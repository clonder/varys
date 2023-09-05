use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, StreamConfig,
};
use log::{debug, error, info, trace, warn};
use simple_moving_average::{NoSumSMA, SMA};

use crate::error::Error;
use crate::listen::audio::AudioData;
use crate::recognise::Recogniser;

pub mod audio;

const CALIBRATION_TIMEOUT: Duration = Duration::from_secs(5);
const RECORDING_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60));
const MOVING_AVERAGE_WINDOW_SIZE: usize = 1024;
/// How many seconds of audio data should be expected by default when starting a recording.
const RECORDING_BUFFER_CAPACITY_SECONDS: usize = 10;

/// A listener that can parse voice input.
pub struct Listener {
    device: Device,
    device_config: StreamConfig,
    /// The optional maximum duration to record for.
    /// Use this to stop any recording longer than the specified duration.
    /// This ensures the listener does not record forever if there is interference or noise.
    ///
    /// Defaults to [`Listener::DEFAULT_RECORDING_TIMEOUT`].
    pub recording_timeout: Option<Duration>,
}

impl Listener {
    /// Create a new listener using the system default input device.
    ///
    /// Returns an error if no input device was found or if it doesn't support the required sample
    /// rate and format.
    ///
    /// # Examples
    ///
    /// ```
    /// # use varys::listen::Listener;
    /// let listener = Listener::new().unwrap();
    /// ```
    pub fn new() -> Result<Self, Error> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or(Error::AudioInputDeviceNotFound)?;
        if let Ok(name) = device.name() {
            debug!("Using audio device {}", name);
        }

        let device_config: StreamConfig = device
            .supported_input_configs()?
            .find(|config| {
                config.sample_format() == SampleFormat::F32
                    && config.max_sample_rate().0 % Recogniser::SAMPLE_RATE == 0
            })
            .ok_or(Error::ConfigurationNotSupported)?
            .with_max_sample_rate()
            .into();
        debug!("Using audio input config {:?}", device_config);

        Ok(Listener {
            device,
            device_config,
            recording_timeout: RECORDING_TIMEOUT,
        })
    }

    /// Start recording audio data.
    ///
    /// Returns an error if the audio stream could not be built or played. This can happen if the
    /// device is no longer available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use varys::listen::Listener;
    /// let listener = Listener::new().unwrap();
    /// let instance = listener.start().unwrap();
    /// # instance.stop().unwrap();
    /// ```
    pub fn start(&self) -> Result<ListenerInstance, Error> {
        info!("Starting recording...");

        let writer = Arc::new(Mutex::new(Vec::with_capacity(
            self.device_config.sample_rate.0 as usize * RECORDING_BUFFER_CAPACITY_SECONDS,
        )));
        let writer_2 = writer.clone();
        let (average_sender, average) = channel();
        let mut running_average = NoSumSMA::<_, f32, { MOVING_AVERAGE_WINDOW_SIZE }>::new();
        let mut sample_count: u32 = 0;

        let stream = self.device.build_input_stream(
            &self.device_config,
            move |data: &[f32], _| {
                if let Ok(mut guard) = writer_2.try_lock() {
                    for &sample in data.iter() {
                        guard.push(sample);
                        running_average.add_sample(sample.abs());
                        sample_count += 1;
                        if sample_count >= MOVING_AVERAGE_WINDOW_SIZE as u32 {
                            trace!("{}", running_average.get_average());
                            if average_sender.send(running_average.get_average()).is_err() {
                                warn!("Unable to send recording average.");
                            }
                            sample_count = 0;
                        }
                    }
                }
            },
            move |err| error!("Audio stream error: {}", err),
            self.recording_timeout,
        )?;
        stream.play()?;

        Ok(ListenerInstance {
            stream,
            writer,
            average,
            channels: u8::try_from(self.device_config.channels).map_err(|_| Error::OutOfRange)?,
            sample_rate: self.device_config.sample_rate.0,
        })
    }

    /// Record until silence is detected for a certain amount of time. The current thread is blocked
    /// until recording is done.
    ///
    /// Returns an error if the audio stream could not be built or played. This can happen if the
    /// device is no longer available.
    ///
    /// # Arguments
    ///
    /// * `silence_duration`: How long a silence must be for the recording to be stopped.
    /// * `silence_threshold`: The highest frequency that is considered silence.
    ///
    /// Returns the recorded [`AudioData`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time;
    /// # use varys::listen::Listener;
    /// let listener = Listener::new().unwrap();
    /// let audio_data = listener.record_until_silent(time::Duration::from_secs(0), 0.01);
    /// ```
    pub fn record_until_silent(
        &self,
        silence_duration: Duration,
        silence_threshold: f32,
    ) -> Result<AudioData, Error> {
        info!(
            "Recording audio until silent for {} seconds...",
            silence_duration.as_secs()
        );

        let instance = self.start()?;
        let started = Instant::now();
        let mut last_audio_detected = Instant::now();
        while let Ok(average) = instance.average.recv() {
            let now = Instant::now();
            if average > silence_threshold {
                last_audio_detected = now;
            }
            if last_audio_detected < now - silence_duration {
                break;
            }
            if let Some(timeout) = self.recording_timeout {
                if started < now - timeout {
                    break;
                }
            }
        }
        instance.stop()
    }

    /// Record for a specified amount of seconds. The current thread is blocked until recording is
    /// done.
    ///
    /// Returns an error if the audio stream could not be built or played. This can happen if the
    /// device is no longer available.
    ///
    /// # Arguments
    ///
    /// * `seconds`: How many seconds to record for.
    ///
    /// Returns the recorded [`AudioData`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use varys::listen::Listener;
    /// let listener = Listener::new().unwrap();
    /// let audio_data = listener.record_for(0);
    /// ```
    pub fn record_for(&self, seconds: u32) -> Result<AudioData, Error> {
        info!("Recording audio for {} seconds", seconds);

        let instance = self.start()?;
        for second in (1..=seconds).rev() {
            debug!("{}...", second);
            thread::sleep(Duration::from_secs(1));
        }
        instance.stop()
    }

    /// Listen for a specified amount of seconds to find the ambient noise threshold to use as
    /// sensitivity. The current thread is blocked until recording is done.
    ///
    /// Returns an error if the audio stream could not be built or played. This can happen if the
    /// device is no longer available.
    pub fn calibrate(&self) -> Result<f32, Error> {
        info!("Calibrating ambient noise...");

        let instance = self.start()?;
        let started = Instant::now();
        let mut averages = Vec::new();
        while let Ok(average) = instance.average.recv() {
            averages.push(average);
            if started < Instant::now() - CALIBRATION_TIMEOUT {
                break;
            }
        }
        instance.stop()?;

        Ok(averages.iter().sum::<f32>() / averages.len() as f32)
    }
}

/// A handle to a running listener instance. It can be stopped with [`ListenerInstance::stop`].
pub struct ListenerInstance {
    stream: Stream,
    writer: Arc<Mutex<Vec<f32>>>,
    average: Receiver<f32>,
    channels: u8,
    sample_rate: u32,
}

impl ListenerInstance {
    /// Stop the running listener consuming the instance and get the recorded audio data.
    ///
    /// Returns the recorded [`AudioData`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use varys::listen::Listener;
    /// let instance = Listener::new().unwrap().start().unwrap();
    /// let audio_data = instance.stop().unwrap();
    /// ```
    pub fn stop(self) -> Result<AudioData, Error> {
        info!("Stopping recording...");

        drop(self.stream);
        let data = Arc::try_unwrap(self.writer)
            .map_err(|_| Error::StillRecording)?
            .into_inner()
            .map_err(|_| Error::RecordingFailed)?;

        Ok(AudioData {
            data,
            channels: self.channels,
            sample_rate: self.sample_rate,
        })
    }
}

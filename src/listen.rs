pub mod audio;

use crate::listen::audio::AudioData;
use crate::recognise::Recogniser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, PlayStreamError, SampleFormat, Stream, StreamConfig};
use log::{debug, error, info, trace, warn};
use simple_moving_average::{NoSumSMA, SMA};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Error that happens if no audio input device was found.
    #[error("Audio input device not found")]
    MissingInputDevice,
    /// Error that happens if the audio input device does not support a required configuration.
    #[error("Audio device does not support required configuration")]
    ConfigurationNotSupported,
    /// Error that happens when trying to access audio data while it is still being recorded.
    #[error("Recording still running")]
    StillRecording,
    #[error(transparent)]
    BuildStream(#[from] BuildStreamError),
    #[error(transparent)]
    PlayStream(#[from] PlayStreamError),
    #[error(transparent)]
    RecordingFailed(#[from] PoisonError<Vec<f32>>),
}

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
    const DEFAULT_MOVING_AVERAGE_WINDOW_SIZE: usize = 1024;
    /// How many seconds of audio data should be expected by default when starting a recording.
    const DEFAULT_RECORDING_BUFFER_CAPACITY_SECONDS: usize = 10;
    const DEFAULT_RECORDING_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60));

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
            .ok_or(Error::MissingInputDevice)?;
        if let Ok(name) = device.name() {
            debug!("Using audio device {}", name);
        }

        let device_config: StreamConfig = device
            .supported_input_configs()
            .map_err(|_| Error::ConfigurationNotSupported)?
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
            recording_timeout: Listener::DEFAULT_RECORDING_TIMEOUT,
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
            self.device_config.sample_rate.0 as usize
                * Listener::DEFAULT_RECORDING_BUFFER_CAPACITY_SECONDS,
        )));
        let writer_2 = writer.clone();
        let (average_sender, average) = channel();
        let mut running_average =
            NoSumSMA::<_, f32, { Listener::DEFAULT_MOVING_AVERAGE_WINDOW_SIZE }>::new();
        let mut sample_count: u32 = 0;

        let stream = self.device.build_input_stream(
            &self.device_config,
            move |data: &[f32], _| {
                if let Ok(mut guard) = writer_2.try_lock() {
                    for &sample in data.iter() {
                        guard.push(sample);
                        running_average.add_sample(sample.abs());
                        sample_count += 1;
                        if sample_count >= Listener::DEFAULT_MOVING_AVERAGE_WINDOW_SIZE as u32 {
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
            channels: self.device_config.channels,
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
    /// let audio_data = listener.record_until_silent(time::Duration::from_secs(0), 0.001);
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
}

/// A handle to a running listener instance. It can be stopped with [`ListenerInstance::stop`].
pub struct ListenerInstance {
    stream: Stream,
    writer: Arc<Mutex<Vec<f32>>>,
    average: Receiver<f32>,
    channels: u16,
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
            .into_inner()?;

        Ok(AudioData {
            data,
            channels: self.channels,
            sample_rate: self.sample_rate,
        })
    }
}
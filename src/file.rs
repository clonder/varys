use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, fs::File};

use flate2::{Compression, GzBuilder};
use log::debug;

use crate::error::Error;

pub mod audio;

/// Compress a file into a gzip wrapper. Returns the path to the compressed file.
///
/// The compressed file is written to the same path as the uncompressed one.
///
/// This returns an error if the compressed file could not be created or written.
///
/// # Arguments
///
/// * `file_path`: The path to the file to compress.
/// * `keep`: Whether to keep the uncompressed file.
///
/// # Examples
///
/// This will try to compress `text.txt` into `text.txt.gz`, keeping the original:
///
/// ```no_run
/// # use std::path::Path;
/// # use varys::file;
/// let file_path_compressed = file::compress_gzip(Path::new("text.txt"), true).unwrap();
/// ```
pub fn compress_gzip(file_path: &Path, keep: bool) -> Result<PathBuf, Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(100, file);

    debug!("Compressing {:?} using gzip.", file_path);

    let mut file_path_gz = file_path.to_owned().into_os_string();
    file_path_gz.push(".gz");
    let file_gz = File::create(Path::new(file_path_gz.as_os_str()))?;
    let mut encoder = GzBuilder::new().write(file_gz, Compression::default());

    reader.bytes().for_each(|b| {
        if let Ok(byte) = b {
            let _ = encoder.write_all(&[byte]);
        }
    });
    encoder.finish()?;

    if !keep {
        fs::remove_file(file_path)?;
    }

    Ok(PathBuf::from(file_path_gz))
}

/// Returns the file name if it exists. Otherwise, returns the full path.
///
/// # Arguments
///
/// * `file_path`: The path to the file to get the name from.
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use varys::file::file_name_or_full;
/// assert_eq!(file_name_or_full(Path::new("path/to/text.txt")), "text.txt");
/// ```
pub fn file_name_or_full(file_path: &Path) -> String {
    file_path
        .file_name()
        .unwrap_or(file_path.as_os_str())
        .to_string_lossy()
        .to_string()
}

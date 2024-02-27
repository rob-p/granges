//! Essential TSV parsing functionality, which wraps the blazingly-fast [`csv`] crate's
//! deserialization method using [`serde`].

use csv::{DeserializeRecordsIntoIter, ReaderBuilder};
use flate2::read::GzDecoder;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::GRangesError;

/// Deserializes some value of type `t` with some possible missing
/// character `missing_chars` into [`Option<T>`].
pub fn deserialize_option_generic<'de, D, T>(
    deserializer: D,
    missing_chars: &'de [&'de str],
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if missing_chars.contains(&s.as_str()) {
        Ok(None)
    } else {
        s.parse::<T>()
            .map(Some)
            .map_err(|e| DeError::custom(format!("parsing error: {}", e)))
    }
}

/// An extensible TSV parser, which uses a supplied parser function to
/// convert a line into a [`GenomicRangeRecord<U>`], a range with generic associated
/// data.
pub struct TsvRecordIterator<T> {
    inner: DeserializeRecordsIntoIter<Box<dyn std::io::Read>, T>,
}

impl<T> std::fmt::Debug for TsvRecordIterator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TsvRecordIterator").finish_non_exhaustive()
    }
}

/// Check if a file is a gzipped by looking for the magic numbers
fn is_gzipped_file(file_path: impl Into<PathBuf>) -> io::Result<bool> {
    let mut file = File::open(file_path.into())?;
    let mut buffer = [0; 2];
    file.read_exact(&mut buffer)?;

    Ok(buffer == [0x1f, 0x8b])
}

impl<T> TsvRecordIterator<T>
where
    for<'de> T: Deserialize<'de>,
{
    pub fn new(filepath: impl Into<PathBuf>) -> Result<Self, GRangesError> {
        let filepath = filepath.into();

        let file = File::open(&filepath)?;
        let is_gzipped = is_gzipped_file(&filepath)?;
        let stream: Box<dyn Read> = if is_gzipped {
            Box::new(GzDecoder::new(file))
        } else {
            Box::new(file)
        };

        let reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(stream);

        let inner = reader.into_deserialize();

        Ok(Self { inner })
    }
}

impl<T> Iterator for TsvRecordIterator<T>
where
    for<'de> T: Deserialize<'de>,
{
    type Item = Result<T, GRangesError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|res| res.map_err(|e| GRangesError::IOError(e.into())))
    }
}

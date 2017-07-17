use std::borrow::Cow;
use std::convert::From;
use std::io;
use std::path::PathBuf;
use std::time;

use espm;

#[derive(Debug)]
pub enum Error {
    InvalidPath(PathBuf),
    IoError(io::Error),
    NoFilename,
    SystemTimeError(time::SystemTimeError),
    NonUTF8FilePath,
    NonUTF8StringData,
    DecodeError(Cow<'static, str>),
    ParsingError,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<time::SystemTimeError> for Error {
    fn from(error: time::SystemTimeError) -> Self {
        Error::SystemTimeError(error)
    }
}

impl From<espm::Error> for Error {
    fn from(error: espm::Error) -> Self {
        match error {
            espm::Error::NonUtf8FilePath => Error::NonUTF8FilePath,
            espm::Error::NonUtf8StringData => Error::NonUTF8StringData,
            espm::Error::IoError(x) => Error::IoError(x),
            espm::Error::NoFilename => Error::NoFilename,
            espm::Error::ParsingIncomplete => Error::ParsingError,
            espm::Error::ParsingError => Error::ParsingError,
            espm::Error::DecodeError(x) => Error::DecodeError(x),
        }
    }
}

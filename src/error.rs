use std::convert::From;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    InvalidPath(PathBuf),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

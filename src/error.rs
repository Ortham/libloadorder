/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2017 Oliver Hamlet
 *
 * libloadorder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * libloadorder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with libloadorder. If not, see <http://www.gnu.org/licenses/>.
 */

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
            espm::Error::ParsingIncomplete |
            espm::Error::ParsingError => Error::ParsingError,
            espm::Error::DecodeError(x) => Error::DecodeError(x),
        }
    }
}

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
use std::string::FromUtf8Error;
use std::time;

use espm;
use espm::GameId as EspmId;
use regex;

#[cfg(windows)]
use app_dirs;

#[derive(Debug, PartialEq)]
pub enum LoadOrderMethod {
    Timestamp,
    Textfile,
    Asterisk,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameId {
    Morrowind,
    Oblivion,
    Skyrim,
    Fallout3,
    FalloutNV,
    Fallout4,
    SkyrimSE,
}

impl GameId {
    pub fn to_libespm_id(&self) -> EspmId {
        match *self {
            GameId::Morrowind => EspmId::Morrowind,
            GameId::Oblivion => EspmId::Oblivion,
            GameId::Skyrim | GameId::SkyrimSE => EspmId::Skyrim,
            GameId::Fallout3 => EspmId::Fallout3,
            GameId::FalloutNV => EspmId::FalloutNV,
            GameId::Fallout4 => EspmId::Fallout4,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidPath(PathBuf),
    IoError(io::Error),
    NoFilename,
    SystemTimeError(time::SystemTimeError),
    NotUtf8(Vec<u8>),
    DecodeError(Cow<'static, str>),
    EncodeError(Cow<'static, str>),
    ParsingError,
    PluginNotFound,
    TooManyActivePlugins,
    InvalidRegex,
    DuplicatePlugin,
    NonMasterBeforeMaster,
    GameMasterMustLoadFirst,
    InvalidPlugin(String),
    ImplicitlyActivePlugin(String),
    NoLocalAppData,
}

#[cfg(windows)]
impl From<app_dirs::AppDirsError> for Error {
    fn from(error: app_dirs::AppDirsError) -> Self {
        match error {
            app_dirs::AppDirsError::Io(x) => Error::IoError(x),
            _ => Error::NoLocalAppData,
        }
    }
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

impl From<regex::Error> for Error {
    fn from(_: regex::Error) -> Self {
        Error::InvalidRegex
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::NotUtf8(error.into_bytes())
    }
}

impl From<espm::Error> for Error {
    fn from(error: espm::Error) -> Self {
        match error {
            espm::Error::IoError(x) => Error::IoError(x),
            espm::Error::NoFilename => Error::NoFilename,
            espm::Error::ParsingIncomplete |
            espm::Error::ParsingError => Error::ParsingError,
            espm::Error::DecodeError(x) => Error::DecodeError(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_id_should_map_to_libespm_id_correctly() {
        assert_eq!(EspmId::Morrowind, GameId::Morrowind.to_libespm_id());
        assert_eq!(EspmId::Oblivion, GameId::Oblivion.to_libespm_id());
        assert_eq!(EspmId::Skyrim, GameId::Skyrim.to_libespm_id());
        assert_eq!(EspmId::Skyrim, GameId::SkyrimSE.to_libespm_id());
        assert_eq!(EspmId::Fallout3, GameId::Fallout3.to_libespm_id());
        assert_eq!(EspmId::FalloutNV, GameId::FalloutNV.to_libespm_id());
        assert_eq!(EspmId::Fallout4, GameId::Fallout4.to_libespm_id());
    }
}

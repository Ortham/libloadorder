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
use std::error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use std::time;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LoadOrderMethod {
    Timestamp,
    Textfile,
    Asterisk,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GameId {
    Morrowind = 1,
    Oblivion,
    Skyrim,
    Fallout3,
    FalloutNV,
    Fallout4,
    SkyrimSE,
    Fallout4VR,
    SkyrimVR,
    Starfield,
}

impl GameId {
    pub fn to_esplugin_id(self) -> esplugin::GameId {
        match self {
            GameId::Morrowind => esplugin::GameId::Morrowind,
            GameId::Oblivion => esplugin::GameId::Oblivion,
            GameId::Skyrim => esplugin::GameId::Skyrim,
            GameId::SkyrimSE => esplugin::GameId::SkyrimSE,
            GameId::SkyrimVR => esplugin::GameId::SkyrimSE,
            GameId::Fallout3 => esplugin::GameId::Fallout3,
            GameId::FalloutNV => esplugin::GameId::FalloutNV,
            GameId::Fallout4 => esplugin::GameId::Fallout4,
            GameId::Fallout4VR => esplugin::GameId::Fallout4,
            GameId::Starfield => esplugin::GameId::Starfield,
        }
    }

    pub fn supports_light_plugins(self) -> bool {
        use self::GameId::*;
        matches!(
            self,
            Fallout4 | Fallout4VR | SkyrimSE | SkyrimVR | Starfield
        )
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
    PluginParsingError,
    PluginNotFound(String),
    TooManyActivePlugins,
    InvalidRegex,
    DuplicatePlugin,
    NonMasterBeforeMaster,
    GameMasterMustLoadFirst,
    InvalidEarlyLoadingPluginPosition {
        name: String,
        pos: usize,
        expected_pos: usize,
    },
    InvalidPlugin(String),
    ImplicitlyActivePlugin(String),
    NoLocalAppData,
    NoDocumentsPath,
    UnrepresentedHoist {
        plugin: String,
        master: String,
    },
    InstalledPlugin(String),
    IniParsingError {
        line: usize,
        column: usize,
        message: String,
    },
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

impl From<esplugin::Error> for Error {
    fn from(error: esplugin::Error) -> Self {
        match error {
            esplugin::Error::IoError(x) => Error::IoError(x),
            esplugin::Error::NoFilename => Error::NoFilename,
            esplugin::Error::ParsingIncomplete | esplugin::Error::ParsingError(_, _) => {
                Error::PluginParsingError
            }
            esplugin::Error::DecodeError => Error::DecodeError("invalid sequence".into()),
        }
    }
}

impl From<ini::Error> for Error {
    fn from(error: ini::Error) -> Self {
        match error {
            ini::Error::Io(x) => Error::IoError(x),
            ini::Error::Parse(x) => Error::from(x),
        }
    }
}

impl From<ini::ParseError> for Error {
    fn from(error: ini::ParseError) -> Self {
        Error::IniParsingError {
            line: error.line,
            column: error.col,
            message: error.msg,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidPath(ref x) => write!(f, "The path \"{:?}\" is invalid", x),
            Error::IoError(ref x) => x.fmt(f),
            Error::NoFilename => write!(f, "The plugin path has no filename part"),
            Error::SystemTimeError(ref x) => x.fmt(f),
            Error::NotUtf8(ref x) => write!(f, "Expected a UTF-8 string, got bytes {:?}", x),
            Error::DecodeError(_) => write!(f, "Text could not be decoded from Windows-1252"),
            Error::EncodeError(_) => write!(f, "Text could not be encoded in Windows-1252"),
            Error::PluginParsingError => {
                write!(f, "An error was encountered while parsing a plugin")
            }
            Error::PluginNotFound(ref x) => {
                write!(f, "The plugin \"{}\" is not in the load order", x)
            }
            Error::TooManyActivePlugins => write!(f, "Maximum number of active plugins exceeded"),
            Error::InvalidRegex => write!(
                f,
                "Internal error: regex is invalid"
            ),
            Error::DuplicatePlugin => write!(f, "The given plugin list contains duplicates"),
            Error::NonMasterBeforeMaster => write!(
                f,
                "Attempted to load a non-master plugin before a master plugin"
            ),
            Error::GameMasterMustLoadFirst => write!(
                f,
                "The game's master file must load first"
            ),
            Error::InvalidEarlyLoadingPluginPosition{ ref name, pos, expected_pos } => write!(
                f,
                "Attempted to load the early-loading plugin {} at position {}, its expected position is {}", name, pos, expected_pos
            ),
            Error::InvalidPlugin(ref x) => write!(f, "The plugin file \"{}\" is invalid", x),
            Error::ImplicitlyActivePlugin(ref x) => write!(
                f,
                "The implicitly active plugin \"{}\" cannot be deactivated",
                x
            ),
            Error::NoLocalAppData => {
                write!(f, "The game's local app data folder could not be detected")
            }
            Error::NoDocumentsPath => {
                write!(f, "The user's Documents path could not be detected")
            }
            Error::UnrepresentedHoist { ref plugin, ref master } => write!(
                f,
                "The plugin \"{}\" is a master of \"{}\", which will hoist it",
                plugin, master
            ),
            Error::InstalledPlugin(ref plugin) => write!(
                f,
                "The plugin \"{}\" is installed, so cannot be removed from the load order",
                plugin
            ),
            Error::IniParsingError {
                line,
                column,
                ref message,
            } => write!(
                f,
                "Failed to parse ini file, error at line {}, column {}: {}",
                line, column, message
            ),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::IoError(ref x) => Some(x),
            Error::SystemTimeError(ref x) => Some(x),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_id_should_map_to_libespm_id_correctly() {
        assert_eq!(
            esplugin::GameId::Morrowind,
            GameId::Morrowind.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::Oblivion,
            GameId::Oblivion.to_esplugin_id()
        );
        assert_eq!(esplugin::GameId::Skyrim, GameId::Skyrim.to_esplugin_id());
        assert_eq!(
            esplugin::GameId::SkyrimSE,
            GameId::SkyrimSE.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::SkyrimSE,
            GameId::SkyrimVR.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::Fallout3,
            GameId::Fallout3.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::FalloutNV,
            GameId::FalloutNV.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::Fallout4,
            GameId::Fallout4.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::Fallout4,
            GameId::Fallout4VR.to_esplugin_id()
        );
        assert_eq!(
            esplugin::GameId::Starfield,
            GameId::Starfield.to_esplugin_id()
        );
    }

    #[test]
    fn game_id_supports_light_plugins_should_be_false_until_fallout_4() {
        assert!(!GameId::Morrowind.supports_light_plugins());
        assert!(!GameId::Oblivion.supports_light_plugins());
        assert!(!GameId::Skyrim.supports_light_plugins());
        assert!(GameId::SkyrimSE.supports_light_plugins());
        assert!(GameId::SkyrimVR.supports_light_plugins());
        assert!(!GameId::Fallout3.supports_light_plugins());
        assert!(!GameId::FalloutNV.supports_light_plugins());
        assert!(GameId::Fallout4.supports_light_plugins());
        assert!(GameId::Fallout4VR.supports_light_plugins());
        assert!(GameId::Starfield.supports_light_plugins());
    }
}

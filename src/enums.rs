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

use std::error;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::slice::EscapeAscii;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum LoadOrderMethod {
    Timestamp,
    Textfile,
    Asterisk,
    OpenMW,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
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
    OpenMW,
    OblivionRemastered,
}

impl GameId {
    pub fn to_esplugin_id(self) -> esplugin::GameId {
        match self {
            GameId::Morrowind | GameId::OpenMW => esplugin::GameId::Morrowind,
            GameId::Oblivion | GameId::OblivionRemastered => esplugin::GameId::Oblivion,
            GameId::Skyrim => esplugin::GameId::Skyrim,
            GameId::SkyrimSE | GameId::SkyrimVR => esplugin::GameId::SkyrimSE,
            GameId::Fallout3 => esplugin::GameId::Fallout3,
            GameId::FalloutNV => esplugin::GameId::FalloutNV,
            GameId::Fallout4 | GameId::Fallout4VR => esplugin::GameId::Fallout4,
            GameId::Starfield => esplugin::GameId::Starfield,
        }
    }

    pub fn supports_light_plugins(self) -> bool {
        matches!(
            self,
            Self::Fallout4 | Self::Fallout4VR | Self::SkyrimSE | Self::SkyrimVR | Self::Starfield
        )
    }

    pub fn supports_medium_plugins(self) -> bool {
        self == GameId::Starfield
    }

    pub fn allow_plugin_ghosting(self) -> bool {
        self != GameId::OpenMW
    }

    pub(crate) fn treats_master_files_differently(self) -> bool {
        !matches!(self, GameId::OpenMW | GameId::OblivionRemastered)
    }
}

#[expect(clippy::error_impl_error)]
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    InvalidPath(PathBuf),
    IoError(PathBuf, io::Error),
    NoFilename(PathBuf),
    DecodeError(Vec<u8>),
    EncodeError(String),
    PluginParsingError(PathBuf, Box<dyn error::Error + Send + Sync + 'static>),
    PluginNotFound(String),
    TooManyActivePlugins {
        light_count: usize,
        medium_count: usize,
        full_count: usize,
    },
    DuplicatePlugin(String),
    NonMasterBeforeMaster {
        master: String,
        non_master: String,
    },
    InvalidEarlyLoadingPluginPosition {
        name: String,
        pos: usize,
        expected_pos: usize,
    },
    ImplicitlyActivePlugin(String),
    NoLocalAppData,
    NoDocumentsPath,
    NoUserConfigPath,
    NoUserDataPath,
    NoProgramFilesPath,
    UnrepresentedHoist {
        plugin: String,
        master: String,
    },
    InstalledPlugin(String),
    IniParsingError {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },
    VdfParsingError(PathBuf, String),
    SystemError(i32, OsString),
    InvalidBlueprintPluginPosition {
        name: String,
        pos: usize,
        expected_pos: usize,
    },
}

#[cfg(windows)]
impl From<windows_result::Error> for Error {
    fn from(error: windows_result::Error) -> Self {
        Error::SystemError(error.code().0, error.message().into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidPath(path) => write!(f, "The path \"{}\" is invalid", escape_ascii(path)),
            Error::IoError(path, error) =>
                write!(f, "I/O error involving the path \"{}\": {error}", escape_ascii(path)),
            Error::NoFilename(path) =>
                write!(f, "The plugin path \"{}\" has no filename part", escape_ascii(path)),
            Error::DecodeError(bytes) => write!(f, "String could not be decoded from Windows-1252: {}", bytes.escape_ascii()),
            Error::EncodeError(string) => write!(f, "The string \"{string}\" could not be encoded to Windows-1252"),
            Error::PluginParsingError(path, err) => {
                write!(f, "An error was encountered while parsing the plugin at \"{}\": {err}", escape_ascii(path))
            }
            Error::PluginNotFound(name) => {
                write!(f, "The plugin \"{name}\" is not in the load order")
            }
            Error::TooManyActivePlugins {light_count, medium_count, full_count } =>
                write!(f, "Maximum number of active plugins exceeded: there are {full_count} active full plugins, {medium_count} active medium plugins and {light_count} active light plugins"),
            Error::DuplicatePlugin(name) =>
                write!(f, "The given plugin list contains more than one instance of \"{name}\""),
            Error::NonMasterBeforeMaster{ master, non_master} =>
                write!(f, "Attempted to load the non-master plugin \"{non_master}\" before the master plugin \"{master}\""),
            Error::InvalidEarlyLoadingPluginPosition{ name, pos, expected_pos } =>
                write!(f, "Attempted to load the early-loading plugin \"{name}\" at position {pos}, its expected position is {expected_pos}"),
            Error::ImplicitlyActivePlugin(name) =>
                write!(f, "The implicitly active plugin \"{name}\" cannot be deactivated"),
            Error::NoLocalAppData => {
                write!(f, "The game's local app data folder could not be detected")
            }
            Error::NoDocumentsPath => write!(f, "The user's Documents path could not be detected"),
            Error::NoUserConfigPath => write!(f, "The user's config path could not be detected"),
            Error::NoUserDataPath => write!(f, "The user's data path could not be detected"),
            Error::NoProgramFilesPath => write!(f, "The Program Files path could not be obtained"),
            Error::UnrepresentedHoist { plugin, master } =>
                write!(f, "The plugin \"{plugin}\" is a master of \"{master}\", which will hoist it"),
            Error::InstalledPlugin(plugin) =>
                write!(f, "The plugin \"{plugin}\" is installed, so cannot be removed from the load order"),
            Error::IniParsingError {
                path,
                line,
                column,
                message,
            } => write!(f, "Failed to parse ini file at \"{}\", error at line {line}, column {column}: {message}", escape_ascii(path)),
            Error::VdfParsingError(path, message) =>
                write!(f, "Failed to parse VDF file at \"{}\": {message}", escape_ascii(path)),
            Error::SystemError(code, message) =>
                write!(f, "Error returned by the operating system, code {code}: \"{}\"", message.as_encoded_bytes().escape_ascii()),
            Error::InvalidBlueprintPluginPosition{ name, pos, expected_pos } =>
                write!(f, "Attempted to load the blueprint plugin \"{name}\" at position {pos}, its expected position is {expected_pos}"),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            Error::IoError(_, x) => Some(x),
            Error::PluginParsingError(_, x) => Some(x.as_ref()),
            _ => None,
        }
    }
}

fn escape_ascii(path: &Path) -> EscapeAscii<'_> {
    path.as_os_str().as_encoded_bytes().escape_ascii()
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
        assert!(!GameId::OpenMW.supports_light_plugins());
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

    #[test]
    fn game_id_supports_medium_plugins_should_be_false_except_for_starfield() {
        assert!(!GameId::OpenMW.supports_medium_plugins());
        assert!(!GameId::Morrowind.supports_medium_plugins());
        assert!(!GameId::Oblivion.supports_medium_plugins());
        assert!(!GameId::Skyrim.supports_medium_plugins());
        assert!(!GameId::SkyrimSE.supports_medium_plugins());
        assert!(!GameId::SkyrimVR.supports_medium_plugins());
        assert!(!GameId::Fallout3.supports_medium_plugins());
        assert!(!GameId::FalloutNV.supports_medium_plugins());
        assert!(!GameId::Fallout4.supports_medium_plugins());
        assert!(!GameId::Fallout4VR.supports_medium_plugins());
        assert!(GameId::Starfield.supports_medium_plugins());
    }

    #[test]
    fn game_id_allow_plugin_ghosting_should_be_false_for_openmw_only() {
        assert!(!GameId::OpenMW.allow_plugin_ghosting());
        assert!(GameId::Morrowind.allow_plugin_ghosting());
        assert!(GameId::Oblivion.allow_plugin_ghosting());
        assert!(GameId::Skyrim.allow_plugin_ghosting());
        assert!(GameId::SkyrimSE.allow_plugin_ghosting());
        assert!(GameId::SkyrimVR.allow_plugin_ghosting());
        assert!(GameId::Fallout3.allow_plugin_ghosting());
        assert!(GameId::FalloutNV.allow_plugin_ghosting());
        assert!(GameId::Fallout4.allow_plugin_ghosting());
        assert!(GameId::Fallout4VR.allow_plugin_ghosting());
        assert!(GameId::Starfield.allow_plugin_ghosting());
    }

    #[test]
    fn game_id_treats_master_files_differently_should_be_false_for_openmw_and_oblivion_remastered_only(
    ) {
        assert!(!GameId::OpenMW.treats_master_files_differently());
        assert!(GameId::Morrowind.treats_master_files_differently());
        assert!(GameId::Oblivion.treats_master_files_differently());
        assert!(!GameId::OblivionRemastered.treats_master_files_differently());
        assert!(GameId::Skyrim.treats_master_files_differently());
        assert!(GameId::SkyrimSE.treats_master_files_differently());
        assert!(GameId::SkyrimVR.treats_master_files_differently());
        assert!(GameId::Fallout3.treats_master_files_differently());
        assert!(GameId::FalloutNV.treats_master_files_differently());
        assert!(GameId::Fallout4.treats_master_files_differently());
        assert!(GameId::Fallout4VR.treats_master_files_differently());
        assert!(GameId::Starfield.treats_master_files_differently());
    }

    #[test]
    fn error_display_should_print_double_quoted_paths() {
        let string = format!("{}", Error::InvalidPath(PathBuf::from("foo")));

        assert_eq!("The path \"foo\" is invalid", string);
    }

    #[test]
    fn error_display_should_print_os_string_as_quoted_string() {
        let string = format!("{}", Error::SystemError(1, OsString::from("foo")));

        assert_eq!(
            "Error returned by the operating system, code 1: \"foo\"",
            string
        );
    }
}

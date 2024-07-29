use std::ffi::OsStr;
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
use std::fs::{File, FileTimes};
use std::path::Path;
use std::time::SystemTime;

use esplugin::ParseOptions;
use unicase::eq;

use crate::enums::{Error, GameId};
use crate::game_settings::GameSettings;
use crate::ghostable_path::{GhostablePath, GHOST_FILE_EXTENSION};

const VALID_EXTENSIONS: &[&str] = &[".esp", ".esm", ".esp.ghost", ".esm.ghost"];

const VALID_EXTENSIONS_WITH_ESL: &[&str] = &[
    ".esp",
    ".esm",
    ".esp.ghost",
    ".esm.ghost",
    ".esl",
    ".esl.ghost",
];

#[derive(Clone, Debug)]
pub struct Plugin {
    active: bool,
    modification_time: SystemTime,
    data: esplugin::Plugin,
    name: String,
}

impl Plugin {
    pub fn new(filename: &str, game_settings: &GameSettings) -> Result<Plugin, Error> {
        Plugin::with_active(filename, game_settings, false)
    }

    pub fn with_active(
        filename: &str,
        game_settings: &GameSettings,
        active: bool,
    ) -> Result<Plugin, Error> {
        let filepath = game_settings.plugin_path(filename);

        let filepath = if active {
            filepath.unghost()?
        } else {
            filepath.resolve_path()?
        };

        Plugin::with_path(&filepath, game_settings.id(), active)
    }

    pub(crate) fn with_path(path: &Path, game_id: GameId, active: bool) -> Result<Plugin, Error> {
        let filename = match path.file_name().and_then(OsStr::to_str) {
            Some(n) => n,
            None => return Err(Error::NoFilename(path.to_path_buf())),
        };

        if !has_plugin_extension(filename, game_id) {
            return Err(Error::InvalidPath(path.to_path_buf()));
        }

        let file = File::open(path).map_err(|e| Error::IoError(path.to_path_buf(), e))?;
        let modification_time = file
            .metadata()
            .and_then(|m| m.modified())
            .map_err(|e| Error::IoError(path.to_path_buf(), e))?;

        let mut data = esplugin::Plugin::new(game_id.to_esplugin_id(), path);
        data.parse_reader(file, ParseOptions::header_only())
            .map_err(|e| file_error(path, e))?;

        Ok(Plugin {
            active,
            modification_time,
            data,
            name: trim_dot_ghost(filename).to_string(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_matches(&self, string: &str) -> bool {
        eq(self.name(), trim_dot_ghost(string))
    }

    pub fn modification_time(&self) -> SystemTime {
        self.modification_time
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_master_file(&self) -> bool {
        self.data.is_master_file()
    }

    pub fn is_light_plugin(&self) -> bool {
        self.data.is_light_plugin()
    }

    pub fn is_medium_plugin(&self) -> bool {
        self.data.is_medium_plugin()
    }

    pub fn is_blueprint_master(&self) -> bool {
        self.is_master_file() && self.data.is_blueprint_plugin()
    }

    pub fn masters(&self) -> Result<Vec<String>, Error> {
        self.data
            .masters()
            .map_err(|e| file_error(self.data.path(), e))
    }

    pub fn set_modification_time(&mut self, time: SystemTime) -> Result<(), Error> {
        // Always write the file time. This has a huge performance impact, but
        // is important for correctness, as otherwise external changes to plugin
        // timestamps between calls to WritableLoadOrder::load() and
        // WritableLoadOrder::save() could lead to libloadorder not setting all
        // the timestamps it needs to and producing an incorrect load order.
        let times = FileTimes::new()
            .set_accessed(SystemTime::now())
            .set_modified(time);

        File::options()
            .write(true)
            .open(self.data.path())
            .and_then(|f| f.set_times(times))
            .map_err(|e| Error::IoError(self.data.path().to_path_buf(), e))?;

        self.modification_time = time;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), Error> {
        if !self.is_active() {
            if self.data.path().is_ghosted() {
                let new_path = self.data.path().unghost()?;

                self.data = esplugin::Plugin::new(self.data.game_id(), &new_path);
                self.data
                    .parse_file(ParseOptions::header_only())
                    .map_err(|e| file_error(self.data.path(), e))?;
                let modification_time = self.modification_time();
                self.set_modification_time(modification_time)?;
            }

            self.active = true;
        }
        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

pub fn has_plugin_extension(filename: &str, game: GameId) -> bool {
    let valid_extensions = if game.supports_light_plugins() {
        VALID_EXTENSIONS_WITH_ESL
    } else {
        VALID_EXTENSIONS
    };

    valid_extensions
        .iter()
        .any(|e| iends_with_ascii(filename, e))
}

fn iends_with_ascii(string: &str, suffix: &str) -> bool {
    // as_bytes().into_iter() is faster than bytes().
    string.len() >= suffix.len()
        && string
            .as_bytes()
            .iter()
            .rev()
            .zip(suffix.as_bytes().iter().rev())
            .all(|(string_byte, suffix_byte)| string_byte.eq_ignore_ascii_case(suffix_byte))
}

pub fn trim_dot_ghost(string: &str) -> &str {
    if iends_with_ascii(string, GHOST_FILE_EXTENSION) {
        &string[..(string.len() - GHOST_FILE_EXTENSION.len())]
    } else {
        string
    }
}

fn file_error(file_path: &Path, error: esplugin::Error) -> Error {
    match error {
        esplugin::Error::IoError(x) => Error::IoError(file_path.to_path_buf(), x),
        esplugin::Error::NoFilename(_) => Error::NoFilename(file_path.to_path_buf()),
        e => Error::PluginParsingError(file_path.to_path_buf(), Box::new(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::copy_to_test_dir;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, UNIX_EPOCH};
    use tempfile::tempdir;

    fn game_settings(game_id: GameId, game_path: &Path) -> GameSettings {
        GameSettings::with_local_and_my_games_paths(
            game_id,
            game_path,
            &PathBuf::default(),
            PathBuf::default(),
        )
        .unwrap()
    }

    #[test]
    fn name_should_return_the_plugin_filename_without_any_ghost_extension() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp.ghost", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.name());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.name());

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", &settings);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();
        assert_eq!("Blank.esm", plugin.name());
    }

    #[test]
    fn name_matches_should_ignore_plugin_ghost_extension() {
        let tmp_dir = tempdir().unwrap();
        let settings = game_settings(GameId::Skyrim, tmp_dir.path());
        copy_to_test_dir("Blank.esp", "BlanK.esp.GHoSt", &settings);

        let plugin = Plugin::new("BlanK.esp.GHoSt", &settings).unwrap();
        assert!(plugin.name_matches("Blank.esp"));
    }

    #[test]
    fn name_matches_should_ignore_string_ghost_suffix() {
        let tmp_dir = tempdir().unwrap();
        let settings = game_settings(GameId::Skyrim, tmp_dir.path());
        copy_to_test_dir("Blank.esp", "BlanK.esp", &settings);

        let plugin = Plugin::new("BlanK.esp", &settings).unwrap();
        assert!(plugin.name_matches("Blank.esp.GHoSt"));
    }

    #[test]
    fn modification_time_should_return_the_plugin_modification_time_at_creation() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin_path = game_dir.join("Data").join("Blank.esp");
        let mtime = plugin_path.metadata().unwrap().modified().unwrap();

        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!(mtime, plugin.modification_time());
    }

    #[test]
    fn is_active_should_be_false() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_active());
    }

    #[test]
    fn is_master_file_should_be_true_if_the_plugin_is_a_master_file() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();

        assert!(plugin.is_master_file());
    }

    #[test]
    fn is_master_file_should_be_false_if_the_plugin_is_not_a_master_file() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_master_file());
    }

    #[test]
    fn is_light_plugin_should_be_true_for_esl_files_only() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::SkyrimSE, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_master_file());

        copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();

        assert!(!plugin.is_light_plugin());

        copy_to_test_dir("Blank.esm", "Blank.esl", &settings);
        let plugin = Plugin::new("Blank.esl", &settings).unwrap();

        assert!(plugin.is_light_plugin());

        copy_to_test_dir("Blank - Different.esp", "Blank - Different.esl", &settings);
        let plugin = Plugin::new("Blank - Different.esl", &settings).unwrap();

        assert!(plugin.is_light_plugin());
    }

    #[test]
    fn set_modification_time_should_update_the_file_modification_time() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);

        let path = game_dir.join("Data").join("Blank.esp");
        let file_size = path.metadata().unwrap().len();

        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert_ne!(UNIX_EPOCH, plugin.modification_time());
        plugin.set_modification_time(UNIX_EPOCH).unwrap();

        let metadata = path.metadata().unwrap();
        let new_mtime = metadata.modified().unwrap();
        let new_size = metadata.len();

        assert_eq!(UNIX_EPOCH, plugin.modification_time());
        assert_eq!(UNIX_EPOCH, new_mtime);
        assert_eq!(file_size, new_size);
    }

    #[test]
    fn set_modification_time_should_be_able_to_handle_pre_unix_timestamps() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();
        let target_mtime = UNIX_EPOCH - Duration::from_secs(1);

        assert_ne!(target_mtime, plugin.modification_time());
        plugin.set_modification_time(target_mtime).unwrap();
        let new_mtime = game_dir
            .join("Data")
            .join("Blank.esp")
            .metadata()
            .unwrap()
            .modified()
            .unwrap();

        assert_eq!(target_mtime, plugin.modification_time());
        assert_eq!(target_mtime, new_mtime);
    }

    #[test]
    fn activate_should_unghost_a_ghosted_plugin() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        plugin.activate().unwrap();

        assert!(plugin.is_active());
        assert_eq!("Blank.esp", plugin.name());
        assert!(game_dir.join("Data").join("Blank.esp").exists());
    }

    #[test]
    fn deactivate_should_not_ghost_a_plugin() {
        let tmp_dir = tempdir().unwrap();
        let game_dir = tmp_dir.path();

        let settings = game_settings(GameId::Oblivion, &game_dir);

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        plugin.deactivate();

        assert!(!plugin.is_active());
        assert!(game_dir.join("Data").join("Blank.esp").exists());
    }
}

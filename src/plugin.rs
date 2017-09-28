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

use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use esplugin;
use filetime::{FileTime, set_file_times};
use unicase::eq;

use enums::{Error, GameId};
use game_settings::GameSettings;
use ghostable_path::GhostablePath;

#[derive(Clone, Debug)]
pub struct Plugin {
    game: GameId,
    active: bool,
    modification_time: SystemTime,
    data: esplugin::Plugin,
}

impl Plugin {
    pub fn new(filename: &str, game_settings: &GameSettings) -> Result<Plugin, Error> {
        let filepath = game_settings
            .plugins_directory()
            .join(filename)
            .resolve_path()?;

        let modification_time = filepath.metadata()?.modified()?;

        let mut data = esplugin::Plugin::new(game_settings.id().to_esplugin_id(), &filepath);
        data.parse_file(true)?;

        Ok(Plugin {
            game: game_settings.id(),
            active: false,
            modification_time,
            data,
        })
    }

    pub fn name(&self) -> Option<String> {
        self.data.filename()
    }

    pub fn unghosted_name(&self) -> Option<String> {
        self.data.filename().map(|f| trim_dot_ghost(&f).to_string())
    }

    pub fn name_matches(&self, string: &str) -> bool {
        match self.unghosted_name() {
            None => false,
            Some(n) => eq(n.as_str(), trim_dot_ghost(string)),
        }
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

    pub fn has_file_changed(&self) -> Result<bool, Error> {
        let current_mtime = self.data.path().metadata()?.modified()?;

        Ok(self.modification_time != current_mtime)
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        self.modification_time = self.data.path().metadata()?.modified()?;
        Ok(self.data.parse_file(true)?)
    }

    pub fn set_modification_time(&mut self, time: SystemTime) -> Result<(), Error> {
        let atime = FileTime::from_last_access_time(&self.data.path().metadata()?);
        let mtime =
            FileTime::from_seconds_since_1970(time.duration_since(UNIX_EPOCH)?.as_secs(), 0);
        set_file_times(&self.data.path(), atime, mtime)?;

        self.modification_time = time;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), Error> {
        if self.is_active() {
            Ok(())
        } else {
            let new_path = self.data.path().unghost()?;

            self.modification_time = new_path.metadata()?.modified()?;

            self.data = esplugin::Plugin::new(*self.data.game_id(), &new_path);
            self.data.parse_file(true)?;
            self.active = true;
            Ok(())
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_valid(filename: &str, game_settings: &GameSettings) -> bool {
        if !filename.ends_with(".esp") && !filename.ends_with(".esm") &&
            !filename.ends_with(".esp.ghost") && !filename.ends_with(".esm.ghost")
        {
            return false;
        }

        match game_settings
            .plugins_directory()
            .join(filename)
            .resolve_path() {
            Err(_) => false,
            Ok(ref x) => esplugin::Plugin::is_valid(game_settings.id().to_esplugin_id(), x, true),
        }
    }
}

fn iends_with_ascii(string: &str, suffix: &str) -> bool {
    use std::ascii::AsciiExt;
    string.chars().rev().zip(suffix.chars().rev()).all(
        |(c1, c2)| {
            c1.eq_ignore_ascii_case(&c2)
        },
    )
}

pub fn trim_dot_ghost(string: &str) -> &str {
    if iends_with_ascii(string, ".ghost") {
        &string[..(string.len() - 6)]
    } else {
        string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use tempdir::TempDir;
    use tests::copy_to_test_dir;

    #[test]
    fn name_should_return_the_plugin_filename_that_exists() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp.ghost", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.name().unwrap());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.name().unwrap());

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", &settings);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();
        assert_eq!("Blank.esm.ghost", plugin.name().unwrap());
    }

    #[test]
    fn unghosted_name_should_return_the_plugin_filename_without_any_ghost_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.unghosted_name().unwrap());

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", &settings);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();
        assert_eq!("Blank.esm", plugin.unghosted_name().unwrap());
    }

    #[test]
    fn unghosted_name_should_check_ghost_extension_case_insensitively() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esm", "Blank.esm.GHoST", &settings);
        let plugin = Plugin::new("Blank.esm.GHoST", &settings).unwrap();
        assert_eq!("Blank.esm", plugin.unghosted_name().unwrap());
    }

    #[test]
    fn name_matches_should_ignore_plugin_ghost_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, tmp_dir.path(), &PathBuf::default());
        copy_to_test_dir("Blank.esp", "BlanK.esp.GHoSt", &settings);

        let plugin = Plugin::new("BlanK.esp.GHoSt", &settings).unwrap();
        assert!(plugin.name_matches("Blank.esp"));
    }

    #[test]
    fn name_matches_should_ignore_string_ghost_suffix() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let settings =
            GameSettings::with_local_path(GameId::Skyrim, tmp_dir.path(), &PathBuf::default());
        copy_to_test_dir("Blank.esp", "BlanK.esp", &settings);

        let plugin = Plugin::new("BlanK.esp", &settings).unwrap();
        assert!(plugin.name_matches("Blank.esp.GHoSt"));
    }

    #[test]
    fn modification_time_should_return_the_plugin_modification_time_at_creation() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin_path = game_dir.join("Data").join("Blank.esp");
        let mtime = plugin_path.metadata().unwrap().modified().unwrap();

        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!(mtime, plugin.modification_time());
    }

    #[test]
    fn is_active_should_be_false() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_active());
    }

    #[test]
    fn is_master_file_should_be_true_if_the_plugin_is_a_master_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();

        assert!(plugin.is_master_file());
    }

    #[test]
    fn is_master_file_should_be_false_if_the_plugin_is_not_a_master_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_master_file());
    }

    #[test]
    fn has_file_changed_should_be_true_if_the_plugin_mtime_is_different_from_when_new_was_called() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        set_file_times(
            &game_dir.join("Data").join("Blank.esp"),
            FileTime::zero(),
            FileTime::from_seconds_since_1970(5, 0),
        ).unwrap();
        assert!(plugin.has_file_changed().unwrap());
    }

    #[test]
    fn has_file_changed_should_be_false_if_the_plugin_mtime_is_the_same_as_when_new_was_called() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.has_file_changed().unwrap());
    }

    #[test]
    fn reload_should_reload_the_plugin_file_data_and_modification_time() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();
        let old_mod_time = plugin.modification_time();

        assert!(!plugin.is_master_file());
        copy_to_test_dir("Blank.esm", "Blank.esp", &settings);
        set_file_times(
            &game_dir.join("Data").join("Blank.esp"),
            FileTime::zero(),
            FileTime::zero(),
        ).unwrap();

        plugin.reload().unwrap();
        assert!(plugin.is_master_file());
        assert_ne!(old_mod_time, plugin.modification_time());
    }

    #[test]
    fn set_modification_time_should_update_the_file_modification_time() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert_ne!(UNIX_EPOCH, plugin.modification_time());
        plugin.set_modification_time(UNIX_EPOCH).unwrap();
        let new_mtime = game_dir
            .join("Data")
            .join("Blank.esp")
            .metadata()
            .unwrap()
            .modified()
            .unwrap();

        assert_eq!(UNIX_EPOCH, plugin.modification_time());
        assert_eq!(UNIX_EPOCH, new_mtime);
    }

    #[test]
    fn activate_should_unghost_a_ghosted_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        plugin.activate().unwrap();

        assert!(plugin.is_active());
        assert_eq!("Blank.esp", plugin.name().unwrap());
        assert!(game_dir.join("Data").join("Blank.esp").exists());
    }

    #[test]
    fn deactivate_should_not_ghost_a_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        plugin.deactivate();

        assert!(!plugin.is_active());
        assert!(game_dir.join("Data").join("Blank.esp").exists());
    }

    #[test]
    fn is_valid_should_return_true_for_a_valid_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        assert!(Plugin::is_valid("Blank.esp", &settings));

        copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
        assert!(Plugin::is_valid("Blank.esm", &settings));

        copy_to_test_dir(
            "Blank - Different.esp",
            "Blank - Different.esp.ghost",
            &settings,
        );
        assert!(Plugin::is_valid("Blank - Different.esp", &settings));

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm.ghost",
            &settings,
        );
        assert!(Plugin::is_valid("Blank - Different.esm", &settings));
    }

    #[test]
    fn is_valid_should_return_false_if_the_plugin_does_not_have_a_esp_or_esm_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.pse", &settings);
        assert!(!Plugin::is_valid("Blank.pse", &settings));
    }

    #[test]
    fn is_valid_should_return_false_if_the_path_given_is_not_a_valid_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.bsa", "Blank.esp", &settings);
        assert!(!Plugin::is_valid("Blank.esp", &settings));
    }
}

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use espm;
use filetime::{FileTime, set_file_times};

use error::Error;
use enums::GameId;
use game_settings::GameSettings;
use ghostable_path::GhostablePath;

struct Plugin {
    game: GameId,
    active: bool,
    modification_time: SystemTime,
    data: espm::Plugin,
}

impl Plugin {
    fn new(filename: &str, gameSettings: &GameSettings) -> Result<Plugin, Error> {
        let filepath = gameSettings.plugins_folder().join(filename).resolve_path()?;

        let modification_time = File::open(&filepath)?.metadata()?.modified()?;

        let mut data = espm::Plugin::new(gameSettings.id().to_libespm_id(), &filepath);
        data.parse_file(true)?;

        Ok(Plugin {
            game: gameSettings.id().clone(),
            active: !filepath.is_ghosted(),
            modification_time,
            data,
        })
    }

    fn name(&self) -> Option<String> {
        self.data.filename()
    }

    fn unghosted_name(&self) -> Option<String> {
        self.data.filename().map(|f| if f.ends_with(".ghost") {
            f[..(f.len() - 6)].to_string()
        } else {
            f
        })
    }

    fn modification_time(&self) -> SystemTime {
        self.modification_time
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn is_master_file(&self) -> bool {
        self.data.is_master_file()
    }

    fn has_file_changed(&self) -> Result<bool, Error> {
        let current_mtime = File::open(&self.data.path())?.metadata()?.modified()?;

        Ok(self.modification_time != current_mtime)
    }

    fn set_modification_time(&mut self, time: SystemTime) -> Result<(), Error> {
        let atime = FileTime::from_last_access_time(&File::open(&self.data.path())?.metadata()?);
        let mtime =
            FileTime::from_seconds_since_1970(time.duration_since(UNIX_EPOCH)?.as_secs(), 0);
        set_file_times(&self.data.path(), atime, mtime)?;

        self.modification_time = time;
        Ok(())
    }

    fn activate(&mut self) -> Result<(), Error> {
        if self.is_active() {
            Ok(())
        } else {
            let new_path = self.data.path().unghost()?;

            self.modification_time = File::open(&new_path)?.metadata()?.modified()?;

            self.data = espm::Plugin::new(*self.data.game_id(), &new_path);
            self.data.parse_file(true)?;
            self.active = true;
            Ok(())
        }
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn is_valid(filename: &str, game_settings: &GameSettings) -> bool {
        if !filename.ends_with(".esp") && !filename.ends_with(".esm") &&
            !filename.ends_with(".esp.ghost") && !filename.ends_with(".esm.ghost")
        {
            return false;
        }

        match game_settings.plugins_folder().join(filename).resolve_path() {
            Err(_) => false,
            Ok(ref x) => espm::Plugin::is_valid(game_settings.id().to_libespm_id(), x, true),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::*;

    use std::fs::{copy, create_dir};
    use self::tempdir::TempDir;

    fn copy_to_test_dir(from_file: &str, to_file: &str, game_dir: &Path) {
        let testing_plugins_dir = Path::new("./tests/testing-plugins/Oblivion/Data");
        let data_dir = game_dir.join("Data");
        if !data_dir.exists() {
            create_dir(&data_dir).unwrap();
        }
        copy(testing_plugins_dir.join(from_file), data_dir.join(to_file)).unwrap();
    }

    #[test]
    fn name_should_return_the_plugin_filename_that_exists() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp.ghost", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.name().unwrap());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.name().unwrap());

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", &game_dir);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();
        assert_eq!("Blank.esm.ghost", plugin.name().unwrap());
    }

    #[test]
    fn unghosted_name_should_return_the_plugin_filename_without_any_ghost_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();
        assert_eq!("Blank.esp", plugin.unghosted_name().unwrap());

        copy_to_test_dir("Blank.esm", "Blank.esm.ghost", &game_dir);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();
        assert_eq!("Blank.esm", plugin.unghosted_name().unwrap());
    }

    #[test]
    fn modification_time_should_return_the_plugin_modification_time_at_creation() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin_path = game_dir.join("Data").join("Blank.esp");
        let mtime = File::open(&plugin_path)
            .unwrap()
            .metadata()
            .unwrap()
            .modified()
            .unwrap();

        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        set_file_times(&plugin_path, FileTime::zero(), FileTime::zero()).unwrap();
        assert_eq!(mtime, plugin.modification_time);
    }

    #[test]
    fn is_active_should_be_true_after_creation_if_plugin_is_not_ghosted() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(plugin.is_active());
    }

    #[test]
    fn is_active_should_be_false_after_creation_if_plugin_is_ghosted() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_active());
    }

    #[test]
    fn is_master_file_should_be_true_if_the_plugin_is_a_master_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esm", "Blank.esm", &game_dir);
        let plugin = Plugin::new("Blank.esm", &settings).unwrap();

        assert!(plugin.is_master_file());
    }

    #[test]
    fn is_master_file_should_be_false_if_the_plugin_is_not_a_master_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.is_master_file());
    }

    #[test]
    fn has_file_changed_should_be_true_if_the_plugin_mtime_is_different_from_when_new_was_called() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        set_file_times(
            &game_dir.join("Data").join("Blank.esp"),
            FileTime::zero(),
            FileTime::zero(),
        ).unwrap();
        assert!(plugin.has_file_changed().unwrap());
    }

    #[test]
    fn has_file_changed_should_be_false_if_the_plugin_mtime_is_the_same_as_when_new_was_called() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let plugin = Plugin::new("Blank.esp", &settings).unwrap();

        assert!(!plugin.has_file_changed().unwrap());
    }

    #[test]
    fn set_modification_time_should_update_the_file_modification_time() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        let mut plugin = Plugin::new("Blank.esp", &settings).unwrap();

        plugin.set_modification_time(UNIX_EPOCH).unwrap();
        let new_mtime = File::open(game_dir.join("Data").join("Blank.esp"))
            .unwrap()
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

        copy_to_test_dir("Blank.esp", "Blank.esp.ghost", &game_dir);
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

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
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

        copy_to_test_dir("Blank.esp", "Blank.esp", &game_dir);
        assert!(Plugin::is_valid("Blank.esp", &settings));

        copy_to_test_dir("Blank.esm", "Blank.esm", &game_dir);
        assert!(Plugin::is_valid("Blank.esm", &settings));

        copy_to_test_dir(
            "Blank - Different.esp",
            "Blank - Different.esp.ghost",
            &game_dir,
        );
        assert!(Plugin::is_valid("Blank - Different.esp", &settings));

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm.ghost",
            &game_dir,
        );
        assert!(Plugin::is_valid("Blank - Different.esm", &settings));
    }

    #[test]
    fn is_valid_should_return_false_if_the_plugin_does_not_have_a_esp_or_esm_extension() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.esp", "Blank.pse", &game_dir);
        assert!(!Plugin::is_valid("Blank.pse", &settings));
    }

    #[test]
    fn is_valid_should_return_false_if_the_path_given_is_not_a_valid_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let game_dir = tmp_dir.path();

        let settings =
            GameSettings::with_local_path(GameId::Oblivion, &game_dir, &PathBuf::default());

        copy_to_test_dir("Blank.bsa", "Blank.esp", &game_dir);
        assert!(!Plugin::is_valid("Blank.esp", &settings));
    }
}

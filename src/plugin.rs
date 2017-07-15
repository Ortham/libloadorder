use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use espm;

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

        let data = espm::Plugin::new(gameSettings.id().to_libespm_id(), &filepath);

        Ok(Plugin {
            game: gameSettings.id().clone(),
            active: false,
            modification_time,
            data,
        })
    }

    fn name(&self) -> Option<String> {
        self.data.filename()
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
}

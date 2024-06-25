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

use std::convert::TryFrom;
use std::fmt::Display;
use std::fs::{create_dir_all, File, FileTimes, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::enums::GameId;
use crate::enums::LoadOrderMethod;
use crate::game_settings::GameSettings;
use crate::load_order::strict_encode;
use crate::plugin::Plugin;
use crate::tests::copy_to_test_dir;

use super::mutable::MutableLoadOrder;

pub fn write_load_order_file<T: AsRef<str> + Display>(
    game_settings: &GameSettings,
    filenames: &[T],
) {
    let mut file = File::create(&game_settings.load_order_file().unwrap()).unwrap();

    for filename in filenames {
        writeln!(file, "{}", filename).unwrap();
    }
}

pub fn write_active_plugins_file<T: AsRef<str>>(game_settings: &GameSettings, filenames: &[T]) {
    let mut file = File::create(&game_settings.active_plugins_file()).unwrap();

    if game_settings.id() == GameId::Morrowind {
        writeln!(file, "isrealmorrowindini=false").unwrap();
        writeln!(file, "[Game Files]").unwrap();
    }

    for filename in filenames {
        if game_settings.id() == GameId::Morrowind {
            write!(file, "GameFile0=").unwrap();
        } else if game_settings.load_order_method() == LoadOrderMethod::Asterisk {
            write!(file, "*").unwrap();
        }

        file.write_all(&strict_encode(filename.as_ref()).unwrap())
            .unwrap();
        writeln!(file, "").unwrap();
    }
}

pub fn set_timestamps<T: AsRef<str>>(plugins_directory: &Path, filenames: &[T]) {
    for (index, filename) in filenames.iter().enumerate() {
        set_file_timestamps(
            &plugins_directory.join(filename.as_ref()),
            u64::try_from(index).unwrap(),
        );
    }
}

pub fn set_file_timestamps(path: &Path, unix_seconds: u64) {
    let times = FileTimes::new()
        .set_accessed(SystemTime::UNIX_EPOCH)
        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(unix_seconds));
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(times)
        .unwrap();
}

pub fn game_settings_for_test(game_id: GameId, game_path: &Path) -> GameSettings {
    let local_path = game_path.join("local");
    create_dir_all(&local_path).unwrap();
    let my_games_path = game_path.join("my games");
    create_dir_all(&my_games_path).unwrap();

    GameSettings::with_local_and_my_games_paths(game_id, game_path, &local_path, my_games_path)
        .unwrap()
}

pub fn set_timestamp_order(plugin_names: &[&str], parent_path: &Path) {
    let mut timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1321009871);
    for plugin_name in plugin_names {
        let path = parent_path.join(plugin_name);
        File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(timestamp)
            .unwrap();
        timestamp += Duration::from_secs(60);
    }
}

pub fn mock_game_files(game_id: GameId, game_dir: &Path) -> (GameSettings, Vec<Plugin>) {
    let mut settings = game_settings_for_test(game_id, game_dir);

    if game_id == GameId::Starfield {
        copy_to_test_dir("Blank.full.esm", settings.master_file(), &settings);
        copy_to_test_dir("Blank.full.esm", "Blank.full.esm", &settings);
        copy_to_test_dir("Blank.medium.esm", "Blank.medium.esm", &settings);
        copy_to_test_dir("Blank.small.esm", "Blank.small.esm", &settings);
        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        copy_to_test_dir("Blank - Override.esp", "Blank - Override.esp", &settings);
    } else {
        copy_to_test_dir("Blank.esm", settings.master_file(), &settings);
        copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        copy_to_test_dir("Blank - Different.esp", "Blank - Different.esp", &settings);
        copy_to_test_dir(
            "Blank - Master Dependent.esp",
            "Blank - Master Dependent.esp",
            &settings,
        );
        copy_to_test_dir("Blank.esp", "Blàñk.esp", &settings);

        let plugin_names = [
            settings.master_file(),
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blàñk.esp",
        ];
        set_timestamp_order(&plugin_names, &settings.plugins_directory());
    }

    // Refresh settings to account for newly-created plugin files.
    settings.refresh_implicitly_active_plugins().unwrap();

    let mut plugins = vec![
        Plugin::new(settings.master_file(), &settings).unwrap(),
        Plugin::with_active("Blank.esp", &settings, true).unwrap(),
    ];

    if game_id != GameId::Starfield {
        plugins.push(Plugin::new("Blank - Different.esp", &settings).unwrap());
    }

    (settings, plugins)
}

pub fn to_owned(strs: Vec<&str>) -> Vec<String> {
    strs.into_iter().map(String::from).collect()
}

/// Set the master flag to be present or not for the given plugin.
/// Only valid for plugins for games other than Morrowind.
pub fn set_master_flag(plugin: &Path, present: bool) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).open(plugin)?;
    file.seek(io::SeekFrom::Start(8))?;

    let value = if present { 1 } else { 0 };
    file.write(&[value])?;

    Ok(())
}

pub fn load_and_insert<T: MutableLoadOrder>(load_order: &mut T, plugin_name: &str) {
    let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();

    match load_order.insert_position(&plugin) {
        Some(position) => {
            load_order.plugins_mut().insert(position, plugin);
        }
        None => {
            load_order.plugins_mut().push(plugin);
        }
    }
}

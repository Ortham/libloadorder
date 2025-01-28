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

use std::fs::{copy, create_dir_all, File, FileTimes};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::enums::GameId;
use crate::game_settings::GameSettings;

pub fn copy_to_test_dir(from_path: &str, to_file: &str, game_settings: &GameSettings) {
    let testing_plugins_dir = testing_plugins_dir(game_settings.id());
    let data_dir = game_settings.plugins_directory();
    if !data_dir.exists() {
        create_dir_all(&data_dir).unwrap();
    }
    copy(testing_plugins_dir.join(from_path), data_dir.join(to_file)).unwrap();
}

pub fn copy_to_dir(from_path: &str, to_dir: &Path, to_file: &str, game_id: GameId) {
    let testing_plugins_dir = testing_plugins_dir(game_id);
    if !to_dir.exists() {
        create_dir_all(to_dir).unwrap();
    }
    copy(testing_plugins_dir.join(from_path), to_dir.join(to_file)).unwrap();
}

fn testing_plugins_dir(game_id: GameId) -> PathBuf {
    use GameId::*;
    let game_folder = match game_id {
        Morrowind | OpenMW => "Morrowind",
        Oblivion => "Oblivion",
        Fallout4 | Fallout4VR | SkyrimSE | SkyrimVR => "SkyrimSE",
        Starfield => "Starfield",
        _ => "Skyrim",
    };

    let plugins_folder = match game_id {
        Morrowind | OpenMW => "Data Files",
        _ => "Data",
    };

    Path::new("testing-plugins")
        .join(game_folder)
        .join(plugins_folder)
}

pub fn create_file(path: &Path) {
    create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "").unwrap();
}

pub fn set_timestamps<T: AsRef<str>>(plugins_directory: &Path, filenames: &[T]) {
    for (index, filename) in filenames.iter().enumerate() {
        set_file_timestamps(
            &plugins_directory.join(filename.as_ref()),
            1321009871 + u64::try_from(index * 60).unwrap(),
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

/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2017 Oliver Hamlet
 *
 * libespm is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * libespm is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with libespm. If not, see <http://www.gnu.org/licenses/>.
 */

use std::fs::File;
use std::io::Write;
use std::path::Path;
use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use filetime::{FileTime, set_file_times};

use enums::GameId;
use game_settings::GameSettings;
use plugin::Plugin;
use tests::copy_to_test_dir;

pub fn write_load_order_file(game_settings: &GameSettings, filenames: &[&str]) {
    let mut file = File::create(&game_settings.load_order_file().unwrap()).unwrap();

    for filename in filenames {
        writeln!(file, "{}", filename).unwrap();
    }
}

pub fn write_active_plugins_file(game_settings: &GameSettings, filenames: &[&str]) {
    let mut file = File::create(&game_settings.active_plugins_file()).unwrap();

    if game_settings.id() == GameId::Morrowind {
        writeln!(file, "isrealmorrowindini=false").unwrap();
        writeln!(file, "[Game Files]").unwrap();
    }

    for filename in filenames {
        if game_settings.id() == GameId::Morrowind {
            write!(file, "GameFile0=").unwrap();
        }
        file.write_all(&WINDOWS_1252.encode(filename, EncoderTrap::Strict).unwrap())
            .unwrap();
        writeln!(file, "").unwrap();
    }
}

pub fn set_timestamps(plugins_directory: &Path, filenames: &[&str]) {
    for (index, filename) in filenames.iter().enumerate() {
        set_file_times(
            &plugins_directory.join(filename),
            FileTime::zero(),
            FileTime::from_seconds_since_1970(index as u64, 0),
        ).unwrap();
    }
}

pub fn mock_game_files(game_id: GameId, game_dir: &Path) -> (GameSettings, Vec<Plugin>) {
    use std::fs::create_dir;

    let local_path = game_dir.join("local");
    create_dir(&local_path).unwrap();
    let settings = GameSettings::with_local_path(game_id, game_dir, &local_path);

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

    write_active_plugins_file(&settings, &["Blank.esm\r", "#Blank.esp", "Blàñk.esp"]);

    set_timestamps(
        &settings.plugins_directory(),
        &[
            "Blank - Master Dependent.esp",
            "Blank.esm",
            "Blank - Different.esp",
            "Blank.esp",
            settings.master_file(),
        ],
    );
    // Give two files the same timestamp.
    set_file_times(
        &settings.plugins_directory().join("Blank.esp"),
        FileTime::zero(),
        FileTime::from_seconds_since_1970(2, 0),
    ).unwrap();

    let mut plugins = vec![
        Plugin::new(settings.master_file(), &settings).unwrap(),
        Plugin::new("Blank.esp", &settings).unwrap(),
        Plugin::new("Blank - Different.esp", &settings).unwrap(),
    ];

    // Activate a plugin that isn't going to be in the active plugins file.
    plugins[1].activate().unwrap();

    (settings, plugins)
}

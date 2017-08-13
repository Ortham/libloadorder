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

use std::path::Path;
use enums::GameId;
use game_settings::GameSettings;
use plugin::Plugin;
use tests::copy_to_test_dir;

fn write_active_plugins_file(path: &Path) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(&path).unwrap();
    writeln!(file, "Blank.esp").unwrap();
}

pub fn mock_game_files(game_id: GameId, game_dir: &Path) -> (GameSettings, Vec<Plugin>) {
    let settings = GameSettings::with_local_path(game_id, &game_dir, &game_dir);

    copy_to_test_dir("Blank.esm", settings.master_file(), &settings);
    copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
    copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
    copy_to_test_dir("Blank - Different.esp", "Blank - Different.esp", &settings);

    let mut plugins = vec![
        Plugin::new(settings.master_file(), &settings).unwrap(),
        Plugin::new("Blank.esp", &settings).unwrap(),
        Plugin::new("Blank - Different.esp", &settings).unwrap(),
    ];

    write_active_plugins_file(&game_dir.join("plugins.txt"));

    //TODO: Remove this once the load order is initialised from the filesystem.
    plugins[0].deactivate();
    plugins[2].deactivate();

    (settings, plugins)
}

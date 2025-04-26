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

use std::fmt::Display;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::path::Path;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::enums::GameId;
use crate::enums::LoadOrderMethod;
use crate::game_settings::GameSettings;
use crate::load_order::strict_encode;
use crate::plugin::Plugin;
use crate::tests::{copy_to_test_dir, set_timestamps, NON_ASCII};

use super::mutable::MutableLoadOrder;

pub(super) fn write_load_order_file<T: AsRef<str> + Display>(
    game_settings: &GameSettings,
    filenames: &[T],
) {
    let mut file = File::create(game_settings.load_order_file().unwrap()).unwrap();

    for filename in filenames {
        writeln!(file, "{filename}").unwrap();
    }
}

pub(super) fn write_active_plugins_file<T: AsRef<str>>(
    game_settings: &GameSettings,
    filenames: &[T],
) {
    let mut file = File::create(game_settings.active_plugins_file()).unwrap();

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
        writeln!(file).unwrap();
    }
}

pub(super) fn game_settings_for_test(game_id: GameId, game_path: &Path) -> GameSettings {
    let local_path = game_path.join("local");
    create_dir_all(&local_path).unwrap();
    let my_games_path = game_path.join("my games");
    create_dir_all(&my_games_path).unwrap();

    if game_id == GameId::OpenMW {
        GameSettings::with_local_path(GameId::OpenMW, game_path, &my_games_path).unwrap()
    } else {
        GameSettings::with_local_and_my_games_paths(game_id, game_path, &local_path, my_games_path)
            .unwrap()
    }
}

pub(super) fn mock_game_files(settings: &mut GameSettings) {
    if settings.id() == GameId::Starfield {
        copy_to_test_dir("Blank.full.esm", "Blank.full.esm", settings);
        copy_to_test_dir("Blank.medium.esm", "Blank.medium.esm", settings);
        copy_to_test_dir("Blank.small.esm", "Blank.small.esm", settings);
        copy_to_test_dir("Blank.esp", "Blank.esp", settings);
        copy_to_test_dir("Blank - Override.esp", "Blank - Override.esp", settings);
    } else {
        copy_to_test_dir("Blank.esm", "Blank.esm", settings);
        copy_to_test_dir("Blank.esp", "Blank.esp", settings);
        copy_to_test_dir("Blank - Different.esp", "Blank - Different.esp", settings);
        copy_to_test_dir(
            "Blank - Master Dependent.esp",
            "Blank - Master Dependent.esp",
            settings,
        );
        copy_to_test_dir("Blank.esp", NON_ASCII, settings);

        let plugin_names = [
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            NON_ASCII,
        ];
        set_timestamps(&settings.plugins_directory(), &plugin_names);
    }

    // Refresh settings to account for newly-created plugin files.
    settings.refresh_implicitly_active_plugins().unwrap();
}

pub(super) fn to_owned(strs: Vec<&str>) -> Vec<String> {
    strs.into_iter().map(String::from).collect()
}

/// Set the master flag to be present or not for the given plugin.
pub(super) fn set_master_flag(
    game_id: GameId,
    plugin_path: &Path,
    present: bool,
) -> io::Result<()> {
    set_flag(game_id, plugin_path, 0x1, present)
}

pub(super) fn set_blueprint_flag(
    game_id: GameId,
    plugin_path: &Path,
    present: bool,
) -> io::Result<()> {
    if game_id != GameId::Starfield {
        return Ok(());
    }

    set_flag(game_id, plugin_path, 0x800, present)
}

fn set_flag(game_id: GameId, plugin_path: &Path, flag: u32, present: bool) -> io::Result<()> {
    let flags_offset = match game_id {
        GameId::Morrowind | GameId::OpenMW => 12,
        _ => 8,
    };

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(plugin_path)?;

    let mut flags_bytes = [0, 0, 0, 0];
    file.seek(io::SeekFrom::Start(flags_offset))?;
    file.read_exact(&mut flags_bytes)?;

    let flags = u32::from_le_bytes(flags_bytes);

    let value = if present { flags | flag } else { flags ^ flag };
    flags_bytes = value.to_le_bytes();

    file.seek(io::SeekFrom::Start(flags_offset))?;
    file.write_all(&flags_bytes)?;

    Ok(())
}

fn insert<T: MutableLoadOrder>(load_order: &mut T, plugin: Plugin) {
    match load_order.insert_position(&plugin) {
        Some(position) => {
            load_order.plugins_mut().insert(position, plugin);
        }
        None => {
            load_order.plugins_mut().push(plugin);
        }
    }
}

pub(super) fn load_and_insert<T: MutableLoadOrder>(load_order: &mut T, plugin_name: &str) {
    let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();

    insert(load_order, plugin);
}

pub(super) fn prepend_master<T: MutableLoadOrder>(load_order: &mut T) {
    let source = match load_order.game_settings().id() {
        GameId::Starfield => "Blank.full.esm",
        _ => "Blank.esm",
    };

    copy_to_test_dir(source, source, load_order.game_settings());

    let plugin = Plugin::new(source, load_order.game_settings()).unwrap();
    load_order.plugins_mut().insert(0, plugin);
}

pub(super) fn prepend_early_loader<T: MutableLoadOrder>(load_order: &mut T) {
    let source = match load_order.game_settings().id() {
        GameId::Starfield => "Blank.full.esm",
        _ => "Blank.esm",
    };

    let target = match load_order.game_settings().id() {
        GameId::SkyrimSE => "Skyrim.esm",
        GameId::Starfield => "Starfield.esm",
        _ => return,
    };

    copy_to_test_dir(source, target, load_order.game_settings());

    let plugin = Plugin::new(target, load_order.game_settings()).unwrap();
    load_order.plugins_mut().insert(0, plugin);
}

pub(super) fn prepare_bulk_plugins<T, F>(
    load_order: &mut T,
    source_plugin_name: &str,
    plugin_count: usize,
    name_generator: F,
) -> Vec<String>
where
    T: MutableLoadOrder,
    F: Fn(usize) -> String,
{
    let names: Vec<_> = (0..plugin_count).map(name_generator).collect();

    let plugins: Vec<_> = names
        .par_iter()
        .map(|name| {
            copy_to_test_dir(source_plugin_name, name, load_order.game_settings());
            Plugin::new(name, load_order.game_settings()).unwrap()
        })
        .collect();

    for plugin in plugins {
        insert(load_order, plugin);
    }

    names
}

pub(super) fn prepare_bulk_full_plugins<T: MutableLoadOrder>(load_order: &mut T) -> Vec<String> {
    let plugin_name = if load_order.game_settings().id() == GameId::Starfield {
        "Blank.full.esm"
    } else {
        "Blank.esm"
    };
    prepare_bulk_plugins(load_order, plugin_name, 260, |i| {
        format!("Blank{i}.full.esm")
    })
}

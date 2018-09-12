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

use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;

use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use rayon::prelude::*;

use super::find_first_non_master_position;
use super::readable::ReadableLoadOrderExt;
use enums::Error;
use plugin::Plugin;

pub trait MutableLoadOrder: ReadableLoadOrderExt {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin>;

    fn deactivate_excess_plugins(&mut self) {
        for index in self.get_excess_active_plugin_indices() {
            self.plugins_mut()[index].deactivate();
        }
    }

    fn move_or_insert_plugin_with_index(
        &mut self,
        plugin_name: &str,
        position: usize,
    ) -> Result<(), Error> {
        if let Some(x) = self.index_of(plugin_name) {
            if x == position {
                return Ok(());
            }
        }

        let plugin = get_plugin_to_insert_at(self, plugin_name, position)?;

        if position >= self.plugins().len() {
            self.plugins_mut().push(plugin);
        } else {
            self.plugins_mut().insert(position, plugin);
        }

        Ok(())
    }

    fn deactivate_all(&mut self) {
        for plugin in self.plugins_mut() {
            plugin.deactivate();
        }
    }

    fn replace_plugins(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        if !are_plugin_names_unique(plugin_names) {
            return Err(Error::DuplicatePlugin);
        }

        let mut plugins = match self.map_to_plugins(plugin_names) {
            Err(x) => return Err(Error::InvalidPlugin(x.to_string())),
            Ok(x) => x,
        };

        if !is_partitioned_by_master_flag(&plugins)? {
            return Err(Error::NonMasterBeforeMaster);
        }

        mem::swap(&mut plugins, self.plugins_mut());

        Ok(())
    }
}

pub fn load_active_plugins<T, F>(load_order: &mut T, line_mapper: F) -> Result<(), Error>
where
    T: MutableLoadOrder,
    F: Fn(&str) -> Option<String> + Send + Sync,
{
    load_order.deactivate_all();

    let plugin_names = read_plugin_names(
        load_order.game_settings().active_plugins_file(),
        line_mapper,
    )?;

    let plugin_indices: Vec<usize> = plugin_names
        .par_iter()
        .filter_map(|p| load_order.index_of(p))
        .collect();

    for index in plugin_indices {
        load_order.plugins_mut()[index].activate()?;
    }

    Ok(())
}

pub fn read_plugin_names<F, T>(file_path: &Path, line_mapper: F) -> Result<Vec<T>, Error>
where
    F: Fn(&str) -> Option<T> + Send + Sync,
    T: Send,
{
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let mut content: Vec<u8> = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut content)?;

    let content = WINDOWS_1252
        .decode(&content, DecoderTrap::Strict)
        .map_err(Error::DecodeError)?;

    Ok(content.lines().filter_map(line_mapper).collect())
}

pub fn plugin_line_mapper(line: &str) -> Option<String> {
    if line.is_empty() || line.starts_with('#') {
        None
    } else {
        Some(line.to_owned())
    }
}

/// If an ESM has an ESP as a master, the ESP will be loaded directly before the
/// ESM instead of in its usual position. This function "hoists" such ESPs
/// further up the load order.
pub fn hoist_masters(plugins: &mut Vec<Plugin>) -> Result<(), Error> {
    // Store plugins' current positions and where they need to move to.
    // Use a BTreeMap so that if a plugin needs to move for more than one ESM,
    // it will move for the earlier one and so also satisfy the later one, and
    // so that it's possible to iterate over content in order.
    let mut from_to_map: BTreeMap<usize, usize> = BTreeMap::new();

    for (index, plugin) in plugins.iter().enumerate() {
        if !plugin.is_master_file() {
            break;
        }

        for master in plugin.masters()? {
            let pos = plugins
                .iter()
                .position(|p| p.name_matches(&master))
                .unwrap_or(0);
            if pos > index && !plugins[pos].is_master_file() {
                // Need to move the plugin to index, but can't do that while
                // iterating, so store it for later.
                from_to_map.insert(pos, index);
            }
        }
    }

    move_elements(plugins, from_to_map);

    Ok(())
}

fn move_elements<T>(vec: &mut Vec<T>, mut from_to_indices: BTreeMap<usize, usize>) {
    // Move elements around. Moving elements doesn't change from_index values,
    // as we're iterating from earliest index to latest, but to_index values can
    // become incorrect, e.g. (5, 2), (6, 3), (7, 1) will insert an element
    // before index 3 so that should become 4, but 1 is still correct.
    // Keeping track of what indices need offsets is probably not worth it as
    // this function is likely to be called with empty or very small maps, so
    // just loop through it after each move and increment any affected to_index
    // values.
    while !from_to_indices.is_empty() {
        // This is a bit gnarly, but it's just popping of the front element.
        let from_index = *from_to_indices
            .iter()
            .next()
            .expect("map should not be empty")
            .0;
        let to_index = from_to_indices
            .remove(&from_index)
            .expect("map key should exist");

        let element = vec.remove(from_index);
        vec.insert(to_index, element);

        for value in from_to_indices.values_mut() {
            if *value > to_index {
                *value += 1;
            }
        }
    }
}

fn get_plugin_to_insert_at<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    plugin_name: &str,
    insert_position: usize,
) -> Result<Plugin, Error> {
    if let Some(p) = load_order.index_of(plugin_name) {
        let is_master = load_order.plugins()[p].is_master_file();
        load_order.validate_index(insert_position, is_master)?;

        Ok(load_order.plugins_mut().remove(p))
    } else {
        let plugin = Plugin::new(plugin_name, load_order.game_settings())
            .map_err(|_| Error::InvalidPlugin(plugin_name.to_string()))?;

        load_order.validate_index(insert_position, plugin.is_master_file())?;

        Ok(plugin)
    }
}

fn are_plugin_names_unique(plugin_names: &[&str]) -> bool {
    let unique_plugin_names: HashSet<String> =
        plugin_names.par_iter().map(|s| s.to_lowercase()).collect();

    unique_plugin_names.len() == plugin_names.len()
}

fn is_partitioned_by_master_flag(plugins: &[Plugin]) -> Result<bool, Error> {
    let first_non_master_pos = match find_first_non_master_position(plugins) {
        None => return Ok(true),
        Some(x) => x,
    };

    let last_master_pos = match plugins.iter().rposition(|p| p.is_master_file()) {
        None => return Ok(true),
        Some(x) => x,
    };

    if first_non_master_pos > last_master_pos {
        return Ok(true);
    }

    let plugins = &plugins[first_non_master_pos..=last_master_pos];

    let mut plugin_names: HashSet<String> = HashSet::new();

    // Add each plugin that isn't a master file to the hashset.
    // When a master file is encountered, remove its masters from the hashset.
    // If there are any plugins left in the hashset, they weren't hoisted there,
    // so fail the check.
    for plugin in plugins {
        if !plugin.is_master_file() {
            plugin_names.insert(plugin.name().to_lowercase());
        } else {
            for master in plugin.masters()? {
                plugin_names.remove(&master.to_lowercase());
            }

            if !plugin_names.is_empty() {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    use enums::GameId;
    use game_settings::GameSettings;
    use load_order::tests::*;
    use tests::copy_to_test_dir;

    use tempfile::tempdir;

    fn prepare(game_path: &Path) -> GameSettings {
        let settings = game_settings_for_test(GameId::SkyrimSE, game_path);

        copy_to_test_dir("Blank.esm", settings.master_file(), &settings);
        copy_to_test_dir("Blank.esm", "Blank.esm", &settings);
        copy_to_test_dir("Blank.esp", "Blank.esp", &settings);
        copy_to_test_dir("Blank - Different.esp", "Blank - Different.esp", &settings);
        copy_to_test_dir(
            "Blank - Plugin Dependent.esp",
            "Blank - Plugin Dependent.esm",
            &settings,
        );

        settings
    }

    #[test]
    fn move_elements_should_correct_later_indices_to_account_for_earlier_moves() {
        let mut vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let mut from_to_indices = BTreeMap::new();
        from_to_indices.insert(6, 3);
        from_to_indices.insert(5, 2);
        from_to_indices.insert(7, 1);

        move_elements(&mut vec, from_to_indices);

        assert_eq!(vec![0, 7, 1, 5, 2, 6, 3, 4, 8], vec);
    }

    #[test]
    fn is_partitioned_by_master_flag_should_be_true_if_there_are_only_master_files() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(&tmp_dir.path());

        let plugins = vec![
            Plugin::new(settings.master_file(), &settings).unwrap(),
            Plugin::new("Blank.esm", &settings).unwrap(),
        ];

        assert!(is_partitioned_by_master_flag(&plugins).unwrap());
    }

    #[test]
    fn is_partitioned_by_master_flag_should_be_true_if_there_are_no_master_files() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(&tmp_dir.path());

        let plugins = vec![
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new("Blank - Different.esp", &settings).unwrap(),
        ];

        assert!(is_partitioned_by_master_flag(&plugins).unwrap());
    }

    #[test]
    fn is_partitioned_by_master_flag_should_be_true_if_master_files_are_before_all_others() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(&tmp_dir.path());

        let plugins = vec![
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
        ];

        assert!(is_partitioned_by_master_flag(&plugins).unwrap());
    }

    #[test]
    fn is_partitioned_by_master_flag_should_be_true_if_hoisted_non_masters_load_before_masters() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(&tmp_dir.path());

        let plugins = vec![
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new("Blank - Plugin Dependent.esm", &settings).unwrap(),
        ];

        assert!(is_partitioned_by_master_flag(&plugins).unwrap());
    }

    #[test]
    fn is_partitioned_by_master_flag_should_be_false_if_non_masters_are_hoisted_earlier_than_needed(
) {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(&tmp_dir.path());

        let plugins = vec![
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank - Plugin Dependent.esm", &settings).unwrap(),
        ];

        assert!(!is_partitioned_by_master_flag(&plugins).unwrap());
    }
}

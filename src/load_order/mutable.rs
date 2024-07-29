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

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::read_dir;
use std::mem;
use std::path::{Path, PathBuf};

use encoding_rs::WINDOWS_1252;
use rayon::prelude::*;
use unicase::{eq, UniCase};

use super::readable::{ReadableLoadOrder, ReadableLoadOrderBase};
use crate::enums::Error;
use crate::game_settings::GameSettings;
use crate::plugin::{has_plugin_extension, trim_dot_ghost, Plugin};
use crate::GameId;

pub trait MutableLoadOrder: ReadableLoadOrder + ReadableLoadOrderBase + Sync {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin>;

    fn insert_position(&self, plugin: &Plugin) -> Option<usize> {
        if self.plugins().is_empty() {
            return None;
        }

        // A blueprint plugin may be listed as an early loader (e.g. in a CCC
        // file) but it still loads as a normal blueprint plugin.
        if !plugin.is_blueprint_master() {
            let mut loaded_plugin_count = 0;
            for plugin_name in self.game_settings().early_loading_plugins() {
                if eq(plugin.name(), plugin_name) {
                    return Some(loaded_plugin_count);
                }

                if self.index_of(plugin_name).is_some() {
                    loaded_plugin_count += 1;
                }
            }
        }

        generic_insert_position(self.plugins(), plugin)
    }

    fn find_plugins(&self) -> Vec<String> {
        // A game might store some plugins outside of its main plugins directory
        // so look for those plugins. They override any of the same names that
        // appear in the main plugins directory, so check for the additional
        // paths first.
        let mut directories = self
            .game_settings()
            .additional_plugins_directories()
            .to_vec();
        directories.push(self.game_settings().plugins_directory());

        find_plugins_in_dirs(&directories, self.game_settings().id())
    }

    fn validate_index(&self, plugin: &Plugin, index: usize) -> Result<(), Error> {
        if plugin.is_blueprint_master() {
            // Blueprint plugins load after all non-blueprint plugins of the
            // same scale, even non-masters.
            validate_blueprint_plugin_index(self.plugins(), plugin, index)
        } else {
            self.validate_early_loading_plugin_indexes(plugin.name(), index)?;

            if plugin.is_master_file() {
                validate_master_file_index(self.plugins(), plugin, index)
            } else {
                validate_non_master_file_index(self.plugins(), plugin, index)
            }
        }
    }

    fn lookup_plugins(&mut self, active_plugin_names: &[&str]) -> Result<Vec<usize>, Error> {
        active_plugin_names
            .par_iter()
            .map(|n| {
                self.plugins()
                    .par_iter()
                    .position_any(|p| p.name_matches(n))
                    .ok_or_else(|| Error::PluginNotFound(n.to_string()))
            })
            .collect()
    }

    fn set_plugin_index(&mut self, plugin_name: &str, position: usize) -> Result<usize, Error> {
        if let Some(x) = self.index_of(plugin_name) {
            if x == position {
                return Ok(position);
            }
        }

        let plugin = get_plugin_to_insert_at(self, plugin_name, position)?;

        if position >= self.plugins().len() {
            self.plugins_mut().push(plugin);
            Ok(self.plugins().len() - 1)
        } else {
            self.plugins_mut().insert(position, plugin);
            Ok(position)
        }
    }

    fn deactivate_all(&mut self) {
        for plugin in self.plugins_mut() {
            plugin.deactivate();
        }
    }

    fn replace_plugins(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        let mut unique_plugin_names = HashSet::new();

        let non_unique_plugin = plugin_names
            .iter()
            .find(|n| !unique_plugin_names.insert(UniCase::new(*n)));

        if let Some(n) = non_unique_plugin {
            return Err(Error::DuplicatePlugin(n.to_string()));
        }

        let mut plugins = map_to_plugins(self, plugin_names)?;

        validate_load_order(&plugins, self.game_settings().early_loading_plugins())?;

        mem::swap(&mut plugins, self.plugins_mut());

        Ok(())
    }

    fn load_unique_plugins(
        &mut self,
        plugin_name_tuples: Vec<(String, bool)>,
        installed_filenames: Vec<String>,
    ) {
        let plugins: Vec<_> = remove_duplicates_icase(plugin_name_tuples, installed_filenames)
            .into_par_iter()
            .filter_map(|(filename, active)| {
                Plugin::with_active(&filename, self.game_settings(), active).ok()
            })
            .collect();

        for plugin in plugins {
            insert(self, plugin);
        }
    }

    fn add_implicitly_active_plugins(&mut self) -> Result<(), Error> {
        let plugin_names = self.game_settings().implicitly_active_plugins().to_vec();

        for plugin_name in plugin_names {
            activate_unvalidated(self, &plugin_name)?;
        }

        Ok(())
    }

    /// Check that the given plugin and index won't cause any early-loading
    /// plugins to load in the wrong positions.
    fn validate_early_loading_plugin_indexes(
        &self,
        plugin_name: &str,
        position: usize,
    ) -> Result<(), Error> {
        let mut next_index = 0;
        for early_loader in self.game_settings().early_loading_plugins() {
            let names_match = eq(plugin_name, early_loader);

            let early_loader_tuple = self
                .plugins()
                .iter()
                .enumerate()
                .find(|(_, p)| p.name_matches(early_loader));

            let expected_index = match early_loader_tuple {
                Some((i, early_loading_plugin)) => {
                    // If the early loader is a blueprint plugin then it doesn't
                    // actually load early and so the index of the next early
                    // loader is unchanged.
                    if !early_loading_plugin.is_blueprint_master() {
                        next_index = i + 1;
                    }

                    if !names_match && position == i {
                        return Err(Error::InvalidEarlyLoadingPluginPosition {
                            name: early_loader.to_string(),
                            pos: i + 1,
                            expected_pos: i,
                        });
                    }

                    i
                }
                None => next_index,
            };

            if names_match && position != expected_index {
                return Err(Error::InvalidEarlyLoadingPluginPosition {
                    name: plugin_name.to_string(),
                    pos: position,
                    expected_pos: expected_index,
                });
            }
        }

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

    let plugin_indices: Vec<_> = plugin_names
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
    F: FnMut(&str) -> Option<T> + Send + Sync,
    T: Send,
{
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read(file_path).map_err(|e| Error::IoError(file_path.to_path_buf(), e))?;

    // This should never fail, as although Windows-1252 has a few unused bytes
    // they get mapped to C1 control characters.
    let decoded_content = WINDOWS_1252
        .decode_without_bom_handling_and_without_replacement(&content)
        .ok_or_else(|| Error::DecodeError(content.clone()))?;

    Ok(decoded_content.lines().filter_map(line_mapper).collect())
}

pub fn plugin_line_mapper(line: &str) -> Option<String> {
    if line.is_empty() || line.starts_with('#') {
        None
    } else {
        Some(line.to_owned())
    }
}

/// If an ESM has a master that is lower down in the load order, the master will
/// be loaded directly before the ESM instead of in its usual position. This
/// function "hoists" such masters further up the load order to match that
/// behaviour.
pub fn hoist_masters(plugins: &mut Vec<Plugin>) -> Result<(), Error> {
    // Store plugins' current positions and where they need to move to.
    // Use a BTreeMap so that if a plugin needs to move for more than one ESM,
    // it will move for the earlier one and so also satisfy the later one, and
    // so that it's possible to iterate over content in order.
    let mut from_to_map: BTreeMap<usize, usize> = BTreeMap::new();

    for (index, plugin) in plugins.iter().enumerate() {
        if !plugin.is_master_file() {
            continue;
        }

        for master in plugin.masters()? {
            let pos = plugins
                .iter()
                .position(|p| {
                    p.name_matches(&master)
                        && (plugin.is_blueprint_master() || !p.is_blueprint_master())
                })
                .unwrap_or(0);
            if pos > index {
                // Need to move the plugin to index, but can't do that while
                // iterating, so store it for later.
                from_to_map.entry(pos).or_insert(index);
            }
        }
    }

    move_elements(plugins, from_to_map);

    Ok(())
}

fn validate_early_loader_positions(
    plugins: &[Plugin],
    early_loading_plugins: &[String],
) -> Result<(), Error> {
    // Check that all early loading plugins that are present load in
    // their hardcoded order.
    let mut missing_plugins_count = 0;
    for (i, plugin_name) in early_loading_plugins.iter().enumerate() {
        match plugins.iter().position(|p| eq(p.name(), plugin_name)) {
            Some(pos) => {
                let expected_pos = i - missing_plugins_count;
                if pos != expected_pos {
                    return Err(Error::InvalidEarlyLoadingPluginPosition {
                        name: plugin_name.clone(),
                        pos,
                        expected_pos,
                    });
                }
            }
            None => missing_plugins_count += 1,
        }
    }

    Ok(())
}

fn generic_insert_position(plugins: &[Plugin], plugin: &Plugin) -> Option<usize> {
    let is_master_of = |p: &Plugin| {
        p.masters()
            .map(|masters| masters.iter().any(|m| plugin.name_matches(m)))
            .unwrap_or(false)
    };

    if plugin.is_blueprint_master() {
        // Blueprint plugins load after all other plugins unless they are
        // hoisted by another blueprint plugin.
        return plugins
            .iter()
            .position(|p| p.is_blueprint_master() && is_master_of(p));
    }

    // Check that there isn't a master that would hoist this plugin.
    let hoisted_index = plugins
        .iter()
        .position(|p| p.is_master_file() && is_master_of(p));

    hoisted_index.or_else(|| {
        if plugin.is_master_file() {
            find_first_non_master_position(plugins)
        } else {
            None
        }
    })
}

fn find_plugins_in_dirs(directories: &[PathBuf], game: GameId) -> Vec<String> {
    let mut dir_entries: Vec<_> = directories
        .iter()
        .flat_map(read_dir)
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|f| f.is_file()).unwrap_or(false))
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|f| has_plugin_extension(f, game))
                .unwrap_or(false)
        })
        .collect();

    // Sort by file modification timestamps, in ascending order. If two timestamps are equal, sort
    // by filenames (in ascending order for Starfield, descending otherwise).
    dir_entries.sort_unstable_by(|e1, e2| {
        let m1 = e1.metadata().and_then(|m| m.modified()).ok();
        let m2 = e2.metadata().and_then(|m| m.modified()).ok();

        match m1.cmp(&m2) {
            Ordering::Equal if game == GameId::Starfield => e1.file_name().cmp(&e2.file_name()),
            Ordering::Equal => e1.file_name().cmp(&e2.file_name()).reverse(),
            x => x,
        }
    });

    let mut set = HashSet::new();

    dir_entries
        .into_iter()
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|filename| set.insert(UniCase::new(trim_dot_ghost(filename).to_string())))
        .collect()
}

fn to_plugin(
    plugin_name: &str,
    existing_plugins: &[Plugin],
    game_settings: &GameSettings,
) -> Result<Plugin, Error> {
    existing_plugins
        .par_iter()
        .find_any(|p| p.name_matches(plugin_name))
        .map_or_else(
            || Plugin::new(plugin_name, game_settings),
            |p| Ok(p.clone()),
        )
}

fn validate_blueprint_plugin_index(
    plugins: &[Plugin],
    plugin: &Plugin,
    index: usize,
) -> Result<(), Error> {
    // Blueprint plugins should only appear before other blueprint plugins, as
    // they get moved after all non-blueprint plugins before conflicts are
    // resolved and don't get hoisted by non-blueprint plugins. However, they
    // do get hoisted by other blueprint plugins.
    let preceding_plugins = if index < plugins.len() {
        &plugins[..index]
    } else {
        plugins
    };

    // Check that none of the preceding blueprint plugins have this plugin as a
    // master.
    for preceding_plugin in preceding_plugins {
        if !preceding_plugin.is_blueprint_master() {
            continue;
        }

        let preceding_masters = preceding_plugin.masters()?;
        if preceding_masters
            .iter()
            .any(|m| eq(m.as_str(), plugin.name()))
        {
            return Err(Error::UnrepresentedHoist {
                plugin: plugin.name().to_string(),
                master: preceding_plugin.name().to_string(),
            });
        }
    }

    Ok(())
}

fn validate_master_file_index(
    plugins: &[Plugin],
    plugin: &Plugin,
    index: usize,
) -> Result<(), Error> {
    let preceding_plugins = if index < plugins.len() {
        &plugins[..index]
    } else {
        plugins
    };

    // Check that none of the preceding plugins have this plugin as a master.
    for preceding_plugin in preceding_plugins {
        let preceding_masters = preceding_plugin.masters()?;
        if preceding_masters
            .iter()
            .any(|m| eq(m.as_str(), plugin.name()))
        {
            return Err(Error::UnrepresentedHoist {
                plugin: plugin.name().to_string(),
                master: preceding_plugin.name().to_string(),
            });
        }
    }

    let previous_master_pos = preceding_plugins
        .iter()
        .rposition(|p| p.is_master_file())
        .unwrap_or(0);

    let masters = plugin.masters()?;
    let master_names: HashSet<_> = masters.iter().map(|m| UniCase::new(m.as_str())).collect();

    // Check that all of the plugins that load between this index and
    // the previous plugin are masters of this plugin.
    if let Some(n) = preceding_plugins
        .iter()
        .skip(previous_master_pos + 1)
        .find(|p| !master_names.contains(&UniCase::new(p.name())))
    {
        return Err(Error::NonMasterBeforeMaster {
            master: plugin.name().to_string(),
            non_master: n.name().to_string(),
        });
    }

    // Check that none of the plugins that load after index are
    // masters of this plugin.
    if let Some(p) = plugins
        .iter()
        .skip(index)
        .find(|p| master_names.contains(&UniCase::new(p.name())))
    {
        Err(Error::UnrepresentedHoist {
            plugin: p.name().to_string(),
            master: plugin.name().to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_non_master_file_index(
    plugins: &[Plugin],
    plugin: &Plugin,
    index: usize,
) -> Result<(), Error> {
    // Check that there aren't any earlier master files that have this
    // plugin as a master.
    for master_file in plugins.iter().take(index).filter(|p| p.is_master_file()) {
        if master_file
            .masters()?
            .iter()
            .any(|m| plugin.name_matches(m))
        {
            return Err(Error::UnrepresentedHoist {
                plugin: plugin.name().to_string(),
                master: master_file.name().to_string(),
            });
        }
    }

    // Check that the next master file has this plugin as a master.
    let next_master = match plugins.iter().skip(index).find(|p| p.is_master_file()) {
        None => return Ok(()),
        Some(p) => p,
    };

    if next_master
        .masters()?
        .iter()
        .any(|m| plugin.name_matches(m))
    {
        Ok(())
    } else {
        Err(Error::NonMasterBeforeMaster {
            master: next_master.name().to_string(),
            non_master: plugin.name().to_string(),
        })
    }
}

fn map_to_plugins<T: ReadableLoadOrderBase + Sync + ?Sized>(
    load_order: &T,
    plugin_names: &[&str],
) -> Result<Vec<Plugin>, Error> {
    plugin_names
        .par_iter()
        .map(|n| to_plugin(n, load_order.plugins(), load_order.game_settings_base()))
        .collect()
}

fn insert<T: MutableLoadOrder + ?Sized>(load_order: &mut T, plugin: Plugin) -> usize {
    match load_order.insert_position(&plugin) {
        Some(position) => {
            load_order.plugins_mut().insert(position, plugin);
            position
        }
        None => {
            load_order.plugins_mut().push(plugin);
            load_order.plugins().len() - 1
        }
    }
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
    while let Some((from_index, to_index)) = from_to_indices.pop_first() {
        let element = vec.remove(from_index);
        vec.insert(to_index, element);

        for value in from_to_indices.values_mut() {
            if *value < from_index && *value > to_index {
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
        let plugin = &load_order.plugins()[p];
        load_order.validate_index(plugin, insert_position)?;

        Ok(load_order.plugins_mut().remove(p))
    } else {
        let plugin = Plugin::new(plugin_name, load_order.game_settings())?;

        load_order.validate_index(&plugin, insert_position)?;

        Ok(plugin)
    }
}

fn validate_load_order(plugins: &[Plugin], early_loading_plugins: &[String]) -> Result<(), Error> {
    validate_early_loader_positions(plugins, early_loading_plugins)?;

    validate_no_unhoisted_non_masters_before_masters(plugins)?;

    validate_plugins_load_before_their_masters(plugins)?;

    Ok(())
}

fn validate_no_unhoisted_non_masters_before_masters(plugins: &[Plugin]) -> Result<(), Error> {
    let first_non_master_pos = match find_first_non_master_position(plugins) {
        None => plugins.len(),
        Some(x) => x,
    };

    // Ignore blueprint plugins because they load after non-masters.
    let last_master_pos = match plugins
        .iter()
        .rposition(|p| p.is_master_file() && !p.is_blueprint_master())
    {
        None => return Ok(()),
        Some(x) => x,
    };

    let mut plugin_names: HashSet<_> = HashSet::new();

    // Add each plugin that isn't a master file to the hashset.
    // When a master file is encountered, remove its masters from the hashset.
    // If there are any plugins left in the hashset, they weren't hoisted there,
    // so fail the check.
    if first_non_master_pos < last_master_pos {
        for plugin in plugins
            .iter()
            .skip(first_non_master_pos)
            .take(last_master_pos - first_non_master_pos + 1)
        {
            if !plugin.is_master_file() {
                plugin_names.insert(UniCase::new(plugin.name().to_string()));
            } else {
                for master in plugin.masters()? {
                    plugin_names.remove(&UniCase::new(master.clone()));
                }

                if let Some(n) = plugin_names.iter().next() {
                    return Err(Error::NonMasterBeforeMaster {
                        master: plugin.name().to_string(),
                        non_master: n.to_string(),
                    });
                }
            }
        }
    }

    Ok(())
}

fn validate_plugins_load_before_their_masters(plugins: &[Plugin]) -> Result<(), Error> {
    let mut plugins_map: HashMap<UniCase<String>, &Plugin> = HashMap::new();

    for plugin in plugins.iter().rev() {
        if plugin.is_master_file() {
            if let Some(m) = plugin
                .masters()?
                .iter()
                .find_map(|m| plugins_map.get(&UniCase::new(m.to_string())))
            {
                // Don't error if a non-blueprint plugin depends on a blueprint plugin.
                if plugin.is_blueprint_master() || !m.is_blueprint_master() {
                    return Err(Error::UnrepresentedHoist {
                        plugin: m.name().to_string(),
                        master: plugin.name().to_string(),
                    });
                }
            }
        }

        plugins_map.insert(UniCase::new(plugin.name().to_string()), plugin);
    }

    Ok(())
}

fn remove_duplicates_icase(
    plugin_tuples: Vec<(String, bool)>,
    filenames: Vec<String>,
) -> Vec<(String, bool)> {
    let mut set: HashSet<_> = HashSet::with_capacity(filenames.len());

    let mut unique_tuples: Vec<(String, bool)> = plugin_tuples
        .into_iter()
        .rev()
        .filter(|(string, _)| set.insert(UniCase::new(trim_dot_ghost(string).to_string())))
        .collect();

    unique_tuples.reverse();

    let unique_file_tuples_iter = filenames
        .into_iter()
        .filter(|string| set.insert(UniCase::new(trim_dot_ghost(string).to_string())))
        .map(|f| (f, false));

    unique_tuples.extend(unique_file_tuples_iter);

    unique_tuples
}

fn activate_unvalidated<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    filename: &str,
) -> Result<(), Error> {
    if let Some(plugin) = load_order
        .plugins_mut()
        .iter_mut()
        .find(|p| p.name_matches(filename))
    {
        plugin.activate()
    } else {
        // Ignore any errors trying to load the plugin to save checking if it's
        // valid and then loading it if it is.
        Plugin::with_active(filename, load_order.game_settings(), true)
            .map(|plugin| {
                insert(load_order, plugin);
            })
            .or(Ok(()))
    }
}

fn find_first_non_master_position(plugins: &[Plugin]) -> Option<usize> {
    plugins.iter().position(|p| !p.is_master_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::GameId;
    use crate::game_settings::GameSettings;
    use crate::load_order::tests::*;
    use crate::load_order::writable::create_parent_dirs;
    use crate::tests::copy_to_test_dir;

    use tempfile::tempdir;

    struct TestLoadOrder {
        game_settings: GameSettings,
        plugins: Vec<Plugin>,
    }

    impl ReadableLoadOrderBase for TestLoadOrder {
        fn game_settings_base(&self) -> &GameSettings {
            &self.game_settings
        }

        fn plugins(&self) -> &[Plugin] {
            &self.plugins
        }
    }

    impl MutableLoadOrder for TestLoadOrder {
        fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
            &mut self.plugins
        }
    }

    fn prepare(game_id: GameId, game_path: &Path) -> TestLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, game_path);

        TestLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn prepare_hoisted(game_id: GameId, game_path: &Path) -> TestLoadOrder {
        let load_order = prepare(game_id, game_path);

        let plugins_dir = &load_order.game_settings().plugins_directory();
        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm",
            load_order.game_settings(),
        );
        set_master_flag(game_id, &plugins_dir.join("Blank - Different.esm"), false).unwrap();
        copy_to_test_dir(
            "Blank - Different Master Dependent.esm",
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        );

        load_order
    }

    fn prepare_plugins(game_path: &Path, blank_esp_source: &str) -> Vec<Plugin> {
        let settings = game_settings_for_test(GameId::SkyrimSE, game_path);

        copy_to_test_dir("Blank.esm", settings.master_file(), &settings);
        copy_to_test_dir(blank_esp_source, "Blank.esp", &settings);

        vec![
            Plugin::new(settings.master_file(), &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
        ]
    }

    #[test]
    fn insert_position_should_return_zero_if_given_the_game_master_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let plugin = Plugin::new("Skyrim.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(0, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_none_for_the_game_master_if_no_plugins_are_loaded() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        load_order.plugins_mut().clear();

        let plugin = Plugin::new("Skyrim.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert!(position.is_none());
    }

    #[test]
    fn insert_position_should_return_the_hardcoded_index_of_an_early_loading_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        load_order.plugins_mut().insert(1, plugin);

        copy_to_test_dir("Blank.esm", "HearthFires.esm", &load_order.game_settings());
        let plugin = Plugin::new("HearthFires.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_not_treat_all_implicitly_active_plugins_as_early_loading_plugins() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Skyrim.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esm").unwrap();

        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir(
            "Blank.esm",
            "Blank - Different.esm",
            &load_order.game_settings(),
        );
        let plugin = Plugin::new("Blank - Different.esm", &load_order.game_settings()).unwrap();
        load_order.plugins_mut().insert(1, plugin);

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(2, position.unwrap());
    }

    #[test]
    fn insert_position_should_not_count_installed_unloaded_early_loading_plugins() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());
        copy_to_test_dir("Blank.esm", "HearthFires.esm", &load_order.game_settings());
        let plugin = Plugin::new("HearthFires.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_not_put_blueprint_plugins_before_non_blueprint_dependents() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(
            dependent_plugin,
            dependent_plugin,
            &load_order.game_settings(),
        );

        let plugin = Plugin::new(dependent_plugin, &load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugins_dir = load_order.game_settings().plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let plugin = Plugin::new(plugin_name, &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert!(position.is_none());
    }

    #[test]
    fn insert_position_should_put_blueprint_plugins_before_blueprint_dependents() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(
            dependent_plugin,
            dependent_plugin,
            &load_order.game_settings(),
        );
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(dependent_plugin), true).unwrap();

        let plugin = Plugin::new(dependent_plugin, &load_order.game_settings()).unwrap();
        load_order.plugins.push(plugin);

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let plugin = Plugin::new(plugin_name, &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(2, position.unwrap());
    }

    #[test]
    fn insert_position_should_not_treat_early_loading_blueprint_plugins_as_early_loading() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        std::fs::write(
            plugins_dir.parent().unwrap().join("Starfield.ccc"),
            plugin_name,
        )
        .unwrap();
        load_order
            .game_settings
            .refresh_implicitly_active_plugins()
            .unwrap();

        let plugin = Plugin::new(plugin_name, &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert!(position.is_none());
    }

    #[test]
    fn insert_position_should_return_none_if_given_a_non_master_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugin =
            Plugin::new("Blank - Master Dependent.esp", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn insert_position_should_return_the_first_non_master_plugin_index_if_given_a_master_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_none_if_no_non_masters_are_present() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        // Remove non-master plugins from the load order.
        load_order.plugins_mut().retain(|p| p.is_master_file());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn insert_position_should_return_the_first_non_master_index_if_given_a_light_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Blank.esl", load_order.game_settings());
        let plugin = Plugin::new("Blank.esl", &load_order.game_settings()).unwrap();

        load_order.plugins_mut().insert(1, plugin);

        let position = load_order.insert_position(&load_order.plugins()[1]);

        assert_eq!(2, position.unwrap());

        copy_to_test_dir(
            "Blank.esp",
            "Blank - Different.esl",
            load_order.game_settings(),
        );
        let plugin = Plugin::new("Blank - Different.esl", &load_order.game_settings()).unwrap();

        let position = load_order.insert_position(&plugin);

        assert_eq!(2, position.unwrap());
    }

    #[test]
    fn insert_position_should_succeed_for_a_non_master_hoisted_after_another_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let plugin = Plugin::new(
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        )
        .unwrap();
        load_order.plugins.insert(1, plugin);

        let other_non_master = "Blank.esm";
        set_master_flag(GameId::Oblivion, &plugins_dir.join(other_non_master), false).unwrap();
        let plugin = Plugin::new(other_non_master, load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let other_master = "Blank - Master Dependent.esm";
        copy_to_test_dir(other_master, other_master, load_order.game_settings());
        let plugin = Plugin::new(other_master, load_order.game_settings()).unwrap();
        load_order.plugins.insert(2, plugin);

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();

        let position = load_order.insert_position(&plugin);

        assert_eq!(3, position.unwrap());
    }

    #[test]
    fn validate_index_should_succeed_for_a_master_plugin_and_index_directly_after_a_master() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 1).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_for_a_master_plugin_and_index_after_a_hoisted_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new(
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        )
        .unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_ok());
    }

    #[test]
    fn validate_index_should_error_for_a_master_plugin_and_index_after_unrelated_non_masters() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new("Blank.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 4).is_err());
    }

    #[test]
    fn validate_index_should_error_for_a_master_plugin_that_has_a_later_non_master_as_a_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        load_order.plugins.insert(2, plugin);

        let plugin = Plugin::new(
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        )
        .unwrap();
        assert!(load_order.validate_index(&plugin, 1).is_err());
    }

    #[test]
    fn validate_index_should_error_for_a_master_plugin_that_has_a_later_master_as_a_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        copy_to_test_dir(
            "Blank - Master Dependent.esm",
            "Blank - Master Dependent.esm",
            load_order.game_settings(),
        );
        copy_to_test_dir("Blank.esm", "Blank.esm", load_order.game_settings());

        let plugin = Plugin::new("Blank.esm", load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin =
            Plugin::new("Blank - Master Dependent.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 1).is_err());
    }

    #[test]
    fn validate_index_should_error_for_a_master_plugin_that_is_a_master_of_an_earlier_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        copy_to_test_dir(
            "Blank - Master Dependent.esm",
            "Blank - Master Dependent.esm",
            load_order.game_settings(),
        );
        copy_to_test_dir("Blank.esm", "Blank.esm", load_order.game_settings());

        let plugin =
            Plugin::new("Blank - Master Dependent.esm", load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new("Blank.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_err());
    }

    #[test]
    fn validate_index_should_succeed_for_a_non_master_plugin_and_an_index_with_no_later_masters() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin =
            Plugin::new("Blank - Master Dependent.esp", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_for_a_non_master_plugin_that_is_a_master_of_the_next_master_file(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new(
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        )
        .unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 1).is_ok());
    }

    #[test]
    fn validate_index_should_error_for_a_non_master_plugin_that_is_not_a_master_of_the_next_master_file(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin =
            Plugin::new("Blank - Master Dependent.esp", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 0).is_err());
    }

    #[test]
    fn validate_index_should_error_for_a_non_master_plugin_and_an_index_not_before_a_master_that_depends_on_it(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::SkyrimSE, &tmp_dir.path());

        let plugin = Plugin::new(
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        )
        .unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_err());
    }

    #[test]
    fn validate_index_should_succeed_for_a_blueprint_plugin_index_that_is_last() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_for_a_blueprint_plugin_index_that_is_only_followed_by_other_blueprint_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let other_plugin_name = "Blank.medium.esm";
        set_blueprint_flag(
            GameId::Starfield,
            &plugins_dir.join(other_plugin_name),
            true,
        )
        .unwrap();

        let other_plugin = Plugin::new(other_plugin_name, load_order.game_settings()).unwrap();
        load_order.plugins.push(other_plugin);

        let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_ok());
    }

    #[test]
    fn validate_index_should_fail_for_a_blueprint_plugin_index_that_is_after_a_dependent_blueprint_plugin_index(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(
            dependent_plugin,
            dependent_plugin,
            load_order.game_settings(),
        );
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(dependent_plugin), true).unwrap();
        let plugin = Plugin::new(dependent_plugin, load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();

        let index = 3;
        match load_order.validate_index(&plugin, index).unwrap_err() {
            Error::UnrepresentedHoist { plugin, master } => {
                assert_eq!(plugin_name, plugin);
                assert_eq!(dependent_plugin, master);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn validate_index_should_succeed_for_a_blueprint_plugin_index_that_is_after_a_dependent_non_blueprint_plugin_index(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(
            dependent_plugin,
            dependent_plugin,
            load_order.game_settings(),
        );
        let plugin = Plugin::new(dependent_plugin, load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();

        assert!(load_order.validate_index(&plugin, 3).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_when_an_early_loader_is_a_blueprint_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        std::fs::write(
            plugins_dir.parent().unwrap().join("Starfield.ccc"),
            format!("Starfield.esm\n{}", plugin_name),
        )
        .unwrap();
        load_order
            .game_settings
            .refresh_implicitly_active_plugins()
            .unwrap();

        let plugin = Plugin::new(plugin_name, load_order.game_settings()).unwrap();
        load_order.plugins.push(plugin);

        let plugin = Plugin::new("Blank.medium.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 1).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_for_an_early_loader_listed_after_a_blueprint_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = load_order.game_settings().plugins_directory();

        let blueprint_plugin = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(blueprint_plugin), true).unwrap();

        let early_loader = "Blank.medium.esm";

        std::fs::write(
            plugins_dir.parent().unwrap().join("Starfield.ccc"),
            format!("Starfield.esm\n{}\n{}", blueprint_plugin, early_loader),
        )
        .unwrap();
        load_order
            .game_settings
            .refresh_implicitly_active_plugins()
            .unwrap();

        let plugin = Plugin::new(blueprint_plugin, load_order.game_settings()).unwrap();
        load_order.plugins.push(plugin);

        let plugin = Plugin::new(early_loader, load_order.game_settings()).unwrap();

        assert!(load_order.validate_index(&plugin, 1).is_ok());
    }

    #[test]
    fn set_plugin_index_should_error_if_inserting_a_non_master_before_a_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        assert!(load_order
            .set_plugin_index("Blank - Master Dependent.esp", 0)
            .is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_moving_a_non_master_before_a_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        assert!(load_order.set_plugin_index("Blank.esp", 0).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_inserting_a_master_after_a_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        assert!(load_order.set_plugin_index("Blank.esm", 2).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_moving_a_master_after_a_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        assert!(load_order.set_plugin_index("Morrowind.esm", 2).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_setting_the_index_of_an_invalid_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        assert!(load_order.set_plugin_index("missing.esm", 0).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_moving_a_plugin_before_an_early_loader() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());

        match load_order.set_plugin_index("Blank.esp", 0).unwrap_err() {
            Error::InvalidEarlyLoadingPluginPosition {
                name,
                pos,
                expected_pos,
            } => {
                assert_eq!("Skyrim.esm", name);
                assert_eq!(1, pos);
                assert_eq!(0, expected_pos);
            }
            e => panic!(
                "Expected InvalidEarlyLoadingPluginPosition error, got {:?}",
                e
            ),
        };

        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_moving_an_early_loader_to_a_different_position() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());

        match load_order.set_plugin_index("Skyrim.esm", 1).unwrap_err() {
            Error::InvalidEarlyLoadingPluginPosition {
                name,
                pos,
                expected_pos,
            } => {
                assert_eq!("Skyrim.esm", name);
                assert_eq!(1, pos);
                assert_eq!(0, expected_pos);
            }
            e => panic!(
                "Expected InvalidEarlyLoadingPluginPosition error, got {:?}",
                e
            ),
        };

        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_inserting_an_early_loader_to_the_wrong_position() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        load_order.set_plugin_index("Blank.esm", 1).unwrap();
        copy_to_test_dir("Blank.esm", "Dragonborn.esm", &load_order.game_settings());

        let existing_filenames = to_owned(load_order.plugin_names());

        match load_order
            .set_plugin_index("Dragonborn.esm", 2)
            .unwrap_err()
        {
            Error::InvalidEarlyLoadingPluginPosition {
                name,
                pos,
                expected_pos,
            } => {
                assert_eq!("Dragonborn.esm", name);
                assert_eq!(2, pos);
                assert_eq!(1, expected_pos);
            }
            e => panic!(
                "Expected InvalidEarlyLoadingPluginPosition error, got {:?}",
                e
            ),
        };

        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_succeed_if_setting_an_early_loader_to_its_current_position() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        assert!(load_order.set_plugin_index("Skyrim.esm", 0).is_ok());
        assert_eq!(
            vec!["Skyrim.esm", "Blank.esp", "Blank - Different.esp"],
            load_order.plugin_names()
        );
    }

    #[test]
    fn set_plugin_index_should_succeed_if_inserting_a_new_early_loader() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Dragonborn.esm", &load_order.game_settings());

        assert!(load_order.set_plugin_index("Dragonborn.esm", 1).is_ok());
        assert_eq!(
            vec![
                "Skyrim.esm",
                "Dragonborn.esm",
                "Blank.esp",
                "Blank - Different.esp"
            ],
            load_order.plugin_names()
        );
    }

    #[test]
    fn set_plugin_index_should_insert_a_new_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let num_plugins = load_order.plugins().len();
        assert_eq!(1, load_order.set_plugin_index("Blank.esm", 1).unwrap());
        assert_eq!(1, load_order.index_of("Blank.esm").unwrap());
        assert_eq!(num_plugins + 1, load_order.plugins().len());
    }

    #[test]
    fn set_plugin_index_should_allow_non_masters_to_be_hoisted() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let filenames = vec!["Blank.esm", "Blank - Different Master Dependent.esm"];

        load_order.replace_plugins(&filenames).unwrap();
        assert_eq!(filenames, load_order.plugin_names());

        let num_plugins = load_order.plugins().len();
        let index = load_order
            .set_plugin_index("Blank - Different.esm", 1)
            .unwrap();
        assert_eq!(1, index);
        assert_eq!(1, load_order.index_of("Blank - Different.esm").unwrap());
        assert_eq!(num_plugins + 1, load_order.plugins().len());
    }

    #[test]
    fn set_plugin_index_should_allow_a_master_file_to_load_after_another_that_hoists_non_masters() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let filenames = vec![
            "Blank - Different.esm",
            "Blank - Different Master Dependent.esm",
        ];

        load_order.replace_plugins(&filenames).unwrap();
        assert_eq!(filenames, load_order.plugin_names());

        let num_plugins = load_order.plugins().len();
        assert_eq!(2, load_order.set_plugin_index("Blank.esm", 2).unwrap());
        assert_eq!(2, load_order.index_of("Blank.esm").unwrap());
        assert_eq!(num_plugins + 1, load_order.plugins().len());
    }

    #[test]
    fn set_plugin_index_should_move_an_existing_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let num_plugins = load_order.plugins().len();
        let index = load_order
            .set_plugin_index("Blank - Different.esp", 1)
            .unwrap();
        assert_eq!(1, index);
        assert_eq!(1, load_order.index_of("Blank - Different.esp").unwrap());
        assert_eq!(num_plugins, load_order.plugins().len());
    }

    #[test]
    fn set_plugin_index_should_move_an_existing_plugin_later_correctly() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        load_and_insert(&mut load_order, "Blank - Master Dependent.esp");
        let num_plugins = load_order.plugins().len();
        assert_eq!(2, load_order.set_plugin_index("Blank.esp", 2).unwrap());
        assert_eq!(2, load_order.index_of("Blank.esp").unwrap());
        assert_eq!(num_plugins, load_order.plugins().len());
    }

    #[test]
    fn set_plugin_index_should_preserve_an_existing_plugins_active_state() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        load_and_insert(&mut load_order, "Blank - Master Dependent.esp");
        assert_eq!(2, load_order.set_plugin_index("Blank.esp", 2).unwrap());
        assert!(load_order.is_active("Blank.esp"));

        let index = load_order
            .set_plugin_index("Blank - Different.esp", 2)
            .unwrap();
        assert_eq!(2, index);
        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn replace_plugins_should_error_if_given_duplicate_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        let filenames = vec!["Blank.esp", "blank.esp"];
        assert!(load_order.replace_plugins(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn replace_plugins_should_error_if_given_an_invalid_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        let filenames = vec!["Blank.esp", "missing.esp"];
        assert!(load_order.replace_plugins(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn replace_plugins_should_error_if_given_a_list_with_plugins_before_masters() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let existing_filenames = to_owned(load_order.plugin_names());
        let filenames = vec!["Blank.esp", "Blank.esm"];
        assert!(load_order.replace_plugins(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn replace_plugins_should_error_if_an_early_loading_plugin_loads_after_another_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());

        let filenames = vec![
            "Skyrim.esm",
            "Blank.esm",
            "Update.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        match load_order.replace_plugins(&filenames).unwrap_err() {
            Error::InvalidEarlyLoadingPluginPosition {
                name,
                pos,
                expected_pos,
            } => {
                assert_eq!("Update.esm", name);
                assert_eq!(2, pos);
                assert_eq!(1, expected_pos);
            }
            e => panic!("Wrong error type: {:?}", e),
        }
    }

    #[test]
    fn replace_plugins_should_not_error_if_an_early_loading_plugin_is_missing() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Dragonborn.esm", &load_order.game_settings());

        let filenames = vec![
            "Skyrim.esm",
            "Dragonborn.esm",
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        assert!(load_order.replace_plugins(&filenames).is_ok());
    }

    #[test]
    fn replace_plugins_should_not_error_if_a_non_early_loading_implicitly_active_plugin_loads_after_another_plugin(
    ) {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Skyrim.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank - Different.esp").unwrap();

        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let filenames = vec![
            "Skyrim.esm",
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        assert!(load_order.replace_plugins(&filenames).is_ok());
    }

    #[test]
    fn replace_plugins_should_not_distinguish_between_ghosted_and_unghosted_filenames() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        copy_to_test_dir(
            "Blank - Different.esm",
            "ghosted.esm.ghost",
            &load_order.game_settings(),
        );

        let filenames = vec![
            "Morrowind.esm",
            "Blank.esm",
            "ghosted.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        assert!(load_order.replace_plugins(&filenames).is_ok());
    }

    #[test]
    fn replace_plugins_should_not_insert_missing_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let filenames = vec![
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
        ];
        load_order.replace_plugins(&filenames).unwrap();

        assert_eq!(filenames, load_order.plugin_names());
    }

    #[test]
    fn replace_plugins_should_not_lose_active_state_of_existing_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let filenames = vec![
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
        ];
        load_order.replace_plugins(&filenames).unwrap();

        assert!(load_order.is_active("Blank.esp"));
    }

    #[test]
    fn replace_plugins_should_accept_hoisted_non_masters() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(GameId::Oblivion, &tmp_dir.path());

        let filenames = vec![
            "Blank.esm",
            "Blank - Different.esm",
            "Blank - Different Master Dependent.esm",
            load_order.game_settings().master_file(),
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "Blàñk.esp",
        ];

        load_order.replace_plugins(&filenames).unwrap();
        assert_eq!(filenames, load_order.plugin_names());
    }

    #[test]
    fn hoist_masters_should_hoist_plugins_that_masters_depend_on_to_load_before_their_first_dependent(
    ) {
        let tmp_dir = tempdir().unwrap();
        let (game_settings, _) = mock_game_files(GameId::SkyrimSE, &tmp_dir.path());

        // Test both hoisting a master before a master and a non-master before a master.

        let master_dependent_master = "Blank - Master Dependent.esm";
        copy_to_test_dir(
            master_dependent_master,
            master_dependent_master,
            &game_settings,
        );

        let plugin_dependent_master = "Blank - Plugin Dependent.esm";
        copy_to_test_dir(
            "Blank - Plugin Dependent.esp",
            plugin_dependent_master,
            &game_settings,
        );

        let plugin_names = vec![
            "Skyrim.esm",
            master_dependent_master,
            "Blank.esm",
            plugin_dependent_master,
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
            "Blank.esp",
        ];
        let mut plugins = plugin_names
            .iter()
            .map(|n| Plugin::new(n, &game_settings).unwrap())
            .collect();

        assert!(hoist_masters(&mut plugins).is_ok());

        let expected_plugin_names = vec![
            "Skyrim.esm",
            "Blank.esm",
            master_dependent_master,
            "Blank.esp",
            plugin_dependent_master,
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        let plugin_names: Vec<_> = plugins.iter().map(Plugin::name).collect();
        assert_eq!(expected_plugin_names, plugin_names);
    }

    #[test]
    fn hoist_masters_should_not_hoist_blueprint_plugins_that_are_masters_of_non_blueprint_plugins()
    {
        let tmp_dir = tempdir().unwrap();
        let (game_settings, _) = mock_game_files(GameId::Starfield, &tmp_dir.path());

        let blueprint_plugin = "Blank.full.esm";
        set_blueprint_flag(
            GameId::Starfield,
            &game_settings.plugins_directory().join(blueprint_plugin),
            true,
        )
        .unwrap();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(dependent_plugin, dependent_plugin, &game_settings);

        let plugin_names = vec![
            "Starfield.esm",
            dependent_plugin,
            "Blank.esp",
            blueprint_plugin,
        ];

        let mut plugins = plugin_names
            .iter()
            .map(|n| Plugin::new(n, &game_settings).unwrap())
            .collect();

        assert!(hoist_masters(&mut plugins).is_ok());

        let expected_plugin_names = plugin_names;

        let plugin_names: Vec<_> = plugins.iter().map(Plugin::name).collect();
        assert_eq!(expected_plugin_names, plugin_names);
    }

    #[test]
    fn hoist_masters_should_hoist_blueprint_plugins_that_are_masters_of_blueprint_plugins() {
        let tmp_dir = tempdir().unwrap();
        let (game_settings, _) = mock_game_files(GameId::Starfield, &tmp_dir.path());

        let plugins_dir = game_settings.plugins_directory();

        let blueprint_plugin = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(blueprint_plugin), true).unwrap();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(dependent_plugin, dependent_plugin, &game_settings);
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(dependent_plugin), true).unwrap();

        let plugin_names = vec![
            "Starfield.esm",
            "Blank.esp",
            dependent_plugin,
            blueprint_plugin,
        ];

        let mut plugins = plugin_names
            .iter()
            .map(|n| Plugin::new(n, &game_settings).unwrap())
            .collect();

        assert!(hoist_masters(&mut plugins).is_ok());

        let expected_plugin_names = vec![
            "Starfield.esm",
            "Blank.esp",
            blueprint_plugin,
            dependent_plugin,
        ];

        let plugin_names: Vec<_> = plugins.iter().map(Plugin::name).collect();
        assert_eq!(expected_plugin_names, plugin_names);
    }

    #[test]
    fn find_plugins_in_dirs_should_sort_files_by_modification_timestamp() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let result = find_plugins_in_dirs(
            &[load_order.game_settings.plugins_directory()],
            load_order.game_settings.id(),
        );

        let plugin_names = [
            load_order.game_settings.master_file(),
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blàñk.esp",
        ];

        assert_eq!(plugin_names.as_slice(), result);
    }

    #[test]
    fn find_plugins_in_dirs_should_sort_files_by_descending_filename_if_timestamps_are_equal() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let timestamp = 1321010051;
        let plugin_path = load_order
            .game_settings
            .plugins_directory()
            .join("Blank - Different.esp");
        set_file_timestamps(&plugin_path, timestamp);
        let plugin_path = load_order
            .game_settings
            .plugins_directory()
            .join("Blank - Master Dependent.esp");
        set_file_timestamps(&plugin_path, timestamp);

        let result = find_plugins_in_dirs(
            &[load_order.game_settings.plugins_directory()],
            load_order.game_settings.id(),
        );

        let plugin_names = [
            load_order.game_settings.master_file(),
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        assert_eq!(plugin_names.as_slice(), result);
    }

    #[test]
    fn find_plugins_in_dirs_should_sort_files_by_ascending_filename_if_timestamps_are_equal_and_game_is_starfield(
    ) {
        let tmp_dir = tempdir().unwrap();
        let (game_settings, plugins) = mock_game_files(GameId::Starfield, &tmp_dir.path());
        let load_order = TestLoadOrder {
            game_settings,
            plugins,
        };

        let timestamp = 1321009991;

        let plugin_names = [
            "Blank - Override.esp",
            "Blank.esp",
            "Blank.full.esm",
            "Blank.medium.esm",
            "Blank.small.esm",
            "Starfield.esm",
        ];

        for plugin_name in plugin_names {
            let plugin_path = load_order
                .game_settings
                .plugins_directory()
                .join(plugin_name);
            set_file_timestamps(&plugin_path, timestamp);
        }

        let result = find_plugins_in_dirs(
            &[load_order.game_settings.plugins_directory()],
            load_order.game_settings.id(),
        );

        assert_eq!(plugin_names.as_slice(), result);
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
    fn validate_load_order_should_be_ok_if_there_are_only_master_files() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        let plugins = vec![
            Plugin::new(settings.master_file(), &settings).unwrap(),
            Plugin::new("Blank.esm", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_ok());
    }

    #[test]
    fn validate_load_order_should_be_ok_if_there_are_no_master_files() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        let plugins = vec![
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new("Blank - Different.esp", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_ok());
    }

    #[test]
    fn validate_load_order_should_be_ok_if_master_files_are_before_all_others() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        let plugins = vec![
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_ok());
    }

    #[test]
    fn validate_load_order_should_be_ok_if_hoisted_non_masters_load_before_masters() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        copy_to_test_dir(
            "Blank - Plugin Dependent.esp",
            "Blank - Plugin Dependent.esm",
            &settings,
        );

        let plugins = vec![
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new("Blank - Plugin Dependent.esm", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_ok());
    }

    #[test]
    fn validate_load_order_should_error_if_non_masters_are_hoisted_earlier_than_needed() {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        copy_to_test_dir(
            "Blank - Plugin Dependent.esp",
            "Blank - Plugin Dependent.esm",
            &settings,
        );

        let plugins = vec![
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank - Plugin Dependent.esm", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_err());
    }

    #[test]
    fn validate_load_order_should_error_if_master_files_load_before_non_masters_they_have_as_masters(
    ) {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        copy_to_test_dir(
            "Blank - Plugin Dependent.esp",
            "Blank - Plugin Dependent.esm",
            &settings,
        );

        let plugins = vec![
            Plugin::new("Blank.esm", &settings).unwrap(),
            Plugin::new("Blank - Plugin Dependent.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_err());
    }

    #[test]
    fn validate_load_order_should_error_if_master_files_load_before_other_masters_they_have_as_masters(
    ) {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::SkyrimSE, &tmp_dir.path()).game_settings;

        copy_to_test_dir(
            "Blank - Master Dependent.esm",
            "Blank - Master Dependent.esm",
            &settings,
        );

        let plugins = vec![
            Plugin::new("Blank - Master Dependent.esm", &settings).unwrap(),
            Plugin::new("Blank.esm", &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_err());
    }

    #[test]
    fn validate_load_order_should_succeed_if_a_blueprint_plugin_loads_after_all_non_blueprint_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::Starfield, &tmp_dir.path()).game_settings;

        let plugins_dir = settings.plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let plugins = vec![
            Plugin::new("Starfield.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new(plugin_name, &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_ok());
    }

    #[test]
    fn validate_load_order_should_succeed_if_a_blueprint_plugin_loads_after_a_non_blueprint_plugin_that_depends_on_it(
    ) {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::Starfield, &tmp_dir.path()).game_settings;

        let plugins_dir = settings.plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(dependent_plugin, dependent_plugin, &settings);

        let plugins = vec![
            Plugin::new("Starfield.esm", &settings).unwrap(),
            Plugin::new(dependent_plugin, &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new(plugin_name, &settings).unwrap(),
        ];

        assert!(validate_load_order(&plugins, &[]).is_ok());
    }

    #[test]
    fn validate_load_order_should_fail_if_a_blueprint_plugin_loads_after_a_blueprint_plugin_that_depends_on_it(
    ) {
        let tmp_dir = tempdir().unwrap();
        let settings = prepare(GameId::Starfield, &tmp_dir.path()).game_settings;

        let plugins_dir = settings.plugins_directory();

        let plugin_name = "Blank.full.esm";
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(plugin_name), true).unwrap();

        let dependent_plugin = "Blank - Override.full.esm";
        copy_to_test_dir(dependent_plugin, dependent_plugin, &settings);
        set_blueprint_flag(GameId::Starfield, &plugins_dir.join(dependent_plugin), true).unwrap();

        let plugins = vec![
            Plugin::new("Starfield.esm", &settings).unwrap(),
            Plugin::new("Blank.esp", &settings).unwrap(),
            Plugin::new(dependent_plugin, &settings).unwrap(),
            Plugin::new(plugin_name, &settings).unwrap(),
        ];

        match validate_load_order(&plugins, &[]).unwrap_err() {
            Error::UnrepresentedHoist { plugin, master } => {
                assert_eq!(plugin_name, plugin);
                assert_eq!(dependent_plugin, master);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn find_first_non_master_should_find_a_full_esp() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare_plugins(&tmp_dir.path(), "Blank.esp");

        let first_non_master = super::find_first_non_master_position(&plugins);
        assert_eq!(1, first_non_master.unwrap());
    }

    #[test]
    fn find_first_non_master_should_find_a_light_flagged_esp() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare_plugins(&tmp_dir.path(), "Blank.esl");

        let first_non_master = super::find_first_non_master_position(&plugins);
        assert_eq!(1, first_non_master.unwrap());
    }
}

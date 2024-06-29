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
use std::collections::HashSet;
use std::fs::create_dir_all;
use std::path::Path;

use unicase::{eq, UniCase};

use super::mutable::MutableLoadOrder;
use super::readable::{ReadableLoadOrder, ReadableLoadOrderBase};
use crate::enums::Error;
use crate::plugin::Plugin;
use crate::GameSettings;

const MAX_ACTIVE_FULL_PLUGINS: usize = 255;
const MAX_ACTIVE_LIGHT_PLUGINS: usize = 4096;
const MAX_ACTIVE_MEDIUM_PLUGINS: usize = 256;

pub trait WritableLoadOrder: ReadableLoadOrder {
    fn game_settings_mut(&mut self) -> &mut GameSettings;

    fn load(&mut self) -> Result<(), Error>;

    fn save(&mut self) -> Result<(), Error>;

    fn add(&mut self, plugin_name: &str) -> Result<usize, Error>;

    fn remove(&mut self, plugin_name: &str) -> Result<(), Error>;

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), Error>;

    fn set_plugin_index(&mut self, plugin_name: &str, position: usize) -> Result<usize, Error>;

    fn is_self_consistent(&self) -> Result<bool, Error>;

    fn is_ambiguous(&self) -> Result<bool, Error>;

    fn activate(&mut self, plugin_name: &str) -> Result<(), Error>;

    fn deactivate(&mut self, plugin_name: &str) -> Result<(), Error>;

    fn set_active_plugins(&mut self, active_plugin_names: &[&str]) -> Result<(), Error>;
}

pub fn add<T: MutableLoadOrder>(load_order: &mut T, plugin_name: &str) -> Result<usize, Error> {
    match load_order.index_of(plugin_name) {
        Some(_) => Err(Error::DuplicatePlugin(plugin_name.to_string())),
        None => {
            let plugin = Plugin::new(plugin_name, load_order.game_settings())?;

            match load_order.insert_position(&plugin) {
                Some(position) => {
                    load_order.validate_index(&plugin, position)?;
                    load_order.plugins_mut().insert(position, plugin);
                    Ok(position)
                }
                None => {
                    load_order.validate_index(&plugin, load_order.plugins().len())?;
                    load_order.plugins_mut().push(plugin);
                    Ok(load_order.plugins().len() - 1)
                }
            }
        }
    }
}

pub fn remove<T: MutableLoadOrder>(load_order: &mut T, plugin_name: &str) -> Result<(), Error> {
    match load_order.index_of(plugin_name) {
        Some(index) => {
            let plugin_path = load_order.game_settings().plugin_path(plugin_name);
            if plugin_path.exists() {
                return Err(Error::InstalledPlugin(plugin_name.to_string()));
            }

            // If this is a master file that depends on a non-master file, it shouldn't be removed
            // without first moving the non-master file later in the load order, unless the next
            // master file also depends on that same non-master file. The non-master file also
            // doesn't need to be moved if this is the last master file in the load order.
            if load_order.plugins()[index].is_master_file() {
                let next_master_index = &load_order
                    .plugins()
                    .iter()
                    .skip(index + 1)
                    .position(|p| p.is_master_file());

                if let Some(next_master_index) = next_master_index {
                    let next_master_masters = load_order.plugins()[*next_master_index].masters()?;
                    let next_master_master_names: HashSet<_> =
                        next_master_masters.iter().map(UniCase::new).collect();

                    let mut masters = load_order.plugins()[index].masters()?;

                    // Remove any masters that are also masters of the next master plugin.
                    masters.retain(|m| !next_master_master_names.contains(&UniCase::new(m)));

                    // Finally, check if any remaining masters are non-master plugins.
                    if let Some(n) = masters.iter().find(|n| {
                        load_order
                            .index_of(n)
                            .map(|i| !load_order.plugins()[i].is_master_file())
                            // If the master isn't installed, assume it's a master file and so
                            // doesn't prevent removal of the target plugin.
                            .unwrap_or(false)
                    }) {
                        return Err(Error::NonMasterBeforeMaster {
                            master: plugin_name.to_string(),
                            non_master: n.to_string(),
                        });
                    }
                }
            }

            load_order.plugins_mut().remove(index);

            Ok(())
        }
        None => Err(Error::PluginNotFound(plugin_name.to_string())),
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PluginCounts {
    light: usize,
    medium: usize,
    full: usize,
}

impl PluginCounts {
    fn count_plugin(&mut self, plugin: &Plugin) {
        if plugin.is_light_plugin() {
            self.light += 1;
        } else if plugin.is_medium_plugin() {
            self.medium += 1;
        } else {
            self.full += 1;
        }
    }
}

fn count_active_plugins<T: ReadableLoadOrderBase>(load_order: &T) -> PluginCounts {
    let mut counts = PluginCounts::default();

    for plugin in load_order.plugins().iter().filter(|p| p.is_active()) {
        counts.count_plugin(plugin);
    }

    counts
}

fn count_plugins(existing_plugins: &[Plugin], existing_plugin_indexes: &[usize]) -> PluginCounts {
    let mut counts = PluginCounts::default();

    for index in existing_plugin_indexes {
        let plugin = &existing_plugins[*index];
        counts.count_plugin(plugin);
    }

    counts
}

pub fn activate<T: MutableLoadOrder>(load_order: &mut T, plugin_name: &str) -> Result<(), Error> {
    let counts = count_active_plugins(load_order);

    let plugin = match load_order
        .plugins_mut()
        .iter_mut()
        .find(|p| p.name_matches(plugin_name))
    {
        Some(p) => p,
        None => return Err(Error::PluginNotFound(plugin_name.to_string())),
    };

    if !plugin.is_active() {
        let is_light = plugin.is_light_plugin();
        let is_medium = plugin.is_medium_plugin();
        let is_full = !is_light && !is_medium;

        if (is_light && counts.light == MAX_ACTIVE_LIGHT_PLUGINS)
            || (is_medium && counts.medium == MAX_ACTIVE_MEDIUM_PLUGINS)
            || (is_full && counts.full == MAX_ACTIVE_FULL_PLUGINS)
        {
            return Err(Error::TooManyActivePlugins {
                light_count: counts.light,
                medium_count: counts.medium,
                full_count: counts.full,
            });
        } else {
            plugin.activate()?;
        }
    }

    Ok(())
}

pub fn deactivate<T: MutableLoadOrder>(load_order: &mut T, plugin_name: &str) -> Result<(), Error> {
    if load_order.game_settings().is_implicitly_active(plugin_name) {
        return Err(Error::ImplicitlyActivePlugin(plugin_name.to_string()));
    }

    load_order
        .plugins_mut()
        .iter_mut()
        .find(|p| p.name_matches(plugin_name))
        .ok_or_else(|| Error::PluginNotFound(plugin_name.to_string()))
        .map(|p| p.deactivate())
}

pub fn set_active_plugins<T: MutableLoadOrder>(
    load_order: &mut T,
    active_plugin_names: &[&str],
) -> Result<(), Error> {
    let existing_plugin_indices = load_order.lookup_plugins(active_plugin_names)?;

    let counts = count_plugins(load_order.plugins(), &existing_plugin_indices);

    if counts.full > MAX_ACTIVE_FULL_PLUGINS
        || counts.medium > MAX_ACTIVE_MEDIUM_PLUGINS
        || counts.light > MAX_ACTIVE_LIGHT_PLUGINS
    {
        return Err(Error::TooManyActivePlugins {
            light_count: counts.light,
            medium_count: counts.medium,
            full_count: counts.full,
        });
    }

    for plugin_name in load_order.game_settings().implicitly_active_plugins() {
        // If the plugin isn't installed, don't check that it's in the active
        // plugins list. Installed plugins will have already been loaded.
        if load_order.index_of(plugin_name).is_some()
            && !active_plugin_names.iter().any(|p| eq(*p, plugin_name))
        {
            return Err(Error::ImplicitlyActivePlugin(plugin_name.to_string()));
        }
    }

    load_order.deactivate_all();

    for index in existing_plugin_indices {
        load_order.plugins_mut()[index].activate()?;
    }

    Ok(())
}

pub fn create_parent_dirs(path: &Path) -> Result<(), Error> {
    if let Some(x) = path.parent() {
        if !x.exists() {
            create_dir_all(x).map_err(|e| Error::IoError(x.to_path_buf(), e))?
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::remove_file;
    use std::path::Path;

    use tempfile::tempdir;

    use crate::enums::GameId;
    use crate::game_settings::GameSettings;
    use crate::load_order::mutable::MutableLoadOrder;
    use crate::load_order::readable::{ReadableLoadOrder, ReadableLoadOrderBase};
    use crate::load_order::tests::{
        load_and_insert, mock_game_files, set_master_flag, write_active_plugins_file,
    };
    use crate::tests::copy_to_test_dir;

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

    fn prepare(game_id: GameId, game_dir: &Path) -> TestLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, game_dir);
        TestLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn prepare_bulk_plugins(
        game_settings: &GameSettings,
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut full_plugins = vec![game_settings.master_file().to_string()];
        full_plugins.extend((0..260).map(|i| format!("Blank{}.full.esm", i)));
        for plugin in &full_plugins {
            copy_to_test_dir("Blank.full.esm", &plugin, game_settings);
        }

        let medium_plugins: Vec<_> = (0..260).map(|i| format!("Blank{}.medium.esm", i)).collect();
        for plugin in &medium_plugins {
            copy_to_test_dir("Blank.medium.esm", &plugin, game_settings);
        }

        let small_plugins: Vec<_> = (0..5000).map(|i| format!("Blank{}.small.esm", i)).collect();
        for plugin in &small_plugins {
            copy_to_test_dir("Blank.small.esm", &plugin, game_settings);
        }

        let mut plugins: Vec<_> = full_plugins.iter().collect();
        plugins.extend(medium_plugins.iter());
        plugins.extend(small_plugins.iter());

        write_active_plugins_file(game_settings, &plugins);

        (full_plugins, medium_plugins, small_plugins)
    }

    #[test]
    fn add_should_error_if_the_plugin_is_already_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(add(&mut load_order, "Blank.esm").is_ok());
        assert!(add(&mut load_order, "Blank.esm").is_err());
    }

    #[test]
    fn add_should_error_if_given_a_master_that_would_hoist_a_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugins_dir = &load_order.game_settings().plugins_directory();
        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm",
            load_order.game_settings(),
        );
        set_master_flag(&plugins_dir.join("Blank - Different.esm"), false).unwrap();
        assert!(add(&mut load_order, "Blank - Different.esm").is_ok());

        copy_to_test_dir(
            "Blank - Different Master Dependent.esm",
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        );

        assert!(add(&mut load_order, "Blank - Different Master Dependent.esm").is_err());
    }

    #[test]
    fn add_should_error_if_the_plugin_is_not_valid() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(add(&mut load_order, "invalid.esm").is_err());
    }

    #[test]
    fn add_should_insert_a_master_before_non_masters() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert_eq!(1, add(&mut load_order, "Blank.esm").unwrap());
        assert_eq!(1, load_order.index_of("Blank.esm").unwrap());
    }

    #[test]
    fn add_should_append_a_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert_eq!(
            3,
            add(&mut load_order, "Blank - Master Dependent.esp").unwrap()
        );
        assert_eq!(
            3,
            load_order.index_of("Blank - Master Dependent.esp").unwrap()
        );
    }

    #[test]
    fn add_should_hoist_a_non_master_that_a_master_depends_on() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugins_dir = &load_order.game_settings().plugins_directory();
        copy_to_test_dir(
            "Blank - Different Master Dependent.esm",
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        );
        assert!(add(&mut load_order, "Blank - Different Master Dependent.esm").is_ok());

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm",
            load_order.game_settings(),
        );
        set_master_flag(&plugins_dir.join("Blank - Different.esm"), false).unwrap();
        assert_eq!(1, add(&mut load_order, "Blank - Different.esm").unwrap());
    }

    #[test]
    fn add_should_hoist_a_master_that_a_master_depends_on() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin_name = "Blank - Master Dependent.esm";
        copy_to_test_dir(plugin_name, plugin_name, load_order.game_settings());
        assert_eq!(1, add(&mut load_order, plugin_name).unwrap());

        assert_eq!(1, add(&mut load_order, "Blank.esm").unwrap());
    }

    #[test]
    fn remove_should_error_if_the_plugin_is_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());
        assert!(remove(&mut load_order, "Blank.esm").is_err());
    }

    #[test]
    fn remove_should_error_if_the_plugin_is_installed() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());
        assert!(remove(&mut load_order, "Blank.esp").is_err());
    }

    #[test]
    fn remove_should_error_if_removing_a_master_would_leave_a_non_master_it_hoisted_loading_too_early(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin_to_remove = "Blank - Different Master Dependent.esm";

        let plugins_dir = &load_order.game_settings().plugins_directory();
        copy_to_test_dir(
            plugin_to_remove,
            plugin_to_remove,
            load_order.game_settings(),
        );
        assert!(add(&mut load_order, plugin_to_remove).is_ok());

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm",
            load_order.game_settings(),
        );
        set_master_flag(&plugins_dir.join("Blank - Different.esm"), false).unwrap();
        assert_eq!(1, add(&mut load_order, "Blank - Different.esm").unwrap());

        copy_to_test_dir(
            "Blank - Master Dependent.esm",
            "Blank - Master Dependent.esm",
            load_order.game_settings(),
        );
        assert!(add(&mut load_order, "Blank - Master Dependent.esm").is_ok());

        let blank_master_dependent = load_order.plugins.remove(1);
        load_order.plugins.insert(3, blank_master_dependent);

        std::fs::remove_file(&plugins_dir.join(plugin_to_remove)).unwrap();

        match remove(&mut load_order, plugin_to_remove).unwrap_err() {
            Error::NonMasterBeforeMaster { master, non_master } => {
                assert_eq!("Blank - Different Master Dependent.esm", master);
                assert_eq!("Blank - Different.esm", non_master);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn remove_should_remove_the_given_plugin_from_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        remove_file(
            load_order
                .game_settings()
                .plugins_directory()
                .join("Blank.esp"),
        )
        .unwrap();

        assert!(remove(&mut load_order, "Blank.esp").is_ok());
        assert!(load_order.index_of("Blank.esp").is_none());
    }

    #[test]
    fn activate_should_activate_the_plugin_with_the_given_filename() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(activate(&mut load_order, "Blank - Different.esp").is_ok());
        assert!(load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn activate_should_error_if_the_plugin_is_not_valid() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(activate(&mut load_order, "missing.esp").is_err());
        assert!(load_order.index_of("missing.esp").is_none());
    }

    #[test]
    fn activate_should_error_if_the_plugin_is_not_already_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(activate(&mut load_order, "Blank.esm").is_err());
        assert!(!load_order.is_active("Blank.esm"));
    }

    #[test]
    fn activate_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(activate(&mut load_order, "Blank - different.esp").is_ok());
        assert!(load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn activate_should_throw_if_increasing_the_number_of_active_plugins_past_the_limit() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        for i in 0..(MAX_ACTIVE_FULL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank.esp", &plugin, &load_order.game_settings());
            load_and_insert(&mut load_order, &plugin);
            activate(&mut load_order, &plugin).unwrap();
        }

        assert!(activate(&mut load_order, "Blank - Different.esp").is_err());
        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn activate_should_succeed_if_at_the_active_plugins_limit_and_the_plugin_is_already_active() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        for i in 0..(MAX_ACTIVE_FULL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank.esp", &plugin, &load_order.game_settings());
            load_and_insert(&mut load_order, &plugin);
            activate(&mut load_order, &plugin).unwrap();
        }

        assert!(load_order.is_active("Blank.esp"));
        assert!(activate(&mut load_order, "Blank.esp").is_ok());
    }

    #[test]
    fn activate_should_fail_if_at_the_active_plugins_limit_and_the_plugin_is_an_update_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        for i in 0..(MAX_ACTIVE_FULL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank.esp", &plugin, &load_order.game_settings());
            load_and_insert(&mut load_order, &plugin);
            activate(&mut load_order, &plugin).unwrap();
        }

        let plugin = "Blank - Override.esp";
        load_and_insert(&mut load_order, &plugin);

        assert!(!load_order.is_active(plugin));

        assert!(activate(&mut load_order, plugin).is_err());
        assert!(!load_order.is_active(plugin));
    }

    #[test]
    fn activate_should_count_active_update_plugins_towards_limit() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        for i in 0..(MAX_ACTIVE_FULL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank - Override.esp", &plugin, &load_order.game_settings());
            load_and_insert(&mut load_order, &plugin);
            activate(&mut load_order, &plugin).unwrap();
        }

        let plugin = "Blank - Override.esp";
        load_and_insert(&mut load_order, &plugin);

        assert!(!load_order.is_active(plugin));

        assert!(activate(&mut load_order, plugin).is_err());
        assert!(!load_order.is_active(plugin));
    }

    #[test]
    fn activate_should_check_full_medium_and_small_plugins_active_limits_separately() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let (full, medium, light) = prepare_bulk_plugins(load_order.game_settings());

        full.iter()
            .for_each(|p| load_and_insert(&mut load_order, &p));
        medium
            .iter()
            .for_each(|p| load_and_insert(&mut load_order, &p));
        light
            .iter()
            .for_each(|p| load_and_insert(&mut load_order, &p));

        let mut plugin_refs: Vec<_> = full[..254].iter().map(String::as_str).collect();
        plugin_refs.extend(medium[..255].iter().map(String::as_str));
        plugin_refs.extend(light[..4095].iter().map(String::as_str));

        assert!(set_active_plugins(&mut load_order, &plugin_refs).is_ok());

        let plugin = &full[254];
        assert!(!load_order.is_active(plugin));
        assert!(activate(&mut load_order, plugin).is_ok());
        assert!(load_order.is_active(plugin));

        let plugin = &medium[255];
        assert!(!load_order.is_active(plugin));
        assert!(activate(&mut load_order, plugin).is_ok());
        assert!(load_order.is_active(plugin));

        let plugin = &light[4095];
        assert!(!load_order.is_active(plugin));
        assert!(activate(&mut load_order, plugin).is_ok());
        assert!(load_order.is_active(plugin));

        let plugin = &full[255];
        assert!(activate(&mut load_order, plugin).is_err());
        assert!(!load_order.is_active(plugin));

        let plugin = &medium[256];
        assert!(activate(&mut load_order, plugin).is_err());
        assert!(!load_order.is_active(plugin));

        let plugin = &light[4096];
        assert!(activate(&mut load_order, plugin).is_err());
        assert!(!load_order.is_active(plugin));
    }

    #[test]
    fn deactivate_should_deactivate_the_plugin_with_the_given_filename() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.is_active("Blank.esp"));
        assert!(deactivate(&mut load_order, "Blank.esp").is_ok());
        assert!(!load_order.is_active("Blank.esp"));
    }

    #[test]
    fn deactivate_should_error_if_the_plugin_is_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(deactivate(&mut load_order, "missing.esp").is_err());
        assert!(load_order.index_of("missing.esp").is_none());
    }

    #[test]
    fn deactivate_should_error_if_given_an_implicitly_active_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(activate(&mut load_order, "Skyrim.esm").is_ok());
        assert!(deactivate(&mut load_order, "Skyrim.esm").is_err());
        assert!(load_order.is_active("Skyrim.esm"));
    }

    #[test]
    fn deactivate_should_error_if_given_a_missing_implicitly_active_plugin() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(deactivate(&mut load_order, "Update.esm").is_err());
        assert!(load_order.index_of("Update.esm").is_none());
    }

    #[test]
    fn deactivate_should_do_nothing_if_the_plugin_is_inactive() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(!load_order.is_active("Blank - Different.esp"));
        assert!(deactivate(&mut load_order, "Blank - Different.esp").is_ok());
        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn set_active_plugins_should_error_if_passed_an_invalid_plugin_name() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["missing.esp"];
        assert!(set_active_plugins(&mut load_order, &active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_the_given_plugins_are_missing_implicitly_active_plugins()
    {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let active_plugins = ["Blank.esp"];
        assert!(set_active_plugins(&mut load_order, &active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_a_missing_implicitly_active_plugin_is_given() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let active_plugins = ["Skyrim.esm", "Update.esm"];
        assert!(set_active_plugins(&mut load_order, &active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_given_plugins_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["Blank - Master Dependent.esp", "Blàñk.esp"];
        assert!(set_active_plugins(&mut load_order, &active_plugins).is_err());
        assert!(!load_order.is_active("Blank - Master Dependent.esp"));
        assert!(load_order
            .index_of("Blank - Master Dependent.esp")
            .is_none());
        assert!(!load_order.is_active("Blàñk.esp"));
        assert!(load_order.index_of("Blàñk.esp").is_none());
    }

    #[test]
    fn set_active_plugins_should_deactivate_all_plugins_not_given() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["Blank - Different.esp"];
        assert!(load_order.is_active("Blank.esp"));
        assert!(set_active_plugins(&mut load_order, &active_plugins).is_ok());
        assert!(!load_order.is_active("Blank.esp"));
    }

    #[test]
    fn set_active_plugins_should_activate_all_given_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["Blank - Different.esp"];
        assert!(!load_order.is_active("Blank - Different.esp"));
        assert!(set_active_plugins(&mut load_order, &active_plugins).is_ok());
        assert!(load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn set_active_plugins_should_count_update_plugins_towards_limit() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let blank_override = "Blank - Override.esp";
        load_and_insert(&mut load_order, blank_override);

        let mut active_plugins = vec!["Starfield.esm".to_string(), blank_override.to_string()];

        for i in 0..(MAX_ACTIVE_FULL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank.esp", &plugin, &load_order.game_settings());
            load_and_insert(&mut load_order, &plugin);

            active_plugins.push(plugin);
        }

        let active_plugins: Vec<&str> = active_plugins.iter().map(|s| s.as_str()).collect();

        assert!(set_active_plugins(&mut load_order, &active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_count_full_medium_and_small_plugins_separately() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let (full, medium, light) = prepare_bulk_plugins(load_order.game_settings());

        let mut plugin_refs: Vec<&str> = full[..255].iter().map(AsRef::as_ref).collect();
        plugin_refs.extend(medium[0..255].iter().map(|s| s.as_str()));
        plugin_refs.extend(light[0..4096].iter().map(|s| s.as_str()));

        plugin_refs
            .iter()
            .for_each(|p| load_and_insert(&mut load_order, &p));

        assert!(set_active_plugins(&mut load_order, &plugin_refs).is_ok());
        assert_eq!(4606, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_given_more_than_255_full_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let (full, _, _) = prepare_bulk_plugins(load_order.game_settings());

        let plugin_refs: Vec<&str> = full[..256].iter().map(AsRef::as_ref).collect();

        assert!(set_active_plugins(&mut load_order, &plugin_refs).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_given_more_than_256_medium_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let (_, medium, _) = prepare_bulk_plugins(load_order.game_settings());

        let plugin_refs: Vec<&str> = medium[..257].iter().map(AsRef::as_ref).collect();

        assert!(set_active_plugins(&mut load_order, &plugin_refs).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_given_more_than_4096_light_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, &tmp_dir.path());

        let (_, _, light) = prepare_bulk_plugins(load_order.game_settings());

        let plugin_refs: Vec<&str> = light[..4097].iter().map(AsRef::as_ref).collect();

        assert!(set_active_plugins(&mut load_order, &plugin_refs).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }
}

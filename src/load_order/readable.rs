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
use std::fs::read_dir;

use rayon::iter::Either;
use rayon::prelude::*;

use enums::Error;
use game_settings::GameSettings;
use plugin::{trim_dot_ghost, Plugin};

pub const MAX_ACTIVE_NORMAL_PLUGINS: usize = 255;
pub const MAX_ACTIVE_LIGHT_MASTERS: usize = 4096;

pub trait ReadableLoadOrder {
    fn game_settings(&self) -> &GameSettings;

    fn plugin_names(&self) -> Vec<&str>;

    fn index_of(&self, plugin_name: &str) -> Option<usize>;

    fn plugin_at(&self, index: usize) -> Option<&str>;

    fn active_plugin_names(&self) -> Vec<&str>;

    fn is_active(&self, plugin_name: &str) -> bool;
}

pub fn plugin_names(plugins: &[Plugin]) -> Vec<&str> {
    plugins.iter().map(Plugin::name).collect()
}

pub fn index_of(plugins: &[Plugin], plugin_name: &str) -> Option<usize> {
    plugins.iter().position(|p| p.name_matches(plugin_name))
}

pub fn plugin_at(plugins: &[Plugin], index: usize) -> Option<&str> {
    plugins.get(index).map(Plugin::name)
}

pub fn active_plugin_names(plugins: &[Plugin]) -> Vec<&str> {
    plugins
        .iter()
        .filter(|p| p.is_active())
        .map(Plugin::name)
        .collect()
}

pub fn is_active(plugins: &[Plugin], plugin_name: &str) -> bool {
    plugins
        .iter()
        .find(|p| p.name_matches(plugin_name))
        .map_or(false, |p| p.is_active())
}

pub trait ReadableLoadOrderExt: ReadableLoadOrder + Sync {
    fn plugins(&self) -> &Vec<Plugin>;

    fn count_active_normal_plugins(&self) -> usize {
        self.plugins()
            .iter()
            .filter(|p| !p.is_light_master_file() && p.is_active())
            .count()
    }

    fn count_active_light_masters(&self) -> usize {
        self.plugins()
            .iter()
            .filter(|p| p.is_light_master_file() && p.is_active())
            .count()
    }

    fn find_plugins_in_dir(&self) -> Vec<String> {
        let entries = match read_dir(&self.game_settings().plugins_directory()) {
            Ok(x) => x,
            _ => return Vec::new(),
        };

        let mut set: HashSet<String> = HashSet::new();

        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|f| f.is_file()).unwrap_or(false))
            .filter_map(|e| e.file_name().to_str().and_then(|f| Some(f.to_owned())))
            .filter(|ref filename| set.insert(trim_dot_ghost(&filename).to_lowercase()))
            .collect()
    }

    fn find_plugins_in_dir_sorted(&self) -> Vec<String> {
        let mut filenames = self.find_plugins_in_dir();
        filenames.sort();

        filenames
    }

    fn get_excess_active_plugin_indices(&self) -> Vec<usize> {
        let implicitly_active_plugins = self.game_settings().implicitly_active_plugins();
        let mut normal_active_count = self.count_active_normal_plugins();
        let mut light_master_active_count = self.count_active_light_masters();

        let mut plugin_indices: Vec<usize> = Vec::new();
        for (index, plugin) in self.plugins().iter().enumerate().rev() {
            if normal_active_count <= MAX_ACTIVE_NORMAL_PLUGINS
                && light_master_active_count <= MAX_ACTIVE_LIGHT_MASTERS
            {
                break;
            }
            let can_deactivate = plugin.is_active()
                && !implicitly_active_plugins
                    .iter()
                    .any(|i| plugin.name_matches(i));
            if can_deactivate {
                if plugin.is_light_master_file()
                    && light_master_active_count > MAX_ACTIVE_LIGHT_MASTERS
                {
                    plugin_indices.push(index);
                    light_master_active_count -= 1;
                } else if !plugin.is_light_master_file()
                    && normal_active_count > MAX_ACTIVE_NORMAL_PLUGINS
                {
                    plugin_indices.push(index);
                    normal_active_count -= 1;
                }
            }
        }

        plugin_indices
    }

    fn validate_index(&self, plugin: &Plugin, index: usize) -> Result<(), Error> {
        if plugin.is_master_file() {
            self.validate_master_file_index(plugin, index)
        } else {
            self.validate_non_master_file_index(plugin, index)
        }
    }

    fn validate_master_file_index(&self, plugin: &Plugin, index: usize) -> Result<(), Error> {
        let plugins = if index < self.plugins().len() {
            &self.plugins()[..index]
        } else {
            &self.plugins()[..]
        };

        let previous_master_pos = plugins
            .iter()
            .rposition(|p| p.is_master_file())
            .unwrap_or(0);

        let master_names: HashSet<String> =
            plugin.masters()?.iter().map(|m| m.to_lowercase()).collect();

        // Check that all of the plugins that load between this index and
        // the previous plugin are masters of this plugin.
        if plugins
            .iter()
            .skip(previous_master_pos + 1)
            .any(|p| !master_names.contains(&p.name().to_lowercase()))
        {
            return Err(Error::NonMasterBeforeMaster);
        }

        // Check that none of the non-masters that load after index are
        // masters of this plugin.
        if let Some(p) = self
            .plugins()
            .iter()
            .skip(index)
            .filter(|p| !p.is_master_file())
            .find(|p| master_names.contains(&p.name().to_lowercase()))
        {
            Err(Error::UnrepresentedHoist(
                p.name().to_string(),
                plugin.name().to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_non_master_file_index(&self, plugin: &Plugin, index: usize) -> Result<(), Error> {
        // Check that there aren't any earlier master files that have this
        // plugin as a master.
        for master_file in self
            .plugins()
            .iter()
            .take(index)
            .filter(|p| p.is_master_file())
        {
            if master_file
                .masters()?
                .iter()
                .any(|m| plugin.name_matches(&m))
            {
                return Err(Error::UnrepresentedHoist(
                    plugin.name().to_string(),
                    master_file.name().to_string(),
                ));
            }
        }

        // Check that the next master file has this plugin as a master.
        let next_master_pos = match self
            .plugins()
            .iter()
            .skip(index)
            .position(|p| p.is_master_file())
        {
            None => return Ok(()),
            Some(i) => index + i,
        };

        if self.plugins()[next_master_pos]
            .masters()?
            .iter()
            .any(|m| plugin.name_matches(&m))
        {
            Ok(())
        } else {
            Err(Error::NonMasterBeforeMaster)
        }
    }

    fn map_to_plugins(&self, plugin_names: &[&str]) -> Result<Vec<Plugin>, Error> {
        plugin_names
            .par_iter()
            .map(|n| to_plugin(n, self.plugins(), self.game_settings()))
            .collect()
    }

    fn lookup_plugins(&mut self, active_plugin_names: &[&str]) -> Result<Vec<usize>, Error> {
        let (existing_plugin_indices, new_plugin_names): (Vec<usize>, Vec<&str>) =
            active_plugin_names.into_par_iter().partition_map(|n| {
                match self
                    .plugins()
                    .par_iter()
                    .position_any(|p| p.name_matches(n))
                {
                    Some(x) => Either::Left(x),
                    None => Either::Right(n),
                }
            });

        if new_plugin_names.is_empty() {
            Ok(existing_plugin_indices)
        } else {
            Err(Error::PluginNotFound(new_plugin_names[0].to_string()))
        }
    }

    fn count_normal_plugins(&mut self, existing_plugin_indices: &[usize]) -> usize {
        count_plugins(self.plugins(), existing_plugin_indices, false)
    }

    fn count_light_masters(&mut self, existing_plugin_indices: &[usize]) -> usize {
        if self.game_settings().id().supports_light_masters() {
            count_plugins(self.plugins(), existing_plugin_indices, true)
        } else {
            0
        }
    }
}

fn to_plugin(
    plugin_name: &str,
    existing_plugins: &[Plugin],
    game_settings: &GameSettings,
) -> Result<Plugin, Error> {
    let existing_plugin = existing_plugins
        .par_iter()
        .find_any(|p| p.name_matches(plugin_name));

    match existing_plugin {
        None => Plugin::new(plugin_name, game_settings),
        Some(x) => Ok(x.clone()),
    }
}

fn count_plugins(
    existing_plugins: &[Plugin],
    existing_plugin_indices: &[usize],
    count_light_masters: bool,
) -> usize {
    existing_plugin_indices
        .iter()
        .filter(|i| existing_plugins[**i].is_light_master_file() == count_light_masters)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use tempfile::tempdir;

    use enums::GameId;
    use load_order::tests::{mock_game_files, set_master_flag};
    use tests::copy_to_test_dir;

    struct TestLoadOrder {
        game_settings: GameSettings,
        plugins: Vec<Plugin>,
    }

    impl ReadableLoadOrder for TestLoadOrder {
        fn game_settings(&self) -> &GameSettings {
            &self.game_settings
        }

        fn plugin_names(&self) -> Vec<&str> {
            plugin_names(&self.plugins)
        }

        fn index_of(&self, plugin_name: &str) -> Option<usize> {
            index_of(&self.plugins, plugin_name)
        }

        fn plugin_at(&self, index: usize) -> Option<&str> {
            plugin_at(&self.plugins, index)
        }

        fn active_plugin_names(&self) -> Vec<&str> {
            active_plugin_names(&self.plugins)
        }

        fn is_active(&self, plugin_name: &str) -> bool {
            is_active(&self.plugins, plugin_name)
        }
    }

    impl ReadableLoadOrderExt for TestLoadOrder {
        fn plugins(&self) -> &Vec<Plugin> {
            &self.plugins
        }
    }

    fn prepare(game_dir: &Path) -> TestLoadOrder {
        let (game_settings, plugins) = mock_game_files(GameId::Oblivion, game_dir);
        TestLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn prepare_hoisted(game_path: &Path) -> TestLoadOrder {
        let load_order = prepare(game_path);

        let plugins_dir = &load_order.game_settings().plugins_directory();
        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm",
            load_order.game_settings(),
        );
        set_master_flag(&plugins_dir.join("Blank - Different.esm"), false).unwrap();
        copy_to_test_dir(
            "Blank - Different Master Dependent.esm",
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        );

        load_order
    }

    fn prepare_with_ghosted_plugin(game_dir: &Path) -> TestLoadOrder {
        let (game_settings, mut plugins) = mock_game_files(GameId::Oblivion, game_dir);

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm.ghost",
            &game_settings,
        );
        plugins.insert(
            1,
            Plugin::new("Blank - Different.esm.ghost", &game_settings).unwrap(),
        );

        TestLoadOrder {
            game_settings,
            plugins,
        }
    }

    #[test]
    fn plugin_names_should_return_filenames_for_plugins_in_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        let expected_plugin_names = vec!["Oblivion.esm", "Blank.esp", "Blank - Different.esp"];
        assert_eq!(expected_plugin_names, load_order.plugin_names());
    }

    #[test]
    fn plugin_names_should_return_unghosted_filenames() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare_with_ghosted_plugin(&tmp_dir.path());

        let expected_plugin_names = vec![
            "Oblivion.esm",
            "Blank - Different.esm",
            "Blank.esp",
            "Blank - Different.esp",
        ];
        assert_eq!(expected_plugin_names, load_order.plugin_names());
    }

    #[test]
    fn index_of_should_return_none_if_the_plugin_is_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.index_of("Blank.esm").is_none());
    }

    #[test]
    fn index_of_should_return_some_index_if_the_plugin_is_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert_eq!(1, load_order.index_of("Blank.esp").unwrap());
    }

    #[test]
    fn index_of_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert_eq!(1, load_order.index_of("blank.esp").unwrap());
    }

    #[test]
    fn plugin_at_should_return_none_if_given_an_out_of_bounds_index() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.plugin_at(3).is_none());
    }

    #[test]
    fn plugin_at_should_return_some_filename_if_given_an_in_bounds_index() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert_eq!("Blank.esp", load_order.plugin_at(1).unwrap());
    }

    #[test]
    fn plugin_at_should_return_some_unghosted_filename() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare_with_ghosted_plugin(&tmp_dir.path());

        assert_eq!("Blank - Different.esm", load_order.plugin_at(1).unwrap());
    }

    #[test]
    fn active_plugin_names_should_return_filenames_for_active_plugins_in_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        let expected_plugin_names = vec!["Blank.esp"];
        assert_eq!(expected_plugin_names, load_order.active_plugin_names());
    }

    #[test]
    fn is_active_should_return_false_for_an_inactive_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn is_active_should_return_false_a_plugin_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(!load_order.is_active("missing.esp"));
    }

    #[test]
    fn is_active_should_return_true_for_an_active_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.is_active("Blank.esp"));
    }

    #[test]
    fn is_active_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.is_active("blank.esp"));
    }

    #[test]
    fn validate_index_should_succeed_for_a_master_plugin_and_index_directly_after_a_master() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 1).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_for_a_master_plugin_and_index_after_a_hoisted_non_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(&tmp_dir.path());

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
        let mut load_order = prepare_hoisted(&tmp_dir.path());

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new("Blank.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 4).is_err());
    }

    #[test]
    fn validate_index_should_error_for_a_master_plugin_that_has_a_later_non_master_as_a_master() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(&tmp_dir.path());

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
    fn validate_index_should_succeed_for_a_non_master_plugin_and_an_index_with_no_later_masters() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(&tmp_dir.path());

        let plugin =
            Plugin::new("Blank - Master Dependent.esp", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_ok());
    }

    #[test]
    fn validate_index_should_succeed_for_a_non_master_plugin_that_is_a_master_of_the_next_master_file(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(&tmp_dir.path());

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
        let load_order = prepare(&tmp_dir.path());

        let plugin =
            Plugin::new("Blank - Master Dependent.esp", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 0).is_err());
    }

    #[test]
    fn validate_index_should_error_for_a_non_master_plugin_and_an_index_not_before_a_master_that_depends_on_it(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_hoisted(&tmp_dir.path());

        let plugin = Plugin::new(
            "Blank - Different Master Dependent.esm",
            load_order.game_settings(),
        )
        .unwrap();
        load_order.plugins.insert(1, plugin);

        let plugin = Plugin::new("Blank - Different.esm", load_order.game_settings()).unwrap();
        assert!(load_order.validate_index(&plugin, 2).is_err());
    }
}

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

use super::find_first_non_master_position;
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

    fn validate_index(&self, index: usize, is_master: bool) -> Result<(), Error> {
        match find_first_non_master_position(self.plugins()) {
            None if !is_master && index < self.plugins().len() => Err(Error::NonMasterBeforeMaster),
            Some(i) if is_master && index > i || !is_master && index < i => {
                Err(Error::NonMasterBeforeMaster)
            }
            _ => Ok(()),
        }
    }

    fn map_to_plugins(&self, plugin_names: &[&str]) -> Result<Vec<Plugin>, Error> {
        plugin_names
            .par_iter()
            .map(|n| to_plugin(n, self.plugins(), self.game_settings()))
            .collect()
    }

    fn lookup_plugins(
        &mut self,
        active_plugin_names: &[&str],
    ) -> Result<(Vec<usize>, Vec<Plugin>), Error> {
        let (existing_plugin_indices, new_plugin_names): (Vec<usize>, Vec<&str>) =
            active_plugin_names.into_par_iter().partition_map(|n| {
                match self.plugins()
                    .par_iter()
                    .position_any(|p| p.name_matches(n))
                {
                    Some(x) => Either::Left(x),
                    None => Either::Right(n),
                }
            });

        let new_plugins = new_plugin_names
            .into_par_iter()
            .map(|n| {
                Plugin::new(n, self.game_settings())
                    .map_err(|_| Error::InvalidPlugin(n.to_string()))
            })
            .collect::<Result<Vec<Plugin>, Error>>()?;

        Ok((existing_plugin_indices, new_plugins))
    }

    fn count_normal_plugins(
        &mut self,
        existing_plugin_indices: &[usize],
        new_plugins: &[Plugin],
    ) -> usize {
        count_plugins(self.plugins(), existing_plugin_indices, new_plugins, false)
    }

    fn count_light_masters(
        &mut self,
        existing_plugin_indices: &[usize],
        new_plugins: &[Plugin],
    ) -> usize {
        if self.game_settings().id().supports_light_masters() {
            count_plugins(self.plugins(), existing_plugin_indices, new_plugins, true)
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
    new_plugins: &[Plugin],
    count_light_masters: bool,
) -> usize {
    let new_count = new_plugins
        .iter()
        .filter(|p| p.is_light_master_file() == count_light_masters)
        .count();

    let existing_count = existing_plugin_indices
        .into_iter()
        .filter(|i| existing_plugins[**i].is_light_master_file() == count_light_masters)
        .count();

    new_count + existing_count
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use tempfile::tempdir;

    use enums::GameId;
    use load_order::tests::mock_game_files;
    use tests::copy_to_test_dir;

    fn prepare(game_dir: &Path) -> Vec<Plugin> {
        let (_, plugins) = mock_game_files(GameId::Oblivion, game_dir);

        plugins
    }

    fn prepare_with_ghosted_plugin(game_dir: &Path) -> Vec<Plugin> {
        let (settings, mut plugins) = mock_game_files(GameId::Oblivion, game_dir);

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm.ghost",
            &settings,
        );
        plugins.insert(
            1,
            Plugin::new("Blank - Different.esm.ghost", &settings).unwrap(),
        );

        plugins
    }

    #[test]
    fn plugin_names_should_return_filenames_for_plugins_in_load_order() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        let expected_plugin_names = vec!["Oblivion.esm", "Blank.esp", "Blank - Different.esp"];
        assert_eq!(expected_plugin_names, plugin_names(&plugins));
    }

    #[test]
    fn plugin_names_should_return_unghosted_filenames() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare_with_ghosted_plugin(&tmp_dir.path());

        let expected_plugin_names = vec![
            "Oblivion.esm",
            "Blank - Different.esm",
            "Blank.esp",
            "Blank - Different.esp",
        ];
        assert_eq!(expected_plugin_names, plugin_names(&plugins));
    }

    #[test]
    fn index_of_should_return_none_if_the_plugin_is_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert!(index_of(&plugins, "Blank.esm").is_none());
    }

    #[test]
    fn index_of_should_return_some_index_if_the_plugin_is_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert_eq!(1, index_of(&plugins, "Blank.esp").unwrap());
    }

    #[test]
    fn index_of_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert_eq!(1, index_of(&plugins, "blank.esp").unwrap());
    }

    #[test]
    fn plugin_at_should_return_none_if_given_an_out_of_bounds_index() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert!(plugin_at(&plugins, 3).is_none());
    }

    #[test]
    fn plugin_at_should_return_some_filename_if_given_an_in_bounds_index() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert_eq!("Blank.esp", plugin_at(&plugins, 1).unwrap());
    }

    #[test]
    fn plugin_at_should_return_some_unghosted_filename() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare_with_ghosted_plugin(&tmp_dir.path());

        assert_eq!("Blank - Different.esm", plugin_at(&plugins, 1).unwrap());
    }

    #[test]
    fn active_plugin_names_should_return_filenames_for_active_plugins_in_load_order() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        let expected_plugin_names = vec!["Blank.esp"];
        assert_eq!(expected_plugin_names, active_plugin_names(&plugins));
    }

    #[test]
    fn is_active_should_return_false_for_an_inactive_plugin() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert!(!is_active(&plugins, "Blank - Different.esp"));
    }

    #[test]
    fn is_active_should_return_false_a_plugin_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert!(!is_active(&plugins, "missing.esp"));
    }

    #[test]
    fn is_active_should_return_true_for_an_active_plugin() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert!(is_active(&plugins, "Blank.esp"));
    }

    #[test]
    fn is_active_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let plugins = prepare(&tmp_dir.path());

        assert!(is_active(&plugins, "blank.esp"));
    }
}

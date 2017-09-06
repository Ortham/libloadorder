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

use plugin::Plugin;
use load_order::match_plugin;

pub trait ReadableLoadOrder {
    fn plugins(&self) -> &Vec<Plugin>;

    fn plugin_names(&self) -> Vec<String> {
        self.plugins()
            .iter()
            .filter_map(|p| p.unghosted_name())
            .collect()
    }

    fn index_of(&self, plugin_name: &str) -> Option<usize> {
        self.plugins().iter().position(
            |p| match_plugin(p, plugin_name),
        )
    }

    fn plugin_at(&self, index: usize) -> Option<String> {
        if index >= self.plugins().len() {
            None
        } else {
            self.plugins()[index].unghosted_name()
        }
    }

    fn active_plugin_names(&self) -> Vec<String> {
        self.plugins()
            .iter()
            .filter_map(|p| if p.is_active() { p.name() } else { None })
            .collect()
    }

    fn is_active(&self, plugin_name: &str) -> bool {
        self.plugins()
            .iter()
            .find(|p| match_plugin(p, plugin_name))
            .map_or(false, |p| p.is_active())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;
    use tempdir::TempDir;
    use enums::GameId;
    use load_order::tests::mock_game_files;
    use tests::copy_to_test_dir;

    struct TestLoadOrder {
        plugins: Vec<Plugin>,
    }

    impl ReadableLoadOrder for TestLoadOrder {
        fn plugins(&self) -> &Vec<Plugin> {
            &self.plugins
        }
    }

    fn prepare(game_dir: &Path) -> TestLoadOrder {
        let (_, plugins) = mock_game_files(GameId::Oblivion, game_dir);
        TestLoadOrder { plugins }
    }

    fn prepare_with_ghosted_plugin(game_dir: &Path) -> TestLoadOrder {
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

        TestLoadOrder { plugins }
    }

    #[test]
    fn plugin_names_should_return_filenames_for_plugins_in_load_order() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        let expected_plugin_names = vec!["Oblivion.esm", "Blank.esp", "Blank - Different.esp"];
        assert_eq!(expected_plugin_names, load_order.plugin_names());
    }

    #[test]
    fn plugin_names_should_return_unghosted_filenames() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
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
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.index_of("Blank.esm").is_none());
    }

    #[test]
    fn index_of_should_return_some_index_if_the_plugin_is_in_the_load_order() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert_eq!(1, load_order.index_of("blank.esp").unwrap());
    }

    #[test]
    fn plugin_at_should_return_none_if_given_an_out_of_bounds_index() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.plugin_at(3).is_none());
    }

    #[test]
    fn plugin_at_should_return_some_filename_if_given_an_in_bounds_index() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert_eq!("Blank.esp", load_order.plugin_at(1).unwrap());
    }

    #[test]
    fn plugin_at_should_return_some_unghosted_filename() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare_with_ghosted_plugin(&tmp_dir.path());

        assert_eq!("Blank - Different.esm", load_order.plugin_at(1).unwrap());
    }

    #[test]
    fn active_plugin_names_should_return_filenames_for_active_plugins_in_load_order() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        let expected_plugin_names = vec!["Blank.esp"];
        assert_eq!(expected_plugin_names, load_order.active_plugin_names());
    }

    #[test]
    fn is_active_should_return_false_for_an_inactive_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn is_active_should_return_true_for_an_active_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(&tmp_dir.path());

        assert!(load_order.is_active("Blank.esp"));
    }
}

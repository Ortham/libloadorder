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
use crate::game_settings::GameSettings;
use crate::plugin::Plugin;

pub trait ReadableLoadOrderBase {
    fn plugins(&self) -> &[Plugin];

    fn game_settings_base(&self) -> &GameSettings;
}

pub trait ReadableLoadOrder {
    fn game_settings(&self) -> &GameSettings;

    fn plugin_names(&self) -> Vec<&str>;

    fn index_of(&self, plugin_name: &str) -> Option<usize>;

    fn plugin_at(&self, index: usize) -> Option<&str>;

    fn active_plugin_names(&self) -> Vec<&str>;

    fn is_active(&self, plugin_name: &str) -> bool;
}

impl<T: ReadableLoadOrderBase> ReadableLoadOrder for T {
    fn game_settings(&self) -> &GameSettings {
        self.game_settings_base()
    }

    fn plugin_names(&self) -> Vec<&str> {
        self.plugins().iter().map(Plugin::name).collect()
    }

    fn index_of(&self, plugin_name: &str) -> Option<usize> {
        self.plugins()
            .iter()
            .position(|p| p.name_matches(plugin_name))
    }

    fn plugin_at(&self, index: usize) -> Option<&str> {
        self.plugins().get(index).map(Plugin::name)
    }

    fn active_plugin_names(&self) -> Vec<&str> {
        self.plugins()
            .iter()
            .filter(|p| p.is_active())
            .map(Plugin::name)
            .collect()
    }

    fn is_active(&self, plugin_name: &str) -> bool {
        self.plugins()
            .iter()
            .find(|p| p.name_matches(plugin_name))
            .is_some_and(|p| p.is_active())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use tempfile::tempdir;

    use crate::enums::GameId;
    use crate::load_order::tests::{game_settings_for_test, mock_game_files};
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

    fn prepare(game_dir: &Path) -> TestLoadOrder {
        let mut game_settings = game_settings_for_test(GameId::Oblivion, game_dir);
        mock_game_files(&mut game_settings);

        let plugins = vec![
            Plugin::with_active("Blank.esp", &game_settings, true).unwrap(),
            Plugin::new("Blank - Different.esp", &game_settings).unwrap(),
        ];

        TestLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn prepare_with_ghosted_plugin(game_dir: &Path) -> TestLoadOrder {
        let mut load_order = prepare(game_dir);

        copy_to_test_dir(
            "Blank - Different.esm",
            "Blank - Different.esm.ghost",
            &load_order.game_settings,
        );
        load_order.plugins.insert(
            0,
            Plugin::new("Blank - Different.esm.ghost", &load_order.game_settings).unwrap(),
        );

        load_order
    }

    #[test]
    fn plugin_names_should_return_filenames_for_plugins_in_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let expected_plugin_names = vec!["Blank.esp", "Blank - Different.esp"];
        assert_eq!(expected_plugin_names, load_order.plugin_names());
    }

    #[test]
    fn plugin_names_should_return_unghosted_filenames() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare_with_ghosted_plugin(tmp_dir.path());

        let expected_plugin_names = vec![
            "Blank - Different.esm",
            "Blank.esp",
            "Blank - Different.esp",
        ];
        assert_eq!(expected_plugin_names, load_order.plugin_names());
    }

    #[test]
    fn index_of_should_return_none_if_the_plugin_is_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(load_order.index_of("Blank.esm").is_none());
    }

    #[test]
    fn index_of_should_return_some_index_if_the_plugin_is_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert_eq!(0, load_order.index_of("Blank.esp").unwrap());
    }

    #[test]
    fn index_of_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert_eq!(0, load_order.index_of("blank.esp").unwrap());
    }

    #[test]
    fn plugin_at_should_return_none_if_given_an_out_of_bounds_index() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(load_order.plugin_at(3).is_none());
    }

    #[test]
    fn plugin_at_should_return_some_filename_if_given_an_in_bounds_index() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert_eq!("Blank.esp", load_order.plugin_at(0).unwrap());
    }

    #[test]
    fn plugin_at_should_return_some_unghosted_filename() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare_with_ghosted_plugin(tmp_dir.path());

        assert_eq!("Blank - Different.esm", load_order.plugin_at(0).unwrap());
    }

    #[test]
    fn active_plugin_names_should_return_filenames_for_active_plugins_in_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let expected_plugin_names = vec!["Blank.esp"];
        assert_eq!(expected_plugin_names, load_order.active_plugin_names());
    }

    #[test]
    fn is_active_should_return_false_for_an_inactive_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn is_active_should_return_false_a_plugin_not_in_the_load_order() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(!load_order.is_active("missing.esp"));
    }

    #[test]
    fn is_active_should_return_true_for_an_active_plugin() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(load_order.is_active("Blank.esp"));
    }

    #[test]
    fn is_active_should_be_case_insensitive() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(load_order.is_active("blank.esp"));
    }
}

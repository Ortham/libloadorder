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

use unicase::eq;

use enums::{Error, GameId};
use load_order::mutable::{MAX_ACTIVE_LIGHT_MASTERS, MAX_ACTIVE_NORMAL_PLUGINS, MutableLoadOrder};
use load_order::readable::ReadableLoadOrder;
use plugin::{iends_with_ascii, Plugin};

pub trait WritableLoadOrder: ReadableLoadOrder + MutableLoadOrder {
    fn load(&mut self) -> Result<(), Error>;
    fn save(&mut self) -> Result<(), Error>;

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), Error>;

    fn set_plugin_index(&mut self, plugin_name: &str, position: usize) -> Result<(), Error>;

    fn is_self_consistent(&self) -> Result<bool, Error>;

    fn activate(&mut self, plugin_name: &str) -> Result<(), Error> {
        if self.index_of(plugin_name).is_none() {
            if !Plugin::is_valid(plugin_name, self.game_settings()) {
                return Err(Error::InvalidPlugin(plugin_name.to_string()));
            }

            self.add_to_load_order(plugin_name)?;
        }

        let at_max_active_normal_plugins = self.count_active_normal_plugins() ==
            MAX_ACTIVE_NORMAL_PLUGINS;
        let at_max_active_light_masters = self.count_active_light_masters() ==
            MAX_ACTIVE_LIGHT_MASTERS;

        let plugin = self.find_plugin_mut(plugin_name).ok_or(
            Error::PluginNotFound(
                plugin_name.to_string(),
            ),
        )?;

        if !plugin.is_active() &&
            ((!plugin.is_light_master_file() && at_max_active_normal_plugins) ||
                 (plugin.is_light_master_file() && at_max_active_light_masters))
        {
            Err(Error::TooManyActivePlugins)
        } else {
            plugin.activate()
        }
    }

    fn deactivate(&mut self, plugin_name: &str) -> Result<(), Error> {
        if self.game_settings().is_implicitly_active(plugin_name) {
            return Err(Error::ImplicitlyActivePlugin(plugin_name.to_string()));
        }

        self.find_plugin_mut(plugin_name)
            .ok_or(Error::PluginNotFound(plugin_name.to_string()))
            .map(|p| p.deactivate())
    }

    fn set_active_plugins(&mut self, active_plugin_names: &[&str]) -> Result<(), Error> {
        if count_normal_plugins(active_plugin_names) > MAX_ACTIVE_NORMAL_PLUGINS ||
            count_light_masters(self, active_plugin_names) > MAX_ACTIVE_LIGHT_MASTERS
        {
            return Err(Error::TooManyActivePlugins);
        }

        for plugin_name in active_plugin_names {
            if self.index_of(plugin_name).is_none() &&
                !Plugin::is_valid(plugin_name, self.game_settings())
            {
                return Err(Error::InvalidPlugin(plugin_name.to_string()));
            }
        }

        for plugin_name in self.game_settings().implicitly_active_plugins() {
            if !Plugin::is_valid(plugin_name, self.game_settings()) {
                continue;
            }

            if !active_plugin_names.iter().any(|p| eq(*p, plugin_name)) {
                return Err(Error::ImplicitlyActivePlugin(plugin_name.to_string()));
            }
        }

        for plugin in self.plugins_mut() {
            plugin.deactivate();
        }

        for plugin_name in active_plugin_names {
            let plugin_exists = self.plugins_mut().iter_mut().any(
                |p| p.name_matches(plugin_name),
            );
            if !plugin_exists {
                self.add_to_load_order(plugin_name)?;
            }

            if let Some(p) = self.find_plugin_mut(plugin_name) {
                p.activate()?;
            }
        }

        Ok(())
    }
}

fn count_normal_plugins(plugin_names: &[&str]) -> usize {
    plugin_names
        .iter()
        .filter(|p| !iends_with_ascii(p, ".esl"))
        .count()
}

fn count_light_masters<T: ReadableLoadOrder + ?Sized>(
    load_order: &T,
    plugin_names: &[&str],
) -> usize {
    match load_order.game_settings().id() {
        GameId::Fallout4 | GameId::SkyrimSE => {
            plugin_names
                .iter()
                .filter(|p| iends_with_ascii(p, ".esl"))
                .count()
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;
    use tempdir::TempDir;
    use enums::GameId;
    use game_settings::GameSettings;
    use load_order::readable::ReadableLoadOrder;
    use load_order::tests::mock_game_files;
    use tests::copy_to_test_dir;

    struct TestLoadOrder {
        game_settings: GameSettings,
        plugins: Vec<Plugin>,
    }

    impl ReadableLoadOrder for TestLoadOrder {
        fn game_settings(&self) -> &GameSettings {
            &self.game_settings
        }

        fn plugins(&self) -> &Vec<Plugin> {
            &self.plugins
        }
    }

    impl MutableLoadOrder for TestLoadOrder {
        fn insert_position(&self, plugin: &Plugin) -> Option<usize> {
            if plugin.is_master_file() {
                Some(1)
            } else {
                None
            }
        }

        fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
            &mut self.plugins
        }
    }

    impl WritableLoadOrder for TestLoadOrder {
        // Dummy method, unused.
        fn load(&mut self) -> Result<(), Error> {
            Ok(())
        }

        // Dummy method, unused.
        fn save(&mut self) -> Result<(), Error> {
            Ok(())
        }

        // Dummy method, unused.
        fn set_load_order(&mut self, _: &[&str]) -> Result<(), Error> {
            Ok(())
        }

        // Dummy method, unused.
        fn set_plugin_index(&mut self, _: &str, _: usize) -> Result<(), Error> {
            Ok(())
        }

        // Dummy method, unused.
        fn is_self_consistent(&self) -> Result<bool, Error> {
            Ok(true)
        }
    }

    fn prepare(game_id: GameId, game_dir: &Path) -> TestLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, game_dir);
        TestLoadOrder {
            game_settings,
            plugins,
        }
    }

    #[test]
    fn activate_should_activate_the_plugin_with_the_given_filename() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.activate("Blank - Different.esp").is_ok());
        assert!(load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn activate_should_error_if_the_plugin_is_not_valid() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.activate("missing.esp").is_err());
        assert!(load_order.index_of("missing.esp").is_none());
    }

    #[test]
    fn activate_should_insert_a_master_before_non_masters_if_it_is_not_present() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.activate("Blank.esm").is_ok());
        assert!(load_order.is_active("Blank.esm"));
        assert_eq!(1, load_order.index_of("Blank.esm").unwrap());
    }

    #[test]
    fn activate_should_append_a_non_master_if_it_is_not_present() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.activate("Blank - Master Dependent.esp").is_ok());
        assert!(load_order.is_active("Blank - Master Dependent.esp"));
        assert_eq!(
            3,
            load_order.index_of("Blank - Master Dependent.esp").unwrap()
        );
    }

    #[test]
    fn activate_should_be_case_insensitive() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.activate("Blank - different.esp").is_ok());
        assert!(load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn activate_should_throw_if_increasing_the_number_of_active_plugins_past_the_limit() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        for i in 0..(MAX_ACTIVE_NORMAL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank.esp", &plugin, &load_order.game_settings());
            load_order.activate(&plugin).unwrap();
        }

        assert!(load_order.activate("Blank - Different.esp").is_err());
        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn activate_should_succeed_if_at_the_active_plugins_limit_and_the_plugin_is_already_active() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        for i in 0..(MAX_ACTIVE_NORMAL_PLUGINS - 1) {
            let plugin = format!("{}.esp", i);
            copy_to_test_dir("Blank.esp", &plugin, &load_order.game_settings());
            load_order.activate(&plugin).unwrap();
        }

        assert!(load_order.is_active("Blank.esp"));
        assert!(load_order.activate("Blank.esp").is_ok());
    }

    #[test]
    fn deactivate_should_deactivate_the_plugin_with_the_given_filename() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.is_active("Blank.esp"));
        assert!(load_order.deactivate("Blank.esp").is_ok());
        assert!(!load_order.is_active("Blank.esp"));
    }

    #[test]
    fn deactivate_should_error_if_the_plugin_is_not_in_the_load_order() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert!(load_order.deactivate("missing.esp").is_err());
        assert!(load_order.index_of("missing.esp").is_none());
    }

    #[test]
    fn deactivate_should_error_if_given_an_implicitly_active_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(load_order.activate("Skyrim.esm").is_ok());
        assert!(load_order.deactivate("Skyrim.esm").is_err());
        assert!(load_order.is_active("Skyrim.esm"));
    }

    #[test]
    fn deactivate_should_error_if_given_a_missing_implicitly_active_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(load_order.deactivate("Update.esm").is_err());
        assert!(load_order.index_of("Update.esm").is_none());
    }

    #[test]
    fn deactivate_should_do_nothing_if_the_plugin_is_inactive() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(!load_order.is_active("Blank - Different.esp"));
        assert!(load_order.deactivate("Blank - Different.esp").is_ok());
        assert!(!load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn set_active_plugins_should_error_if_given_more_plugins_than_the_max_limit() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = [""; 256];
        assert!(load_order.set_active_plugins(&active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_passed_an_invalid_plugin_name() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["missing.esp"];
        assert!(load_order.set_active_plugins(&active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
fn set_active_plugins_should_error_if_the_given_plugins_are_missing_implicitly_active_plugins(){
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let active_plugins = ["Blank.esp"];
        assert!(load_order.set_active_plugins(&active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_a_missing_implicitly_active_plugin_is_given() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let active_plugins = ["Skyrim.esm", "Update.esm"];
        assert!(load_order.set_active_plugins(&active_plugins).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_deactivate_all_plugins_not_given() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["Blank - Different.esp"];
        assert!(load_order.is_active("Blank.esp"));
        assert!(load_order.set_active_plugins(&active_plugins).is_ok());
        assert!(!load_order.is_active("Blank.esp"));
    }

    #[test]
    fn set_active_plugins_should_activate_all_given_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["Blank - Different.esp"];
        assert!(!load_order.is_active("Blank - Different.esp"));
        assert!(load_order.set_active_plugins(&active_plugins).is_ok());
        assert!(load_order.is_active("Blank - Different.esp"));
    }

    #[test]
    fn set_active_plugins_should_add_given_plugins_not_in_the_load_order_in_the_given_order() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let active_plugins = ["Blank - Master Dependent.esp", "Blàñk.esp"];
        assert!(load_order.set_active_plugins(&active_plugins).is_ok());
        assert!(load_order.is_active("Blank - Master Dependent.esp"));
        assert_eq!(
            3,
            load_order.index_of("Blank - Master Dependent.esp").unwrap()
        );
        assert!(load_order.is_active("Blàñk.esp"));
        assert_eq!(4, load_order.index_of("Blàñk.esp").unwrap());
    }
}

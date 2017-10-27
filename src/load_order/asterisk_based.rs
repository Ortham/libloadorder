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
use std::fs::File;
use std::io::{BufWriter, Write};

use encoding::{Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use unicase::eq;

use enums::Error;
use game_settings::GameSettings;
use plugin::Plugin;
use load_order::{create_parent_dirs, find_first_non_master_position};
use load_order::mutable::{MutableLoadOrder, read_plugin_names};
use load_order::readable::ReadableLoadOrder;
use load_order::writable::WritableLoadOrder;

#[derive(Clone, Debug)]
pub struct AsteriskBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl AsteriskBasedLoadOrder {
    pub fn new(game_settings: GameSettings) -> AsteriskBasedLoadOrder {
        AsteriskBasedLoadOrder {
            game_settings,
            plugins: Vec::new(),
        }
    }
}

impl ReadableLoadOrder for AsteriskBasedLoadOrder {
    fn game_settings(&self) -> &GameSettings {
        &self.game_settings
    }

    fn plugins(&self) -> &Vec<Plugin> {
        &self.plugins
    }
}

impl MutableLoadOrder for AsteriskBasedLoadOrder {
    fn insert_position(&self, plugin: &Plugin) -> Option<usize> {
        if let Some(name) = plugin.name() {
            if self.game_settings().is_implicitly_active(&name) {
                let mut installed_plugin_count = 0;
                for plugin_name in self.game_settings().implicitly_active_plugins() {
                    if eq(name.as_str(), plugin_name) {
                        return Some(installed_plugin_count);
                    }

                    if self.index_of(plugin_name).is_some() ||
                        Plugin::is_valid(plugin_name, self.game_settings())
                    {
                        installed_plugin_count += 1;
                    }
                }
            }
        }

        if plugin.is_master_file() || plugin.is_light_master_file() {
            find_first_non_master_position(self.plugins())
        } else {
            None
        }
    }

    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl WritableLoadOrder for AsteriskBasedLoadOrder {
    fn load(&mut self) -> Result<(), Error> {
        self.plugins_mut().clear();

        let plugin_tuples = self.read_from_active_plugins_file()?;
        let filenames = self.find_plugins_in_dir_sorted();

        self.load_unique_plugins(plugin_tuples, filenames);

        self.add_implicitly_active_plugins()?;

        self.deactivate_excess_plugins();

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        create_parent_dirs(self.game_settings().active_plugins_file())?;

        let file = File::create(self.game_settings().active_plugins_file())?;
        let mut writer = BufWriter::new(file);
        for plugin in self.plugins() {
            let name = match plugin.unghosted_name() {
                Some(x) => x,
                None => continue,
            };

            if self.game_settings().is_implicitly_active(&name) {
                continue;
            }

            if plugin.is_active() {
                write!(writer, "*")?;
            }
            writer.write_all(&WINDOWS_1252
                .encode(&name, EncoderTrap::Strict)
                .map_err(Error::EncodeError)?)?;
            writeln!(writer, "")?;
        }

        Ok(())
    }

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        if plugin_names.is_empty() || !eq(plugin_names[0], self.game_settings().master_file()) {
            return Err(Error::GameMasterMustLoadFirst);
        }

        self.replace_plugins(plugin_names)
    }

    fn set_plugin_index(&mut self, plugin_name: &str, position: usize) -> Result<(), Error> {
        if position != 0 && !self.plugins().is_empty() &&
            eq(plugin_name, self.game_settings().master_file())
        {
            return Err(Error::GameMasterMustLoadFirst);
        }
        if position == 0 && !eq(plugin_name, self.game_settings().master_file()) {
            return Err(Error::GameMasterMustLoadFirst);
        }

        self.move_or_insert_plugin_with_index(plugin_name, position)
    }

    fn is_self_consistent(&self) -> Result<bool, Error> {
        Ok(true)
    }
}

impl AsteriskBasedLoadOrder {
    fn read_from_active_plugins_file(&self) -> Result<Vec<(String, bool)>, Error> {
        read_plugin_names(
            self.game_settings().active_plugins_file(),
            plugin_line_mapper,
        )
    }
}

fn plugin_line_mapper(line: &str) -> Option<(String, bool)> {
    if line.is_empty() || line.starts_with('#') {
        None
    } else if line.as_bytes()[0] == b'*' {
        Some((line[1..].to_owned(), true))
    } else {
        Some((line.to_owned(), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{File, remove_dir_all};
    use std::io;
    use std::io::{BufRead, BufReader};
    use std::path::Path;
    use filetime::{FileTime, set_file_times};
    use tempdir::TempDir;
    use enums::GameId;
    use load_order::tests::*;
    use tests::copy_to_test_dir;

    fn prepare(game_id: GameId, game_dir: &Path) -> AsteriskBasedLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, game_dir);
        AsteriskBasedLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn prepare_bulk_plugins(game_settings: &GameSettings) -> Vec<String> {
        let mut plugins: Vec<String> = vec![game_settings.master_file().to_string()];
        plugins.extend((0..260).map(|i| format!("Blank{}.esm", i)));
        plugins.extend((0..5000).map(|i| format!("Blank{}.esl", i)));

        for plugin in &plugins {
            copy_to_test_dir("Blank - Different.esm", &plugin, game_settings);
        }

        write_active_plugins_file(game_settings, &plugins);

        plugins
    }

    #[test]
    fn insert_position_should_return_the_hardcoded_index_of_an_implicitly_active_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        load_order.plugins_mut().insert(1, plugin);

        copy_to_test_dir("Blank.esm", "Hearthfires.esm", &load_order.game_settings());
        let plugin = Plugin::new("Hearthfires.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_none_if_given_a_non_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugin = Plugin::new("Blank - Master Dependent.esp", &load_order.game_settings())
            .unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn insert_position_should_return_the_first_non_master_plugin_index_if_given_a_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_none_if_no_non_masters_are_present() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        // Remove non-master plugins from the load order.
        load_order.plugins_mut().retain(|p| p.is_master_file());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn insert_position_should_return_the_first_non_master_index_if_given_a_light_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
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
    fn load_should_reload_existing_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        assert!(!load_order.plugins()[1].is_master_file());
        copy_to_test_dir("Blank.esm", "Blank.esp", &load_order.game_settings());
        let plugin_path = load_order.game_settings().plugins_directory().join(
            "Blank.esp",
        );
        set_file_times(&plugin_path, FileTime::zero(), FileTime::zero()).unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins()[1].is_master_file());
    }

    #[test]
    fn load_should_remove_plugins_that_fail_to_load() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        assert!(load_order.index_of("Blank.esp").is_some());
        assert!(load_order.index_of("Blank - Different.esp").is_some());

        let plugin_path = load_order.game_settings().plugins_directory().join(
            "Blank.esp",
        );
        File::create(&plugin_path).unwrap();
        set_file_times(&plugin_path, FileTime::zero(), FileTime::zero()).unwrap();

        let plugin_path = load_order.game_settings().plugins_directory().join(
            "Blank - Different.esp",
        );
        File::create(&plugin_path).unwrap();
        set_file_times(&plugin_path, FileTime::zero(), FileTime::zero()).unwrap();

        load_order.load().unwrap();
        assert!(load_order.index_of("Blank.esp").is_none());
        assert!(load_order.index_of("Blank - Different.esp").is_none());
    }

    #[test]
    fn load_should_get_load_order_from_active_plugins_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["Blank.esp", "Blank - Master Dependent.esp"],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_decode_active_plugins_file_from_windows_1252() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &["Blàñk.esp", "Blank.esm"]);

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blàñk.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_handle_crlf_and_lf_in_active_plugins_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &["Blàñk.esp", "Blank.esm\r"]);

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blàñk.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_ignore_active_plugins_file_lines_starting_with_a_hash() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["#Blank.esp", "Blàñk.esp", "Blank.esm"],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blàñk.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_ignore_plugins_in_active_plugins_file_that_are_not_installed() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["Blàñk.esp", "Blank.esm", "missing.esp"],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blàñk.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_add_missing_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        assert!(load_order.index_of("Blank.esm").is_none());
        assert!(
            load_order
                .index_of("Blank - Master Dependent.esp")
                .is_none()
        );
        assert!(load_order.index_of("Blàñk.esp").is_none());

        load_order.load().unwrap();

        assert!(load_order.index_of("Blank.esm").is_some());
        assert!(
            load_order
                .index_of("Blank - Master Dependent.esp")
                .is_some()
        );
        assert!(load_order.index_of("Blàñk.esp").is_some());
    }

    #[test]
    fn load_should_recognise_light_master_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "ccTest.esl", &load_order.game_settings());

        load_order.load().unwrap();

        assert!(load_order.plugin_names().contains(&"ccTest.esl".to_owned()));
    }

    #[test]
    fn load_should_add_missing_implicitly_active_plugins_in_their_hardcoded_positions() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());
        load_order.load().unwrap();
        assert_eq!(Some(1), load_order.index_of("Update.esm"));
        assert!(load_order.is_active("Update.esm"));
    }

    #[test]
    fn load_should_empty_the_load_order_if_the_plugins_directory_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());
        tmp_dir.close().unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins().is_empty());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &["Blàñk.esp", "Blank.esm"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Skyrim.esm", "Blank.esm", "Blàñk.esp"];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_succeed_when_active_plugins_file_is_missing() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        assert!(load_order.load().is_ok());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn load_should_deactivate_excess_normal_plugins_and_light_masters_using_separate_limits() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugins = prepare_bulk_plugins(load_order.game_settings());

        load_order.load().unwrap();
        let active_plugin_names = load_order.active_plugin_names();

        assert_eq!(4351, active_plugin_names.len());
        assert_eq!(plugins[..255], active_plugin_names[..255]);
        assert_eq!(plugins[261..4357], active_plugin_names[255..]);
    }

    #[test]
    fn load_should_not_duplicate_a_plugin_that_has_a_ghosted_duplicate() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        use std::fs::copy;

        copy(
            load_order.game_settings().plugins_directory().join(
                "Blank.esm",
            ),
            load_order.game_settings().plugins_directory().join(
                "Blank.esm.ghost",
            ),
        ).unwrap();

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
            "Blàñk.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn save_should_create_active_plugins_file_parent_directory_if_it_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        remove_dir_all(
            load_order
                .game_settings()
                .active_plugins_file()
                .parent()
                .unwrap(),
        ).unwrap();

        load_order.save().unwrap();

        assert!(
            load_order
                .game_settings()
                .active_plugins_file()
                .parent()
                .unwrap()
                .exists()
        );
    }

    #[test]
    fn save_should_write_active_plugins_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(
            vec!["Skyrim.esm", "Blank.esp"],
            load_order.active_plugin_names()
        );
    }

    #[test]
    fn save_should_write_unghosted_plugin_names() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        copy_to_test_dir(
            "Blank - Different.esm",
            "ghosted.esm.ghost",
            &load_order.game_settings(),
        );
        let plugin = Plugin::new("ghosted.esm.ghost", &load_order.game_settings()).unwrap();
        load_order.plugins_mut().push(plugin);

        load_order.save().unwrap();

        let reader = BufReader::new(
            File::open(load_order.game_settings().active_plugins_file()).unwrap(),
        );

        let lines = reader
            .lines()
            .collect::<Result<Vec<String>, io::Error>>()
            .unwrap();

        assert_eq!(
            vec!["*Blank.esp", "Blank - Different.esp", "ghosted.esm"],
            lines
        );
    }

    #[test]
    fn set_load_order_should_error_if_given_an_empty_list() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        let filenames = vec![];
        assert!(load_order.set_load_order(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_load_order_should_error_if_the_first_element_given_is_not_the_game_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        let filenames = vec!["Blank.esp"];
        assert!(load_order.set_load_order(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_load_order_should_add_and_activate_implicitly_active_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let filenames = vec![
            "Skyrim.esm",
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
        ];
        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());
        load_order.plugins_mut().remove(0); // Remove the existing Skyrim.esm entry.
        load_order.set_load_order(&filenames).unwrap();

        let expected_filenames = vec![
            "Skyrim.esm",
            "Update.esm",
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];
        assert_eq!(expected_filenames, load_order.plugin_names());
        assert!(load_order.is_active("Skyrim.esm"));
        assert!(load_order.is_active("Update.esm"));
    }

    #[test]
    fn set_plugin_index_should_error_if_setting_the_game_master_index_to_non_zero_in_bounds() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        assert!(load_order.set_plugin_index("Skyrim.esm", 1).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_setting_a_zero_index_for_a_non_game_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        assert!(load_order.set_plugin_index("Blank.esm", 0).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_insert_a_new_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let num_plugins = load_order.plugins().len();
        load_order.set_plugin_index("Blank.esm", 1).unwrap();
        assert_eq!(1, load_order.index_of("Blank.esm").unwrap());
        assert_eq!(num_plugins + 1, load_order.plugins().len());
    }

    #[test]
    fn is_self_consistent_should_return_true() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn activate_should_check_normal_plugins_and_light_masters_active_limits_separately() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugins = prepare_bulk_plugins(load_order.game_settings());

        let mut plugin_refs: Vec<&str> = plugins[..254].iter().map(AsRef::as_ref).collect();
        plugin_refs.extend(plugins[261..4356].iter().map(|s| s.as_str()));

        assert!(load_order.set_active_plugins(&plugin_refs).is_ok());

        let i = 4356;
        assert!(load_order.activate(&plugins[i]).is_ok());
        assert!(load_order.is_active(&plugins[i]));

        let i = 254;
        assert!(load_order.activate(&plugins[i]).is_ok());
        assert!(load_order.is_active(&plugins[i]));

        let i = 256;
        assert!(load_order.activate(&plugins[i]).is_err());
        assert!(!load_order.is_active(&plugins[i]));

        let i = 4357;
        assert!(load_order.activate(&plugins[i]).is_err());
        assert!(!load_order.is_active(&plugins[i]));
    }

    #[test]
    fn set_active_plugins_should_count_light_masters_and_normal_plugins_separately() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugins = prepare_bulk_plugins(load_order.game_settings());

        let mut plugin_refs: Vec<&str> = plugins[..255].iter().map(AsRef::as_ref).collect();
        plugin_refs.extend(plugins[261..4357].iter().map(|s| s.as_str()));

        assert!(load_order.set_active_plugins(&plugin_refs).is_ok());
        assert_eq!(4351, load_order.active_plugin_names().len());
    }

    #[test]
    fn set_active_plugins_should_error_if_given_more_than_4096_light_masters() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, &tmp_dir.path());

        let plugins = prepare_bulk_plugins(load_order.game_settings());

        let mut plugin_refs: Vec<&str> = plugins[..255].iter().map(AsRef::as_ref).collect();
        plugin_refs.extend(plugins[261..4358].iter().map(|s| s.as_str()));

        assert!(load_order.set_active_plugins(&plugin_refs).is_err());
        assert_eq!(1, load_order.active_plugin_names().len());
    }
}

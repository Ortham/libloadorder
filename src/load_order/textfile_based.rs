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
use std::io::Write;
use encoding::{DecoderTrap, Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use unicase::eq;

use enums::Error;
use game_settings::GameSettings;
use plugin::Plugin;
use load_order::{create_parent_dirs, find_first_non_master_position, read_plugin_names};
use load_order::mutable::{load_active_plugins, MutableLoadOrder};
use load_order::readable::ReadableLoadOrder;
use load_order::writable::WritableLoadOrder;

pub struct TextfileBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl TextfileBasedLoadOrder {
    pub fn new(game_settings: GameSettings) -> TextfileBasedLoadOrder {
        TextfileBasedLoadOrder {
            game_settings,
            plugins: Vec::new(),
        }
    }
}

impl ReadableLoadOrder for TextfileBasedLoadOrder {
    fn game_settings(&self) -> &GameSettings {
        &self.game_settings
    }

    fn plugins(&self) -> &Vec<Plugin> {
        &self.plugins
    }
}

impl MutableLoadOrder for TextfileBasedLoadOrder {
    fn insert_position(&self, plugin: &Plugin) -> Option<usize> {
        let is_game_master = plugin
            .name()
            .map(|n| eq(n.as_str(), self.game_settings().master_file()))
            .unwrap_or(false);

        if is_game_master {
            Some(0)
        } else if plugin.is_master_file() {
            find_first_non_master_position(self.plugins())
        } else {
            None
        }
    }

    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl WritableLoadOrder for TextfileBasedLoadOrder {
    fn load(&mut self) -> Result<(), Error> {
        self.reload_changed_plugins();

        let load_order_file_exists = self.game_settings()
            .load_order_file()
            .map(|p| p.exists())
            .unwrap_or(false);

        let mut active_plugins_file_parsed = false;
        if load_order_file_exists {
            load_from_load_order_file(self)?;
        } else {
            load_from_active_plugins_file(self)?;
            active_plugins_file_parsed = true;
        }

        self.add_missing_plugins()?;

        if !active_plugins_file_parsed {
            load_active_plugins(self, active_plugin_line_mapper)?;
        }

        self.add_implicitly_active_plugins()?;

        self.deactivate_excess_plugins();

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        save_load_order(self)?;
        save_active_plugins(self)
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
}

fn load_from_load_order_file<T: MutableLoadOrder>(load_order: &mut T) -> Result<(), Error> {
    let line_mapper = |l| String::from_utf8(l).map_err(Error::from);

    let plugin_names = if let Some(file_path) = load_order.game_settings().load_order_file() {
        read_plugin_names(file_path, line_mapper)?
    } else {
        Vec::new()
    };
    for plugin_name in plugin_names {
        load_order.move_or_insert_plugin_if_valid(&plugin_name)?;
    }

    Ok(())
}

fn load_from_active_plugins_file<T: MutableLoadOrder>(load_order: &mut T) -> Result<(), Error> {
    load_order.deactivate_all();

    let plugin_names = read_plugin_names(
        load_order.game_settings().active_plugins_file(),
        active_plugin_line_mapper,
    )?;

    for plugin_name in plugin_names {
        load_order.move_or_insert_plugin_if_valid(&plugin_name)?;
        load_order.activate_unvalidated(&plugin_name)?;
    }

    Ok(())
}

fn active_plugin_line_mapper(line: Vec<u8>) -> Result<String, Error> {
    WINDOWS_1252.decode(&line, DecoderTrap::Strict).map_err(
        |e| {
            Error::DecodeError(e)
        },
    )
}

fn save_load_order<T: MutableLoadOrder>(load_order: &mut T) -> Result<(), Error> {
    if let Some(file_path) = load_order.game_settings().load_order_file() {
        create_parent_dirs(file_path)?;

        let mut file = File::create(file_path)?;
        for plugin_name in load_order.plugin_names() {
            writeln!(file, "{}", plugin_name)?;
        }
    }
    Ok(())
}

fn save_active_plugins<T: MutableLoadOrder>(load_order: &mut T) -> Result<(), Error> {
    create_parent_dirs(load_order.game_settings().active_plugins_file())?;

    let mut file = File::create(load_order.game_settings().active_plugins_file())?;
    for plugin_name in load_order.active_plugin_names() {
        file.write_all(&WINDOWS_1252
            .encode(&plugin_name, EncoderTrap::Strict)
            .map_err(Error::EncodeError)?)?;
        writeln!(file, "")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{File, remove_dir_all, remove_file};
    use std::io::Write;
    use std::path::Path;
    use filetime::{FileTime, set_file_times};
    use tempdir::TempDir;
    use enums::GameId;
    use load_order::tests::*;
    use tests::copy_to_test_dir;

    fn prepare(game_id: GameId, game_dir: &Path) -> TextfileBasedLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, game_dir);
        TextfileBasedLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn write_file(path: &Path) {
        let mut file = File::create(&path).unwrap();
        writeln!(file, "").unwrap();
    }

    #[test]
    fn insert_position_should_return_zero_if_given_the_game_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let plugin = Plugin::new("Skyrim.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(0, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_none_if_given_a_non_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let plugin = Plugin::new("Blank - Master Dependent.esp", &load_order.game_settings())
            .unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn insert_position_should_return_the_first_non_master_plugin_index_if_given_a_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_none_if_no_non_masters_are_present() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        // Remove non-master plugins from the load order.
        load_order.plugins_mut().retain(|p| p.is_master_file());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn load_should_reload_changed_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

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
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        assert!(load_order.index_of("Blank.esp").is_some());
        assert!(load_order.index_of("Blank - Different.esp").is_some());

        let plugin_path = load_order.game_settings().plugins_directory().join(
            "Blank.esp",
        );
        write_file(&plugin_path);
        set_file_times(&plugin_path, FileTime::zero(), FileTime::zero()).unwrap();

        let plugin_path = load_order.game_settings().plugins_directory().join(
            "Blank - Different.esp",
        );
        write_file(&plugin_path);
        set_file_times(&plugin_path, FileTime::zero(), FileTime::zero()).unwrap();

        load_order.load().unwrap();
        assert!(load_order.index_of("Blank.esp").is_none());
        assert!(load_order.index_of("Blank - Different.esp").is_none());
    }

    #[test]
    fn load_should_get_load_order_from_load_order_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let expected_filenames = vec![
            "Skyrim.esm",
            "Blank.esm",
            "Blàñk.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "missing.esp",
        ];
        write_load_order_file(load_order.game_settings(), &expected_filenames);

        load_order.load().unwrap();
        assert_eq!(
            &expected_filenames[..6],
            load_order.plugin_names().as_slice()
        );
    }

    #[test]
    fn load_should_get_load_order_from_active_plugins_file_if_load_order_file_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        load_order.load().unwrap();

        let expected_filenames = vec![
            load_order.game_settings().master_file(),
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
            "Blank - Master Dependent.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_add_missing_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

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
    fn load_should_add_missing_implicitly_active_plugins_after_other_missing_masters() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());
        load_order.load().unwrap();
        assert_eq!(Some(2), load_order.index_of("Update.esm"));
        assert!(load_order.is_active("Update.esm"));
    }

    #[test]
    fn load_should_empty_the_load_order_if_the_plugins_directory_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());
        tmp_dir.close().unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins().is_empty());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        load_order.load().unwrap();
        let expected_filenames = vec!["Skyrim.esm", "Blank.esm", "Blàñk.esp"];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_succeed_when_load_order_and_active_plugins_files_are_missing() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        remove_file(load_order.game_settings().active_plugins_file()).unwrap();

        assert!(load_order.load().is_ok());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn load_should_deactivate_excess_plugins_not_including_implicitly_active_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let mut plugins: Vec<String> = Vec::new();
        plugins.push(load_order.game_settings().master_file().to_string());
        for i in 0..260 {
            plugins.push(format!("Blank{}.esm", i));
            copy_to_test_dir(
                "Blank.esm",
                &plugins.last().unwrap(),
                load_order.game_settings(),
            );
        }
        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());

        {
            let plugins_as_ref: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();
            write_active_plugins_file(load_order.game_settings(), &plugins_as_ref);
            set_timestamps(
                &load_order.game_settings().plugins_directory(),
                &plugins_as_ref,
            );
        }

        plugins = plugins[0..254].to_vec();
        plugins.push("Update.esm".to_string());

        load_order.load().unwrap();
        let active_plugin_names = load_order.active_plugin_names();

        assert_eq!(255, active_plugin_names.len());
        for i in 0..255 {
            assert_eq!(plugins[i], active_plugin_names[i]);
        }
        assert_eq!(plugins, active_plugin_names);
    }

    #[test]
    fn save_should_write_all_plugins_to_load_order_file() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        load_order.save().unwrap();

        let expected_filenames = vec!["Skyrim.esm", "Blank.esp", "Blank - Different.esp"];
        let line_mapper = |l| String::from_utf8(l).map_err(Error::from);
        let plugin_names = read_plugin_names(
            load_order.game_settings().load_order_file().unwrap(),
            line_mapper,
        ).unwrap();
        assert_eq!(expected_filenames, plugin_names);
    }

    #[test]
    fn save_should_create_active_plugins_file_parent_directory_if_it_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

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
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(
            vec!["Skyrim.esm", "Blank.esp"],
            load_order.active_plugin_names()
        );
    }

    #[test]
    fn set_load_order_should_error_if_given_an_empty_list() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        let filenames = vec![];
        assert!(load_order.set_load_order(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_load_order_should_error_if_the_first_element_given_is_not_the_game_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        let filenames = vec!["Blank.esp"];
        assert!(load_order.set_load_order(&filenames).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_load_order_should_add_and_activate_implicitly_active_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

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
            "Blank.esm",
            "Update.esm",
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
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        assert!(load_order.set_plugin_index("Skyrim.esm", 1).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_error_if_setting_a_zero_index_for_a_non_game_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        let existing_filenames = load_order.plugin_names();
        assert!(load_order.set_plugin_index("Blank.esm", 0).is_err());
        assert_eq!(existing_filenames, load_order.plugin_names());
    }

    #[test]
    fn set_plugin_index_should_insert_a_new_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let num_plugins = load_order.plugins().len();
        load_order.set_plugin_index("Blank.esm", 1).unwrap();
        assert_eq!(1, load_order.index_of("Blank.esm").unwrap());
        assert_eq!(num_plugins + 1, load_order.plugins().len());
    }
}

/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2017 Oliver Hamlet
 *
 * libespm is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * libespm is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with libespm. If not, see <http://www.gnu.org/licenses/>.
 */
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufReader, BufRead};
use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_1252;
use regex::bytes::Regex;

use enums::GameId;
use game_settings::GameSettings;
use plugin::Plugin;
use super::error::LoadOrderError;
use super::readable::ReadableLoadOrder;
use super::writable::ExtensibleLoadOrder;
use super::writable::MutableLoadOrder;
use super::reload_changed_plugins;
use super::find_first_non_master_position;

struct TimestampBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl ReadableLoadOrder for TimestampBasedLoadOrder {
    fn plugins(&self) -> &Vec<Plugin> {
        &self.plugins
    }
}

impl ExtensibleLoadOrder for TimestampBasedLoadOrder {
    fn insert_position(&self, plugin: &Plugin) -> Option<usize> {
        if plugin.is_master_file() {
            find_first_non_master_position(self.plugins())
        } else {
            None
        }
    }

    fn game_settings(&self) -> &GameSettings {
        &self.game_settings
    }

    fn mut_plugins(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl MutableLoadOrder for TimestampBasedLoadOrder {
    fn load(&mut self) -> Result<(), LoadOrderError> {
        reload_changed_plugins(self.mut_plugins());

        //TODO: Profile vs. C++ libloadorder to see if caching plugins folder timestamp is worth it
        self.add_missing_plugins()?;

        load_active_plugins(self)?;

        self.add_implicitly_active_plugins()?;

        self.mut_plugins().sort_by(|a, b| if a.is_master_file() ==
            b.is_master_file()
        {
            a.modification_time().cmp(&b.modification_time())
        } else if a.is_master_file() {
            Ordering::Less
        } else {
            Ordering::Greater
        });

        self.deactivate_excess_plugins();

        Ok(())
    }

    fn save(&mut self) -> Result<(), LoadOrderError> {
        unimplemented!();
    }

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), LoadOrderError> {
        unimplemented!();
    }

    fn set_plugin_index(
        &mut self,
        plugin_name: &str,
        position: usize,
    ) -> Result<(), LoadOrderError> {
        unimplemented!();
    }
}

fn load_active_plugins<T: ExtensibleLoadOrder>(load_order: &mut T) -> Result<(), LoadOrderError> {
    for plugin in load_order.mut_plugins() {
        plugin.deactivate();
    }

    if !load_order.game_settings().active_plugins_file().exists() {
        return Ok(());
    }

    let input = File::open(load_order.game_settings().active_plugins_file())?;
    let buffered = BufReader::new(input);

    const CARRIAGE_RETURN: u8 = b'\r';
    let regex = Regex::new(r"(?i-u)GameFile[0-9]{1,3}=(.+\.es(?:m|p))")?;
    for line in buffered.split(b'\n') {
        let mut line = line?;
        if line.last().unwrap_or(&CARRIAGE_RETURN) == &CARRIAGE_RETURN {
            line.pop();
        }
        if *load_order.game_settings().id() == GameId::Morrowind {
            line = regex.captures(&line).and_then(|c| c.get(1)).map_or(
                Vec::new(),
                |m| {
                    m.as_bytes().to_vec()
                },
            )
        }
        if line.is_empty() || line[0] == b'#' {
            continue;
        }

        let line = WINDOWS_1252.decode(&line, DecoderTrap::Strict)?;

        let index = load_order.find_or_add(&line)?;
        load_order.mut_plugins()[index].activate()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::*;

    use std::path::Path;
    use self::tempdir::TempDir;
    use enums::GameId;
    use filetime::{FileTime, set_file_times};
    use load_order::tests::*;
    use tests::copy_to_test_dir;

    fn prepare(game_id: GameId, game_dir: &Path) -> TimestampBasedLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, &game_dir);
        TimestampBasedLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn write_file(path: &Path) {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(&path).unwrap();
        writeln!(file, "").unwrap();
    }

    #[test]
    fn insert_position_should_return_the_size_of_the_load_order_if_given_a_non_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new("Blank - Master Dependent.esp", &load_order.game_settings())
            .unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn insert_position_should_return_the_first_non_master_plugin_index_if_given_a_master_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(1, position.unwrap());
    }

    #[test]
    fn insert_position_should_return_the_load_order_size_if_no_non_masters_are_present() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        // Remove non-master plugins from the load order.
        load_order.mut_plugins().retain(|p| p.is_master_file());

        let plugin = Plugin::new("Blank.esm", &load_order.game_settings()).unwrap();
        let position = load_order.insert_position(&plugin);

        assert_eq!(None, position);
    }

    #[test]
    fn load_should_reload_changed_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

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
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

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
    fn load_should_add_missing_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        assert_eq!(3, load_order.plugins().len());
        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Oblivion.esm",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "Blàñk.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    // TODO: Move to textfile and asterisk-based load order implementations
    fn load_should_add_missing_implicitly_active_plugins_after_other_missing_masters() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Skyrim, &tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Update.esm", &load_order.game_settings());
        load_order.load().unwrap();
        assert_eq!(Some(2), load_order.index_of("Update.esm"));
        assert!(load_order.is_active("Update.esm"));
    }

    #[test]
    fn load_should_sort_plugins_into_their_timestamp_order_with_master_files_first() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            load_order.game_settings().master_file(),
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "Blàñk.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_empty_the_load_order_if_the_plugins_directory_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());
        tmp_dir.close().unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins().is_empty());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file_for_oblivion() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", "Blàñk.esp"];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file_for_morrowind() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", "Blàñk.esp"];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    // TODO: Move to textfile and asterisk-based load order implementations
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

}

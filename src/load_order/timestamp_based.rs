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
use std::collections::BTreeSet;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufRead, Error, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use encoding::{DecoderTrap, Encoding, EncoderTrap};
use encoding::all::WINDOWS_1252;
use regex::bytes::Regex;

use enums::GameId;
use game_settings::GameSettings;
use plugin::Plugin;
use load_order::find_first_non_master_position;
use load_order::error::LoadOrderError;
use load_order::mutable::MutableLoadOrder;
use load_order::readable::ReadableLoadOrder;
use load_order::writable::WritableLoadOrder;

struct TimestampBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl ReadableLoadOrder for TimestampBasedLoadOrder {
    fn plugins(&self) -> &Vec<Plugin> {
        &self.plugins
    }
}

impl MutableLoadOrder for TimestampBasedLoadOrder {
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

impl WritableLoadOrder for TimestampBasedLoadOrder {
    fn load(&mut self) -> Result<(), LoadOrderError> {
        self.reload_changed_plugins();

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
        let mut timestamps: BTreeSet<SystemTime> = self.plugins()
            .iter()
            .map(Plugin::modification_time)
            .collect();

        while timestamps.len() < self.plugins().len() {
            let timestamp = *timestamps.iter().rev().nth(0).unwrap_or(&UNIX_EPOCH) +
                Duration::from_secs(60);
            timestamps.insert(timestamp);
        }

        for (plugin, timestamp) in self.mut_plugins().iter_mut().zip(timestamps.into_iter()) {
            plugin.set_modification_time(timestamp)?;
        }

        save_active_plugins(self)?;

        Ok(())
    }

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), LoadOrderError> {
        self.replace_plugins(plugin_names)
    }

    fn set_plugin_index(
        &mut self,
        plugin_name: &str,
        position: usize,
    ) -> Result<(), LoadOrderError> {
        self.move_or_insert_plugin(plugin_name, position)
    }
}

fn load_active_plugins<T: MutableLoadOrder>(load_order: &mut T) -> Result<(), LoadOrderError> {
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

fn save_active_plugins<T: MutableLoadOrder>(load_order: &mut T) -> Result<(), LoadOrderError> {
    if let Some(x) = load_order.game_settings().active_plugins_file().parent() {
        if !x.exists() {
            create_dir_all(x)?;
        }
    }

    let prelude = get_file_prelude(load_order.game_settings())?;

    let mut file = File::create(&load_order.game_settings().active_plugins_file())?;
    file.write_all(&prelude)?;
    for (index, plugin_name) in load_order.active_plugin_names().iter().enumerate() {
        if load_order.game_settings().id() == &GameId::Morrowind {
            write!(file, "GameFile{}=", index)?;
        }
        file.write_all(
            &WINDOWS_1252.encode(plugin_name, EncoderTrap::Strict)?,
        )?;
        writeln!(file, "")?;
    }

    Ok(())
}

fn get_file_prelude(game_settings: &GameSettings) -> Result<Vec<u8>, Error> {
    let mut prelude: Vec<u8> = Vec::new();
    if game_settings.id() == &GameId::Morrowind && game_settings.active_plugins_file().exists() {
        let input = File::open(game_settings.active_plugins_file())?;
        let buffered = BufReader::new(input);

        let game_files_header: &'static [u8] = b"[Game Files]";
        for line in buffered.split(b'\n') {
            let line = line?;
            prelude.append(&mut line.clone());
            prelude.push(b'\n');

            if line.starts_with(game_files_header) {
                break;
            }
        }

    }

    Ok(prelude)
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::*;

    use std::fs::{File, remove_dir_all};
    use std::io::{Read, Write};
    use std::path::Path;
    use self::tempdir::TempDir;
    use enums::GameId;
    use filetime::{FileTime, set_file_times};
    use load_order::tests::*;
    use tests::copy_to_test_dir;

    fn prepare(game_id: GameId, game_dir: &Path) -> TimestampBasedLoadOrder {
        let (game_settings, plugins) = mock_game_files(game_id, game_dir);
        TimestampBasedLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn write_file(path: &Path) {

        let mut file = File::create(&path).unwrap();
        writeln!(file, "").unwrap();
    }

    #[test]
    fn insert_position_should_return_none_if_given_a_non_master_plugin() {
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
    fn insert_position_should_return_none_if_no_non_masters_are_present() {
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
            "Blank.esp",
            "Blank - Different.esp",
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
            "Blank.esp",
            "Blank - Different.esp",
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

    #[test]
    fn save_should_preserve_and_extend_the_existing_set_of_timestamps() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        let mapper = |p: &Plugin| {
            p.modification_time()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };

        let mut old_timestamps: Vec<u64> = load_order.plugins().iter().map(&mapper).collect();

        load_order.save().unwrap();

        let timestamps: Vec<u64> = load_order.plugins().iter().map(&mapper).collect();

        assert_ne!(old_timestamps, timestamps);

        old_timestamps.sort();
        old_timestamps.dedup_by_key(|t| *t);
        let last_timestamp = *old_timestamps.last().unwrap();
        old_timestamps.push(last_timestamp + 60);

        assert_eq!(old_timestamps, timestamps);
    }

    #[test]
    fn save_should_create_active_plugins_file_parent_directory_if_it_does_not_exist() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

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
    fn save_should_write_active_plugins_file_for_oblivion() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Oblivion, &tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(vec!["Blank.esp"], load_order.active_plugin_names());
    }

    #[test]
    fn save_should_write_active_plugins_file_for_morrowind() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(vec!["Blank.esp"], load_order.active_plugin_names());

        let mut content = String::new();
        File::open(load_order.game_settings().active_plugins_file())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert!(content.contains("isrealmorrowindini=false\n[Game Files]\n"));
    }

    #[test]
    fn set_load_order_should_error_if_given_duplicate_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let filenames = vec!["Blank.esp", "blank.esp"];
        assert!(load_order.set_load_order(&filenames).is_err());
    }

    #[test]
    fn set_load_order_should_error_if_given_an_invalid_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let filenames = vec!["Blank.esp", "missing.esp"];
        assert!(load_order.set_load_order(&filenames).is_err());
    }

    #[test]
    fn set_load_order_should_error_if_given_a_list_with_plugins_before_masters() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let filenames = vec!["Blank.esp", "Blank.esm"];
        assert!(load_order.set_load_order(&filenames).is_err());
    }

    #[test]
    fn set_load_order_should_not_lose_active_state_of_existing_plugins() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let filenames = vec![
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
        ];
        load_order.set_load_order(&filenames).unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Morrowind.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blàñk.esp",
        ];
        assert_eq!(expected_filenames, load_order.plugin_names());
        assert!(load_order.is_active("Blank.esp"));
    }

    #[test]
    fn set_plugin_index_should_error_if_inserting_a_non_master_before_a_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        assert!(
            load_order
                .set_plugin_index("Blank - Master Dependent.esp", 0)
                .is_err()
        );
    }

    #[test]
    fn set_plugin_index_should_error_if_moving_a_non_master_before_a_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        assert!(load_order.set_plugin_index("Blank.esp", 0).is_err());
    }

    #[test]
    fn set_plugin_index_should_error_if_inserting_a_master_after_a_non_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        assert!(load_order.set_plugin_index("Blank.esm", 2).is_err());
    }

    #[test]
    fn set_plugin_index_should_error_if_moving_a_master_after_a_non_master() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        assert!(load_order.set_plugin_index("Morrowind.esm", 2).is_err());
    }

    #[test]
    fn set_plugin_index_should_error_if_setting_the_index_of_an_invalid_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        assert!(load_order.set_plugin_index("missing.esm", 0).is_err());
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

    #[test]
    fn set_plugin_index_should_move_an_existing_plugin() {
        let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
        let mut load_order = prepare(GameId::Morrowind, &tmp_dir.path());

        let num_plugins = load_order.plugins().len();
        load_order
            .set_plugin_index("Blank - Different.esp", 1)
            .unwrap();
        assert_eq!(1, load_order.index_of("Blank - Different.esp").unwrap());
        assert_eq!(num_plugins, load_order.plugins().len());
    }
}

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
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rayon::prelude::*;
use unicase::UniCase;

use super::mutable::{hoist_masters, load_active_plugins, MutableLoadOrder};
use super::readable::{ReadableLoadOrder, ReadableLoadOrderBase};
use super::strict_encode;
use super::writable::{
    activate, add, create_parent_dirs, deactivate, remove, set_active_plugins, WritableLoadOrder,
};
use crate::enums::{Error, GameId};
use crate::game_settings::GameSettings;
use crate::ini::read_morrowind_active_plugins;
use crate::plugin::{trim_dot_ghost, Plugin};

const GAME_FILES_HEADER: &[u8] = b"[Game Files]";

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct TimestampBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

/// Retains the first occurrence for each unique filename that is valid Unicode.
fn get_unique_filenames(file_paths: &[PathBuf], game_id: GameId) -> Vec<String> {
    let mut set = HashSet::new();

    file_paths
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .filter(|n| set.insert(UniCase::new(trim_dot_ghost(n, game_id))))
        .map(ToOwned::to_owned)
        .collect()
}

impl TimestampBasedLoadOrder {
    pub(crate) fn new(game_settings: GameSettings) -> Self {
        Self {
            game_settings,
            plugins: Vec::new(),
        }
    }

    fn load_plugins_from_dir(&self) -> Vec<Plugin> {
        let paths = self.game_settings.find_plugins();

        let filenames = get_unique_filenames(&paths, self.game_settings.id());

        filenames
            .par_iter()
            .filter_map(|f| Plugin::new(f, &self.game_settings).ok())
            .collect()
    }

    fn save_active_plugins(&mut self) -> Result<(), Error> {
        let path = self.game_settings().active_plugins_file();
        create_parent_dirs(path)?;

        let prelude = get_file_prelude(self.game_settings())?;

        let file = File::create(path).map_err(|e| Error::IoError(path.clone(), e))?;
        let mut writer = BufWriter::new(file);
        writer
            .write_all(&prelude)
            .map_err(|e| Error::IoError(path.clone(), e))?;
        for (index, plugin_name) in self.active_plugin_names().iter().enumerate() {
            if self.game_settings().id() == GameId::Morrowind {
                write!(writer, "GameFile{index}=").map_err(|e| Error::IoError(path.clone(), e))?;
            }
            writer
                .write_all(&strict_encode(plugin_name)?)
                .map_err(|e| Error::IoError(path.clone(), e))?;
            writeln!(writer).map_err(|e| Error::IoError(path.clone(), e))?;
        }

        Ok(())
    }

    fn load_active_morrowind_plugins(&mut self) -> Result<(), Error> {
        self.deactivate_all();

        let plugin_names =
            read_morrowind_active_plugins(self.game_settings().active_plugins_file())?;

        for plugin_name in plugin_names {
            if let Some(plugin) = self.find_plugin_mut(&plugin_name) {
                plugin.activate()?;
            }
        }

        Ok(())
    }
}

impl ReadableLoadOrderBase for TimestampBasedLoadOrder {
    fn game_settings_base(&self) -> &GameSettings {
        &self.game_settings
    }

    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}

impl MutableLoadOrder for TimestampBasedLoadOrder {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl WritableLoadOrder for TimestampBasedLoadOrder {
    fn game_settings_mut(&mut self) -> &mut GameSettings {
        &mut self.game_settings
    }

    fn load(&mut self) -> Result<(), Error> {
        self.plugins_mut().clear();

        self.plugins = self.load_plugins_from_dir();
        self.plugins.par_sort_by(plugin_sorter);

        let game_id = self.game_settings().id();
        if game_id == GameId::Morrowind {
            self.load_active_morrowind_plugins()?;
        } else {
            load_active_plugins(self, plugin_line_mapper)?;
        }

        self.add_implicitly_active_plugins()?;

        hoist_masters(&mut self.plugins)?;

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        save_load_order_using_timestamps(self)?;

        self.save_active_plugins()
    }

    fn add(&mut self, plugin_name: &str) -> Result<usize, Error> {
        add(self, plugin_name)
    }

    fn remove(&mut self, plugin_name: &str) -> Result<(), Error> {
        remove(self, plugin_name)
    }

    fn set_load_order(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        self.replace_plugins(plugin_names)
    }

    fn set_plugin_index(&mut self, plugin_name: &str, position: usize) -> Result<usize, Error> {
        MutableLoadOrder::set_plugin_index(self, plugin_name, position)
    }

    fn is_self_consistent(&self) -> Result<bool, Error> {
        Ok(true)
    }

    /// A timestamp-based load order is never ambiguous, as even if two or more plugins share the
    /// same timestamp, they load in descending filename order.
    fn is_ambiguous(&self) -> Result<bool, Error> {
        Ok(false)
    }

    fn activate(&mut self, plugin_name: &str) -> Result<(), Error> {
        activate(self, plugin_name)
    }

    fn deactivate(&mut self, plugin_name: &str) -> Result<(), Error> {
        deactivate(self, plugin_name)
    }

    fn set_active_plugins(&mut self, active_plugin_names: &[&str]) -> Result<(), Error> {
        set_active_plugins(self, active_plugin_names)
    }
}

pub(super) fn save_load_order_using_timestamps<T: MutableLoadOrder>(
    load_order: &mut T,
) -> Result<(), Error> {
    let timestamps = padded_unique_timestamps(load_order.plugins());

    load_order
        .plugins_mut()
        .par_iter_mut()
        .zip(timestamps.into_par_iter())
        .map(|(ref mut plugin, timestamp)| plugin.set_modification_time(timestamp))
        .collect::<Result<Vec<_>, Error>>()
        .map(|_| ())
}

fn plugin_sorter(a: &Plugin, b: &Plugin) -> Ordering {
    if a.is_master_file() == b.is_master_file() {
        match a.modification_time().cmp(&b.modification_time()) {
            Ordering::Equal => a.name().cmp(b.name()).reverse(),
            x => x,
        }
    } else if a.is_master_file() {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

fn plugin_line_mapper(line: &str) -> Option<String> {
    if line.is_empty() || line.starts_with('#') {
        None
    } else {
        Some(line.to_owned())
    }
}

fn padded_unique_timestamps(plugins: &[Plugin]) -> Vec<SystemTime> {
    let mut timestamps: Vec<SystemTime> = plugins.iter().map(Plugin::modification_time).collect();

    timestamps.sort();
    timestamps.dedup();

    while timestamps.len() < plugins.len() {
        let timestamp = *timestamps.last().unwrap_or(&UNIX_EPOCH) + Duration::from_secs(60);
        timestamps.push(timestamp);
    }

    timestamps
}

fn get_file_prelude(game_settings: &GameSettings) -> Result<Vec<u8>, Error> {
    let mut prelude: Vec<u8> = Vec::new();

    let path = game_settings.active_plugins_file();

    if game_settings.id() == GameId::Morrowind && path.exists() {
        let input = File::open(path).map_err(|e| Error::IoError(path.clone(), e))?;
        let buffered = BufReader::new(input);

        for line in buffered.split(b'\n') {
            let line = line.map_err(|e| Error::IoError(path.clone(), e))?;
            prelude.append(&mut line.clone());
            prelude.push(b'\n');

            if line.starts_with(GAME_FILES_HEADER) {
                break;
            }
        }
    }

    Ok(prelude)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::load_order::tests::*;
    use crate::tests::{copy_to_test_dir, set_file_timestamps, set_timestamps, NON_ASCII};
    use std::fs::remove_dir_all;
    use std::io::Read;
    use std::path::Path;
    use tempfile::tempdir;

    fn prepare(game_id: GameId, game_dir: &Path) -> TimestampBasedLoadOrder {
        let mut game_settings = game_settings_for_test(game_id, game_dir);
        mock_game_files(&mut game_settings);

        let plugins = vec![
            Plugin::with_active("Blank.esp", &game_settings, true).unwrap(),
            Plugin::new("Blank - Different.esp", &game_settings).unwrap(),
        ];

        TimestampBasedLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn write_file(path: &Path) {
        let mut file = File::create(path).unwrap();
        writeln!(file).unwrap();
    }

    #[test]
    fn load_should_reload_existing_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        assert!(!load_order.plugins()[1].is_master_file());
        copy_to_test_dir("Blank.esm", "Blank.esp", load_order.game_settings());
        let plugin_path = load_order
            .game_settings()
            .plugins_directory()
            .join("Blank.esp");
        set_file_timestamps(&plugin_path, 0);

        load_order.load().unwrap();

        assert!(load_order.plugins()[1].is_master_file());
    }

    #[test]
    fn load_should_remove_plugins_that_fail_to_load() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        assert!(load_order.index_of("Blank.esp").is_some());
        assert!(load_order.index_of("Blank - Different.esp").is_some());

        let plugin_path = load_order
            .game_settings()
            .plugins_directory()
            .join("Blank.esp");
        write_file(&plugin_path);
        set_file_timestamps(&plugin_path, 0);

        let plugin_path = load_order
            .game_settings()
            .plugins_directory()
            .join("Blank - Different.esp");
        write_file(&plugin_path);
        set_file_timestamps(&plugin_path, 0);

        load_order.load().unwrap();
        assert!(load_order.index_of("Blank.esp").is_none());
        assert!(load_order.index_of("Blank - Different.esp").is_none());
    }

    #[test]
    fn load_should_sort_installed_plugins_into_their_timestamp_order_with_master_files_first() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        set_timestamps(
            &load_order.game_settings().plugins_directory(),
            &[
                "Blank - Master Dependent.esp",
                "Blank.esm",
                "Blank - Different.esp",
                "Blank.esp",
            ],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            NON_ASCII,
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_hoist_masters_that_masters_depend_on_to_load_before_their_dependents() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        let master_dependent_master = "Blank - Master Dependent.esm";
        copy_to_test_dir(
            master_dependent_master,
            master_dependent_master,
            load_order.game_settings(),
        );

        let filenames = vec![
            "Blank - Master Dependent.esm",
            "Blank - Master Dependent.esp",
            "Blank.esm",
            "Blank - Different.esp",
            NON_ASCII,
            "Blank.esp",
        ];
        set_timestamps(&load_order.game_settings().plugins_directory(), &filenames);

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Blank - Master Dependent.esm",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            NON_ASCII,
            "Blank.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_empty_the_load_order_if_the_plugins_directory_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());
        tmp_dir.close().unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins().is_empty());
    }

    #[test]
    fn load_should_decode_active_plugins_file_from_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_handle_crlf_and_lf_in_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm\r"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_ignore_active_plugins_file_lines_starting_with_a_hash_for_oblivion() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["#Blank.esp", NON_ASCII, "Blank.esm"],
        );

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_ignore_plugins_in_active_plugins_file_that_are_not_installed() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file_for_oblivion() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_succeed_when_active_plugins_file_is_missing() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        assert!(load_order.load().is_ok());
        assert!(load_order.active_plugin_names().is_empty());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file_for_morrowind() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_skip_morrowind_gamefile_entries_after_a_break_in_their_indexes() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, tmp_dir.path());

        let filenames: [(u8, &str); 3] = [
            (0, "Blank.esm"),
            (1, "Blank.esp"),
            (3, "Blank - Different.esp"),
        ];
        {
            let mut file = File::create(load_order.game_settings().active_plugins_file()).unwrap();

            writeln!(file, "[Game Files]").unwrap();

            for (i, filename) in filenames {
                write!(file, "GameFile{i}=").unwrap();

                file.write_all(&strict_encode(filename).unwrap()).unwrap();
                writeln!(file).unwrap();
            }
        }

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", "Blank.esp"];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn save_should_preserve_the_existing_set_of_timestamps() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        let mapper = |p: &Plugin| {
            p.modification_time()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };

        set_timestamps(
            &load_order.game_settings().plugins_directory(),
            &[
                "Blank - Master Dependent.esp",
                "Blank.esm",
                "Blank - Different.esp",
                "Blank.esp",
            ],
        );

        load_order.load().unwrap();

        let mut old_timestamps: Vec<u64> = load_order.plugins().iter().map(&mapper).collect();
        old_timestamps.sort_unstable();

        load_order.save().unwrap();

        let timestamps: Vec<u64> = load_order.plugins().iter().map(&mapper).collect();

        assert_eq!(old_timestamps, timestamps);
    }

    #[test]
    fn save_should_deduplicate_plugin_timestamps() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        let mapper = |p: &Plugin| {
            p.modification_time()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };

        set_timestamps(
            &load_order.game_settings().plugins_directory(),
            &[
                "Blank - Master Dependent.esp",
                "Blank.esm",
                "Blank - Different.esp",
                "Blank.esp",
            ],
        );

        // Give two files the same timestamp.
        load_order.plugins_mut()[1]
            .set_modification_time(UNIX_EPOCH + Duration::new(2, 0))
            .unwrap();

        load_order.load().unwrap();

        let mut old_timestamps: Vec<u64> = load_order.plugins().iter().map(&mapper).collect();

        load_order.save().unwrap();

        let timestamps: Vec<u64> = load_order.plugins().iter().map(&mapper).collect();

        assert_ne!(old_timestamps, timestamps);

        old_timestamps.sort_unstable();
        old_timestamps.dedup_by_key(|t| *t);

        assert_eq!(old_timestamps, timestamps);
    }

    #[test]
    fn save_should_create_active_plugins_file_parent_directory_if_it_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        remove_dir_all(
            load_order
                .game_settings()
                .active_plugins_file()
                .parent()
                .unwrap(),
        )
        .unwrap();

        load_order.save().unwrap();

        assert!(load_order
            .game_settings()
            .active_plugins_file()
            .parent()
            .unwrap()
            .exists());
    }

    #[test]
    fn save_should_write_active_plugins_file_for_oblivion() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(vec!["Blank.esp"], load_order.active_plugin_names());
    }

    #[test]
    fn save_should_write_active_plugins_file_for_morrowind() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

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
    fn save_should_error_if_an_active_plugin_filename_cannot_be_encoded_in_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Oblivion, tmp_dir.path());

        let filename = "Bl\u{0227}nk.esm";
        copy_to_test_dir(
            "Blank - Different.esm",
            filename,
            load_order.game_settings(),
        );
        let mut plugin = Plugin::new(filename, load_order.game_settings()).unwrap();
        plugin.activate().unwrap();
        load_order.plugins_mut().push(plugin);

        match load_order.save().unwrap_err() {
            Error::EncodeError(s) => assert_eq!("Bl\u{227}nk.esm", s),
            e => panic!("Expected encode error, got {e:?}"),
        }
    }

    #[test]
    fn is_self_consistent_should_return_true() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Morrowind, tmp_dir.path());

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_false_if_all_loaded_plugins_have_unique_timestamps() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, tmp_dir.path());

        for (index, plugin) in load_order.plugins_mut().iter_mut().enumerate() {
            plugin
                .set_modification_time(UNIX_EPOCH + Duration::new(index.try_into().unwrap(), 0))
                .unwrap();
        }

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_false_if_two_loaded_plugins_have_the_same_timestamp() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Morrowind, tmp_dir.path());

        // Give two files the same timestamp.
        load_order.plugins_mut()[0]
            .set_modification_time(UNIX_EPOCH + Duration::new(2, 0))
            .unwrap();
        load_order.plugins_mut()[1]
            .set_modification_time(UNIX_EPOCH + Duration::new(2, 0))
            .unwrap();

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn plugin_sorter_should_sort_in_descending_filename_order_if_timestamps_are_equal() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Morrowind, tmp_dir.path());

        let mut plugin1 = Plugin::new("Blank.esp", load_order.game_settings()).unwrap();
        let mut plugin2 = Plugin::new("Blank - Different.esp", load_order.game_settings()).unwrap();

        plugin1
            .set_modification_time(UNIX_EPOCH + Duration::new(2, 0))
            .unwrap();

        plugin2
            .set_modification_time(UNIX_EPOCH + Duration::new(2, 0))
            .unwrap();

        let ordering = plugin_sorter(&plugin1, &plugin2);

        assert_eq!(Ordering::Less, ordering);
    }
}

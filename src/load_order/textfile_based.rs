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
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use unicase::{eq, UniCase};

use super::mutable::{
    hoist_masters, load_active_plugins, plugin_line_mapper, read_plugin_names, MutableLoadOrder,
};
use super::readable::{ReadableLoadOrder, ReadableLoadOrderBase};
use super::strict_encode;
use super::writable::{
    activate, add, create_parent_dirs, deactivate, remove, set_active_plugins, WritableLoadOrder,
};
use crate::enums::Error;
use crate::game_settings::GameSettings;
use crate::plugin::{trim_dot_ghost, trim_dot_ghost_unchecked, Plugin};
use crate::GameId;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct TextfileBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl TextfileBasedLoadOrder {
    pub(crate) fn new(game_settings: GameSettings) -> Self {
        Self {
            game_settings,
            plugins: Vec::new(),
        }
    }

    fn read_from_load_order_file(&self) -> Result<Vec<(String, bool)>, Error> {
        match self.game_settings().load_order_file() {
            Some(file_path) => read_utf8_plugin_names(file_path, load_order_line_mapper)
                .or_else(|_| read_plugin_names(file_path, load_order_line_mapper)),
            None => Ok(Vec::new()),
        }
    }

    fn read_from_active_plugins_file(&self) -> Result<Vec<(String, bool)>, Error> {
        read_plugin_names(
            self.game_settings().active_plugins_file(),
            active_plugin_line_mapper,
        )
    }

    fn save_load_order(&self) -> Result<(), Error> {
        if let Some(file_path) = self.game_settings().load_order_file() {
            create_parent_dirs(file_path)?;

            let file = File::create(file_path).map_err(|e| Error::IoError(file_path.clone(), e))?;
            let mut writer = BufWriter::new(file);
            for plugin_name in self.plugin_names() {
                writeln!(writer, "{plugin_name}")
                    .map_err(|e| Error::IoError(file_path.clone(), e))?;
            }
        }
        Ok(())
    }

    fn save_active_plugins(&self) -> Result<(), Error> {
        let path = self.game_settings().active_plugins_file();
        create_parent_dirs(path)?;

        let file = File::create(path).map_err(|e| Error::IoError(path.clone(), e))?;
        let mut writer = BufWriter::new(file);
        for plugin_name in self.active_plugin_names() {
            writer
                .write_all(&strict_encode(plugin_name)?)
                .map_err(|e| Error::IoError(path.clone(), e))?;
            writeln!(writer).map_err(|e| Error::IoError(path.clone(), e))?;
        }

        Ok(())
    }
}

impl ReadableLoadOrderBase for TextfileBasedLoadOrder {
    fn game_settings_base(&self) -> &GameSettings {
        &self.game_settings
    }

    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}

impl MutableLoadOrder for TextfileBasedLoadOrder {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl WritableLoadOrder for TextfileBasedLoadOrder {
    fn game_settings_mut(&mut self) -> &mut GameSettings {
        &mut self.game_settings
    }

    fn load(&mut self) -> Result<(), Error> {
        self.plugins_mut().clear();

        let load_order_file_exists = self
            .game_settings()
            .load_order_file()
            .is_some_and(|p| p.exists());

        let plugin_tuples = if load_order_file_exists {
            self.read_from_load_order_file()?
        } else {
            self.read_from_active_plugins_file()?
        };

        let paths = self.game_settings.find_plugins();
        self.load_unique_plugins(&plugin_tuples, &paths);

        if load_order_file_exists {
            load_active_plugins(self, plugin_line_mapper)?;
        }

        self.add_implicitly_active_plugins()?;

        if self.game_settings.id().treats_master_files_differently() {
            hoist_masters(&mut self.plugins)?;
        }

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        self.save_load_order()?;
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
        match check_self_consistency(self.game_settings())? {
            SelfConsistency::Inconsistent => Ok(false),
            _ => Ok(true),
        }
    }

    /// A textfile-based load order is ambiguous when it's not self-consistent
    /// (because an app that prefers loadorder.txt may give a different load
    /// order to one that prefers plugins.txt) or when there are installed
    /// plugins that are not present in one or both of the text files.
    fn is_ambiguous(&self) -> Result<bool, Error> {
        let plugin_names = match check_self_consistency(self.game_settings())? {
            SelfConsistency::Inconsistent => {
                return Ok(true);
            }
            SelfConsistency::ConsistentWithNames(plugin_names) => plugin_names,
            SelfConsistency::ConsistentNoLoadOrderFile => read_plugin_names(
                self.game_settings().active_plugins_file(),
                plugin_line_mapper,
            )?,
            SelfConsistency::ConsistentOnlyLoadOrderFile(load_order_file) => {
                read_utf8_plugin_names(&load_order_file, plugin_line_mapper)
                    .or_else(|_| read_plugin_names(&load_order_file, plugin_line_mapper))?
            }
        };

        let set: HashSet<_> = plugin_names
            .iter()
            .map(|name| UniCase::new(trim_dot_ghost(name, self.game_settings.id())))
            .collect();

        let all_plugins_listed = self
            .plugins
            .iter()
            .all(|plugin| set.contains(&UniCase::new(plugin.name())));

        Ok(!all_plugins_listed)
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

pub(super) fn read_utf8_plugin_names<F, T>(
    file_path: &Path,
    line_mapper: F,
) -> Result<Vec<T>, Error>
where
    F: Fn(&str) -> Option<T> + Send + Sync,
    T: Send,
{
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(file_path)
        .map_err(|e| Error::IoError(file_path.to_path_buf(), e))?;

    Ok(content.lines().filter_map(line_mapper).collect())
}

enum SelfConsistency {
    ConsistentNoLoadOrderFile,
    ConsistentOnlyLoadOrderFile(PathBuf),
    ConsistentWithNames(Vec<String>),
    Inconsistent,
}

fn check_self_consistency(game_settings: &GameSettings) -> Result<SelfConsistency, Error> {
    match game_settings.load_order_file() {
        None => Ok(SelfConsistency::ConsistentNoLoadOrderFile),
        Some(load_order_file) => {
            if !load_order_file.exists() {
                return Ok(SelfConsistency::ConsistentNoLoadOrderFile);
            }

            if !game_settings.active_plugins_file().exists() {
                return Ok(SelfConsistency::ConsistentOnlyLoadOrderFile(
                    load_order_file.clone(),
                ));
            }

            // First get load order according to loadorder.txt.
            let load_order_plugin_names =
                read_utf8_plugin_names(load_order_file, plugin_line_mapper)
                    .or_else(|_| read_plugin_names(load_order_file, plugin_line_mapper))?;

            // Get load order from plugins.txt.
            let active_plugin_names =
                read_plugin_names(game_settings.active_plugins_file(), plugin_line_mapper)?;

            let are_equal = load_order_plugin_names
                .iter()
                .filter(|l| {
                    active_plugin_names
                        .iter()
                        .any(|a| plugin_names_match(game_settings.id(), a, l))
                })
                .zip(active_plugin_names.iter())
                .all(|(l, a)| plugin_names_match(game_settings.id(), l, a));

            if are_equal {
                Ok(SelfConsistency::ConsistentWithNames(
                    load_order_plugin_names,
                ))
            } else {
                Ok(SelfConsistency::Inconsistent)
            }
        }
    }
}

fn load_order_line_mapper(line: &str) -> Option<(String, bool)> {
    plugin_line_mapper(line).map(|s| (s, false))
}

fn active_plugin_line_mapper(line: &str) -> Option<(String, bool)> {
    plugin_line_mapper(line).map(|s| (s, true))
}

fn plugin_names_match(game_id: GameId, name1: &str, name2: &str) -> bool {
    if game_id.allow_plugin_ghosting() {
        eq(
            trim_dot_ghost_unchecked(name1),
            trim_dot_ghost_unchecked(name2),
        )
    } else {
        eq(name1, name2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::load_order::tests::*;
    use crate::tests::{copy_to_test_dir, set_file_timestamps, NON_ASCII};
    use std::fs::remove_dir_all;
    use tempfile::tempdir;

    fn prepare(game_dir: &Path) -> TextfileBasedLoadOrder {
        prepare_game(GameId::Skyrim, game_dir)
    }

    fn prepare_oblivion_remastered(game_dir: &Path) -> TextfileBasedLoadOrder {
        prepare_game(GameId::OblivionRemastered, game_dir)
    }

    fn prepare_game(game_id: GameId, game_dir: &Path) -> TextfileBasedLoadOrder {
        let mut game_settings = game_settings_for_test(game_id, game_dir);
        mock_game_files(&mut game_settings);

        let plugins = vec![
            Plugin::with_active("Blank.esp", &game_settings, true).unwrap(),
            Plugin::new("Blank - Different.esp", &game_settings).unwrap(),
        ];

        TextfileBasedLoadOrder {
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
        let mut load_order = prepare(tmp_dir.path());

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
        let mut load_order = prepare(tmp_dir.path());

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
    fn load_should_get_load_order_from_load_order_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "missing.esp",
        ];
        write_load_order_file(load_order.game_settings(), &expected_filenames);

        load_order.load().unwrap();
        assert_eq!(
            &expected_filenames[..5],
            load_order.plugin_names().as_slice()
        );
    }

    #[test]
    fn load_should_hoist_masters_that_masters_depend_on_to_load_before_their_dependents() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

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
        write_load_order_file(load_order.game_settings(), &filenames);

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
    fn load_should_not_hoist_masters_for_oblivion_remastered() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare_oblivion_remastered(tmp_dir.path());

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
        write_load_order_file(load_order.game_settings(), &filenames);

        load_order.load().unwrap();

        assert_eq!(filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_read_load_order_file_as_windows_1252_if_not_utf8() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "missing.esp",
        ];

        let mut file = File::create(load_order.game_settings().load_order_file().unwrap()).unwrap();

        for filename in &expected_filenames {
            file.write_all(&strict_encode(filename).unwrap()).unwrap();
            writeln!(file).unwrap();
        }

        load_order.load().unwrap();
        assert_eq!(
            &expected_filenames[..5],
            load_order.plugin_names().as_slice()
        );
    }

    #[test]
    fn load_should_get_load_order_from_active_plugins_file_if_load_order_file_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["Blank.esp", "Blank - Master Dependent.esp"],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Blank.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            NON_ASCII,
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_add_missing_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        assert!(load_order.index_of("Blank.esm").is_none());
        assert!(load_order
            .index_of("Blank - Master Dependent.esp")
            .is_none());
        assert!(load_order.index_of(NON_ASCII).is_none());

        load_order.load().unwrap();

        assert!(load_order.index_of("Blank.esm").is_some());
        assert!(load_order
            .index_of("Blank - Master Dependent.esp")
            .is_some());
        assert!(load_order.index_of(NON_ASCII).is_some());
    }

    #[test]
    fn load_should_empty_the_load_order_if_the_plugins_directory_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());
        tmp_dir.close().unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins().is_empty());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["Blank.esm", "Blank - Master Dependent.esp"],
        );

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", "Blank - Master Dependent.esp"];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_decode_active_plugins_file_from_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_handle_crlf_and_lf_in_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm\r"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_ignore_active_plugins_file_lines_starting_with_a_hash() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

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
        let mut load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_succeed_when_load_order_and_active_plugins_files_are_missing() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Skyrim.esm", load_order.game_settings());

        assert!(load_order.load().is_ok());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn load_should_not_duplicate_a_plugin_that_is_ghosted_and_in_load_order_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        std::fs::rename(
            load_order
                .game_settings()
                .plugins_directory()
                .join("Blank.esm"),
            load_order
                .game_settings()
                .plugins_directory()
                .join("Blank.esm.ghost"),
        )
        .unwrap();

        let filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
            "missing.esp",
        ];
        write_load_order_file(load_order.game_settings(), &filenames);

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            "Blank.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn save_should_write_all_plugins_to_load_order_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        load_order.save().unwrap();

        let expected_filenames = vec!["Blank.esp", "Blank - Different.esp"];
        let plugin_names = read_utf8_plugin_names(
            load_order.game_settings().load_order_file().unwrap(),
            plugin_line_mapper,
        )
        .unwrap();
        assert_eq!(expected_filenames, plugin_names);
    }

    #[test]
    fn save_should_create_active_plugins_file_parent_directory_if_it_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

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
    fn save_should_write_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(vec!["Blank.esp"], load_order.active_plugin_names());
    }

    #[test]
    fn save_should_error_if_an_active_plugin_filename_cannot_be_encoded_in_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

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
    fn is_self_consistent_should_return_true_when_no_load_order_file_exists() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_self_consistent_should_return_true_when_no_active_plugins_file_exists() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let filenames = vec!["Blank - Master Dependent.esp"];
        write_load_order_file(load_order.game_settings(), &filenames);

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_self_consistent_should_return_false_when_load_order_and_active_plugins_files_mismatch() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        let filenames = vec![NON_ASCII, "missing.esp", "Blank.esm\r"];
        write_load_order_file(load_order.game_settings(), &filenames);

        assert!(!load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_self_consistent_should_return_true_when_load_order_and_active_plugins_files_match() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        // loadorder.txt should be a case-insensitive sorted superset of plugins.txt.
        let filenames = vec![NON_ASCII, "Blank.esm\r", "missing.esp"];
        write_load_order_file(load_order.game_settings(), &filenames);

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_self_consistent_should_read_load_order_file_as_windows_1252_if_not_utf8() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        // loadorder.txt should be a case-insensitive sorted superset of plugins.txt.
        let filenames = vec![NON_ASCII, "Blank.esm\r", "missing.esp"];

        let mut file = File::create(load_order.game_settings().load_order_file().unwrap()).unwrap();

        for filename in &filenames {
            file.write_all(&strict_encode(filename).unwrap()).unwrap();
            writeln!(file).unwrap();
        }

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_true_if_load_order_is_not_self_consistent() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        let expected_filenames = vec![NON_ASCII, "missing.esp", "Blank.esm\r"];
        write_load_order_file(load_order.game_settings(), &expected_filenames);

        assert!(!load_order.is_self_consistent().unwrap());
        assert!(load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_true_if_active_plugins_and_load_order_files_do_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        assert!(load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_true_if_only_active_plugins_file_exists_and_does_not_list_all_loaded_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let mut loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        loaded_plugin_names.pop();

        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_false_if_only_active_plugins_file_exists_and_lists_all_loaded_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_true_if_only_load_order_file_exists_and_does_not_list_all_loaded_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let mut loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        loaded_plugin_names.pop();

        write_load_order_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_false_if_only_load_order_file_exists_and_lists_all_loaded_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        write_load_order_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_read_load_order_file_as_windows_1252_if_not_utf8() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        let mut file = File::create(load_order.game_settings().load_order_file().unwrap()).unwrap();

        for filename in &loaded_plugin_names {
            file.write_all(&strict_encode(filename).unwrap()).unwrap();
            writeln!(file).unwrap();
        }

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_true_if_active_plugins_and_load_order_files_exist_and_load_order_file_does_not_list_all_loaded_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let mut loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        loaded_plugin_names.pop();

        write_load_order_file(load_order.game_settings(), &loaded_plugin_names);
        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_false_if_active_plugins_and_load_order_files_exist_and_load_order_file_lists_all_loaded_plugins(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(tmp_dir.path());

        let mut loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        write_load_order_file(load_order.game_settings(), &loaded_plugin_names);

        loaded_plugin_names.pop();

        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(!load_order.is_ambiguous().unwrap());
    }
}

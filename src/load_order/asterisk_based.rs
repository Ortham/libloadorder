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

use unicase::UniCase;

use super::mutable::{hoist_masters, read_plugin_names, MutableLoadOrder};
use super::readable::{ReadableLoadOrder, ReadableLoadOrderBase};
use super::strict_encode;
use super::timestamp_based::save_load_order_using_timestamps;
use super::writable::{
    activate, add, create_parent_dirs, deactivate, remove, set_active_plugins, WritableLoadOrder,
};
use crate::enums::{Error, GameId};
use crate::game_settings::GameSettings;
use crate::load_order::timestamp_based::save_partial_load_order_using_timestamps;
use crate::load_order::writable::{blueprint_ships_base_plugin_name, starts_with_blueprint_ships};
use crate::plugin::{trim_dot_ghost, Plugin};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AsteriskBasedLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl AsteriskBasedLoadOrder {
    pub(crate) fn new(game_settings: GameSettings) -> Self {
        Self {
            game_settings,
            plugins: Vec::new(),
        }
    }

    fn read_from_active_plugins_file(&self) -> Result<Vec<(String, bool)>, Error> {
        if self.ignore_active_plugins_file() {
            if self.game_settings.id() == GameId::Starfield {
                // For Starfield, if the active plugins file is being ignored, it's because there
                // are test files set, and they load in the order of their entries indexes, before
                // any other non-early-loader plugins that are found, which effectively means that
                // the entries replace plugins.txt.
                Ok(self
                    .game_settings
                    .test_files()
                    .iter()
                    .map(|s| (s.clone(), true))
                    .collect())
            } else {
                Ok(Vec::new())
            }
        } else {
            read_plugin_names(
                self.game_settings().active_plugins_file(),
                owning_plugin_line_mapper,
            )
        }
    }

    fn ignore_active_plugins_file(&self) -> bool {
        // Fallout 4 and Starfield ignore plugins.txt if there are any sTestFile plugins listed in
        // the ini files.
        matches!(
            self.game_settings.id(),
            GameId::Fallout4 | GameId::Fallout4VR | GameId::Starfield
        ) && !self.game_settings.test_files().is_empty()
    }

    fn implicitly_activate_blueprint_ships_plugins(&mut self) -> Result<(), Error> {
        let active_base_names: HashSet<UniCase<&str>> = self
            .plugins()
            .iter()
            // Implicitly-active plugins can activate BlueprintShips plugins.
            .filter(|p| p.is_active())
            .map(|p| UniCase::new(p.name_without_extension()))
            .collect();

        let indexes: Vec<_> = self
            .plugins
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                blueprint_ships_base_plugin_name(p.name())
                    .map(UniCase::new)
                    .is_some_and(|n| active_base_names.contains(&n))
            })
            .map(|(i, _)| i)
            .collect();

        for index in indexes {
            if let Some(plugin) = self.plugins.get_mut(index) {
                plugin.implicitly_activate()?;
            }
        }

        Ok(())
    }
}

impl ReadableLoadOrderBase for AsteriskBasedLoadOrder {
    fn game_settings_base(&self) -> &GameSettings {
        &self.game_settings
    }

    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}

impl MutableLoadOrder for AsteriskBasedLoadOrder {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }
}

impl WritableLoadOrder for AsteriskBasedLoadOrder {
    fn game_settings_mut(&mut self) -> &mut GameSettings {
        &mut self.game_settings
    }

    fn load(&mut self) -> Result<(), Error> {
        self.plugins_mut().clear();

        let plugin_tuples = self.read_from_active_plugins_file()?;
        let paths = self.game_settings.find_plugins();

        self.load_unique_plugins(&plugin_tuples, &paths);

        self.add_implicitly_active_plugins()?;

        if self.game_settings.id() == GameId::Starfield {
            self.implicitly_activate_blueprint_ships_plugins()?;
        }

        hoist_masters(&mut self.plugins)?;

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        let path = self.game_settings().active_plugins_file();
        create_parent_dirs(path)?;

        let file = File::create(path).map_err(|e| Error::IoError(path.clone(), e))?;
        let mut writer = BufWriter::new(file);
        for plugin in self.plugins() {
            if self.game_settings().loads_early(plugin.name()) {
                // Skip early loading plugins, but not implicitly active plugins
                // as they may need load order positions defined.
                continue;
            }

            if self.game_settings().id() == GameId::Starfield
                && (starts_with_blueprint_ships(plugin.name()) || plugin.is_blueprint_plugin())
            {
                // Skip these since they get removed from plugins.txt by the
                // game. This means that the load order being saved might not be
                // respected by the game when it loads, if the affected plugins
                // have different load order positions than where they load when
                // only implicitly active, but that's an unusual edge-case that
                // can only be enforced by basically fighting the game.
                continue;
            }

            if plugin.is_active() {
                write!(writer, "*").map_err(|e| Error::IoError(path.clone(), e))?;
            }
            writer
                .write_all(&strict_encode(plugin.name())?)
                .map_err(|e| Error::IoError(path.clone(), e))?;
            writeln!(writer).map_err(|e| Error::IoError(path.clone(), e))?;
        }

        if self.ignore_active_plugins_file() {
            // If the active plugins file is being ignored there's no harm in
            // writing to it, but it won't actually have any impact on the load
            // order used by the game. In that case, the only way to set the
            // load order is to modify plugin timestamps, so do that.
            save_load_order_using_timestamps(&mut self.plugins)?;
        } else if self.game_settings.id() == GameId::Starfield {
            // Blueprint plugins and BlueprintShips plugins get removed from
            // plugins.txt by Starfield after the file is read. However,
            // BlueprintShips plugins are still implicitly active if the plugin
            // referenced by their filename suffix is active, so their load
            // order is relatively important.
            // Blueprint masters get loaded after all other plugins, and if not
            // explicitly active they get loaded in timestamp order, so set
            // their timestamps to reflect their load order, so that they'll
            // load in the intended order even though they're not written to
            // plugins.txt.
            // This doesn't help with non-master blueprint plugins, or with
            // BlueprintShips plugins that are not blueprint plugins, but all
            // official BlueprintShips plugins (as of 2026-04-13) are blueprint
            // masters, and there are no other official blueprint plugins.
            // I don't know how common blueprint plugins are in mods.
            let blueprint_masters_iter =
                self.plugins.iter_mut().filter(|p| p.is_blueprint_master());
            save_partial_load_order_using_timestamps(blueprint_masters_iter)?;
        }

        Ok(())
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

    /// An asterisk-based load order can be ambiguous if there are installed
    /// plugins that are not implicitly active and not listed in plugins.txt.
    fn is_ambiguous(&self) -> Result<bool, Error> {
        let mut set = HashSet::new();

        // Read plugins from the active plugins file. A set of plugin names is
        // more useful than the returned vec, so insert into the set during the
        // line mapping and then discard the line.
        if !self.ignore_active_plugins_file() {
            read_plugin_names(self.game_settings().active_plugins_file(), |line| {
                plugin_line_mapper(line).and_then::<(), _>(|(name, _)| {
                    set.insert(UniCase::new(
                        trim_dot_ghost(name, self.game_settings.id()).to_owned(),
                    ));
                    None
                })
            })?;
        }

        // All implicitly active plugins have a defined load order position,
        // even if they're not in plugins.txt or the early loaders.
        // Plugins that are active but not implicitly active, and plugins that
        // are inactive, only have a load order position if they're listed in
        // plugins.txt, so check that they're all listed.
        // Starfield removes blueprint plugins from plugins.txt, which means
        // inactive blueprint plugins' positions become ambiguous after each
        // game session, but resolving that ambiguity will just be undone the
        // next time the game is loaded, so there's not really any point
        // reporting it.
        // Starfield will also load plugins named BlueprintShips-<X>.esm for any
        // active plugins with the basename <X> (e.g. <X>.esp, <X>.esm,
        // <X>.esl), even if the BlueprintShips plugin is not a blueprint
        // plugin and/or not listed in plugins.txt. Like blueprint plugins,
        // BlueprintShips plugins are removed from plugins.txt whether they're
        // active or not, so also skip them.
        let plugins_listed = self
            .plugins
            .iter()
            .filter(|plugin| {
                !(self.game_settings.is_implicitly_active(plugin.name())
                    || plugin.is_blueprint_plugin()
                    || (self.game_settings.supports_blueprint_ships_plugins()
                        && starts_with_blueprint_ships(plugin.name())))
            })
            .all(|plugin| set.contains(&UniCase::new(plugin.name().to_owned())));

        Ok(!plugins_listed)
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

fn plugin_line_mapper(line: &str) -> Option<(&str, bool)> {
    if line.is_empty() || line.starts_with('#') {
        None
    } else if let Some(remainder) = line.strip_prefix('*') {
        Some((remainder, true))
    } else {
        Some((line, false))
    }
}

fn owning_plugin_line_mapper(line: &str) -> Option<(String, bool)> {
    plugin_line_mapper(line).map(|(name, explicitly_active)| (name.to_owned(), explicitly_active))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::load_order::tests::*;
    use crate::plugin::ActiveState;
    use crate::tests::{copy_to_dir, copy_to_test_dir, set_file_timestamps, NON_ASCII};
    use std::fs::{create_dir_all, remove_dir_all};
    use std::path::Path;
    use std::time::Duration;
    use tempfile::tempdir;

    fn prepare(game_id: GameId, game_dir: &Path) -> AsteriskBasedLoadOrder {
        let mut game_settings = game_settings_for_test(game_id, game_dir);
        mock_game_files(&mut game_settings);

        let mut plugins =
            vec![
                Plugin::with_active("Blank.esp", &game_settings, ActiveState::ExplicitlyActive)
                    .unwrap(),
            ];

        if game_id != GameId::Starfield {
            plugins.push(Plugin::new("Blank - Different.esp", &game_settings).unwrap());
        }

        AsteriskBasedLoadOrder {
            game_settings,
            plugins,
        }
    }

    fn read_lines(file_path: &Path) -> Vec<String> {
        let bytes = std::fs::read(file_path).unwrap();
        let text = encoding_rs::WINDOWS_1252.decode(&bytes).0;

        text.lines().map(std::borrow::ToOwned::to_owned).collect()
    }

    fn copy_as_blueprint_plugin(settings: &GameSettings, plugin_name: &str) {
        copy_to_test_dir("Blank.full.esm", plugin_name, settings);
        set_blueprint_flag(
            settings.id(),
            &settings.plugins_directory().join(plugin_name),
            true,
        )
        .unwrap();
    }

    #[test]
    fn ignore_active_plugins_file_should_be_true_for_fallout4_when_test_files_are_configured() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Fallout4.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        let load_order = prepare(GameId::Fallout4, tmp_dir.path());

        assert!(load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_false_for_fallout4_when_test_files_are_not_configured()
    {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Fallout4, tmp_dir.path());

        assert!(!load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_true_for_fallout4vr_when_test_files_are_configured() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Fallout4VR.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        let load_order = prepare(GameId::Fallout4VR, tmp_dir.path());

        assert!(load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_false_for_fallout4vr_when_test_files_are_not_configured(
    ) {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Fallout4VR, tmp_dir.path());

        assert!(!load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_true_for_starfield_when_test_files_are_configured() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/StarfieldCustom.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        let load_order = prepare(GameId::Starfield, tmp_dir.path());

        assert!(load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_false_for_starfield_when_test_files_are_not_configured()
    {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::Starfield, tmp_dir.path());

        assert!(!load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_false_for_skyrimse() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Skyrim.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        assert!(!load_order.ignore_active_plugins_file());
    }

    #[test]
    fn ignore_active_plugins_file_should_be_false_for_skyrimvr() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/SkyrimVR.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=a").unwrap();

        let load_order = prepare(GameId::SkyrimVR, tmp_dir.path());

        assert!(!load_order.ignore_active_plugins_file());
    }

    #[test]
    fn load_should_reload_existing_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

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
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        assert!(load_order.index_of("Blank.esp").is_some());
        assert!(load_order.index_of("Blank - Different.esp").is_some());

        let plugin_path = load_order
            .game_settings()
            .plugins_directory()
            .join("Blank.esp");
        File::create(&plugin_path).unwrap();
        set_file_timestamps(&plugin_path, 0);

        let plugin_path = load_order
            .game_settings()
            .plugins_directory()
            .join("Blank - Different.esp");
        File::create(&plugin_path).unwrap();
        set_file_timestamps(&plugin_path, 0);

        load_order.load().unwrap();
        assert!(load_order.index_of("Blank.esp").is_none());
        assert!(load_order.index_of("Blank - Different.esp").is_none());
    }

    #[test]
    fn load_should_get_load_order_from_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

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
    fn load_should_hoist_masters_that_masters_depend_on_to_load_before_their_dependents() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

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
        write_active_plugins_file(load_order.game_settings(), &filenames);

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
    fn load_should_decode_active_plugins_file_from_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_handle_crlf_and_lf_in_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm\r"]);

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_ignore_active_plugins_file_lines_starting_with_a_hash() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["#Blank.esp", NON_ASCII, "Blank.esm"],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_ignore_plugins_in_active_plugins_file_that_are_not_installed() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &[NON_ASCII, "Blank.esm", "missing.esp"],
        );

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            NON_ASCII,
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_add_missing_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

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
    fn load_should_recognise_light_master_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        copy_to_test_dir("Blank.esm", "ccTest.esl", load_order.game_settings());

        load_order.load().unwrap();

        assert!(load_order.plugin_names().contains(&"ccTest.esl"));
    }

    #[test]
    fn load_should_add_missing_early_loading_plugins_in_their_hardcoded_positions() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Skyrim.esm", load_order.game_settings());
        copy_to_test_dir("Blank.esm", "Update.esm", load_order.game_settings());
        load_order.load().unwrap();
        assert_eq!(Some(1), load_order.index_of("Update.esm"));
        assert!(load_order.is_active("Update.esm"));
    }

    #[test]
    fn load_should_empty_the_load_order_if_the_plugins_directory_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());
        tmp_dir.close().unwrap();

        load_order.load().unwrap();

        assert!(load_order.plugins().is_empty());
    }

    #[test]
    fn load_should_load_plugin_states_from_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &[NON_ASCII, "Blank.esm"]);

        load_order.load().unwrap();
        let expected_filenames = vec!["Blank.esm", NON_ASCII];

        assert_eq!(expected_filenames, load_order.active_plugin_names());
    }

    #[test]
    fn load_should_succeed_when_active_plugins_file_is_missing() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        copy_to_test_dir("Blank.esm", "Skyrim.esm", load_order.game_settings());

        assert!(load_order.load().is_ok());
        assert_eq!(1, load_order.active_plugin_names().len());
    }

    #[test]
    fn load_should_not_duplicate_a_plugin_that_has_a_ghosted_duplicate() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        std::fs::copy(
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

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            NON_ASCII,
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_not_move_light_master_esp_files_before_non_masters() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        copy_to_test_dir("Blank.esl", "Blank.esl.esp", load_order.game_settings());

        load_order.load().unwrap();

        let expected_filenames = vec![
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            NON_ASCII,
            "Blank.esl.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_add_plugins_in_additional_plugins_directory_before_those_in_main_plugins_directory(
    ) {
        let tmp_dir = tempdir().unwrap();
        let game_path = tmp_dir.path().join("Fallout 4/Content");
        create_dir_all(&game_path).unwrap();

        File::create(game_path.join("appxmanifest.xml")).unwrap();

        let mut load_order = prepare(GameId::Fallout4, &game_path);

        copy_to_test_dir("Blank.esm", "Blank.esm", load_order.game_settings());

        let dlc_path = tmp_dir
            .path()
            .join("Fallout 4- Far Harbor (PC)/Content/Data");
        create_dir_all(&dlc_path).unwrap();
        copy_to_dir("Blank.esm", &dlc_path, "DLCCoast.esm", GameId::Fallout4);
        copy_to_dir("Blank.esp", &dlc_path, "Blank DLC.esp", GameId::Fallout4);

        load_order.load().unwrap();

        let expected_filenames = vec![
            "DLCCoast.esm",
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            NON_ASCII,
            "Blank DLC.esp",
        ];

        assert_eq!(expected_filenames, load_order.plugin_names());
    }

    #[test]
    fn load_should_ignore_active_plugins_file_for_fallout4_when_test_files_are_configured() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Fallout4.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        let mut load_order = prepare(GameId::Fallout4, tmp_dir.path());

        write_active_plugins_file(
            load_order.game_settings(),
            &["Blank.esp", "Blank - Master Dependent.esp"],
        );

        load_order.load().unwrap();

        assert_eq!(vec!["Blank.esp"], load_order.active_plugin_names());
    }

    #[test]
    fn load_should_use_test_files_in_place_of_plugins_txt_for_starfield() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/StarfieldCustom.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(
            &ini_path,
            "[General]\nsTestFile1=Blank.full.esm\nsTestFile2=Blank.medium.esm",
        )
        .unwrap();

        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &["Blank.esp"]);

        load_order.load().unwrap();

        assert_eq!(
            vec!["Blank.full.esm", "Blank.medium.esm"],
            load_order.active_plugin_names()
        );
        assert_eq!(
            vec![
                "Blank.full.esm",
                "Blank.medium.esm",
                "Blank.small.esm",
                "Blank.esp",
                "Blank - Override.esp"
            ],
            load_order.plugin_names()
        );
    }

    #[test]
    fn load_should_activate_blueprint_ships_plugins_for_active_starfield_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let filenames = &[
            "starfield.esm",
            "BlueprintShips-Starfield.esm",
            "A.esm",
            "BlueprintShips-a.esm",
            "BlueprintShips-B.esm",
            "BlueprintShips-Blank.esm",
        ];

        for filename in filenames {
            copy_to_test_dir("Blank.full.esm", filename, load_order.game_settings());
        }

        write_active_plugins_file(load_order.game_settings(), &["A.esm"]);

        load_order.load().unwrap();

        assert_eq!(
            &[
                "starfield.esm",
                "A.esm",
                "BlueprintShips-Starfield.esm",
                "BlueprintShips-a.esm",
            ],
            load_order.active_plugin_names().as_slice()
        );
        assert!(!load_order
            .find_plugin("starfield.esm")
            .unwrap()
            .is_explicitly_active());
        assert!(load_order
            .find_plugin("A.esm")
            .unwrap()
            .is_explicitly_active());
        assert!(!load_order
            .find_plugin("BlueprintShips-Starfield.esm")
            .unwrap()
            .is_explicitly_active());
        assert!(!load_order
            .find_plugin("BlueprintShips-a.esm")
            .unwrap()
            .is_explicitly_active());
    }

    #[test]
    fn save_should_create_active_plugins_file_parent_directory_if_it_does_not_exist() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

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
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        load_order.save().unwrap();

        load_order.load().unwrap();
        assert_eq!(vec!["Blank.esp"], load_order.active_plugin_names());
    }

    #[test]
    fn save_should_write_unghosted_plugin_names() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        copy_to_test_dir(
            "Blank - Different.esm",
            "ghosted.esm.ghost",
            load_order.game_settings(),
        );
        let plugin = Plugin::new("ghosted.esm.ghost", load_order.game_settings()).unwrap();
        load_order.plugins_mut().push(plugin);

        load_order.save().unwrap();

        let lines = read_lines(load_order.game_settings().active_plugins_file());

        assert_eq!(
            vec!["*Blank.esp", "Blank - Different.esp", "ghosted.esm"],
            lines
        );
    }

    #[test]
    fn save_should_error_if_a_plugin_filename_cannot_be_encoded_in_windows_1252() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        let filename = "Bl\u{0227}nk.esm";
        copy_to_test_dir(
            "Blank - Different.esm",
            filename,
            load_order.game_settings(),
        );
        let plugin = Plugin::new(filename, load_order.game_settings()).unwrap();
        load_order.plugins_mut().push(plugin);

        match load_order.save().unwrap_err() {
            Error::EncodeError(s) => assert_eq!("Bl\u{227}nk.esm", s),
            e => panic!("Expected encode error, got {e:?}"),
        }
    }

    #[test]
    fn save_should_omit_early_loading_plugins_from_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        copy_to_test_dir("Blank.esm", "HearthFires.esm", load_order.game_settings());
        let plugin = Plugin::new("HearthFires.esm", load_order.game_settings()).unwrap();
        load_order.plugins_mut().push(plugin);

        load_order.save().unwrap();

        let lines = read_lines(load_order.game_settings().active_plugins_file());

        assert_eq!(vec!["*Blank.esp", "Blank - Different.esp"], lines);
    }

    #[test]
    fn save_should_not_omit_implicitly_active_plugins_that_do_not_load_early() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Skyrim.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank - Different.esp").unwrap();

        let mut load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        load_order.load().unwrap();

        load_order.save().unwrap();

        let lines = read_lines(load_order.game_settings().active_plugins_file());

        assert_eq!(
            vec![
                "Blank.esm",
                "Blank.esp",
                "*Blank - Different.esp",
                "Blank - Master Dependent.esp",
                NON_ASCII,
            ],
            lines
        );
    }

    #[test]
    fn save_should_modify_plugin_timestamps_if_active_plugins_file_is_ignored() {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Fallout4.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        let mut load_order = prepare(GameId::Fallout4, tmp_dir.path());

        prepend_master(&mut load_order);
        let game_master_time = load_order.plugins[1].modification_time() - Duration::from_secs(1);
        load_order.plugins[0]
            .set_modification_time(game_master_time)
            .unwrap();

        let filename = "Blank.esp";
        let plugin_path = load_order.game_settings.plugins_directory().join(filename);

        let original_timestamp = plugin_path.metadata().unwrap().modified().unwrap();

        assert_eq!(1, load_order.index_of(filename).unwrap());
        MutableLoadOrder::set_plugin_index(&mut load_order, filename, 2).unwrap();

        load_order.save().unwrap();

        let new_timestamp = plugin_path.metadata().unwrap().modified().unwrap();

        assert_eq!(
            original_timestamp + std::time::Duration::from_secs(60),
            new_timestamp
        );
    }

    #[test]
    fn save_should_not_modify_plugin_timestamps_if_active_plugins_file_is_not_ignored() {
        let tmp_dir = tempdir().unwrap();

        let mut load_order = prepare(GameId::Fallout4, tmp_dir.path());

        prepend_master(&mut load_order);
        let game_master_time = load_order.plugins[1].modification_time() - Duration::from_secs(1);
        load_order.plugins[0]
            .set_modification_time(game_master_time)
            .unwrap();

        let filename = "Blank.esp";
        let plugin_path = load_order.game_settings.plugins_directory().join(filename);

        let original_timestamp = plugin_path.metadata().unwrap().modified().unwrap();

        assert_eq!(1, load_order.index_of(filename).unwrap());
        MutableLoadOrder::set_plugin_index(&mut load_order, filename, 2).unwrap();

        load_order.save().unwrap();

        let new_timestamp = plugin_path.metadata().unwrap().modified().unwrap();

        assert_eq!(original_timestamp, new_timestamp);
    }

    #[test]
    fn save_should_not_write_blueprint_plugins_to_plugins_txt() {
        let tmp_dir = tempdir().unwrap();

        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let plugin_name1 = "Blueprint1.esp";
        let plugin_name2 = "Blueprint2.esp";
        copy_as_blueprint_plugin(&load_order.game_settings, plugin_name1);
        copy_as_blueprint_plugin(&load_order.game_settings, plugin_name2);
        load_order.add(plugin_name1).unwrap();
        load_order
            .find_plugin_mut(plugin_name1)
            .unwrap()
            .activate()
            .unwrap();
        load_order.add(plugin_name2).unwrap();

        load_order.save().unwrap();

        let contents =
            std::fs::read_to_string(load_order.game_settings.active_plugins_file()).unwrap();
        assert_eq!("*Blank.esp\n", contents);
    }

    #[test]
    fn save_should_not_write_blueprint_ships_plugins_to_plugins_txt() {
        let tmp_dir = tempdir().unwrap();

        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let plugin_name1 = "BlueprintShips-A.esm";
        let plugin_name2 = "BlueprintShips-B.esm";
        copy_to_test_dir("Blank.full.esm", plugin_name1, &load_order.game_settings);
        copy_to_test_dir("Blank.full.esm", plugin_name2, &load_order.game_settings);
        load_order.add(plugin_name1).unwrap();
        load_order
            .find_plugin_mut(plugin_name1)
            .unwrap()
            .activate()
            .unwrap();
        load_order.add(plugin_name2).unwrap();

        load_order.save().unwrap();

        let contents =
            std::fs::read_to_string(load_order.game_settings.active_plugins_file()).unwrap();
        assert_eq!("*Blank.esp\n", contents);
    }

    #[test]
    fn save_should_not_write_plugins_that_start_with_blueprint_ships_to_plugins_txt() {
        let tmp_dir = tempdir().unwrap();

        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let plugin_name1 = "BlueprintShips-A.esp";
        let plugin_name2 = "BlueprintShips-B.esp";
        copy_to_test_dir("Blank.full.esm", plugin_name1, &load_order.game_settings);
        copy_to_test_dir("Blank.full.esm", plugin_name2, &load_order.game_settings);
        load_order.add(plugin_name1).unwrap();
        load_order
            .find_plugin_mut(plugin_name1)
            .unwrap()
            .activate()
            .unwrap();
        load_order.add(plugin_name2).unwrap();

        load_order.save().unwrap();

        let contents =
            std::fs::read_to_string(load_order.game_settings.active_plugins_file()).unwrap();
        assert_eq!("*Blank.esp\n", contents);
    }

    #[test]
    fn save_should_update_blueprint_master_timestamps_to_reflect_load_order() {
        let tmp_dir = tempdir().unwrap();

        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let plugin_name1 = "Blueprint1.esp";
        let plugin_name2 = "Blueprint2.esp";
        copy_as_blueprint_plugin(&load_order.game_settings, plugin_name1);
        copy_as_blueprint_plugin(&load_order.game_settings, plugin_name2);

        let plugin_path1 = load_order
            .game_settings
            .plugins_directory()
            .join(plugin_name1);
        let plugin_path2 = load_order
            .game_settings
            .plugins_directory()
            .join(plugin_name2);

        let first_timestamp = plugin_path1.metadata().unwrap().modified().unwrap();
        File::options()
            .write(true)
            .open(&plugin_path2)
            .unwrap()
            .set_modified(first_timestamp + Duration::from_secs(1))
            .unwrap();

        load_order.load().unwrap();

        let last_index = load_order.plugins.len() - 1;
        load_order.plugins.swap(last_index - 1, last_index);

        load_order.save().unwrap();

        let plugin_timestamp1 = plugin_path1.metadata().unwrap().modified().unwrap();
        let plugin_timestamp2 = plugin_path2.metadata().unwrap().modified().unwrap();

        assert_eq!(first_timestamp, plugin_timestamp2);
        assert_eq!(first_timestamp + Duration::from_secs(1), plugin_timestamp1);
    }

    #[test]
    fn is_self_consistent_should_return_true() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        assert!(load_order.is_self_consistent().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_false_if_all_loaded_plugins_are_listed_in_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        let loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();
        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_ignore_plugins_that_are_listed_in_active_plugins_file_but_not_loaded() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

        assert!(load_order.index_of("missing.esp").is_none());

        let mut loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();
        loaded_plugin_names.push("missing.esp");

        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_ignore_blueprint_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        let blueprint_plugin_name = "Blueprint.esp";
        copy_as_blueprint_plugin(&load_order.game_settings, blueprint_plugin_name);
        let plugin = Plugin::new(blueprint_plugin_name, load_order.game_settings()).unwrap();
        load_order.plugins_mut().push(plugin);

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_ignore_blueprint_ships_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(GameId::Starfield, tmp_dir.path());

        let loaded_plugin_names: Vec<&str> = load_order
            .plugins
            .iter()
            .map(crate::plugin::Plugin::name)
            .collect();

        write_active_plugins_file(load_order.game_settings(), &loaded_plugin_names);

        copy_to_test_dir(
            "Blank.full.esm",
            "BlueprintShips-Blank.esm",
            load_order.game_settings(),
        );
        let plugin = Plugin::new("BlueprintShips-Blank.esm", load_order.game_settings()).unwrap();
        load_order.plugins_mut().push(plugin);

        assert!(!load_order.is_ambiguous().unwrap());
    }

    #[test]
    fn is_ambiguous_should_return_true_if_there_are_loaded_plugins_not_in_active_plugins_file() {
        let tmp_dir = tempdir().unwrap();
        let load_order = prepare(GameId::SkyrimSE, tmp_dir.path());

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
    fn is_ambiguous_should_ignore_the_active_plugins_file_for_fallout4_when_test_files_are_configured(
    ) {
        let tmp_dir = tempdir().unwrap();

        let ini_path = tmp_dir.path().join("my games/Fallout4.ini");
        create_parent_dirs(&ini_path).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        let load_order = prepare(GameId::Fallout4, tmp_dir.path());

        write_active_plugins_file(load_order.game_settings(), &load_order.plugin_names());

        assert!(load_order.is_ambiguous().unwrap());
    }
}

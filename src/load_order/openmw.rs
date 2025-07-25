use std::{collections::HashSet, path::PathBuf};

use unicase::UniCase;

use crate::{
    load_order::mutable::filename_str,
    openmw_config::{non_user_additional_data_paths, read_active_plugin_names, write_openmw_cfg},
    plugin::{iends_with_ascii, Plugin},
    Error, GameId, GameSettings,
};

use super::{
    mutable::MutableLoadOrder,
    readable::{ReadableLoadOrder, ReadableLoadOrderBase},
    writable::{activate, add, deactivate, remove, set_active_plugins},
    WritableLoadOrder,
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct OpenMWLoadOrder {
    game_settings: GameSettings,
    plugins: Vec<Plugin>,
}

impl OpenMWLoadOrder {
    pub(crate) fn new(game_settings: GameSettings) -> Self {
        Self {
            game_settings,
            plugins: Vec::new(),
        }
    }

    fn read_from_active_plugins_file(&self) -> Result<Vec<(String, bool)>, Error> {
        let path = self.game_settings().active_plugins_file();

        let active_plugin_tuples: Vec<_> = read_active_plugin_names(path)?
            .into_iter()
            .map(|v| (v, true))
            .collect();

        Ok(active_plugin_tuples)
    }

    fn apply_load_order(&mut self, active_plugins: &[(String, bool)]) {
        // This takes a similar approach to that of the OpenMW Launcher so that
        // the load order that libloadorder reads should be the same as
        // displayed in the OpenMW Launcher, though the Launcher hides some
        // plugins when their master is inactive.
        // For reference the launcher implementation is at:
        // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-0.49.0/components/contentselector/model/contentmodel.cpp?ref_type=tags#L538>

        // At this point libloadorder has inserted the early loaders in order at
        // the top of the load order, but everything else in their file path
        // order.

        // The launcher does a few things at once:
        //
        // - it moves Tribunal.esm before Bloodmoon.esm
        // - it moves the game file that's currently selected in the Launcher to
        //   the top of the load order
        // - it moves plugins so that they load immediately before the earliest
        //   plugin that has them as a master.

        // From testing, it seems that the Launcher defaults the game file to
        // the first active game file in the load order (which is a little
        // awkward because it removes the current game file from the list, so
        // you can't see where it actually loads in relation to the others).
        // A plugin is a game file if it has no masters and ends with .esm or
        // .omwgame.
        let game_file_name = self
            .plugins()
            .iter()
            .find(|p| {
                (iends_with_ascii(p.name(), ".esm") || iends_with_ascii(p.name(), ".omwgame"))
                    && p.masters().unwrap_or_default().is_empty()
            })
            .map(|p| p.name().to_owned());

        let first_modifiable_index = self
            .plugins
            .iter()
            .position(|p| !self.game_settings.loads_early(p.name()))
            .unwrap_or(self.plugins.len());

        // This is adapted from OpenMW's logic at
        // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-0.49.0/components/contentselector/model/contentmodel.cpp?ref_type=tags#L575>
        let mut moved = HashSet::new();

        if self.plugins.is_empty() {
            // Return early to prevent underflow panic if there are no plugins
            // loaded.
            return;
        }

        let mut i = self.plugins.len() - 1;
        while i > first_modifiable_index {
            let Some(later_plugin) = self.plugins.get(i) else {
                // This should never happen.
                break;
            };

            let key = UniCase::new(later_plugin.name().to_owned());
            if !moved.contains(&key) {
                let index = self
                    .plugins
                    .iter()
                    .skip(first_modifiable_index)
                    .take(i - first_modifiable_index)
                    .position(|earlier_plugin| {
                        game_file_name
                            .as_ref()
                            .is_some_and(|n| n == later_plugin.name())
                            || (later_plugin.name_matches("Tribunal.esm")
                                && earlier_plugin.name_matches("Bloodmoon.esm"))
                            || earlier_plugin.has_master(later_plugin.name())
                    });

                if let Some(index) = index {
                    let plugin = self.plugins.remove(i);
                    self.plugins.insert(first_modifiable_index + index, plugin);
                    moved.insert(key);
                    continue;
                }
            }
            i -= 1;
            moved.clear();
        }

        // Finally, sort the active plugins according to their defined load
        // order. This is equivalent to the approach that the OpenMW Launcher
        // takes:
        // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-0.49.0/components/contentselector/model/contentmodel.cpp?ref_type=tags#L655>
        let mut previous_index = 0;
        for name_tuple in active_plugins {
            if !name_tuple.1 {
                // The name tuples should all be for active plugins, but check
                // just in case.
                continue;
            }

            if let Some(current_index) = self
                .plugins
                .iter()
                .position(|p| p.name_matches(&name_tuple.0))
            {
                if current_index < previous_index {
                    let plugin = self.plugins.remove(current_index);
                    self.plugins.insert(previous_index, plugin);
                } else {
                    previous_index = current_index;
                }
            }
        }
    }
}

impl ReadableLoadOrderBase for OpenMWLoadOrder {
    fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }

    fn game_settings_base(&self) -> &GameSettings {
        &self.game_settings
    }
}

impl MutableLoadOrder for OpenMWLoadOrder {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin> {
        &mut self.plugins
    }

    fn max_active_full_plugins(&self) -> usize {
        // Stated as the limit in the FAQs here:
        // <https://openmw.org/faq/>
        // The code shows that plugin indexes are stored as int32_t, with 0
        // being used as a null value and presumably save data taking up
        // another slot like in other games:
        // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-49-rc3/components/esm/formid.hpp?ref_type=tags#L16>
        0x7FFF_FFFE
    }

    fn total_insertion_order(
        defined_load_order: &[(String, bool)],
        installed_files: &[PathBuf],
        _: GameId,
    ) -> Vec<(String, bool)> {
        // The OpenMW Launcher lists files by the order of their data
        // directories, and sorting files by case-sensitive name within each
        // directory. That's already handled by GameSettings::find_plugins().
        // The OpenMW Launcher implementation is here:
        // <https://gitlab.com/OpenMW/openmw/-/blob/openmw-0.49.0/apps/launcher/datafilespage.cpp?ref_type=tags#L348>

        fn get_key_from_filename(filename: &str) -> UniCase<&str> {
            UniCase::new(filename)
        }

        let active_set: HashSet<_> = defined_load_order
            .iter()
            .map(|(n, _)| get_key_from_filename(n))
            .collect();

        let mut set: HashSet<_> = HashSet::with_capacity(installed_files.len());

        // If multiple file paths have the same filename, keep the first
        // occurrence. The file path used to load the plugin is the last one,
        // but that's handled by GameSettings::plugin_path().
        let unique_tuples: Vec<_> = installed_files
            .iter()
            .filter_map(|p| filename_str(p))
            .filter_map(|f| {
                let key = get_key_from_filename(f);
                set.insert(key)
                    .then_some((f.to_owned(), active_set.contains(&key)))
            })
            .collect();

        // At this point the active plugins are not in their load order, but the
        // OpenMW Launcher doesn't apply the load order until after it has done
        // a few things, like move the early loaders into their proper
        // positions. libloadorder moves early loaders after this function
        // completes, so defer setting the rest of the load order until later.

        unique_tuples
    }
}

impl WritableLoadOrder for OpenMWLoadOrder {
    fn game_settings_mut(&mut self) -> &mut GameSettings {
        &mut self.game_settings
    }

    fn load(&mut self) -> Result<(), Error> {
        self.plugins_mut().clear();

        let plugin_tuples = self.read_from_active_plugins_file()?;
        let paths = self.game_settings.find_plugins();

        self.load_unique_plugins(&plugin_tuples, &paths);

        self.add_implicitly_active_plugins()?;

        self.apply_load_order(&plugin_tuples);

        Ok(())
    }

    fn save(&mut self) -> Result<(), Error> {
        let read_only_data_paths: HashSet<_> =
            non_user_additional_data_paths(self.game_settings.game_path())?
                .into_iter()
                .collect();

        // Filter out the additional plugins dirs that come from read-only
        // sources (e.g. hardcoded values or global config).
        let data_paths: Vec<_> = self
            .game_settings
            .additional_plugins_directories()
            .iter()
            .filter(|p| !read_only_data_paths.contains(p.as_path()))
            .cloned()
            .collect();

        let cfg_path = self.game_settings.active_plugins_file();
        write_openmw_cfg(cfg_path, &data_paths, &self.active_plugin_names())?;

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

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir_all, write},
        path::Path,
    };

    use tempfile::tempdir;

    use crate::{
        load_order::tests::{game_settings_for_test, mock_game_files, prepare_bulk_full_plugins},
        tests::{copy_to_dir, create_file, NON_ASCII},
    };

    use super::*;

    fn cfg_path(tmp_path: &Path) -> PathBuf {
        tmp_path.join("my games").join("openmw.cfg")
    }

    fn prepare(tmp_path: &Path) -> OpenMWLoadOrder {
        let mut game_settings = game_settings_for_test(GameId::OpenMW, tmp_path);
        mock_game_files(&mut game_settings);

        OpenMWLoadOrder {
            game_settings,
            plugins: Vec::new(),
        }
    }

    fn write_cfg(cfg_path: &Path, data_paths: &[&str], content: &[&str]) {
        use std::fmt::Write;

        let mut file_content = String::new();
        for data_path in data_paths {
            writeln!(file_content, "data=\"{data_path}\"").unwrap();
        }

        for entry in content {
            writeln!(file_content, "content={entry}").unwrap();
        }

        if !cfg_path.exists() {
            create_dir_all(cfg_path.parent().unwrap()).unwrap();
        }

        write(cfg_path, file_content).unwrap();
    }

    fn read_lines(path: &Path) -> Vec<String> {
        let content = std::fs::read_to_string(path).unwrap();
        content.lines().map(ToOwned::to_owned).collect()
    }

    #[test]
    fn load_should_not_panic_if_no_plugins_are_installed() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = OpenMWLoadOrder {
            game_settings: game_settings_for_test(GameId::OpenMW, tmp_dir.path()),
            plugins: Vec::new(),
        };

        load_order.load().unwrap();

        assert!(load_order.plugins.is_empty());
    }

    #[test]
    fn load_should_read_active_plugin_load_order_from_cfg_file() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let cfg_path = cfg_path(tmp_dir.path());
        let active_plugin_names = &[
            "Blank.esm",
            "Blank - Master Dependent.esp",
            "Blank - Different.esp",
            NON_ASCII,
            "Blank.esp",
        ];
        write_cfg(&cfg_path, &[], active_plugin_names);

        load_order.load().unwrap();

        assert_eq!(active_plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn load_should_list_inactive_plugins_in_the_same_data_path_in_case_sensitive_lexicographical_order_unless_overridden(
    ) {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        load_order.load().unwrap();

        // Blank.esm is moved up because it looks like a game file.
        let plugin_names = &[
            "Blank.esm",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
            NON_ASCII,
        ];

        assert_eq!(plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn load_should_list_inactive_plugins_in_order_of_their_data_paths() {
        let tmp_dir = tempdir().unwrap();

        let other_dir_1 = tmp_dir.path().join("other1");
        let other_dir_2 = tmp_dir.path().join("other2");
        create_dir_all(&other_dir_1).unwrap();
        create_dir_all(&other_dir_2).unwrap();

        let cfg_path = cfg_path(tmp_dir.path());
        write_cfg(
            &cfg_path,
            &[other_dir_1.to_str().unwrap(), other_dir_2.to_str().unwrap()],
            &[],
        );

        let mut load_order = prepare(tmp_dir.path());
        let main_dir = load_order.game_settings.plugins_directory();

        copy_to_dir(
            "Blank - Different Master Dependent.esp",
            &main_dir,
            "Blank - Different Master Dependent.esp",
            GameId::OpenMW,
        );
        copy_to_dir(
            "Blank.esm",
            &main_dir,
            "Blank - Different.esm",
            GameId::OpenMW,
        );
        copy_to_dir("Blank.esp", &other_dir_1, "Blank.esp", GameId::OpenMW);
        copy_to_dir(
            "Blank - Different.esp",
            &other_dir_2,
            "Blank - Different.esp",
            GameId::OpenMW,
        );
        copy_to_dir("Blank.esp", &other_dir_2, NON_ASCII, GameId::OpenMW);

        // std::fs::remove_file(main_dir.join("Blank.esm")).unwrap();
        std::fs::remove_file(main_dir.join("Blank.esp")).unwrap();
        std::fs::remove_file(main_dir.join("Blank - Different.esp")).unwrap();
        std::fs::remove_file(main_dir.join(NON_ASCII)).unwrap();

        load_order.load().unwrap();

        // Blank - Different.esm is moved up because it looks like a game file.
        // Blank.esm is moved up because it's a master of
        // Blank - Master Dependent.esp.
        let plugin_names = &[
            "Blank - Different.esm",
            "Blank - Different Master Dependent.esp",
            "Blank.esm",
            "Blank - Master Dependent.esp",
            "Blank.esp",
            "Blank - Different.esp",
            NON_ASCII,
        ];

        assert_eq!(plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn load_should_move_game_file_immediately_below_last_early_loader() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());
        let main_dir = load_order.game_settings.plugins_directory();

        create_file(&main_dir.join("builtin.omwscripts"));

        load_order.load().unwrap();

        // Blank.esm is moved up because it looks like a game file.
        let plugin_names = &[
            "builtin.omwscripts",
            "Blank.esm",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
            NON_ASCII,
        ];

        assert_eq!(plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn load_should_list_tribunal_before_bloodmoon() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let main_dir = load_order.game_settings.plugins_directory();
        copy_to_dir("Blank.esm", &main_dir, "Tribunal.esm", GameId::OpenMW);
        copy_to_dir("Blank.esm", &main_dir, "Bloodmoon.esm", GameId::OpenMW);

        load_order.load().unwrap();

        // Blank.esm is moved up because it looks like a game file.
        let plugin_names = &[
            "Blank.esm",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank.esp",
            "Tribunal.esm",
            "Bloodmoon.esm",
            NON_ASCII,
        ];

        assert_eq!(plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn load_should_interleave_active_and_inactive_plugins_by_parent_path_and_lexicographically() {
        let tmp_dir = tempdir().unwrap();

        let other_dir_1 = tmp_dir.path().join("other1");
        let other_dir_2 = tmp_dir.path().join("other2");
        create_dir_all(&other_dir_1).unwrap();
        create_dir_all(&other_dir_2).unwrap();

        let cfg_path = cfg_path(tmp_dir.path());
        write_cfg(
            &cfg_path,
            &[other_dir_1.to_str().unwrap(), other_dir_2.to_str().unwrap()],
            &["Blank - Different.esp", "Blank - Master Dependent.esp"],
        );

        copy_to_dir("Blank.esm", &other_dir_1, "Blank.esm", GameId::OpenMW);
        copy_to_dir("Blank.esp", &other_dir_1, "Blank.esp", GameId::OpenMW);
        copy_to_dir(
            "Blank - Different.esp",
            &other_dir_2,
            "Blank - Different.esp",
            GameId::OpenMW,
        );
        copy_to_dir("Blank.esp", &other_dir_2, NON_ASCII, GameId::OpenMW);

        let mut load_order = prepare(tmp_dir.path());
        let main_dir = load_order.game_settings.plugins_directory();

        std::fs::remove_file(main_dir.join("Blank.esp")).unwrap();
        std::fs::remove_file(main_dir.join("Blank - Different.esp")).unwrap();
        std::fs::remove_file(main_dir.join(NON_ASCII)).unwrap();

        load_order.load().unwrap();

        let plugin_names = &[
            "Blank.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            NON_ASCII,
        ];

        assert_eq!(plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn load_should_support_openmw_plugins() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let parent_path = load_order.game_settings.plugins_directory();
        let cfg_path = cfg_path(tmp_dir.path());
        let active_plugin_names = &[
            "Blank.omwgame",
            "Blank - Master Dependent.esp",
            "Blank - Different.omwaddon",
            NON_ASCII,
            "Blank.omwscripts",
            "Blank.esp",
        ];
        write_cfg(&cfg_path, &[], active_plugin_names);

        std::fs::rename(
            parent_path.join("Blank.esm"),
            parent_path.join("Blank.omwgame"),
        )
        .unwrap();
        std::fs::rename(
            parent_path.join("Blank - Different.esp"),
            parent_path.join("Blank - Different.omwaddon"),
        )
        .unwrap();
        std::fs::write(parent_path.join("Blank.omwscripts"), "").unwrap();

        load_order.load().unwrap();

        assert_eq!(active_plugin_names, load_order.plugin_names().as_slice());
    }

    #[test]
    fn save_should_write_active_openmw_plugin_positions() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let cfg_path = cfg_path(tmp_dir.path());

        let parent_path = load_order.game_settings.plugins_directory();
        let active_plugin_names = &[
            "Blank.omwgame",
            "Blank - Master Dependent.esp",
            "Blank - Different.omwaddon",
            NON_ASCII,
            "Blank.omwscripts",
            "Blank.esp",
        ];

        std::fs::rename(
            parent_path.join("Blank.esm"),
            parent_path.join("Blank.omwgame"),
        )
        .unwrap();
        std::fs::rename(
            parent_path.join("Blank - Different.esp"),
            parent_path.join("Blank - Different.omwaddon"),
        )
        .unwrap();
        std::fs::write(parent_path.join("Blank.omwscripts"), "").unwrap();

        for plugin_name in active_plugin_names {
            let plugin =
                Plugin::with_active(plugin_name, load_order.game_settings(), true).unwrap();
            load_order.plugins.push(plugin);
        }

        load_order.save().unwrap();

        let lines = read_lines(&cfg_path);

        let expected_lines: Vec<_> = active_plugin_names
            .iter()
            .map(|n| format!("content={n}"))
            .collect();

        assert_eq!(expected_lines, lines);
    }

    #[test]
    fn save_should_write_data_paths() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let cfg_path = cfg_path(tmp_dir.path());

        write_cfg(
            &cfg_path,
            &["C:\\Games\\Morrowind\\Data Files", "C:\\Other\\Directory"],
            &["a.esm", "b.esm"],
        );

        load_order
            .game_settings
            .set_additional_plugins_directories(vec!["C:\\Path\\&\"a&&\\Data Files".into()]);

        load_order.save().unwrap();

        let lines = read_lines(&cfg_path);

        let expected_lines = vec!["data=\"C:\\Path\\&&&\"a&&&&\\Data Files\"".to_owned()];

        assert_eq!(expected_lines, lines);
    }

    #[test]
    fn save_should_skip_writing_data_paths_that_are_in_global_config() {
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let global_cfg_path = load_order.game_settings.game_path().join("openmw.cfg");

        write_cfg(
            &global_cfg_path,
            &["C:\\Games\\Morrowind\\Data Files", "C:\\Other\\Directory"],
            &["a.esm", "b.esm"],
        );

        load_order.save().unwrap();

        let lines = read_lines(&cfg_path(tmp_dir.path()));

        assert!(lines.is_empty());
    }

    #[test]
    fn activate_should_allow_more_than_255_plugins_to_be_active() {
        // The limit is over 2 billion, don't test activating that many plugins.
        let tmp_dir = tempdir().unwrap();
        let mut load_order = prepare(tmp_dir.path());

        let plugins = prepare_bulk_full_plugins(&mut load_order);
        for plugin in &plugins[..260] {
            load_order.activate(plugin).unwrap();
            assert!(load_order.is_active(plugin));
        }
    }
}

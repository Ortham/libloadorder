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

use std::collections::HashSet;
use std::mem;

use walkdir::WalkDir;

use game_settings::GameSettings;
use load_order::{find_first_non_master_position, match_plugin};
use load_order::error::LoadOrderError;
use load_order::readable::ReadableLoadOrder;
use plugin::Plugin;

pub const MAX_ACTIVE_PLUGINS: usize = 255;

pub trait MutableLoadOrder: ReadableLoadOrder {
    fn game_settings(&self) -> &GameSettings;
    fn mut_plugins(&mut self) -> &mut Vec<Plugin>;

    fn insert_position(&self, plugin: &Plugin) -> Option<usize>;

    fn add_to_load_order(&mut self, plugin_name: &str) -> Result<usize, LoadOrderError> {
        let plugin = Plugin::new(plugin_name, self.game_settings())?;

        let index = match self.insert_position(&plugin) {
            Some(x) => {
                self.mut_plugins().insert(x, plugin);
                x
            }
            None => {
                self.mut_plugins().push(plugin);
                self.plugins().len() - 1
            }
        };

        Ok(index)
    }

    fn count_active_plugins(&self) -> usize {
        self.plugins().iter().filter(|p| p.is_active()).count()
    }

    fn add_missing_plugins(&mut self) -> Result<(), LoadOrderError> {
        let filenames: Vec<String> = WalkDir::new(self.game_settings().plugins_directory())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                e.file_name().to_str().and_then(
                    |f| if !self.game_settings().is_implicitly_active(f) &&
                        self.index_of(f).is_none() &&
                        Plugin::is_valid(
                            f,
                            self.game_settings(),
                        )
                    {
                        Some(f.to_string())
                    } else {
                        None
                    },
                )
            })
            .collect();

        for filename in filenames {
            self.add_to_load_order(&filename)?;
        }

        Ok(())
    }

    fn find_or_add(&mut self, filename: &str) -> Result<usize, LoadOrderError> {
        let index = match self.index_of(filename) {
            Some(x) => x,
            None => self.add_to_load_order(filename)?,
        };

        Ok(index)
    }

    fn add_implicitly_active_plugins(&mut self) -> Result<(), LoadOrderError> {
        for filename in self.game_settings().implicitly_active_plugins() {
            if self.is_active(filename) || !Plugin::is_valid(filename, self.game_settings()) {
                continue;
            }

            let index = self.find_or_add(filename)?;
            self.mut_plugins()[index].activate()?;
        }

        Ok(())
    }

    fn deactivate_excess_plugins(&mut self) {
        let implicitly_active_plugins = self.game_settings().implicitly_active_plugins();
        let mut count = self.count_active_plugins();

        for plugin in self.mut_plugins().iter_mut().rev() {
            if count <= MAX_ACTIVE_PLUGINS {
                break;
            }
            if plugin.is_active() &&
                !implicitly_active_plugins.iter().any(
                    |i| match_plugin(plugin, i),
                )
            {
                plugin.deactivate();
                count -= 1;
            }
        }
    }

    fn move_or_insert_plugin(
        &mut self,
        plugin_name: &str,
        position: usize,
    ) -> Result<(), LoadOrderError> {
        let plugin = get_plugin_to_insert(self, plugin_name, position)?;

        if position >= self.plugins().len() {
            self.mut_plugins().push(plugin);
        } else {
            self.mut_plugins().insert(position, plugin);
        }

        Ok(())
    }

    fn replace_plugins(&mut self, plugin_names: &[&str]) -> Result<(), LoadOrderError> {
        validate_plugin_names(plugin_names, self.game_settings())?;

        let mut plugins = map_to_plugins(self, plugin_names)?;

        if !is_partitioned_by_master_flag(&plugins) {
            return Err(LoadOrderError::NonMasterBeforeMaster);
        }

        mem::swap(&mut plugins, self.mut_plugins());

        self.add_missing_plugins()?;

        Ok(())
    }

    fn find_plugin_mut<'a>(&'a mut self, plugin_name: &str) -> Option<&'a mut Plugin> {
        self.mut_plugins().iter_mut().find(|p| {
            match_plugin(p, plugin_name)
        })
    }

    //TODO: Profile if the 'has changed' check is actually necessary.
    fn reload_changed_plugins(&mut self) {
        let plugins = self.mut_plugins();
        for i in (0..plugins.len()).rev() {
            let should_remove = plugins[i]
                .has_file_changed()
                .and_then(|has_changed| if has_changed {
                    plugins[i].reload()
                } else {
                    Ok(())
                })
                .is_err();
            if should_remove {
                plugins.remove(i);
            }
        }
    }
}

fn validate_index<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    index: usize,
    is_master: bool,
) -> Result<(), LoadOrderError> {
    match find_first_non_master_position(load_order.plugins()) {
        None if !is_master && index < load_order.plugins().len() => Err(
            LoadOrderError::NonMasterBeforeMaster,
        ),
        Some(i) if is_master && index > i || !is_master && index < i => Err(
            LoadOrderError::NonMasterBeforeMaster,
        ),
        _ => Ok(()),
    }
}

fn get_plugin_to_insert<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    plugin_name: &str,
    position: usize,
) -> Result<Plugin, LoadOrderError> {
    if let Some(i) = load_order.plugins().iter().position(
        |p| match_plugin(p, plugin_name),
    )
    {
        let is_master = load_order.plugins()[i].is_master_file();
        validate_index(load_order, position, is_master)?;

        Ok(load_order.mut_plugins().remove(i))
    } else {
        if !Plugin::is_valid(plugin_name, load_order.game_settings()) {
            return Err(LoadOrderError::InvalidPlugin(plugin_name.to_string()));
        }

        let plugin = Plugin::new(plugin_name, load_order.game_settings())?;

        validate_index(load_order, position, plugin.is_master_file())?;

        Ok(plugin)
    }
}

fn validate_plugin_names(
    plugin_names: &[&str],
    game_settings: &GameSettings,
) -> Result<(), LoadOrderError> {
    let unique_plugin_names: HashSet<String> =
        plugin_names.iter().map(|s| s.to_lowercase()).collect();

    if unique_plugin_names.len() != plugin_names.len() {
        return Err(LoadOrderError::DuplicatePlugin);
    }

    let invalid_plugin = plugin_names.iter().find(
        |p| !Plugin::is_valid(p, game_settings),
    );

    match invalid_plugin {
        Some(x) => Err(LoadOrderError::InvalidPlugin(x.to_string())),
        None => Ok(()),
    }
}

fn to_plugin(
    plugin_name: &str,
    existing_plugins: &[Plugin],
    game_settings: &GameSettings,
) -> Result<Plugin, LoadOrderError> {
    match existing_plugins.iter().find(
        |p| match_plugin(p, plugin_name),
    ) {
        None => Ok(Plugin::new(plugin_name, game_settings)?),
        Some(x) => Ok(x.clone()),
    }
}

fn map_to_plugins<T: MutableLoadOrder + ?Sized>(
    load_order: &T,
    plugin_names: &[&str],
) -> Result<Vec<Plugin>, LoadOrderError> {
    plugin_names
        .iter()
        .map(|n| {
            to_plugin(n, load_order.plugins(), load_order.game_settings())
        })
        .collect()
}

fn is_partitioned_by_master_flag(plugins: &[Plugin]) -> bool {
    let plugin_pos = match find_first_non_master_position(plugins) {
        None => return true,
        Some(x) => x,
    };
    match plugins.iter().rposition(|p| p.is_master_file()) {
        None => true,
        Some(master_pos) => master_pos < plugin_pos,
    }
}

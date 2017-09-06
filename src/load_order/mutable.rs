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
use std::mem;

use walkdir::WalkDir;

use enums::Error;
use game_settings::GameSettings;
use load_order::{find_first_non_master_position, read_plugin_names};
use load_order::readable::ReadableLoadOrder;
use plugin::Plugin;

pub const MAX_ACTIVE_PLUGINS: usize = 255;

pub trait MutableLoadOrder: ReadableLoadOrder {
    fn game_settings(&self) -> &GameSettings;
    fn plugins_mut(&mut self) -> &mut Vec<Plugin>;

    fn insert_position(&self, plugin: &Plugin) -> Option<usize>;

    fn add_to_load_order(&mut self, plugin_name: &str) -> Result<usize, Error> {
        let plugin = Plugin::new(plugin_name, self.game_settings())?;

        let index = match self.insert_position(&plugin) {
            Some(x) => {
                self.plugins_mut().insert(x, plugin);
                x
            }
            None => {
                self.plugins_mut().push(plugin);
                self.plugins().len() - 1
            }
        };

        Ok(index)
    }

    fn count_active_plugins(&self) -> usize {
        self.plugins().iter().filter(|p| p.is_active()).count()
    }

    //TODO: Profile vs. C++ libloadorder to see if caching plugins folder timestamp is worth it
    fn add_missing_plugins(&mut self) -> Result<(), Error> {
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

    fn find_or_add(&mut self, filename: &str) -> Result<usize, Error> {
        let index = match self.index_of(filename) {
            Some(x) => x,
            None => self.add_to_load_order(filename)?,
        };

        Ok(index)
    }

    fn add_implicitly_active_plugins(&mut self) -> Result<(), Error> {
        for filename in self.game_settings().implicitly_active_plugins() {
            if self.is_active(filename) || !Plugin::is_valid(filename, self.game_settings()) {
                continue;
            }

            let index = self.find_or_add(filename)?;
            self.plugins_mut()[index].activate()?;
        }

        Ok(())
    }

    fn deactivate_excess_plugins(&mut self) {
        let implicitly_active_plugins = self.game_settings().implicitly_active_plugins();
        let mut count = self.count_active_plugins();

        for plugin in self.plugins_mut().iter_mut().rev() {
            if count <= MAX_ACTIVE_PLUGINS {
                break;
            }
            if plugin.is_active() &&
                !implicitly_active_plugins.iter().any(
                    |i| plugin.name_matches(i),
                )
            {
                plugin.deactivate();
                count -= 1;
            }
        }
    }

    fn move_or_insert_plugin(&mut self, plugin_name: &str) -> Result<(), Error> {
        let plugin = get_plugin_to_insert(self, plugin_name, None)?;
        let position = self.insert_position(&plugin);

        match position {
            None => self.plugins_mut().push(plugin),
            Some(x) => self.plugins_mut().insert(x, plugin),
        }

        Ok(())
    }

    fn move_or_insert_plugin_with_index(
        &mut self,
        plugin_name: &str,
        position: usize,
    ) -> Result<(), Error> {
        if let Some(x) = self.index_of(plugin_name) {
            if x == position {
                return Ok(());
            }
        }

        let plugin = get_plugin_to_insert(self, plugin_name, Some(position))?;

        if position >= self.plugins().len() {
            self.plugins_mut().push(plugin);
        } else {
            self.plugins_mut().insert(position, plugin);
        }

        Ok(())
    }

    fn replace_plugins(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        validate_plugin_names(plugin_names, self.game_settings())?;

        let mut plugins = map_to_plugins(self, plugin_names)?;

        if !is_partitioned_by_master_flag(&plugins) {
            return Err(Error::NonMasterBeforeMaster);
        }

        mem::swap(&mut plugins, self.plugins_mut());

        self.add_missing_plugins()?;

        self.add_implicitly_active_plugins()
    }

    fn find_plugin_mut<'a>(&'a mut self, plugin_name: &str) -> Option<&'a mut Plugin> {
        self.plugins_mut().iter_mut().find(
            |p| p.name_matches(plugin_name),
        )
    }

    //TODO: Profile if the 'has changed' check is actually necessary.
    fn reload_changed_plugins(&mut self) {
        let plugins = self.plugins_mut();
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

pub fn load_active_plugins<T, F>(load_order: &mut T, line_mapper: F) -> Result<(), Error>
where
    T: MutableLoadOrder,
    F: Fn(Vec<u8>) -> Result<String, Error>,
{
    for plugin in load_order.plugins_mut() {
        plugin.deactivate();
    }

    let plugin_names = read_plugin_names(
        load_order.game_settings().active_plugins_file(),
        line_mapper,
    )?;

    for plugin_name in plugin_names {
        let index = load_order.find_or_add(&plugin_name)?;
        load_order.plugins_mut()[index].activate()?;
    }

    Ok(())
}

fn validate_index<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    index: usize,
    is_master: bool,
) -> Result<(), Error> {
    match find_first_non_master_position(load_order.plugins()) {
        None if !is_master && index < load_order.plugins().len() => Err(
            Error::NonMasterBeforeMaster,
        ),
        Some(i) if is_master && index > i || !is_master && index < i => Err(
            Error::NonMasterBeforeMaster,
        ),
        _ => Ok(()),
    }
}

fn get_plugin_to_insert<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    plugin_name: &str,
    insert_position: Option<usize>,
) -> Result<Plugin, Error> {
    let plugin_position = load_order.plugins().iter().position(
        |p| p.name_matches(plugin_name),
    );
    if let Some(p) = plugin_position {
        if let Some(i) = insert_position {
            let is_master = load_order.plugins()[p].is_master_file();
            validate_index(load_order, i, is_master)?;
        }

        Ok(load_order.plugins_mut().remove(p))
    } else {
        if !Plugin::is_valid(plugin_name, load_order.game_settings()) {
            return Err(Error::InvalidPlugin(plugin_name.to_string()));
        }

        let plugin = Plugin::new(plugin_name, load_order.game_settings())?;

        if let Some(i) = insert_position {
            validate_index(load_order, i, plugin.is_master_file())?;
        }

        Ok(plugin)
    }
}

fn validate_plugin_names(plugin_names: &[&str], game_settings: &GameSettings) -> Result<(), Error> {
    let unique_plugin_names: HashSet<String> =
        plugin_names.iter().map(|s| s.to_lowercase()).collect();

    if unique_plugin_names.len() != plugin_names.len() {
        return Err(Error::DuplicatePlugin);
    }

    let invalid_plugin = plugin_names.iter().find(
        |p| !Plugin::is_valid(p, game_settings),
    );

    match invalid_plugin {
        Some(x) => Err(Error::InvalidPlugin(x.to_string())),
        None => Ok(()),
    }
}

fn to_plugin(
    plugin_name: &str,
    existing_plugins: &[Plugin],
    game_settings: &GameSettings,
) -> Result<Plugin, Error> {
    match existing_plugins.iter().find(
        |p| p.name_matches(plugin_name),
    ) {
        None => Ok(Plugin::new(plugin_name, game_settings)?),
        Some(x) => Ok(x.clone()),
    }
}

fn map_to_plugins<T: MutableLoadOrder + ?Sized>(
    load_order: &T,
    plugin_names: &[&str],
) -> Result<Vec<Plugin>, Error> {
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

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

use rayon::prelude::*;

use super::mutable::MutableLoadOrder;
use enums::Error;
use plugin::{trim_dot_ghost, Plugin};

pub trait InsertableLoadOrder: MutableLoadOrder {
    fn insert_position(&self, plugin: &Plugin) -> Option<usize>;

    fn insert(&mut self, plugin: Plugin) -> usize {
        match self.insert_position(&plugin) {
            Some(position) => {
                self.plugins_mut().insert(position, plugin);
                position
            }
            None => {
                self.plugins_mut().push(plugin);
                self.plugins().len() - 1
            }
        }
    }

    fn add_to_load_order(&mut self, plugin_name: &str) -> Result<usize, Error> {
        let plugin = Plugin::new(plugin_name, self.game_settings())?;

        Ok(self.insert(plugin))
    }

    fn find_or_add(&mut self, plugin_name: &str) -> Result<usize, Error> {
        match self.index_of(plugin_name) {
            Some(i) => Ok(i),
            None => self
                .add_to_load_order(plugin_name)
                .map_err(|_| Error::InvalidPlugin(plugin_name.to_string())),
        }
    }

    fn load_unique_plugins(
        &mut self,
        plugin_name_tuples: Vec<(String, bool)>,
        installed_filenames: Vec<String>,
    ) {
        let plugins: Vec<Plugin> = {
            let game_settings = self.game_settings();

            remove_duplicates_icase(plugin_name_tuples, installed_filenames)
                .into_par_iter()
                .filter_map(|(filename, active)| {
                    Plugin::with_active(&filename, game_settings, active).ok()
                })
                .collect()
        };

        for plugin in plugins {
            self.insert(plugin);
        }
    }

    fn add_implicitly_active_plugins(&mut self) -> Result<(), Error> {
        let plugin_names: Vec<String> = self
            .game_settings()
            .implicitly_active_plugins()
            .iter()
            .filter(|p| !self.is_active(p))
            .cloned()
            .collect();

        for plugin_name in plugin_names {
            activate_unvalidated(self, &plugin_name)?;
        }

        Ok(())
    }
}

fn remove_duplicates_icase(
    plugin_tuples: Vec<(String, bool)>,
    filenames: Vec<String>,
) -> Vec<(String, bool)> {
    let mut set: HashSet<String> = HashSet::with_capacity(filenames.len());

    let mut unique_tuples: Vec<(String, bool)> = plugin_tuples
        .into_iter()
        .rev()
        .filter(|&(ref string, _)| set.insert(trim_dot_ghost(&string).to_lowercase()))
        .collect();

    unique_tuples.reverse();

    let unique_file_tuples_iter = filenames
        .into_iter()
        .filter(|ref string| set.insert(trim_dot_ghost(&string).to_lowercase()))
        .map(|f| (f, false));

    unique_tuples.extend(unique_file_tuples_iter);

    unique_tuples
}

fn activate_unvalidated<T: InsertableLoadOrder + ?Sized>(
    load_order: &mut T,
    filename: &str,
) -> Result<(), Error> {
    let index = {
        let index = load_order.index_of(filename);
        if index.is_none() && Plugin::is_valid(&filename, load_order.game_settings()) {
            Some(load_order.add_to_load_order(filename)?)
        } else {
            index
        }
    };

    if let Some(x) = index {
        load_order.plugins_mut()[x].activate()?;
    }

    Ok(())
}

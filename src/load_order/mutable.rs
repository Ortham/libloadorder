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
use std::io::Read;
use std::mem;
use std::path::Path;

use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use rayon::prelude::*;

use super::find_first_non_master_position;
use super::readable::ReadableLoadOrderExt;
use enums::Error;
use plugin::Plugin;

pub trait MutableLoadOrder: ReadableLoadOrderExt {
    fn plugins_mut(&mut self) -> &mut Vec<Plugin>;

    fn deactivate_excess_plugins(&mut self) {
        for index in self.get_excess_active_plugin_indices() {
            self.plugins_mut()[index].deactivate();
        }
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

        let plugin = get_plugin_to_insert_at(self, plugin_name, position)?;

        if position >= self.plugins().len() {
            self.plugins_mut().push(plugin);
        } else {
            self.plugins_mut().insert(position, plugin);
        }

        Ok(())
    }

    fn deactivate_all(&mut self) {
        for plugin in self.plugins_mut() {
            plugin.deactivate();
        }
    }

    fn replace_plugins(&mut self, plugin_names: &[&str]) -> Result<(), Error> {
        if !are_plugin_names_unique(plugin_names) {
            return Err(Error::DuplicatePlugin);
        }

        let mut plugins = match self.map_to_plugins(plugin_names) {
            Err(x) => return Err(Error::InvalidPlugin(x.to_string())),
            Ok(x) => x,
        };

        if !is_partitioned_by_master_flag(&plugins) {
            return Err(Error::NonMasterBeforeMaster);
        }

        mem::swap(&mut plugins, self.plugins_mut());

        Ok(())
    }
}

pub fn load_active_plugins<T, F>(load_order: &mut T, line_mapper: F) -> Result<(), Error>
where
    T: MutableLoadOrder,
    F: Fn(&str) -> Option<String> + Send + Sync,
{
    load_order.deactivate_all();

    let plugin_names = read_plugin_names(
        load_order.game_settings().active_plugins_file(),
        line_mapper,
    )?;

    let plugin_indices: Vec<usize> = plugin_names
        .par_iter()
        .filter_map(|p| load_order.index_of(p))
        .collect();

    for index in plugin_indices {
        load_order.plugins_mut()[index].activate()?;
    }

    Ok(())
}

pub fn read_plugin_names<F, T>(file_path: &Path, line_mapper: F) -> Result<Vec<T>, Error>
where
    F: Fn(&str) -> Option<T> + Send + Sync,
    T: Send,
{
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let mut content: Vec<u8> = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut content)?;

    let content = WINDOWS_1252
        .decode(&content, DecoderTrap::Strict)
        .map_err(Error::DecodeError)?;

    Ok(content.lines().filter_map(line_mapper).collect())
}

pub fn plugin_line_mapper(line: &str) -> Option<String> {
    if line.is_empty() || line.starts_with('#') {
        None
    } else {
        Some(line.to_owned())
    }
}

fn get_plugin_to_insert_at<T: MutableLoadOrder + ?Sized>(
    load_order: &mut T,
    plugin_name: &str,
    insert_position: usize,
) -> Result<Plugin, Error> {
    if let Some(p) = load_order.index_of(plugin_name) {
        let is_master = load_order.plugins()[p].is_master_file();
        load_order.validate_index(insert_position, is_master)?;

        Ok(load_order.plugins_mut().remove(p))
    } else {
        let plugin = Plugin::new(plugin_name, load_order.game_settings())
            .map_err(|_| Error::InvalidPlugin(plugin_name.to_string()))?;

        load_order.validate_index(insert_position, plugin.is_master_file())?;

        Ok(plugin)
    }
}

fn are_plugin_names_unique(plugin_names: &[&str]) -> bool {
    let unique_plugin_names: HashSet<String> =
        plugin_names.par_iter().map(|s| s.to_lowercase()).collect();

    unique_plugin_names.len() == plugin_names.len()
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

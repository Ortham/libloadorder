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

mod error;
mod readable;
mod writable;
mod tests;
mod timestamp_based;

use unicase::eq;

use plugin::Plugin;

fn match_plugin(plugin: &Plugin, name: &str) -> bool {
    match plugin.name() {
        None => false,
        Some(n) => eq(n.as_str(), name),
    }
}

fn find_first_non_master_position(plugins: &Vec<Plugin>) -> Option<usize> {
    plugins.iter().position(|p| !p.is_master_file())
}

//TODO: Profile if the 'has changed' check is actually necessary.
fn reload_changed_plugins(plugins: &mut Vec<Plugin>) {
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

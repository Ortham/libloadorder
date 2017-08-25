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
mod mutable;
mod readable;
mod writable;
mod tests;
mod textfile_based;
mod timestamp_based;

use unicase::eq;

use plugin::Plugin;

fn match_plugin(plugin: &Plugin, name: &str) -> bool {
    match plugin.name() {
        None => false,
        Some(n) => eq(n.as_str(), name),
    }
}

fn find_first_non_master_position(plugins: &[Plugin]) -> Option<usize> {
    plugins.iter().position(|p| !p.is_master_file())
}

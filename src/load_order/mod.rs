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

mod asterisk_based;
mod error;
mod mutable;
mod readable;
mod writable;
mod tests;
mod textfile_based;
mod timestamp_based;

use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufRead, Error};
use std::path::Path;

use unicase::eq;

use plugin::Plugin;
use load_order::error::LoadOrderError;

fn match_plugin(plugin: &Plugin, name: &str) -> bool {
    match plugin.name() {
        None => false,
        Some(n) => eq(n.as_str(), name),
    }
}

fn find_first_non_master_position(plugins: &[Plugin]) -> Option<usize> {
    plugins.iter().position(|p| !p.is_master_file())
}

fn trim_cr(mut buffer: Vec<u8>) -> Vec<u8> {
    if buffer.last() == Some(&b'\r') {
        buffer.pop();
    }
    buffer
}

fn read_plugin_names<F>(file_path: &Path, line_mapper: F) -> Result<Vec<String>, LoadOrderError>
where
    F: Fn(Vec<u8>) -> Result<String, LoadOrderError>,
{
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let input = File::open(file_path)?;
    let buffered = BufReader::new(input);

    let mut names: Vec<String> = Vec::new();
    for line in buffered.split(b'\n') {
        let line = line_mapper(trim_cr(line?))?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        names.push(line);
    }

    Ok(names)
}

fn create_parent_dirs(path: &Path) -> Result<(), Error> {
    if let Some(x) = path.parent() {
        if !x.exists() {
            create_dir_all(x)?
        }
    }
    Ok(())
}

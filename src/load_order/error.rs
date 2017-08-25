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

use std::borrow::Cow;
use std::io;
use regex;
use error::Error;

#[derive(Debug)]
pub enum LoadOrderError {
    PluginError(Error),
    PluginNotFound,
    TooManyActivePlugins,
    InvalidPlugin(String),
    ImplicitlyActivePlugin(String),
    IoError(io::Error),
    DecodeError(Cow<'static, str>),
    InvalidRegex,
    DuplicatePlugin,
    NonMasterBeforeMaster,
    GameMasterMustLoadFirst,
}

impl From<Error> for LoadOrderError {
    fn from(error: Error) -> Self {
        LoadOrderError::PluginError(error)
    }
}

impl From<io::Error> for LoadOrderError {
    fn from(error: io::Error) -> Self {
        LoadOrderError::IoError(error)
    }
}

impl From<Cow<'static, str>> for LoadOrderError {
    fn from(error: Cow<'static, str>) -> Self {
        LoadOrderError::DecodeError(error)
    }
}

impl From<regex::Error> for LoadOrderError {
    fn from(error: regex::Error) -> Self {
        LoadOrderError::InvalidRegex
    }
}

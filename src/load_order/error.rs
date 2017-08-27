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

use std::borrow::Cow;
use std::io;
use std::string::FromUtf8Error;
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
    NotUtf8(Vec<u8>),
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
    fn from(_: regex::Error) -> Self {
        LoadOrderError::InvalidRegex
    }
}

impl From<FromUtf8Error> for LoadOrderError {
    fn from(error: FromUtf8Error) -> Self {
        LoadOrderError::NotUtf8(error.into_bytes())
    }
}

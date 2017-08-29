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

use std::error::Error;
use std::ptr;
use libc::{c_char, size_t, c_uint};

use constants::*;
use helpers::{error, handle_error, to_c_string_array, to_str, to_str_vec};
use super::lo_game_handle;

/// Gets the list of currently active plugins.
///
/// If no plugins are active, the value pointed to by `plugins` will be null and `num_plugins` will
/// point to zero.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_active_plugins(
    handle: lo_game_handle,
    plugins: *mut *mut *mut c_char,
    num_plugins: *mut size_t,
) -> c_uint {
    if handle.is_null() || plugins.is_null() || num_plugins.is_null() {
        return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
    }
    let mut handle = match (*handle).write() {
        Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
        Ok(h) => h,
    };

    *plugins = ptr::null_mut();
    *num_plugins = 0;

    if let Err(x) = handle.load() {
        handle.plugins_mut().clear();
        return handle_error(x);
    }

    let active_plugins = handle.active_plugin_names();

    if active_plugins.is_empty() {
        return LIBLO_OK;
    }

    match to_c_string_array(active_plugins) {
        Ok((pointer, size)) => {
            *plugins = pointer;
            *num_plugins = size;
        }
        Err(x) => return error(x, "A filename contained a null byte"),
    }

    LIBLO_OK
}

/// Sets the list of currently active plugins.
///
/// Replaces the current active plugins list with the plugins in the given array. The replacement
/// list must be valid. If, for Skyrim or Fallout 4, a plugin to be activated does not have a
/// defined load order position, this function will append it to the load order. If multiple such
/// plugins exist, they are appended in the order they are given.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_set_active_plugins(
    handle: lo_game_handle,
    plugins: *const *const c_char,
    num_plugins: size_t,
) -> c_uint {
    if handle.is_null() || plugins.is_null() {
        return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
    }
    let mut handle = match (*handle).write() {
        Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
        Ok(h) => h,
    };

    if let Err(x) = handle.load() {
        handle.plugins_mut().clear();
        return handle_error(x);
    }

    let plugins: Vec<&str> = match to_str_vec(plugins, num_plugins) {
        Ok(x) => x,
        Err(x) => return error(x, "A filename contained a null byte"),
    };

    if let Err(x) = handle.set_active_plugins(&plugins) {
        return handle_error(x);
    }

    if let Err(x) = handle.save() {
        handle.plugins_mut().clear();
        return handle_error(x);
    }

    LIBLO_OK
}

/// Activates or deactivates a given plugin.
///
/// If `active` is true, the plugin will be activated. If `active` is false, the plugin will be
/// deactivated.
///
/// When activating a plugin that is ghosted, the ".ghost" extension is removed. If a plugin is
/// already in its target state, ie. a plugin to be activated is already activate, or a plugin to
/// be deactivated is already inactive, no changes are made.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_set_plugin_active(
    handle: lo_game_handle,
    plugin: *const c_char,
    active: bool,
) -> c_uint {
    if handle.is_null() || plugin.is_null() {
        return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
    }
    let mut handle = match (*handle).write() {
        Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
        Ok(h) => h,
    };

    if let Err(x) = handle.load() {
        handle.plugins_mut().clear();
        return handle_error(x);
    }

    let plugin = match to_str(plugin) {
        Ok(x) => x,
        Err(x) => return error(x, "The filename contained a null byte"),
    };

    let result = if active {
        handle.activate(plugin)
    } else {
        handle.deactivate(plugin)
    };

    if let Err(x) = result {
        return handle_error(x);
    }

    if let Err(x) = handle.save() {
        handle.plugins_mut().clear();
        return handle_error(x);
    }

    LIBLO_OK
}

/// Checks if a given plugin is active.
///
/// Outputs `true` if the plugin is active, and false otherwise.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_plugin_active(
    handle: lo_game_handle,
    plugin: *const c_char,
    result: *mut bool,
) -> c_uint {
    if handle.is_null() || plugin.is_null() || result.is_null() {
        return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
    }
    let mut handle = match (*handle).write() {
        Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
        Ok(h) => h,
    };

    if let Err(x) = handle.load() {
        handle.plugins_mut().clear();
        return handle_error(x);
    }

    let plugin = match to_str(plugin) {
        Ok(x) => x,
        Err(x) => return error(x, "The filename contained a null byte"),
    };

    *result = handle.is_active(plugin);

    LIBLO_OK
}

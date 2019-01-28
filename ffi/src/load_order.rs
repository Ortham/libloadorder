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
use std::panic::catch_unwind;
use std::ptr;

use libc::{c_char, c_uint, size_t};
use loadorder::LoadOrderMethod;

use super::lo_game_handle;
use constants::*;
use helpers::{error, handle_error, to_c_string, to_c_string_array, to_str, to_str_vec};

/// Get which method is used for the load order.
///
/// The output is one of the `LIBLO_METHOD_*` constants.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_load_order_method(
    handle: lo_game_handle,
    method: *mut c_uint,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || method.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }
        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
            Ok(h) => h,
        };

        *method = match handle.game_settings().load_order_method() {
            LoadOrderMethod::Timestamp => LIBLO_METHOD_TIMESTAMP,
            LoadOrderMethod::Textfile => LIBLO_METHOD_TEXTFILE,
            LoadOrderMethod::Asterisk => LIBLO_METHOD_ASTERISK,
        };

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Get the current load order.
///
/// If no plugins are in the current order, the value pointed to by `plugins` will be null and
/// `num_plugins` will point to zero.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_load_order(
    handle: lo_game_handle,
    plugins: *mut *mut *mut c_char,
    num_plugins: *mut size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugins.is_null() || num_plugins.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }
        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
            Ok(h) => h,
        };

        *plugins = ptr::null_mut();
        *num_plugins = 0;

        let plugin_names = handle.plugin_names();

        if plugin_names.is_empty() {
            return LIBLO_OK;
        }

        match to_c_string_array(&plugin_names) {
            Ok((pointer, size)) => {
                *plugins = pointer;
                *num_plugins = size;
            }
            Err(x) => return error(x, "A filename contained a null byte"),
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Set the load order.
///
/// Sets the load order to the passed plugin array, then scans the plugins directory and inserts
/// any plugins not included in the passed array.
///
/// Plugin files are inserted at the end of the load order, and master files are inserted after the
/// last master file in the load order. The order of plugin insertion is undefined besides the
/// distinction made between master files and plugin files.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_set_load_order(
    handle: lo_game_handle,
    plugins: *const *const c_char,
    num_plugins: size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugins.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }
        if num_plugins == 0 {
            return error(LIBLO_ERROR_INVALID_ARGS, "Zero-length plugin array passed.");
        }
        let mut handle = match (*handle).write() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
            Ok(h) => h,
        };

        let plugins: Vec<&str> = match to_str_vec(plugins, num_plugins) {
            Ok(x) => x,
            Err(x) => return error(x, "A filename contained a null byte"),
        };

        if let Err(x) = handle.set_load_order(&plugins) {
            return handle_error(x);
        }

        if let Err(x) = handle.save() {
            return handle_error(x);
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Get the load order position of a plugin.
///
/// Load order positions are zero-based, so the first plugin in the load order has a position of
/// `0`, the next has a position of `1`, and so on.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_plugin_position(
    handle: lo_game_handle,
    plugin: *const c_char,
    index: *mut size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugin.is_null() || index.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }
        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
            Ok(h) => h,
        };

        let plugin = match to_str(plugin) {
            Ok(x) => x,
            Err(x) => return error(x, "The filename contained a null byte"),
        };

        match handle.index_of(plugin) {
            Some(x) => *index = x,
            None => return error(LIBLO_ERROR_FILE_NOT_FOUND, "Plugin not found"),
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Set the load order position of a plugin.
///
/// Sets the load order position of a plugin, removing it from its current position if it has one.
/// If the supplied position is greater than the last position in the load order, the plugin will
/// be positioned at the end of the load order. Load order positions are zero-based, so the first
/// plugin in the load order has a position of `0`, the next has a position of `1`, and so on.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_set_plugin_position(
    handle: lo_game_handle,
    plugin: *const c_char,
    index: size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugin.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }
        let mut handle = match (*handle).write() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
            Ok(h) => h,
        };

        let plugin = match to_str(plugin) {
            Ok(x) => x,
            Err(x) => return error(x, "The filename contained a null byte"),
        };

        if let Err(x) = handle.set_plugin_index(plugin, index) {
            return handle_error(x);
        }

        if let Err(x) = handle.save() {
            return handle_error(x);
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Get filename of the plugin at a specific load order position.
///
/// Load order positions are zero-based, so the first plugin in the load order has a position of
/// `0`, the next has a position of `1`, and so on.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_indexed_plugin(
    handle: lo_game_handle,
    index: size_t,
    plugin: *mut *mut c_char,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugin.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }
        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, e.description()),
            Ok(h) => h,
        };

        *plugin = ptr::null_mut();

        let plugin_name = match handle.plugin_at(index) {
            Some(x) => x,
            None => return error(LIBLO_ERROR_INVALID_ARGS, "Plugin is not in the load order"),
        };

        match to_c_string(plugin_name) {
            Ok(x) => *plugin = x,
            Err(x) => return error(x, "The filename contained a null byte"),
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

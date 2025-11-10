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
#![allow(
    unsafe_code,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]
#![cfg_attr(
    test,
    allow(
        clippy::multiple_unsafe_ops_per_block,
        clippy::undocumented_unsafe_blocks,
        clippy::unwrap_used,
    )
)]

//! # libloadorder-ffi
//!
//! libloadorder-ffi provides a C API wrapper around libloadorder, a free software library for
//! manipulating the load order and active status of plugins for the following games:
//!
//! - TES III: Morrowind
//! - TES IV: Oblivion
//! - TES V: Skyrim
//! - TES V: Skyrim Special Edition
//! - TES V: Skyrim VR
//! - Fallout 3
//! - Fallout: New Vegas
//! - Fallout 4
//! - Fallout 4 VR
//! - Starfield
//! - OpenMW
//!
//! ## Variable Types
//!
//! libloadorder-ffi uses character strings and integers for information input/output.
//!
//! - All strings are null-terminated byte character strings encoded in UTF-8.
//! - All return, game and load order method codes are unsigned integers at least 16 bits in size.
//! - All array sizes are unsigned integers at least 16 bits in size.
//! - File paths are case-sensitive if and only if the underlying file system is case-sensitive.
//!
//! ## Thread Safety
//!
//! libloadorder-ffi is thread-safe. Reading and writing data for a single game handle is protected
//! by a reader-writer lock, and error messages are stored thread-locally.
//!
//! Game handles operate independently, so using more than one game handle for a single game across
//! multiple threads is not advised, as filesystem changes made when writing data are not atomic
//! and data races may occur under such usage.
//!
//! ## Data Caching
//!
//! libloadorder caches plugin data to improve performance. Each game handle has its own unique
//! cache, which is cleared when
//!
//! - calling `lo_load_current_state()`
//! - calling `lo_fix_plugin_lists()`
//! - an error is encountered writing a change.
//!
//! ## Plugin Validity
//!
//! Where libloadorder functions take one or more plugin filenames, it checks that these filenames
//! correspond to valid plugins. libloadorder defines a valid plugin as one that:
//!
//! - Ends with `.esp`, `.esm`, `.esp.ghost` or `.esm.ghost` (or `.esl` or `.esl.ghost` for Skyrim
//!   Special Edition, Fallout 4 and Fallout 4 VR).
//! - Contains a header record with:
//!
//!     - The correct type (`TES3` for Morrowind, `TES4` otherwise).
//!     - A size that is not larger than the total file size.
//!     - Subrecords with sizes that do not together sum to larger than the expected total
//!       subrecords size.
//!
//! When the game is OpenMW:
//!
//! - `.omwgame`, `.omwaddon` and `.omwscripts` plugins are also supported. `.omwgame` and
//!   `.omwaddon` plugins must be valid according to the same rules as Morrowind `.esm` and `.esp`
//!   plugins. The contents of `.omwscripts` plugins are not used by libloadorder.
//! - `.esm.ghost` and `.esp.ghost` are not valid plugin filename extensions.
//!
//! This definition is substantially more permissive than games or other utilities may be for
//! performance reasons, and because libloadorder uses no plugin data beyond the header record, so
//! later corruption or invalid data would not affect its behaviour.
//!
//! This permissivity does allow more strictly invalid plugins to be positioned in the load order
//! and activated, which may cause game issues, but protecting against such circumstances is beyond
//! the scope of libloadorder.
//!
//! ## Valid Load Orders
//!
//! Any load order that is set using libloadorder must meet all the following conditions:
//!
//! - Contains only installed plugins.
//! - Contains no duplicate entries.
//! - Loads all master files (including light masters and false-flagged plugins) before all plugin
//!   files, with the exception that a non-master file that is a master of a master file must load
//!   between that master file and the previous master file. Note that OpenMW treats all plugins as
//!   non-master files.
//! - Contains no more than 255 active plugins, excluding light plugins.
//! - Contains no more than 4096 active light plugins.
//! - Contains all the game's implicitly active plugins that are installed (e.g. `Skyrim.esm` and
//!   `Update.esm` for Skyrim).
//! - Starts with the game's main master file (Skyrim, Skyrim SE, Fallout 4 and Fallout 4 VR only).
//!
//! Libloadorder considers a load order that fulfills all the above conditions to be valid, though
//! there may be additional conditions that may be enforced by the game.
//!
//! Libloadorder is less strict when loading load orders and will adjust them at load time to be
//! valid, similar to game behaviour. The exception to this adjustment is that the set of active
//! plugins is not reduced in size if too many plugins are active, so that it is preserved when
//! changing the order in which plugins are loaded. As such, while libloadorder will refuse to
//! activate too many plugins, it will preserve a load order that already has too many plugins
//! activated.

use std::cell::RefCell;
use std::ffi::{c_char, c_uint, CString};
use std::panic::catch_unwind;
use std::ptr;

use libc::size_t;

mod active_plugins;
mod constants;
mod handle;
mod helpers;
mod load_order;

pub use crate::active_plugins::*;
pub use crate::constants::*;
pub use crate::handle::*;
use crate::helpers::error;
pub use crate::load_order::*;

thread_local!(static ERROR_MESSAGE: RefCell<CString> = RefCell::new(CString::default()));

/// Gets the library version.
///
/// Outputs the major, minor and patch version numbers for the loaded libloadorder. The version
/// numbering used is major.minor.patch.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
///
/// # Safety
///
/// - `major`, `minor` and `patch` must be dereferenceable pointers.
#[no_mangle]
pub unsafe extern "C" fn lo_get_version(
    major: *mut c_uint,
    minor: *mut c_uint,
    patch: *mut c_uint,
) -> c_uint {
    catch_unwind(|| {
        if major.is_null() || minor.is_null() || patch.is_null() {
            error(LIBLO_ERROR_INVALID_ARGS, "Null pointer(s) passed")
        } else {
            match env!("CARGO_PKG_VERSION_MAJOR").parse::<c_uint>() {
                Ok(x) => *major = x,
                Err(_) => {
                    return error(
                        LIBLO_ERROR_INVALID_ARGS,
                        "Failed to parse major version number",
                    );
                }
            }
            match env!("CARGO_PKG_VERSION_MINOR").parse::<c_uint>() {
                Ok(x) => *minor = x,
                Err(_) => {
                    return error(
                        LIBLO_ERROR_INVALID_ARGS,
                        "Failed to parse minor version number",
                    );
                }
            }
            match env!("CARGO_PKG_VERSION_PATCH").parse::<c_uint>() {
                Ok(x) => *patch = x,
                Err(_) => {
                    return error(
                        LIBLO_ERROR_INVALID_ARGS,
                        "Failed to parse patch version number",
                    );
                }
            }

            LIBLO_OK
        }
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Get the message for the last error or warning encountered.
///
/// Outputs a string giving a message containing the details of the last error or warning
/// encountered by a function. The message uses thread-local storage, and only one message is
/// stored at any one time.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
///
/// # Safety
///
/// - `message` must be a dereferenceable pointer.
#[no_mangle]
pub unsafe extern "C" fn lo_get_error_message(message: *mut *const c_char) -> c_uint {
    catch_unwind(|| {
        if message.is_null() {
            error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed")
        } else {
            ERROR_MESSAGE.with(|f| {
                if f.borrow().as_bytes().is_empty() {
                    *message = ptr::null();
                } else {
                    *message = f.borrow().as_ptr();
                }
            });

            LIBLO_OK
        }
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Free memory allocated to string output.
///
/// This function should be called to free memory allocated by any API function that outputs a
/// string, excluding `lo_get_error_message()`.
///
/// # Safety
///
/// - `string` must be a C string that was previously allocated by this library, excluding the
///   string allocated by `lo_get_error_message()` and strings that are elements of a string array.
///   The length of the string must not have changed since it was allocated.
///
/// This function must not be called more than once with the same input value.
#[no_mangle]
pub unsafe extern "C" fn lo_free_string(string: *mut c_char) {
    if !string.is_null() {
        drop(CString::from_raw(string));
    }
}

/// Free memory allocated to string array output.
///
/// This function should be called to free memory allocated by any API function that outputs an
/// array of strings.
///
/// # Safety
///
/// - `paths` must be a non-null aligned pointer to a sequence of `num_paths` initialised C strings
///   within a single allocated object.
/// - `num_paths * std::mem::size_of::<*const c_char>()` must be no larger than `isize::MAX`.
/// - `paths` and `num_paths` must represent a single complete array of C strings that was allocated
///   by this library.
///
/// This function must not be called more than once with the same `array` value.
#[no_mangle]
pub unsafe extern "C" fn lo_free_string_array(array: *mut *mut c_char, size: size_t) {
    if array.is_null() || size == 0 {
        return;
    }

    let strings = Box::from_raw(std::ptr::slice_from_raw_parts_mut(array, size));
    for string in &strings {
        lo_free_string(*string);
    }

    drop(strings);
}

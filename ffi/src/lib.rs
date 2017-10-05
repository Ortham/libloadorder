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

//! # libloadorder-ffi
//!
//! libloadorder-ffi provides a C API wrapper around libloadorder, a free software library for
//! manipulating the load order and active status of plugins for the following games:
//!
//! * TES III: Morrowind
//! * TES IV: Oblivion
//! * TES V: Skyrim
//! * TES V: Skyrim Special Edition
//! * Fallout 3
//! * Fallout: New Vegas
//! * Fallout 4
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
//! by mutual exclusion, and error messages are stored thread-locally.
//!
//! Game handles operate independently, so using more than one game handle for a single game across
//! multiple threads is not advised, as filesystem changes made when writing data are not atomic
//! and data races may occur under such usage.
//!
//! ## Data Caching
//!
//! libloadorder caches plugin data to improve performance. Each game handle has its own unique
//! cache, and change detection is performed whenever an API function that takes a game handle is
//! called. If changes are detected, the necessary data are reloaded before the function operates
//! on the data.
//!
//! Edits made to a plugin will only be detected if they that plugin's timestamp changes. If edits
//! are made and the timestamp is unchanged, the changes can only be detected by destroying the
//! existing game handle and creating a new game handle to use.
//!
//! ## Plugin Validity
//!
//! Where libloadorder functions take one or more plugin filenames, it checks that these filenames
//! correspond to valid plugins. libloadorder defines a valid plugin as one that:
//!
//! - Ends with `.esp`, `.esm`, `.esp.ghost` or `.esm.ghost`.
//! - Contains a header record with:
//!
//!     - The correct type (`TES3` for Morrowind, `TES4` otherwise).
//!     - A size that is not larger than the total file size.
//!     - Subrecords with sizes that do not together sum to larger than the expected total
//!       subrecords size.
//!
//! This definition is substantially more permissive than games or other utilities may be for
//! performance reasons, and because libloadorder uses no plugin data beyond the header record, so
//! later corruption or invalid data would not affect its behaviour.
//!
//! This permissivity does allow more strictly invalid plugins to be positioned in the load order
//! and activated, which may cause game issues, but protecting against such circumstances is beyond
//! the scope of libloadorder.
//!
//! ## Valid Active Plugin Lists
//!
//! Any active plugin list that is set using libloadorder must be valid,
//! ie. it must meet all the following conditions:
//!
//! - Contains only installed plugins.
//! - Contains no duplicate entries.
//! - Contains no more than 255 plugins.
//! - If a Skyrim or Fallout 4 load order, contains `Skyrim.esm` or `Fallout4.esm` respectively.
//! - If a Skyrim load order and `Update.esm` is installed, contains `Update.esm`.
//!
//! Libloadorder is less strict when loading active plugin lists. If loading a Skyrim or Fallout 4
//! list and the relevant main master file is missing, it will be inferred to load first.
//!
//! Similarly, if Update.esm is installed but not in the active list, it will be inferred to load
//! after all other master files.
//!
//! ## Valid Load Orders
//!
//! Any load order that is set using libloadorder must be valid, ie. it must meet all the following
//! conditions:
//!
//! - Contains only installed plugins.
//! - Contains no duplicate entries.
//! - Loads all master files before all plugin files. Master bit flag value, rather than file
//!   extension, is checked.
//! - For Skyrim or Fallout 4, the first plugin in the load order must be `Skyrim.esm` or
//!   `Fallout4.esm` respectively.

extern crate loadorder;
extern crate libc;

use std::cell::RefCell;
use std::ffi::CString;
use std::io;
use std::ptr;
use libc::{c_char, c_uint, size_t};
use loadorder::Error;

mod active_plugins;
mod constants;
mod handle;
mod helpers;
mod load_order;

pub use active_plugins::*;
pub use constants::*;
pub use handle::*;
use helpers::error;
pub use load_order::*;

thread_local!(static ERROR_MESSAGE: RefCell<CString> = RefCell::new(CString::default()));

/// Gets the library version.
///
/// Outputs the major, minor and patch version numbers for the loaded libloadorder. The version
/// numbering used is major.minor.patch.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_version(
    major: *mut c_uint,
    minor: *mut c_uint,
    patch: *mut c_uint,
) -> c_uint {
    if major.is_null() || minor.is_null() || patch.is_null() {
        error(LIBLO_ERROR_INVALID_ARGS, "Null pointer(s) passed")
    } else {
        match env!("CARGO_PKG_VERSION_MAJOR").parse::<c_uint>() {
            Ok(x) => *major = x,
            Err(_) => {
                return error(
                    LIBLO_ERROR_INVALID_ARGS,
                    "Failed to parse major version number",
                )
            }
        }
        match env!("CARGO_PKG_VERSION_MINOR").parse::<c_uint>() {
            Ok(x) => *minor = x,
            Err(_) => {
                return error(
                    LIBLO_ERROR_INVALID_ARGS,
                    "Failed to parse minor version number",
                )
            }
        }
        match env!("CARGO_PKG_VERSION_PATCH").parse::<c_uint>() {
            Ok(x) => *patch = x,
            Err(_) => {
                return error(
                    LIBLO_ERROR_INVALID_ARGS,
                    "Failed to parse patch version number",
                )
            }
        }

        LIBLO_OK
    }
}

/// Get the message for the last error or warning encountered.
///
/// Outputs a string giving a message containing the details of the last error or warning
/// encountered by a function. The message uses thread-local storage, and only one message is
/// stored at any one time.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_error_message(message: *mut *const c_char) -> c_uint {
    if message.is_null() {
        error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed")
    } else {
        ERROR_MESSAGE.with(|f| if f.borrow().as_bytes().is_empty() {
            *message = ptr::null();
        } else {
            *message = f.borrow().as_ptr() as *const i8;
        });

        LIBLO_OK
    }
}

/// Free memory allocated to string output.
///
/// This function should be called to free memory allocated by any API function that outputs a
/// string, excluding `lo_get_error_message()`.
#[no_mangle]
pub unsafe extern "C" fn lo_free_string(string: *mut c_char) {
    if !string.is_null() {
        CString::from_raw(string);
    }
}

/// Free memory allocated to string array output.
///
/// This function should be called to free memory allocated by any API function that outputs an
/// array of strings.
#[no_mangle]
pub unsafe extern "C" fn lo_free_string_array(array: *mut *mut c_char, size: size_t) {
    if array.is_null() || size == 0 {
        return;
    }

    let vec = Vec::from_raw_parts(array, size, size);
    for string in vec {
        lo_free_string(string);
    }
}

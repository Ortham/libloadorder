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

use libc::c_uint;

/// Success return code.
#[no_mangle]
pub static LIBLO_OK: c_uint = 0;

/// There is a mismatch between the files used to keep track of load order.
///
/// This warning can only occur when using libloadorder with a game that uses the textfile-based
/// load order system. The load order in the active plugins list file (`plugins.txt`) does not
/// match the load order in the full load order file (`loadorder.txt`). Synchronisation between
/// the two is automatic when load order is managed through libloadorder. It is left to the client
/// to decide how best to restore synchronisation.
#[no_mangle]
pub static LIBLO_WARN_LO_MISMATCH: c_uint = 2;

/// The specified file could not be found.
#[no_mangle]
pub static LIBLO_ERROR_FILE_NOT_FOUND: c_uint = 6;

/// A file could not be renamed.
#[no_mangle]
pub static LIBLO_ERROR_FILE_RENAME_FAIL: c_uint = 7;

/// There was an error parsing a plugin file.
#[no_mangle]
pub static LIBLO_ERROR_FILE_PARSE_FAIL: c_uint = 10;

/// Invalid arguments were given for the function.
#[no_mangle]
pub static LIBLO_ERROR_INVALID_ARGS: c_uint = 12;

/// A thread lock was poisoned.
#[no_mangle]
pub static LIBLO_ERROR_POISONED_THREAD_LOCK: c_uint = 14;

/// An unknown I/O error occurred. This is used when the I/O error kind doesn't fit another error
/// code.
#[no_mangle]
pub static LIBLO_ERROR_IO_ERROR: c_uint = 15;

/// Permission denied while trying to access a filesystem path.
#[no_mangle]
pub static LIBLO_ERROR_IO_PERMISSION_DENIED: c_uint = 16;

/// A plugin filename contains characters that do not have Windows-1252 code points, or a character
/// string contains a null character.
#[no_mangle]
pub static LIBLO_ERROR_TEXT_ENCODE_FAIL: c_uint = 17;

/// Text expected to be encoded in Windows-1252 could not be decoded to UTF-8.
#[no_mangle]
pub static LIBLO_ERROR_TEXT_DECODE_FAIL: c_uint = 18;

/// The library encountered an error that should not have been possible to encounter.
#[no_mangle]
pub static LIBLO_ERROR_INTERNAL_LOGIC_ERROR: c_uint = 19;

/// Something panicked.
#[no_mangle]
pub static LIBLO_ERROR_PANICKED: c_uint = 20;

/// A path cannot be encoded in UTF-8.
#[no_mangle]
pub static LIBLO_ERROR_PATH_ENCODE_FAIL: c_uint = 21;

/// An unknown operating system error occurred.
#[no_mangle]
pub static LIBLO_ERROR_SYSTEM_ERROR: c_uint = 22;

/// Matches the value of the highest-numbered return code.
///
/// Provided in case clients wish to incorporate additional return codes in their implementation
/// and desire some method of avoiding value conflicts.
#[no_mangle]
pub static LIBLO_RETURN_MAX: c_uint = 22;

/// The game handle is using the timestamp-based load order system. Morrowind, Oblivion, Fallout 3
/// and Fallout: New Vegas all use this system.
#[no_mangle]
pub static LIBLO_METHOD_TIMESTAMP: c_uint = 0;

/// he game handle is using the textfile-based load order system. Skyrim uses this system.
#[no_mangle]
pub static LIBLO_METHOD_TEXTFILE: c_uint = 1;

/// The asterisk load order system, used by Fallout 4 and Skyrim Special Edition.
#[no_mangle]
pub static LIBLO_METHOD_ASTERISK: c_uint = 2;

/// Game code for The Elder Scrolls III: Morrowind.
#[no_mangle]
pub static LIBLO_GAME_TES3: c_uint = 1;

/// Game code for The Elder Scrolls IV: Oblivion.
#[no_mangle]
pub static LIBLO_GAME_TES4: c_uint = 2;

/// Game code for The Elder Scrolls V: Skyrim.
#[no_mangle]
pub static LIBLO_GAME_TES5: c_uint = 3;

/// Game code for Fallout 3.
#[no_mangle]
pub static LIBLO_GAME_FO3: c_uint = 4;

/// Game code for Fallout: New Vegas.
#[no_mangle]
pub static LIBLO_GAME_FNV: c_uint = 5;

/// Game code for Fallout 4.
#[no_mangle]
pub static LIBLO_GAME_FO4: c_uint = 6;

/// Game code for The Elder Scrolls V: Skyrim Special Edition.
#[no_mangle]
pub static LIBLO_GAME_TES5SE: c_uint = 7;

/// Game code for The Elder Scrolls V: Skyrim VR.
#[no_mangle]
pub static LIBLO_GAME_TES5VR: c_uint = 9;

/// Game code for Fallout 4 VR.
#[no_mangle]
pub static LIBLO_GAME_FO4VR: c_uint = 8;

/// Game code for Starfield.
#[no_mangle]
pub static LIBLO_GAME_STARFIELD: c_uint = 10;

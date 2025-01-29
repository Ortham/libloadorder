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

use std::panic::catch_unwind;
use std::path::Path;
use std::ptr;
use std::sync::RwLock;

use libc::{c_char, c_uint, size_t};
use loadorder::GameId;
use loadorder::GameSettings;
use loadorder::WritableLoadOrder;

use crate::constants::*;
use crate::helpers::{
    error, handle_error, to_c_string, to_c_string_array, to_path_buf_vec, to_str,
};

/// A structure that holds all game-specific data used by libloadorder.
///
/// Used to keep each game's data independent. Abstracts the definition of libloadorder's internal
/// state while still providing type safety across the library.
#[allow(non_camel_case_types)]
pub type lo_game_handle = *mut GameHandle;

// This type alias is necessary to make cbindgen treat lo_game_handle as a
// pointer to an undefined type, rather than an undefined type itself.
type GameHandle = RwLock<Box<dyn WritableLoadOrder>>;

fn map_game_id(game_id: u32) -> Result<GameId, u32> {
    match game_id {
        x if x == LIBLO_GAME_TES3 => Ok(GameId::Morrowind),
        x if x == LIBLO_GAME_TES4 => Ok(GameId::Oblivion),
        x if x == LIBLO_GAME_TES5 => Ok(GameId::Skyrim),
        x if x == LIBLO_GAME_TES5SE => Ok(GameId::SkyrimSE),
        x if x == LIBLO_GAME_TES5VR => Ok(GameId::SkyrimVR),
        x if x == LIBLO_GAME_FO3 => Ok(GameId::Fallout3),
        x if x == LIBLO_GAME_FNV => Ok(GameId::FalloutNV),
        x if x == LIBLO_GAME_FO4 => Ok(GameId::Fallout4),
        x if x == LIBLO_GAME_FO4VR => Ok(GameId::Fallout4VR),
        x if x == LIBLO_GAME_STARFIELD => Ok(GameId::Starfield),
        _ => Err(LIBLO_ERROR_INVALID_ARGS),
    }
}

/// Initialise a new game handle.
///
/// Creates a handle for a game, which is then used by all load order and active plugin functions.
/// If the game uses the textfile-based load order system, this function also checks if the two
/// load order files are in sync, provided they both exist.
///
/// The game ID is one of the `LIBLO_GAME_*` constants.
///
/// The game path is the directory where the game's executable is found.
///
/// The local path is the game's local application data folder, found within `%LOCALAPPDATA%` on
/// Windows. If running on Windows, the `local_path` can be null, in which case libloadorder will
/// look the path up itself. However, on other operating systems, lookup is not possible, and this
/// function must be used to provide the necessary path.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_create_handle(
    handle: *mut lo_game_handle,
    game_id: c_uint,
    game_path: *const c_char,
    local_path: *const c_char,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || game_path.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer(s) passed");
        }

        let game_id = match map_game_id(game_id) {
            Ok(x) => x,
            Err(x) => return error(x, "Invalid game specified"),
        };

        let game_path = match to_str(game_path) {
            Ok(x) => Path::new(x),
            Err(x) => return x,
        };

        if !game_path.is_dir() {
            return error(
                LIBLO_ERROR_INVALID_ARGS,
                &format!(
                    "Given game path \"{:?}\" is not a valid directory",
                    game_path
                ),
            );
        }

        let load_order: Box<dyn WritableLoadOrder>;
        if local_path.is_null() {
            match GameSettings::new(game_id, game_path) {
                Ok(x) => load_order = x.into_load_order(),
                Err(x) => return handle_error(x),
            }
        } else {
            let local_path = match to_str(local_path) {
                Ok(x) => Path::new(x),
                Err(x) => return x,
            };

            if local_path.exists() && !local_path.is_dir() {
                return error(
                    LIBLO_ERROR_INVALID_ARGS,
                    &format!(
                        "Given local data path \"{:?}\" exists but is not a valid directory",
                        local_path
                    ),
                );
            }

            match GameSettings::with_local_path(game_id, game_path, local_path) {
                Ok(x) => load_order = x.into_load_order(),
                Err(x) => return handle_error(x),
            }
        }

        let is_self_consistent = load_order.is_self_consistent();

        *handle = Box::into_raw(Box::new(RwLock::new(load_order)));

        match is_self_consistent {
            Ok(true) => LIBLO_OK,
            Ok(false) => LIBLO_WARN_LO_MISMATCH,
            Err(x) => handle_error(x),
        }
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Destroy an existing game handle.
///
/// Destroys the given game handle, freeing up memory allocated during its use, excluding any
/// memory allocated to error messages.
#[no_mangle]
pub unsafe extern "C" fn lo_destroy_handle(handle: lo_game_handle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Load the current load order state, discarding any previously held state.
///
/// This function should be called whenever the load order or active state of plugins "on disk"
/// changes, so that cached state is updated to reflect the changes.
#[no_mangle]
pub unsafe extern "C" fn lo_load_current_state(handle: lo_game_handle) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let mut handle = match (*handle).write() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        if let Err(x) = handle
            .game_settings_mut()
            .refresh_implicitly_active_plugins()
        {
            return handle_error(x);
        }

        if let Err(x) = handle.load() {
            return handle_error(x);
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Check if the load order is ambiguous, by checking that all plugins in the current load order
/// state have a well-defined position in the "on disk" state, and that all data sources are
/// consistent. If the load order is ambiguous, different applications may read different load
/// orders from the same source data.
///
/// Outputs `true` if the load order state is ambiguous, and false otherwise.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_is_ambiguous(handle: lo_game_handle, result: *mut bool) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || result.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        match handle.is_ambiguous() {
            Ok(x) => *result = x,
            Err(x) => return handle_error(x),
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Fix up the text file(s) used by the load order and active plugins systems.
///
/// This checks that the load order and active plugin lists conform to libloadorder's validity
/// requirements (see Miscellaneous Details for details), and resolves any issues encountered, then
/// saves the fixed lists.
///
/// For the case of a plugin appearing multiple times in a load order / active plugin list,
/// libloadorder discards all but the last instance of that plugin.
///
/// This can be useful for when plugins are uninstalled manually or by a utility that does not also
/// update the load order / active plugins systems correctly.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_fix_plugin_lists(handle: lo_game_handle) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let mut handle = match (*handle).write() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        if let Err(x) = handle.load() {
            return handle_error(x);
        }

        if let Err(x) = handle.save() {
            return handle_error(x);
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Get the list of implicitly active plugins for the given handle's game.
///
/// The list may be empty if the game has no implicitly active plugins. The list may include plugins
/// that are not installed. Plugins are not necessarily listed in their load order.
///
/// If the list is empty, the `plugins` pointer will be null and `num_plugins` will be `0`.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_implicitly_active_plugins(
    handle: lo_game_handle,
    plugins: *mut *mut *mut c_char,
    num_plugins: *mut size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugins.is_null() || num_plugins.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        *plugins = ptr::null_mut();
        *num_plugins = 0;

        let plugin_names = handle.game_settings().implicitly_active_plugins();

        if plugin_names.is_empty() {
            return LIBLO_OK;
        }

        match to_c_string_array(plugin_names) {
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

/// Get the list of implicitly active plugins for the given handle's game.
///
/// The list may be empty if the game has no early loading plugins. The list may include plugins
/// that are not installed. Plugins are listed in their hardcoded load order.
///
/// Note that for the original Skyrim, `Update.esm` is hardcoded to always load, but not in a
/// specific location, unlike all other early loading plugins for all games, which must load in the
/// given order, before any other plugins.
///
/// The order of Creation Club plugins as listed in `Fallout4.ccc` or `Skyrim.ccc` is as their
/// hardcoded load order for libloadorder's purposes.
///
/// If the list is empty, the `plugins` pointer will be null and `num_plugins` will be `0`.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_early_loading_plugins(
    handle: lo_game_handle,
    plugins: *mut *mut *mut c_char,
    num_plugins: *mut size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || plugins.is_null() || num_plugins.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        *plugins = ptr::null_mut();
        *num_plugins = 0;

        let plugin_names = handle.game_settings().early_loading_plugins();

        if plugin_names.is_empty() {
            return LIBLO_OK;
        }

        match to_c_string_array(plugin_names) {
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

/// Get the active plugins file path for the given game handle.
///
/// The active plugins file path is often within the game's local path, but its name and location
/// varies by game and game configuration, so this function exposes the path that libloadorder
/// chooses to use.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is
/// returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_active_plugins_file_path(
    handle: lo_game_handle,
    path: *mut *mut c_char,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || path.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        let file_path = match handle.game_settings().active_plugins_file().to_str() {
            Some(p) => p,
            None => {
                return error(
                    LIBLO_ERROR_PATH_ENCODE_FAIL,
                    "The active plugins file path could not be encoded in UTF-8",
                )
            }
        };

        match to_c_string(file_path) {
            Ok(x) => *path = x,
            Err(x) => return error(x, "The filename contained a null byte"),
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Gets the additional plugins directories that are used when looking up plugin filenames.
///
/// Some games (Fallout 4, Starfield and OpenMW) support loading plugins from outside of the game's
/// main plugins directory, and each game handle is initialised with those additional directories
/// set.
///
/// If the list is empty, the `paths` pointer will be null and `num_paths` will be `0`.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_get_additional_plugins_directories(
    handle: lo_game_handle,
    paths: *mut *mut *mut c_char,
    num_paths: *mut size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || paths.is_null() || num_paths.is_null() {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let handle = match (*handle).read() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        *paths = ptr::null_mut();
        *num_paths = 0;

        let path_strings: Vec<_> = handle
            .game_settings()
            .additional_plugins_directories()
            .iter()
            .filter_map(|p| p.to_str())
            .collect();

        if path_strings.is_empty() {
            return LIBLO_OK;
        }

        match to_c_string_array(&path_strings) {
            Ok((pointer, size)) => {
                *paths = pointer;
                *num_paths = size;
            }
            Err(x) => return error(x, "A path contained a null byte"),
        }

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

/// Sets the additional plugins directories to be recognised by the given handle.
///
/// If the load order contains plugins that are installed outside of the game's plugins directory,
/// this function can be used to provide the paths to the directories that those plugins are in so
/// that libloadorder is able to find them.
///
/// If external plugins exist, this function must be called before performing any operations on
/// the load order to avoid any unexpected behaviour.
///
/// Returns `LIBLO_OK` if successful, otherwise a `LIBLO_ERROR_*` code is returned.
#[no_mangle]
pub unsafe extern "C" fn lo_set_additional_plugins_directories(
    handle: lo_game_handle,
    paths: *const *const c_char,
    num_paths: size_t,
) -> c_uint {
    catch_unwind(|| {
        if handle.is_null() || (paths.is_null() && num_paths != 0) {
            return error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed");
        }

        let mut handle = match (*handle).write() {
            Err(e) => return error(LIBLO_ERROR_POISONED_THREAD_LOCK, &e.to_string()),
            Ok(h) => h,
        };

        let plugin_paths = match to_path_buf_vec(paths, num_paths) {
            Ok(x) => x,
            Err(x) => return error(x, "A plugin path was null or not valid UTF-8"),
        };

        handle
            .game_settings_mut()
            .set_additional_plugins_directories(plugin_paths);

        LIBLO_OK
    })
    .unwrap_or(LIBLO_ERROR_PANICKED)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn lo_create_handle_should_allow_a_non_existent_local_path() {
        let mut handle: lo_game_handle = std::ptr::null_mut();
        let game_path = CString::new(".").unwrap();
        let local_path = CString::new("does-not-exist").unwrap();

        assert!(!PathBuf::from(local_path.to_string_lossy().to_string()).exists());

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES5,
                game_path.as_ptr(),
                local_path.as_ptr(),
            );
            lo_destroy_handle(handle);

            assert_eq!(LIBLO_OK, result);
        }
    }

    #[test]
    fn lo_create_handle_should_allow_a_directory_local_path_that_exists() {
        let mut handle: lo_game_handle = std::ptr::null_mut();
        let game_path = CString::new(".").unwrap();
        let local_path = CString::new(".").unwrap();

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES5,
                game_path.as_ptr(),
                local_path.as_ptr(),
            );
            lo_destroy_handle(handle);

            assert_eq!(LIBLO_OK, result);
        }
    }

    #[test]
    fn lo_create_handle_should_error_if_given_a_non_directory_local_path_that_exists() {
        let mut handle: lo_game_handle = std::ptr::null_mut();
        let game_path = CString::new(".").unwrap();
        let local_path = CString::new(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .as_bytes(),
        )
        .unwrap();

        let local_path_buf = PathBuf::from(local_path.to_string_lossy().to_string());

        assert!(local_path_buf.exists());
        assert!(!local_path_buf.is_dir());

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES5,
                game_path.as_ptr(),
                local_path.as_ptr(),
            );
            lo_destroy_handle(handle);

            assert_eq!(LIBLO_ERROR_INVALID_ARGS, result);
        }
    }

    #[test]
    fn lo_create_handle_should_always_accept_a_null_local_path_if_game_is_morrowind() {
        let mut handle: lo_game_handle = std::ptr::null_mut();
        let game_path = CString::new(".").unwrap();

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES3,
                game_path.as_ptr(),
                std::ptr::null(),
            );
            lo_destroy_handle(handle);

            assert_eq!(LIBLO_OK, result);
        }
    }

    #[test]
    #[cfg(windows)]
    fn lo_create_handle_should_accept_a_null_local_path_if_game_is_not_morrowind() {
        let mut handle: lo_game_handle = std::ptr::null_mut();
        let game_path = CString::new(".").unwrap();

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES4,
                game_path.as_ptr(),
                std::ptr::null(),
            );
            lo_destroy_handle(handle);

            assert_eq!(LIBLO_OK, result);
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn lo_create_handle_should_not_accept_a_null_local_path_if_game_is_not_morrowind() {
        let mut handle: lo_game_handle = std::ptr::null_mut();
        let game_path = CString::new(".").unwrap();

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES4,
                game_path.as_ptr(),
                std::ptr::null(),
            );
            lo_destroy_handle(handle);

            assert_eq!(LIBLO_ERROR_INVALID_ARGS, result);
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn lo_load_current_state_should_refresh_implicitly_active_plugins() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let game_path = tmp_dir.path();
        let local_path = game_path.join("AppData/Local/Oblivion");
        let ini_path = game_path.join("Documents/My Games/Oblivion/Oblivion.ini");

        let mut handle: lo_game_handle = std::ptr::null_mut();

        let game_path = CString::new(game_path.to_str().unwrap()).unwrap();
        let local_path = CString::new(local_path.to_str().unwrap()).unwrap();

        unsafe {
            let result = lo_create_handle(
                &mut handle,
                LIBLO_GAME_TES4,
                game_path.as_ptr(),
                local_path.as_ptr(),
            );
            assert_eq!(LIBLO_OK, result);
        }

        unsafe {
            let mut plugins: *mut *mut c_char = std::ptr::null_mut();
            let mut num_plugins: size_t = 0;

            let result = lo_get_implicitly_active_plugins(handle, &mut plugins, &mut num_plugins);
            assert_eq!(LIBLO_OK, result);
            assert_eq!(0, num_plugins);

            crate::lo_free_string_array(plugins, num_plugins);
        }

        std::fs::create_dir_all(&ini_path.parent().unwrap()).unwrap();
        std::fs::write(&ini_path, "[General]\nsTestFile1=Blank.esp").unwrap();

        unsafe {
            let result = lo_load_current_state(handle);
            assert_eq!(LIBLO_OK, result);
        }

        unsafe {
            let mut plugins: *mut *mut c_char = std::ptr::null_mut();
            let mut num_plugins: size_t = 0;

            let result = lo_get_implicitly_active_plugins(handle, &mut plugins, &mut num_plugins);
            assert_eq!(LIBLO_OK, result);
            assert_eq!(1, num_plugins);

            crate::lo_free_string_array(plugins, num_plugins);
        }

        unsafe {
            lo_destroy_handle(handle);
        }
    }
}

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
use std::ffi::{c_char, c_uint, CStr, CString};
use std::io;
use std::path::PathBuf;
use std::slice;

use loadorder::Error;

use super::ERROR_MESSAGE;
use crate::constants::{
    LIBLO_ERROR_FILE_NOT_FOUND, LIBLO_ERROR_FILE_PARSE_FAIL, LIBLO_ERROR_FILE_RENAME_FAIL,
    LIBLO_ERROR_INTERNAL_LOGIC_ERROR, LIBLO_ERROR_INVALID_ARGS, LIBLO_ERROR_IO_ERROR,
    LIBLO_ERROR_IO_PERMISSION_DENIED, LIBLO_ERROR_NO_PATH, LIBLO_ERROR_SYSTEM_ERROR,
    LIBLO_ERROR_TEXT_DECODE_FAIL, LIBLO_ERROR_TEXT_ENCODE_FAIL,
};

pub(crate) fn error(code: c_uint, message: &str) -> c_uint {
    ERROR_MESSAGE.with(|f| {
        *f.borrow_mut() = CString::new(message.as_bytes())
            .or_else(|_e| CString::new(message.replace('\0', "\\0").as_bytes()))
            .unwrap_or_else(|_e| c"Failed to retrieve error message".into());
    });
    code
}

pub(crate) fn handle_error(err: &Error) -> c_uint {
    let code = map_error(err);
    error(code, &format!("{err}"))
}

fn map_io_error(err: &io::Error) -> c_uint {
    use io::ErrorKind::{AlreadyExists, NotFound, PermissionDenied};
    match err.kind() {
        NotFound => LIBLO_ERROR_FILE_NOT_FOUND,
        AlreadyExists => LIBLO_ERROR_FILE_RENAME_FAIL,
        PermissionDenied => LIBLO_ERROR_IO_PERMISSION_DENIED,
        _ => LIBLO_ERROR_IO_ERROR,
    }
}

fn map_error(err: &Error) -> c_uint {
    match err {
        Error::InvalidPath(_) => LIBLO_ERROR_FILE_NOT_FOUND,
        Error::IoError(_, x) => map_io_error(x),
        Error::NoFilename(_)
        | Error::PluginParsingError(_, _)
        | Error::IniParsingError { .. }
        | Error::VdfParsingError(_, _) => LIBLO_ERROR_FILE_PARSE_FAIL,
        Error::DecodeError(_) => LIBLO_ERROR_TEXT_DECODE_FAIL,
        Error::EncodeError(_) => LIBLO_ERROR_TEXT_ENCODE_FAIL,
        Error::PluginNotFound(_)
        | Error::TooManyActivePlugins { .. }
        | Error::DuplicatePlugin(_)
        | Error::NonMasterBeforeMaster { .. }
        | Error::InvalidEarlyLoadingPluginPosition { .. }
        | Error::ImplicitlyActivePlugin(_)
        | Error::NoLocalAppData
        | Error::NoDocumentsPath
        | Error::UnrepresentedHoist { .. }
        | Error::InstalledPlugin(_)
        | Error::InvalidBlueprintPluginPosition { .. } => LIBLO_ERROR_INVALID_ARGS,
        Error::NoUserConfigPath | Error::NoUserDataPath | Error::NoProgramFilesPath => {
            LIBLO_ERROR_NO_PATH
        }
        Error::SystemError(_, _) => LIBLO_ERROR_SYSTEM_ERROR,
        _ => LIBLO_ERROR_INTERNAL_LOGIC_ERROR,
    }
}

pub(crate) unsafe fn to_str<'a>(c_string: *const c_char) -> Result<&'a str, u32> {
    if c_string.is_null() {
        Err(error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed"))
    } else {
        CStr::from_ptr(c_string)
            .to_str()
            .map_err(|_e| error(LIBLO_ERROR_INVALID_ARGS, "Non-UTF-8 string passed"))
    }
}

pub(crate) fn to_c_string<S: AsRef<str>>(string: S) -> Result<*mut c_char, u32> {
    CString::new(string.as_ref())
        .map(CString::into_raw)
        .map_err(|_e| LIBLO_ERROR_TEXT_ENCODE_FAIL)
}

pub(crate) fn to_c_string_array<S: AsRef<str>>(
    strings: &[S],
) -> Result<(*mut *mut c_char, usize), u32> {
    let c_strings = strings
        .iter()
        .map(to_c_string)
        .collect::<Result<Box<[*mut c_char]>, u32>>()?;

    let size = c_strings.len();

    // Although this is a pointer for the box and we want a pointer to the
    // start of the slice in the box, they're actually the same value, and we
    // can recover the box as well as the slice from the pointer and size.
    let pointer = Box::into_raw(c_strings);

    Ok((pointer.cast(), size))
}

pub(crate) unsafe fn to_str_vec<'a>(
    array: *const *const c_char,
    array_size: usize,
) -> Result<Vec<&'a str>, u32> {
    slice::from_raw_parts(array, array_size)
        .iter()
        .map(|c| to_str(*c))
        .collect()
}

pub(crate) unsafe fn to_path_buf_vec(
    array: *const *const c_char,
    array_size: usize,
) -> Result<Vec<PathBuf>, u32> {
    if array_size == 0 {
        Ok(Vec::new())
    } else {
        slice::from_raw_parts(array, array_size)
            .iter()
            .map(|c| to_str(*c).map(PathBuf::from))
            .collect()
    }
}

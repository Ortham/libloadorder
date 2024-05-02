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
use std::ffi::{CStr, CString};
use std::io;
use std::mem;
use std::path::PathBuf;
use std::slice;

use libc::{c_char, c_uint, size_t};
use loadorder::Error;

use super::ERROR_MESSAGE;
use crate::constants::*;

pub fn error(code: c_uint, message: &str) -> c_uint {
    ERROR_MESSAGE.with(|f| {
        *f.borrow_mut() = unsafe { CString::from_vec_unchecked(message.as_bytes().to_vec()) }
    });
    code
}

pub fn handle_error(err: Error) -> c_uint {
    let code = map_error(&err);
    error(code, &format!("{}", err))
}

fn map_io_error(err: &io::Error) -> c_uint {
    use io::ErrorKind::*;
    match err.kind() {
        NotFound => LIBLO_ERROR_FILE_NOT_FOUND,
        AlreadyExists => LIBLO_ERROR_FILE_RENAME_FAIL,
        PermissionDenied => LIBLO_ERROR_IO_PERMISSION_DENIED,
        _ => LIBLO_ERROR_IO_ERROR,
    }
}

fn map_error(err: &Error) -> c_uint {
    use Error::*;
    match *err {
        InvalidPath(_) => LIBLO_ERROR_FILE_NOT_FOUND,
        IoError(_, ref x) => map_io_error(x),
        NoFilename(_) => LIBLO_ERROR_FILE_PARSE_FAIL,
        SystemTimeError(_) => LIBLO_ERROR_TIMESTAMP_WRITE_FAIL,
        NotUtf8(_) => LIBLO_ERROR_FILE_NOT_UTF8,
        DecodeError(_) => LIBLO_ERROR_TEXT_DECODE_FAIL,
        EncodeError(_) => LIBLO_ERROR_TEXT_ENCODE_FAIL,
        PluginParsingError(_, _) => LIBLO_ERROR_FILE_PARSE_FAIL,
        PluginNotFound(_) => LIBLO_ERROR_INVALID_ARGS,
        TooManyActivePlugins { .. } => LIBLO_ERROR_INVALID_ARGS,
        DuplicatePlugin(_) => LIBLO_ERROR_INVALID_ARGS,
        NonMasterBeforeMaster { .. } => LIBLO_ERROR_INVALID_ARGS,
        GameMasterMustLoadFirst(_) => LIBLO_ERROR_INVALID_ARGS,
        InvalidEarlyLoadingPluginPosition { .. } => LIBLO_ERROR_INVALID_ARGS,
        ImplicitlyActivePlugin(_) => LIBLO_ERROR_INVALID_ARGS,
        NoLocalAppData => LIBLO_ERROR_INVALID_ARGS,
        NoDocumentsPath => LIBLO_ERROR_INVALID_ARGS,
        UnrepresentedHoist { .. } => LIBLO_ERROR_INVALID_ARGS,
        InstalledPlugin(_) => LIBLO_ERROR_INVALID_ARGS,
        IniParsingError { .. } => LIBLO_ERROR_FILE_PARSE_FAIL,
        VdfParsingError(_, _) => LIBLO_ERROR_FILE_PARSE_FAIL,
        SystemError(_, _) => LIBLO_ERROR_SYSTEM_ERROR,
    }
}

pub unsafe fn to_str<'a>(c_string: *const c_char) -> Result<&'a str, u32> {
    if c_string.is_null() {
        Err(error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed"))
    } else {
        CStr::from_ptr(c_string)
            .to_str()
            .map_err(|_| error(LIBLO_ERROR_INVALID_ARGS, "Non-UTF-8 string passed"))
    }
}

pub fn to_c_string<S: AsRef<str>>(string: S) -> Result<*mut c_char, u32> {
    CString::new(string.as_ref())
        .map(CString::into_raw)
        .map_err(|_| LIBLO_ERROR_TEXT_ENCODE_FAIL)
}

pub fn to_c_string_array<S: AsRef<str>>(strings: &[S]) -> Result<(*mut *mut c_char, size_t), u32> {
    let mut c_strings = strings
        .iter()
        .map(to_c_string)
        .collect::<Result<Vec<*mut c_char>, u32>>()?;

    c_strings.shrink_to_fit();

    let pointer = c_strings.as_mut_ptr();
    let size = c_strings.len();
    mem::forget(c_strings);

    Ok((pointer, size))
}

pub unsafe fn to_str_vec<'a>(
    array: *const *const c_char,
    array_size: usize,
) -> Result<Vec<&'a str>, u32> {
    slice::from_raw_parts(array, array_size)
        .iter()
        .map(|c| to_str(*c))
        .collect()
}

pub unsafe fn to_path_buf_vec(
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

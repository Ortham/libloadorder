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
use std::slice;
use libc::{c_char, c_uint, size_t};
use loadorder::Error;

use constants::*;
use super::ERROR_MESSAGE;

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
    match err {
        &InvalidPath(_) => LIBLO_ERROR_FILE_NOT_FOUND,
        &IoError(ref x) => map_io_error(x),
        &NoFilename => LIBLO_ERROR_FILE_PARSE_FAIL,
        &SystemTimeError(_) => LIBLO_ERROR_TIMESTAMP_WRITE_FAIL,
        &NotUtf8(_) => LIBLO_ERROR_FILE_NOT_UTF8,
        &DecodeError(_) => LIBLO_ERROR_TEXT_DECODE_FAIL,
        &EncodeError(_) => LIBLO_ERROR_TEXT_ENCODE_FAIL,
        &PluginParsingError => LIBLO_ERROR_FILE_PARSE_FAIL,
        &PluginNotFound(_) => LIBLO_ERROR_INVALID_ARGS,
        &TooManyActivePlugins => LIBLO_ERROR_INVALID_ARGS,
        &InvalidRegex => LIBLO_ERROR_INTERNAL_LOGIC_ERROR,
        &DuplicatePlugin => LIBLO_ERROR_INVALID_ARGS,
        &NonMasterBeforeMaster => LIBLO_ERROR_INVALID_ARGS,
        &GameMasterMustLoadFirst => LIBLO_ERROR_INVALID_ARGS,
        &InvalidPlugin(_) => LIBLO_ERROR_INVALID_ARGS,
        &ImplicitlyActivePlugin(_) => LIBLO_ERROR_INVALID_ARGS,
        &NoLocalAppData => LIBLO_ERROR_INVALID_ARGS,
    }
}

pub unsafe fn to_str<'a>(c_string: *const c_char) -> Result<&'a str, u32> {
    if c_string.is_null() {
        return Err(error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed"));
    }

    let rust_c_string = CStr::from_ptr(c_string);

    Ok(rust_c_string.to_str().map_err(|_| {
        error(LIBLO_ERROR_INVALID_ARGS, "Non-UTF-8 string passed")
    })?)
}

pub fn to_c_string(string: &str) -> Result<*mut c_char, u32> {
    let c_string_name = CString::new(string.to_string()).map_err(|_| {
        LIBLO_ERROR_TEXT_ENCODE_FAIL
    })?;

    Ok(c_string_name.into_raw())
}

pub fn to_c_string_array(strings: Vec<String>) -> Result<(*mut *mut c_char, size_t), u32> {
    let c_strings = strings.iter().map(|s| to_c_string(s)).collect();

    let mut c_strings: Vec<*mut c_char> = match c_strings {
        Ok(x) => x,
        Err(x) => return Err(x),
    };

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
    let plugins: &[*const c_char] = slice::from_raw_parts(array, array_size);

    let plugins: Result<Vec<&str>, u32> = plugins.iter().map(|c| to_str(*c)).collect();

    Ok(plugins?)
}

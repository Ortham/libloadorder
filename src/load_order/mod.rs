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

mod asterisk_based;
mod mutable;
mod openmw;
mod readable;
#[cfg(test)]
mod tests;
mod textfile_based;
mod timestamp_based;
mod writable;

use std::borrow::Cow;

use encoding_rs::WINDOWS_1252;

use super::enums::Error;

pub(crate) use self::asterisk_based::AsteriskBasedLoadOrder;
pub(crate) use self::openmw::OpenMWLoadOrder;
pub use self::readable::ReadableLoadOrder;
pub(crate) use self::textfile_based::TextfileBasedLoadOrder;
pub(crate) use self::timestamp_based::TimestampBasedLoadOrder;
pub use self::writable::WritableLoadOrder;

fn strict_encode(string: &str) -> Result<Cow<'_, [u8]>, Error> {
    let (output, _, had_unmappable_chars) = WINDOWS_1252.encode(string);

    if had_unmappable_chars {
        Err(Error::EncodeError(string.to_owned()))
    } else {
        Ok(output)
    }
}

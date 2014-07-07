/*  libloadorder

    A library for reading and writing the load order of plugin files for
    TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 and
    Fallout: New Vegas.

    Copyright (C) 2012    WrinklyNinja

    This file is part of libloadorder.

    libloadorder is free software: you can redistribute
    it and/or modify it under the terms of the GNU General Public License
    as published by the Free Software Foundation, either version 3 of
    the License, or (at your option) any later version.

    libloadorder is distributed in the hope that it will
    be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with libloadorder.  If not, see
    <http://www.gnu.org/licenses/>.
*/

#include "constants.h"

const unsigned int LIBLO_OK = 0;
const unsigned int LIBLO_WARN_BAD_FILENAME = 1;
const unsigned int LIBLO_WARN_LO_MISMATCH = 2;
const unsigned int LIBLO_ERROR_FILE_READ_FAIL = 3;
const unsigned int LIBLO_ERROR_FILE_WRITE_FAIL = 4;
const unsigned int LIBLO_ERROR_FILE_NOT_UTF8 = 5;
const unsigned int LIBLO_ERROR_FILE_NOT_FOUND = 6;
const unsigned int LIBLO_ERROR_FILE_RENAME_FAIL = 7;
const unsigned int LIBLO_ERROR_TIMESTAMP_READ_FAIL = 8;
const unsigned int LIBLO_ERROR_TIMESTAMP_WRITE_FAIL = 9;
const unsigned int LIBLO_ERROR_FILE_PARSE_FAIL = 10;
const unsigned int LIBLO_ERROR_NO_MEM = 11;
const unsigned int LIBLO_ERROR_INVALID_ARGS = 12;
const unsigned int LIBLO_RETURN_MAX = LIBLO_ERROR_INVALID_ARGS;

const unsigned int LIBLO_METHOD_TIMESTAMP = 0;
const unsigned int LIBLO_METHOD_TEXTFILE  = 1;

const unsigned int LIBLO_GAME_TES3 = 1;
const unsigned int LIBLO_GAME_TES4 = 2;
const unsigned int LIBLO_GAME_TES5 = 3;
const unsigned int LIBLO_GAME_FO3  = 4;
const unsigned int LIBLO_GAME_FNV  = 5;

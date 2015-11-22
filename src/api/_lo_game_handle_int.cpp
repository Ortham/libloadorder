/*  libloadorder

    A library for reading and writing the load order of plugin files for
    TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3,
    Fallout: New Vegas and Fallout 4.

    Copyright (C) 2012-2015 Oliver Hamlet

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

#include "_lo_game_handle_int.h"

using namespace liblo;

_lo_game_handle_int::_lo_game_handle_int(unsigned int id, const boost::filesystem::path& gamePath, const boost::filesystem::path& localPath)
    : GameSettings(id, gamePath, localPath),
    loadOrder(*this),
    extString(nullptr),
    extStringArray(nullptr),
    extStringArraySize(0) {}

_lo_game_handle_int::~_lo_game_handle_int() {
    delete[] extString;

    if (extStringArray != nullptr) {
        for (size_t i = 0; i < extStringArraySize; i++)
            delete[] extStringArray[i];  //Clear all the char strings created.
        delete[] extStringArray;  //Clear the string array.
    }
}

void _lo_game_handle_int::freeStringArray() {
    if (extStringArray != nullptr) {
        for (size_t i = 0; i < extStringArraySize; i++)
            delete[] extStringArray[i];  //Clear all the char strings created.
        delete[] extStringArray;  //Clear the string array.
        extStringArray = nullptr;
        extStringArraySize = 0;
    }
}

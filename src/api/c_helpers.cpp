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

#include "c_helpers.h"
#include "backend/error.h"

#include <cstring>

using namespace std;

namespace liblo {
    // std::string to null-terminated char string converter.
    char * copyString(const string& str) {
        char * p = new char[str.length() + 1];
        return strcpy(p, str.c_str());
    }

    char * extErrorString = nullptr;

    unsigned int c_error(const error& e) {
        delete[] extErrorString;
        try {
            extErrorString = copyString(e.what());
        }
        catch (std::bad_alloc&) {
            extErrorString = nullptr;
        }
        return e.code();
    }

    unsigned int c_error(const unsigned int code, const std::string& what) {
        return c_error(error(code, what.c_str()));
    }
}

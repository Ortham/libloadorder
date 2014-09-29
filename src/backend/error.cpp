/*      libloadorder

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

#include "error.h"
#include <cstring>

namespace liblo {
    char * extErrorString = nullptr;

    error::error(const unsigned int code, const std::string& what) : _code(code), _what(what) {}

    error::~error() throw() {}

    unsigned int error::code() const {
        return _code;
    }

    const char * error::what() const throw() {
        return _what.c_str();
    }

    unsigned int c_error(const error& e) {
        delete[] extErrorString;
        try {
            extErrorString = new char[strlen(e.what()) + 1];
            strcpy(extErrorString, e.what());
        }
        catch (std::bad_alloc& /*e*/) {
            extErrorString = nullptr;
        }
        return e.code();
    }

    unsigned int c_error(const unsigned int code, const std::string& what) {
        return c_error(error(code, what.c_str()));
    }
}

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

#ifndef __LIBLO_HELPERS_H__
#define __LIBLO_HELPERS_H__

#include <string>
#include <boost/filesystem.hpp>

namespace liblo {
    // std::string to null-terminated char string converter.
    char * ToNewCString(const std::string& str);

    //Reads an entire file into a string buffer.
    void fileToBuffer(const boost::filesystem::path& file, std::string& buffer);

    //Only ever have to convert between UTF-8 and Windows-1252.
    std::string ToUTF8(const std::string& str);
    std::string FromUTF8(const std::string& str);
}

#endif

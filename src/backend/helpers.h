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
#include <boost/unordered_map.hpp>
#include <boost/filesystem.hpp>

namespace liblo {
    // std::string to null-terminated char string converter.
    char * ToNewCString(const std::string& str);

    //UTF-8 file validator.
    bool ValidateUTF8File(const boost::filesystem::path& file);

    //Reads an entire file into a string buffer.
    void fileToBuffer(const boost::filesystem::path& file, std::string& buffer);

    //Only ever have to convert between UTF-8 and Windows-1252.
    std::string ToUTF8(const std::string& str);
    std::string FromUTF8(const std::string& str);

    //Version class for more robust version comparisons.
    class Version {
    private:
        std::string verString;

        //Converts an integer to a string using BOOST's Spirit.Karma, which is apparently a lot faster than a stringstream conversion...
        std::string IntToString(const unsigned int n);
    public:
        Version();
        Version(const char * ver);
        Version(const boost::filesystem::path& file);

        std::string AsString() const;

        bool operator > (const Version& rhs) const;
        bool operator < (const Version& rhs) const;
        bool operator >= (const Version& rhs) const;
        bool operator <= (const Version& rhs) const;
        bool operator == (const Version& rhs) const;
        bool operator != (const Version& rhs) const;
    };
}

#endif

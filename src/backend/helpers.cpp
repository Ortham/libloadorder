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

#include "../api/constants.h"
#include "helpers.h"
#include "error.h"
#include "streams.h"
#include <cstring>
#include <boost/locale.hpp>

using namespace std;

namespace liblo {
    // std::string to null-terminated char string converter.
    char * ToNewCString(const string& str) {
        char * p = new char[str.length() + 1];
        return strcpy(p, str.c_str());
    }

    //Reads an entire file into a string buffer.
    void fileToBuffer(const boost::filesystem::path& file, string& buffer) {
        try {
            liblo::ifstream ifile(file);
            ifile.exceptions(std::ios_base::badbit);
            if (ifile.fail())
                return;
            ifile.unsetf(ios::skipws); // No white space skipping!
            copy(
                istream_iterator<char>(ifile),
                istream_iterator<char>(),
                back_inserter(buffer)
                );
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
        }
    }

    std::string ToUTF8(const std::string& str) {
        try {
            return boost::locale::conv::to_utf<char>(str, "Windows-1252", boost::locale::conv::stop);
        }
        catch (boost::locale::conv::conversion_error& /*e*/) {
            throw error(LIBLO_WARN_BAD_FILENAME, "\"" + str + "\" cannot be encoded in Windows-1252.");
        }
    }

    std::string FromUTF8(const std::string& str) {
        try {
            return boost::locale::conv::from_utf<char>(str, "Windows-1252", boost::locale::conv::stop);
        }
        catch (boost::locale::conv::conversion_error& /*e*/) {
            throw error(LIBLO_WARN_BAD_FILENAME, "\"" + str + "\" cannot be encoded in Windows-1252.");
        }
    }
}

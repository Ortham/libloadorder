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

#include "libloadorder/constants.h"
#include "helpers.h"
#include "error.h"
#include <cstring>
#include <boost/locale.hpp>
#include <boost/filesystem/fstream.hpp>

using namespace std;

namespace liblo {
    // std::string to null-terminated char string converter.
    char * copyString(const string& str) {
        char * p = new char[str.length() + 1];
        return strcpy(p, str.c_str());
    }

    //Reads an entire file into a string buffer.
    std::string fileToBuffer(const boost::filesystem::path& file) {
        try {
            string buffer;
            boost::filesystem::ifstream ifile(file);
            ifile.exceptions(std::ios_base::badbit);

            if (!ifile.good())
                throw error(LIBLO_ERROR_FILE_NOT_FOUND, "\"" + file.string() + "\" could not be found.");

            ifile.unsetf(ios::skipws); // No white space skipping!
            copy(
                istream_iterator<char>(ifile),
                istream_iterator<char>(),
                back_inserter(buffer)
                );

            return buffer;
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
        }
    }

    std::string windows1252toUtf8(const std::string& str) {
        try {
            return boost::locale::conv::to_utf<char>(str, "Windows-1252", boost::locale::conv::stop);
        }
        catch (boost::locale::conv::conversion_error&) {
            throw error(LIBLO_WARN_BAD_FILENAME, "\"" + str + "\" cannot be encoded in Windows-1252.");
        }
    }

    std::string utf8ToWindows1252(const std::string& str) {
        try {
            return boost::locale::conv::from_utf<char>(str, "Windows-1252", boost::locale::conv::stop);
        }
        catch (boost::locale::conv::conversion_error&) {
            throw error(LIBLO_WARN_BAD_FILENAME, "\"" + str + "\" cannot be encoded in Windows-1252.");
        }
    }
}

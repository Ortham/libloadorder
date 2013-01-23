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

#include "helpers.h"
#include "libloadorder.h"
#include "error.h"
#include <cstring>
#include <fstream>
#include <sstream>
#include <source/utf8.h>
#include <boost/spirit/include/support_istream_iterator.hpp>
#include <boost/spirit/include/karma.hpp>
#include <boost/regex.hpp>
#include <boost/locale.hpp>

#if _WIN32 || _WIN64
#   ifndef UNICODE
#       define UNICODE
#   endif
#   ifndef _UNICODE
#      define _UNICODE
#   endif
#   include "windows.h"
#endif

using namespace std;

namespace liblo {

    // std::string to null-terminated char string converter.
    char * ToNewCString(const string& str) {
        char * p = new char[str.length() + 1];
        return strcpy(p, str.c_str());
    }

    //UTF-8 file validator.
    bool ValidateUTF8File(const boost::filesystem::path& file) {
        ifstream ifs(file.string().c_str());

        istreambuf_iterator<char> it(ifs.rdbuf());
        istreambuf_iterator<char> eos;

        if (!utf8::is_valid(it, eos))
            return false;
        else
            return true;
    }

    //Reads an entire file into a string buffer.
    void fileToBuffer(const boost::filesystem::path& file, string& buffer) {
        ifstream ifile(file.string().c_str());
        if (ifile.fail())
            return;
        ifile.unsetf(ios::skipws); // No white space skipping!
        copy(
            istream_iterator<char>(ifile),
            istream_iterator<char>(),
            back_inserter(buffer)
        );
    }

    std::string ToUTF8(const std::string& str) {
        try {
            return boost::locale::conv::to_utf<char>(str, "Windows-1252", boost::locale::conv::stop);
        } catch (boost::locale::conv::conversion_error& e) {
            throw error(LIBLO_WARN_BAD_FILENAME, "\"" + str + "\" cannot be encoded in Windows-1252.");
        }
    }

    std::string FromUTF8(const std::string& str) {
        try {
            return boost::locale::conv::from_utf<char>(str, "Windows-1252", boost::locale::conv::stop);
        } catch (boost::locale::conv::conversion_error& e) {
            throw error(LIBLO_WARN_BAD_FILENAME, "\"" + str + "\" cannot be encoded in Windows-1252.");
        }
    }


    //////////////////////////////
    // Version Class Functions
    //////////////////////////////

    Version::Version() {}

    Version::Version(const char * ver)
        : verString(ver) {}

    Version::Version(const boost::filesystem::path& file) {
#if _WIN32 || _WIN64
        DWORD dummy = 0;
        DWORD size = GetFileVersionInfoSize(file.wstring().c_str(), &dummy);

        if (size > 0) {
            LPBYTE point = new BYTE[size];
            UINT uLen;
            VS_FIXEDFILEINFO *info;
            string ver;

            GetFileVersionInfo(file.wstring().c_str(),0,size,point);

            VerQueryValue(point,L"\\",(LPVOID *)&info,&uLen);

            DWORD dwLeftMost     = HIWORD(info->dwFileVersionMS);
            DWORD dwSecondLeft   = LOWORD(info->dwFileVersionMS);
            DWORD dwSecondRight  = HIWORD(info->dwFileVersionLS);
            DWORD dwRightMost    = LOWORD(info->dwFileVersionLS);

            delete [] point;

            verString = IntToString(dwLeftMost) + '.' + IntToString(dwSecondLeft) + '.' + IntToString(dwSecondRight) + '.' + IntToString(dwRightMost);
        }
#else
        // ensure filename has no quote characters in it to avoid command injection attacks
        if (string::npos == file.string().find('"')) {
            // command mostly borrowed from the gnome-exe-thumbnailer.sh script
            // wrestool is part of the icoutils package
            string cmd = "wrestool --extract --raw --type=version \"" + file.string() + "\" | tr '\\0, ' '\\t.\\0' | sed 's/\\t\\t/_/g' | tr -c -d '[:print:]' | sed -r 's/.*Version[^0-9]*([0-9]+(\\.[0-9]+)+).*/\\1/'";

            FILE *fp = popen(cmd.c_str(), "r");

            // read out the version string
            static const unsigned int BUFSIZE = 32;
            char buf[BUFSIZE];
            if (NULL != fgets(buf, BUFSIZE, fp))
                verString = string(buf);
            pclose(fp);
        }
#endif
    }

    string Version::AsString() const {
        return verString;
    }

    bool Version::operator < (const Version& ver) const {
        /* In libloadorder, the version comparison is only used for checking the versions of games,
           which always have the format "a.b.c.d" where a, b, c and d are all integers. */

        istringstream parser1(verString);
        istringstream parser2(ver.AsString());
        while (parser1.good() || parser2.good()) {
            //Check if each stringstream is OK for i/o before doing anything with it. If not, replace its extracted value with a 0.
            unsigned int n1, n2;
            if (parser1.good()) {
                parser1 >> n1;
                parser1.get();
            } else
                n1 = 0;
            if (parser2.good()) {
                parser2 >> n2;
                parser2.get();
            } else
                n2 = 0;
            if (n1 < n2)
                return true;
            else if (n1 > n2)
                return false;
        }
        return false;
    }

    bool Version::operator > (const Version& rhs) const {
        return *this != rhs && !(*this < rhs);
    }

    bool Version::operator >= (const Version& rhs) const {
        return *this == rhs || *this > rhs;
    }

    bool Version::operator <= (const Version& rhs) const {
        return *this == rhs || *this < rhs;
    }

    bool Version::operator == (const Version& rhs) const {
        return verString == rhs.AsString();
    }

    bool Version::operator != (const Version& rhs) const {
        return !(*this == rhs);
    }

    //Converts an integer to a string using BOOST's Spirit.Karma, which is apparently a lot faster than a stringstream conversion...
    string Version::IntToString(const unsigned int n) {
        string out;
        back_insert_iterator<string> sink(out);
        boost::spirit::karma::generate(sink,boost::spirit::karma::upper[boost::spirit::karma::uint_],n);
        return out;
    }
}

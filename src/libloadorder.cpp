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

#include "libloadorder.h"
#include "helpers.h"
#include "game.h"
#include "error.h"
#include <boost/filesystem/detail/utf8_codecvt_facet.hpp>
#include <locale>

using namespace std;
using namespace liblo;

/*------------------------------
   Global variables
------------------------------*/

const unsigned int LIBLO_VERSION_MAJOR = 2;
const unsigned int LIBLO_VERSION_MINOR = 0;
const unsigned int LIBLO_VERSION_PATCH = 0;


/*------------------------------
   Constants
------------------------------*/

/* The following are the possible codes that the library can return. */
const unsigned int LIBLO_OK                         = 0;
const unsigned int LIBLO_WARN_BAD_FILENAME          = 1;
const unsigned int LIBLO_WARN_LO_MISMATCH           = 2;
const unsigned int LIBLO_ERROR_FILE_READ_FAIL       = 3;
const unsigned int LIBLO_ERROR_FILE_WRITE_FAIL      = 4;
const unsigned int LIBLO_ERROR_FILE_NOT_UTF8        = 5;
const unsigned int LIBLO_ERROR_FILE_NOT_FOUND       = 6;
const unsigned int LIBLO_ERROR_FILE_RENAME_FAIL     = 7;
const unsigned int LIBLO_ERROR_TIMESTAMP_READ_FAIL  = 8;
const unsigned int LIBLO_ERROR_TIMESTAMP_WRITE_FAIL = 9;
const unsigned int LIBLO_ERROR_FILE_PARSE_FAIL      = 10;
const unsigned int LIBLO_ERROR_NO_MEM               = 11;
const unsigned int LIBLO_ERROR_INVALID_ARGS         = 12;
const unsigned int LIBLO_RETURN_MAX                 = LIBLO_ERROR_INVALID_ARGS;

/* The following are for signifying what load order method is being used. */
const unsigned int LIBLO_METHOD_TIMESTAMP           = 0;
const unsigned int LIBLO_METHOD_TEXTFILE            = 1;

/* The following are the games identifiers used by the library. */
const unsigned int LIBLO_GAME_TES3                  = 1;
const unsigned int LIBLO_GAME_TES4                  = 2;
const unsigned int LIBLO_GAME_TES5                  = 3;
const unsigned int LIBLO_GAME_FO3                   = 4;
const unsigned int LIBLO_GAME_FNV                   = 5;


/*------------------------------
   Version Functions
------------------------------*/

/* Returns whether this version of libloadorder is compatible with the given
   version of libloadorder. */
LIBLO bool lo_is_compatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch) {
    if (versionMajor == 2 && versionMinor == 0 && versionPatch == 0)
        return true;
    else
        return false;
}

LIBLO void lo_get_version(unsigned int * versionMajor, unsigned int * versionMinor, unsigned int * versionPatch) {
    *versionMajor = LIBLO_VERSION_MAJOR;
    *versionMinor = LIBLO_VERSION_MINOR;
    *versionPatch = LIBLO_VERSION_PATCH;
}


/*------------------------------
   Error Handling Functions
------------------------------*/

/* Outputs a string giving the a message containing the details of the
   last error or warning encountered by a function called for the given
   game handle. */
LIBLO unsigned int lo_get_error_message(const char ** details) {
    if (details == NULL)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    *details = extErrorString;

    return LIBLO_OK;
}

LIBLO void lo_cleanup() {
    delete [] extErrorString;
    extErrorString = NULL;
}


/*----------------------------------
   Lifecycle Management Functions
----------------------------------*/

/* Creates a handle for the game given by gameId, which is found at gamePath. This handle allows
   clients to free memory when they want to. gamePath is case-sensitive if the underlying filesystem
   is case-sensitive. */
LIBLO unsigned int lo_create_handle(lo_game_handle * gh, const unsigned int gameId, const char * gamePath) {
    if (gh == NULL || gamePath == NULL) //Check for valid args.
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");
    else if (gameId != LIBLO_GAME_TES3 && gameId != LIBLO_GAME_TES4 && gameId != LIBLO_GAME_TES5 && gameId != LIBLO_GAME_FO3 && gameId != LIBLO_GAME_FNV)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Invalid game specified.");

    //Set the locale to get encoding conversions working correctly.
    setlocale(LC_CTYPE, "");
    locale global_loc = locale();
    locale loc(global_loc, new boost::filesystem::detail::utf8_codecvt_facet());
    boost::filesystem::path::imbue(loc);

    //Create handle.
    try {
        *gh = new _lo_game_handle_int(gameId, string(reinterpret_cast<const char *>(gamePath)));
    } catch (error& e) {
        return c_error(e);
    }

    if ((**gh).LoadOrderMethod() == LIBLO_METHOD_TEXTFILE && boost::filesystem::exists((**gh).ActivePluginsFile()) && boost::filesystem::exists((**gh).LoadOrderFile())) {
        //Check for desync.
        LoadOrder PluginsFileLO;
        LoadOrder LoadOrderFileLO;

        try {
            //First get load order according to loadorder.txt.
            LoadOrderFileLO.LoadFromFile(**gh, (**gh).LoadOrderFile());
            //Get load order from plugins.txt.
            PluginsFileLO.LoadFromFile(**gh, (**gh).ActivePluginsFile());
        } catch (error& e) {
            delete *gh;
            return c_error(e);
        }

        //Remove any plugins from LoadOrderFileLO that are not in PluginsFileLO.
        vector<Plugin>::iterator it=LoadOrderFileLO.begin(), endIt=LoadOrderFileLO.end(), pEndIt=PluginsFileLO.end();
        while (it != endIt) {
            if (PluginsFileLO.begin() + PluginsFileLO.Find(*it) == pEndIt)
                it = LoadOrderFileLO.erase(it);
            else
                ++it;
        }

        //Compare the two LoadOrder objects: they should be identical (since mtimes for each have not been touched).
        if (PluginsFileLO != LoadOrderFileLO)
            return c_error(LIBLO_WARN_LO_MISMATCH, "The order of plugins present in both loadorder.txt and plugins.txt differs between the two files.");
    }

    return LIBLO_OK;
}

/* Destroys the given game handle, freeing up memory allocated during its use. */
LIBLO void lo_destroy_handle(lo_game_handle gh) {
    delete gh;
}

/* Sets the game's master file to a given filename, eg. for use with total conversions where
   the original main master file is replaced. */
LIBLO unsigned int lo_set_game_master(lo_game_handle gh, const char * masterFile) {
    if (gh == NULL || masterFile == NULL) //Check for valid args.
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    try {
        gh->SetMasterFile(string(reinterpret_cast<const char *>(masterFile)));
    } catch (error& e) {
        return c_error(e);
    }

    return LIBLO_OK;
}

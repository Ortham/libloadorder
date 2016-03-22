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

#include "libloadorder/libloadorder.h"
#include "../api/_lo_game_handle_int.h"
#include "../backend/helpers.h"
#include "../backend/error.h"

#include <codecvt>

using namespace std;
using namespace liblo;

/*------------------------------
   Version Functions
   ------------------------------*/

const unsigned int LIBLO_VERSION_MAJOR = 8;
const unsigned int LIBLO_VERSION_MINOR = 0;
const unsigned int LIBLO_VERSION_PATCH = 0;

/* Returns whether this version of libloadorder is compatible with the given
   version of libloadorder. */
LIBLO bool lo_is_compatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch) {
    if (versionMajor == LIBLO_VERSION_MAJOR)
        return true;
    else
        return false;
}

LIBLO unsigned int lo_get_version(unsigned int * const versionMajor, unsigned int * const versionMinor, unsigned int * const versionPatch) {
    if (versionMajor == nullptr || versionMinor == nullptr || versionPatch == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    *versionMajor = LIBLO_VERSION_MAJOR;
    *versionMinor = LIBLO_VERSION_MINOR;
    *versionPatch = LIBLO_VERSION_PATCH;

    return LIBLO_OK;
}

/*------------------------------
   Error Handling Functions
   ------------------------------*/

/* Outputs a string giving the a message containing the details of the
   last error or warning encountered by a function called for the given
   game handle. */
LIBLO unsigned int lo_get_error_message(const char ** const details) {
    if (details == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    if (extErrorString.empty())
        *details = nullptr;
    else
        *details = extErrorString.c_str();

    return LIBLO_OK;
}

LIBLO void lo_cleanup() {
    extErrorString.clear();
}

/*----------------------------------
   Lifecycle Management Functions
   ----------------------------------*/

/* Creates a handle for the game given by gameId, which is found at gamePath. This handle allows
   clients to free memory when they want to. gamePath is case-sensitive if the underlying filesystem
   is case-sensitive. */
LIBLO unsigned int lo_create_handle(lo_game_handle * const gh,
                                    const unsigned int gameId,
                                    const char * const gamePath,
                                    const char * const localPath) {
    if (gh == nullptr || gamePath == nullptr) //Check for valid args.
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");
    else if (gameId != LIBLO_GAME_TES3
             && gameId != LIBLO_GAME_TES4
             && gameId != LIBLO_GAME_TES5
             && gameId != LIBLO_GAME_FO3
             && gameId != LIBLO_GAME_FNV
             && gameId != LIBLO_GAME_FO4)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Invalid game specified.");

    // Set the locale to get UTF-8 conversions working correctly.
    std::locale::global(std::locale(std::locale(), new std::codecvt_utf8_utf16<wchar_t>));
    boost::filesystem::path::imbue(std::locale());

    try {
        // Check for valid paths.
        if (!boost::filesystem::is_directory(gamePath))
            return c_error(LIBLO_ERROR_INVALID_ARGS, "Given game path \"" + string(gamePath) + "\" is not a valid directory.");

        string localPathString;
        if (localPath != nullptr) {
            if (!boost::filesystem::is_directory(localPath))
                return c_error(LIBLO_ERROR_INVALID_ARGS, "Given local data path \"" + string(localPath) + "\" is not a valid directory.");

            localPathString = localPath;
        }
#ifndef _WIN32
        else
            return c_error(LIBLO_ERROR_INVALID_ARGS, "A local data path must be supplied on non-Windows platforms.");
#endif

        //Create handle.
        *gh = new _lo_game_handle_int(gameId, gamePath, localPathString);
    }
    catch (error& e) {
        return c_error(e);
    }
    catch (std::bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
    }
    catch (std::exception& e) {
        return c_error(LIBLO_ERROR_INVALID_ARGS, e.what());
    }

    try {
        // Check for desync. This is a textfile-specific issue, but the checking
        // function will handle that distinction.
        if (!LoadOrder::isSynchronised(**gh))
            return c_error(LIBLO_WARN_LO_MISMATCH, "The order of plugins present in both loadorder.txt and plugins.txt differs between the two files.");
    }
    catch (error& e) {
        delete *gh;
        *gh = nullptr;
        return c_error(e);
    }
    catch (std::exception& e) {
        delete *gh;
        *gh = nullptr;
        return c_error(LIBLO_ERROR_FILE_READ_FAIL, e.what());
    }

    return LIBLO_OK;
}

/* Destroys the given game handle, freeing up memory allocated during its use. */
LIBLO void lo_destroy_handle(lo_game_handle gh) {
    delete gh;
}

/* Sets the game's master file to a given filename, eg. for use with total conversions where
   the original main master file is replaced. */
LIBLO unsigned int lo_set_game_master(lo_game_handle gh, const char * const masterFile) {
    if (gh == nullptr || masterFile == nullptr) //Check for valid args.
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");
    if (gh->getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Cannot change main master file from " + gh->getMasterFile());

    return LIBLO_OK;
}

/*----------------------------------
   Misc Functions
   ----------------------------------*/

/* Removes any plugins that are not present in the filesystem from plugins.txt (and loadorder.txt if used). */
LIBLO unsigned int lo_fix_plugin_lists(lo_game_handle gh) {
    if (gh == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    try {
        gh->loadOrder.load();
        gh->loadOrder.save();
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

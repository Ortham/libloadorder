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
#include "../backend/helpers.h"
#include "../backend/game.h"
#include "../backend/error.h"
#include <boost/filesystem/detail/utf8_codecvt_facet.hpp>
#include <locale>

using namespace std;
using namespace liblo;


/*------------------------------
   Version Functions
------------------------------*/

const unsigned int LIBLO_VERSION_MAJOR = 4;
const unsigned int LIBLO_VERSION_MINOR = 0;
const unsigned int LIBLO_VERSION_PATCH = 1;

/* Returns whether this version of libloadorder is compatible with the given
   version of libloadorder. */
LIBLO bool lo_is_compatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch) {
    if (versionMajor == LIBLO_VERSION_MAJOR)
        return true;
    else
        return false;
}

LIBLO void lo_get_version(unsigned int * const versionMajor, unsigned int * const versionMinor, unsigned int * const versionPatch) {
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
LIBLO unsigned int lo_get_error_message(const char ** const details) {
    if (details == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    *details = extErrorString;

    return LIBLO_OK;
}

LIBLO void lo_cleanup() {
    delete [] extErrorString;
    extErrorString = nullptr;
}


/*----------------------------------
   Lifecycle Management Functions
----------------------------------*/

/* Creates a handle for the game given by gameId, which is found at gamePath. This handle allows
   clients to free memory when they want to. gamePath is case-sensitive if the underlying filesystem
   is case-sensitive. */
LIBLO unsigned int lo_create_handle(lo_game_handle * const gh, const unsigned int gameId, const char * const gamePath) {
    if (gh == nullptr || gamePath == nullptr) //Check for valid args.
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
        *gh = new _lo_game_handle_int(gameId, gamePath);
    } catch (error& e) {
        return c_error(e);
    } catch (std::bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
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
        vector<Plugin>::iterator it=LoadOrderFileLO.begin();
        while (it != LoadOrderFileLO.end()) {
            if (PluginsFileLO.Find(*it) == PluginsFileLO.size())
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
LIBLO unsigned int lo_set_game_master(lo_game_handle gh, const char * const masterFile) {
    if (gh == nullptr || masterFile == nullptr) //Check for valid args.
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    try {
        gh->SetMasterFile(masterFile);
    } catch (error& e) {
        return c_error(e);
    }

    return LIBLO_OK;
}

/*----------------------------------
   Misc Functions
----------------------------------*/

/* Removes any plugins that are not present in the filesystem from plugins.txt (and loadorder.txt if used). */
LIBLO unsigned int lo_fix_plugin_lists(lo_game_handle gh) {
    if (gh == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    //Only need to update loadorder.txt if it is used.
    if (gh->LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
        try {
            //Update cache if necessary.
            if (gh->loadOrder.HasChanged(*gh))
                gh->loadOrder.Load(*gh);

            //Now check all plugins' existences.
            vector<Plugin>::iterator it = gh->loadOrder.begin();
            while (it != gh->loadOrder.end()) {
                if (!it->Exists(*gh))  //Active plugin is not installed.
                    it = gh->loadOrder.erase(it);
                else
                    ++it;
            }
        } catch (error& e) {
            return c_error(e);
        }
    }

    try {
        //Update cache if necessary.
        if (gh->activePlugins.HasChanged(*gh))
            gh->activePlugins.Load(*gh);

        //Now check all plugins' existences.
        auto it = gh->activePlugins.begin();
        while (it != gh->activePlugins.end()) {
            if (!it->Exists(*gh))  //Active plugin is not installed.
                it = gh->activePlugins.erase(it);
            else
                ++it;
        }
    } catch (error& e) {
        return c_error(e);
    }

    return LIBLO_OK;
}

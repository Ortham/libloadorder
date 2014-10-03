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
#include <boost/locale.hpp>
#include <locale>

using namespace std;
using namespace liblo;

/*------------------------------
   Version Functions
   ------------------------------*/

const unsigned int LIBLO_VERSION_MAJOR = 5;
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

    *details = extErrorString;

    return LIBLO_OK;
}

LIBLO void lo_cleanup() {
    delete[] extErrorString;
    extErrorString = nullptr;
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
    else if (gameId != LIBLO_GAME_TES3 && gameId != LIBLO_GAME_TES4 && gameId != LIBLO_GAME_TES5 && gameId != LIBLO_GAME_FO3 && gameId != LIBLO_GAME_FNV)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Invalid game specified.");

    //Set the locale to get encoding conversions working correctly.
    std::locale::global(boost::locale::generator().generate(""));
    boost::filesystem::path::imbue(std::locale());

    // Check for valid paths.
    boost::filesystem::path testPath(gamePath);
    for (const auto& name : testPath) {
        if (!boost::filesystem::native(name.string()))
            return c_error(LIBLO_ERROR_INVALID_ARGS, "Invalid game path specified.");
    }
    testPath = localPath;
    for (const auto& name : testPath) {
        if (!boost::filesystem::native(name.string()))
            return c_error(LIBLO_ERROR_INVALID_ARGS, "Invalid local data path specified.");
    }

    //Create handle.
    try {
        *gh = new _lo_game_handle_int(gameId, gamePath);
        if (localPath != nullptr)
            (*gh)->SetLocalAppData(localPath);
    }
    catch (error& e) {
        return c_error(e);
    }
    catch (std::bad_alloc& e) {
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
        }
        catch (error& e) {
            delete *gh;
            return c_error(e);
        }

        try {
            LoadOrderFileLO.CheckValidity(**gh);
            PluginsFileLO.CheckValidity(**gh);
        }
        catch (error& e) {
            return c_error(e);
        }

        //Remove any plugins from LoadOrderFileLO that are not in PluginsFileLO.
        vector<Plugin>::iterator it = LoadOrderFileLO.begin();
        while (it != LoadOrderFileLO.end()) {
            if (PluginsFileLO.Find(*it) == PluginsFileLO.cend())
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
    }
    catch (error& e) {
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

    unsigned int successRetCode = LIBLO_OK;

    //Only need to update loadorder.txt if it is used.
    if (gh->LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
        try {
            //Update cache if necessary.
            if (gh->loadOrder.HasChanged(*gh)) {
                gh->loadOrder.Load(*gh);
                try {
                    gh->activePlugins.CheckValidity(*gh);
                }
                catch (error& e) {
                    successRetCode = c_error(e);
                }
            }

            // Ensure that the first plugin is the game's master file.
            if (!gh->loadOrder.empty() && !boost::iequals(gh->loadOrder.begin()->Name(), gh->MasterFile())) {
                gh->loadOrder.Move(gh->MasterFile(), gh->loadOrder.begin());
            }

            // Now check all plugins' existences.
            // Also check that no plugin appears more than once.
            // Also ensure that all master files load before all plugin files.
            unordered_set<Plugin> hashset;
            auto firstNonMaster = gh->loadOrder.FindFirstNonMaster(*gh);
            vector<Plugin>::iterator it = gh->loadOrder.begin();
            while (it != gh->loadOrder.end()) {
                if (!it->Exists(*gh)) {
                    //Active plugin is not installed.
                    it = gh->loadOrder.erase(it);
                    continue;
                }

                if (hashset.find(*it) != hashset.end()) {
                    // Plugin has already appeared, remove this appearance.
                    it = gh->loadOrder.erase(it);
                    continue;
                }

                if (it->IsMasterFile(*gh)) {
                    if (distance(gh->loadOrder.begin(), it) > distance(gh->loadOrder.cbegin(), firstNonMaster)) {
                        // Master amongst plugins, move it after the last master.
                        firstNonMaster = ++gh->loadOrder.Move(*it, firstNonMaster);
                    }
                }
            }

            // Now write changes.
            gh->loadOrder.Save(*gh);
        }
        catch (error& e) {
            return c_error(e);
        }
    }

    try {
        //Update cache if necessary.
        if (gh->activePlugins.HasChanged(*gh)) {
            gh->activePlugins.Load(*gh);
            try {
                gh->activePlugins.CheckValidity(*gh);
            }
            catch (error& e) {
                successRetCode = c_error(e);
            }
        }

        // Check that there aren't more than 255 plugins, and remove those
        // at the end of the load order if so.
        if (gh->activePlugins.size() > 255) {
            size_t toRemove = gh->activePlugins.size() - 255;
            while (toRemove > 0) {
                for (auto rit = gh->loadOrder.crbegin(); rit != gh->loadOrder.crend(); ++rit) {
                    auto pos = gh->activePlugins.find(*rit);
                    if (pos != gh->activePlugins.end())
                        gh->activePlugins.erase(pos);
                }
            }
        }

        if (gh->Id() == LIBLO_GAME_TES5) {
            // Ensure Skyrim.esm is active.
            if (gh->activePlugins.find(Plugin("Skyrim.esm")) == gh->activePlugins.end())
                gh->activePlugins.insert(Plugin("Skyrim.esm"));

            // Ensure Update.esm is active, if it is installed.
            if (Plugin("Update.esm").Exists(*gh) && gh->activePlugins.find(Plugin("Update.esm")) == gh->activePlugins.end())
                gh->activePlugins.insert(Plugin("Update.esm"));
        }

        //Now check all plugins' existences.
        auto it = gh->activePlugins.begin();
        while (it != gh->activePlugins.end()) {
            if (!it->Exists(*gh))  //Active plugin is not installed.
                it = gh->activePlugins.erase(it);
            else
                ++it;
        }

        // Now write changes.
        gh->activePlugins.Save(*gh);
    }
    catch (error& e) {
        return c_error(e);
    }

    return successRetCode;
}

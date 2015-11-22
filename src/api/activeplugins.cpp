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

#include "libloadorder/activeplugins.h"
#include "../api/_lo_game_handle_int.h"
#include "../backend/helpers.h"
#include "../backend/error.h"

using namespace std;
using namespace liblo;

/*----------------------------------
   Plugin Active Status Functions
   ----------------------------------*/

/* Returns the list of active plugins. */
LIBLO unsigned int lo_get_active_plugins(lo_game_handle gh, char *** const plugins, size_t * const numPlugins) {
    if (gh == nullptr || plugins == nullptr || numPlugins == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    unsigned int successRetCode = LIBLO_OK;

    //Free memory if in use.
    if (gh->extStringArray != nullptr) {
        for (size_t i = 0; i < gh->extStringArraySize; i++)
            delete[] gh->extStringArray[i];  //Clear all the char strings created.
        delete[] gh->extStringArray;  //Clear the string array.
        gh->extStringArray = nullptr;
        gh->extStringArraySize = 0;
    }

    //Set initial outputs.
    *plugins = gh->extStringArray;
    *numPlugins = gh->extStringArraySize;

    //Update cache if necessary.
    try {
        if (gh->loadOrder.hasFilesystemChanged())
            gh->loadOrder.load();
    }
    catch (error& e) {
        return c_error(e);
    }

    //Check array size. Exit if zero.
    unordered_set<string> loadOrder = gh->loadOrder.getActivePlugins();
    if (loadOrder.empty())
        return LIBLO_OK;

    //Allocate memory.
    gh->extStringArraySize = loadOrder.size();
    try {
        gh->extStringArray = new char*[gh->extStringArraySize];
        size_t i = 0;
        for (const auto &activePlugin : loadOrder) {
            gh->extStringArray[i] = copyString(activePlugin);
            i++;
        }
    }
    catch (bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
    }

    //Set outputs.
    *plugins = gh->extStringArray;
    *numPlugins = gh->extStringArraySize;

    return successRetCode;
}

/* Replaces the current list of active plugins with the given list. */
LIBLO unsigned int lo_set_active_plugins(lo_game_handle gh, const char * const * const plugins, const size_t numPlugins) {
    if (gh == nullptr || plugins == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    //Update cache if necessary.
    try {
        if (gh->loadOrder.hasFilesystemChanged())
            gh->loadOrder.load();
    }
    catch (error& e) {
        return c_error(e);
    }

    //Put input into activePlugins object.
    unordered_set<string> activePlugins;
    for (size_t i = 0; i < numPlugins; i++) {
        activePlugins.insert(plugins[i]);
    }

    try {
        gh->loadOrder.setActivePlugins(activePlugins);
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(LIBLO_ERROR_INVALID_ARGS, string("Invalid active plugins list supplied. Details: ") + e.what());
    }

    //Now save changes.
    try {
        gh->loadOrder.save();
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

/* Activates or deactivates the given plugin depending on the value of the active argument. */
LIBLO unsigned int lo_set_plugin_active(lo_game_handle gh, const char * const plugin, const bool active) {
    if (gh == nullptr || plugin == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    //Update cache if necessary.
    try {
        if (gh->loadOrder.hasFilesystemChanged())
            gh->loadOrder.load();
    }
    catch (error& e) {
        return c_error(e);
    }

    //Look for plugin in active plugins list.
    try {
        if (active)
            gh->loadOrder.activate(plugin);
        else
            gh->loadOrder.deactivate(plugin);
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(LIBLO_ERROR_INVALID_ARGS, string("The operation results in an invalid active plugins list. Details: ") + e.what());
    }

    //Now save changes.
    try {
        gh->loadOrder.save();
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

/* Checks to see if the given plugin is active. */
LIBLO unsigned int lo_get_plugin_active(lo_game_handle gh, const char * const plugin, bool * const result) {
    if (gh == nullptr || plugin == nullptr || result == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    unsigned int successRetCode = LIBLO_OK;

    //Update cache if necessary.
    try {
        if (gh->loadOrder.hasFilesystemChanged())
            gh->loadOrder.load();
    }
    catch (error& e) {
        return c_error(e);
    }

    *result = gh->loadOrder.isActive(plugin);

    return successRetCode;
}

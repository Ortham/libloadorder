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

#include "libloadorder/activeplugins.h"
#include "_lo_game_handle_int.h"
#include "c_helpers.h"
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

    //Set initial outputs.
    *plugins = nullptr;
    *numPlugins = 0;

    try {
        //Update cache if necessary.
        gh->loadOrder.load();

        unordered_set<string> activePlugins = gh->loadOrder.getActivePlugins();

        //Check set size. Exit early if zero.
        if (activePlugins.empty())
            return LIBLO_OK;

        //Allocate memory.
        gh->setExternalStringArray(activePlugins);
    }
    catch (bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    //Set outputs.
    *plugins = const_cast<char **>(&gh->getExternalStringArray()[0]);
    *numPlugins = gh->getExternalStringArray().size();

    return LIBLO_OK;
}

/* Replaces the current list of active plugins with the given list. */
LIBLO unsigned int lo_set_active_plugins(lo_game_handle gh, const char * const * const plugins, const size_t numPlugins) {
    if (gh == nullptr || plugins == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    //Update cache if necessary.
    try {
        gh->loadOrder.load();
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    try {
        gh->loadOrder.setActivePlugins(copyToContainer<unordered_set<string>>(plugins, numPlugins));
    }
    catch (error& e) {
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
        gh->loadOrder.load();
    }
    catch (error& e) {
        gh->loadOrder.clear();
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

    //Update cache if necessary.
    try {
        gh->loadOrder.load();
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    *result = gh->loadOrder.isActive(plugin);

    return LIBLO_OK;
}

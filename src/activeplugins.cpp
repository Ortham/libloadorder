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
#include "game.h"
#include "helpers.h"
#include "error.h"

using namespace std;
using namespace liblo;

/*----------------------------------
   Plugin Active Status Functions
----------------------------------*/

/* Returns the list of active plugins. */
LIBLO unsigned int lo_get_active_plugins(lo_game_handle gh, char *** const plugins, size_t * const numPlugins) {
    if (gh == NULL || plugins == NULL || numPlugins == NULL)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    //Free memory if in use.
    if (gh->extStringArray != NULL) {
        for (size_t i=0; i < gh->extStringArraySize; i++)
            delete[] gh->extStringArray[i];  //Clear all the char strings created.
        delete[] gh->extStringArray;  //Clear the string array.
        gh->extStringArray = NULL;
        gh->extStringArraySize = 0;
    }

    //Update cache if necessary.
    try {
        if (gh->activePlugins.HasChanged(*gh))
            gh->activePlugins.Load(*gh);
    } catch (error& e) {
        return c_error(e);
    }

    //Check array size. Exit if zero.
    if (gh->activePlugins.empty())
        return LIBLO_OK;

    //Allocate memory.
    gh->extStringArraySize = gh->activePlugins.size();
    try {
        gh->extStringArray = new char*[gh->extStringArraySize];
        size_t i = 0;
        for (boost::unordered_set<Plugin>::const_iterator it = gh->activePlugins.begin(), endIt = gh->activePlugins.end(); it != endIt; ++it) {
            gh->extStringArray[i] = ToNewCString(it->Name());
            i++;
        }
    } catch(bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
    }

    //Set outputs.
    *plugins = gh->extStringArray;
    *numPlugins = gh->extStringArraySize;

    return LIBLO_OK;
}

/* Replaces the current list of active plugins with the given list. */
LIBLO unsigned int lo_set_active_plugins(lo_game_handle gh, char ** const plugins, const size_t numPlugins) {
    if (gh == NULL || plugins == NULL)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    //Put input into activePlugins object.
    gh->activePlugins.clear();
    for (size_t i=0; i < numPlugins; i++) {
        Plugin plugin(plugins[i]);
        if (gh->activePlugins.find(plugin) != gh->activePlugins.end()) {  //Not necessary for unordered set, but present so that invalid active plugin lists are refused.
            gh->activePlugins.clear();
            return c_error(LIBLO_ERROR_INVALID_ARGS, "The supplied active plugins list is invalid.");
        } else if (plugin.Exists(*gh))
            gh->activePlugins.emplace(plugin);
        else {
            gh->activePlugins.clear();
            return c_error(LIBLO_ERROR_FILE_NOT_FOUND, "\"" + plugin.Name() + "\" cannot be found.");
        }
    }

    //Check to see if basic rules are being obeyed.
    if (!gh->activePlugins.IsValid(*gh)) {
        gh->activePlugins.clear();
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Invalid active plugins list supplied.");
    }

    //Now save changes.
    try {
        gh->activePlugins.Save(*gh);
    } catch (error& e) {
        gh->activePlugins.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

/* Activates or deactivates the given plugin depending on the value of the active argument. */
LIBLO unsigned int lo_set_plugin_active(lo_game_handle gh, const char * const plugin, const bool active) {
    if (gh == NULL || plugin == NULL)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    Plugin pluginObj(plugin);

    //Check that plugin exists if activating it.
    if (active && !pluginObj.Exists(*gh))
        return c_error(LIBLO_ERROR_FILE_NOT_FOUND, "\"" + pluginObj.Name() + "\" cannot be found.");

    //Unghost plugin if ghosted.
    try {
        pluginObj.UnGhost(*gh);
    } catch (error& e) {
        return c_error(e);
    }

    //Update cache if necessary.
    try {
        if (gh->activePlugins.HasChanged(*gh))
            gh->activePlugins.Load(*gh);
    } catch (error& e) {
        return c_error(e);
    }

    //Look for plugin in active plugins list.
    boost::unordered_set<Plugin>::const_iterator it = gh->activePlugins.find(pluginObj);
    if (active)  //No need to check for duplication, unordered set will silently handle avoidance.
        gh->activePlugins.emplace(pluginObj);
    else if (!active && it != gh->activePlugins.end())
        gh->activePlugins.erase(it);

    //Check that active plugins list is valid.
    if (!gh->activePlugins.IsValid(*gh)) {
        gh->activePlugins.clear();
        return c_error(LIBLO_ERROR_INVALID_ARGS, "The operation results in an invalid active plugins list.");
    }

    //Now save changes.
    try {
        gh->activePlugins.Save(*gh);
    } catch (error& e) {
        gh->activePlugins.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

/* Checks to see if the given plugin is active. */
LIBLO unsigned int lo_get_plugin_active(lo_game_handle gh, const char * const plugin, bool * const result) {
    if (gh == NULL || plugin == NULL || result == NULL)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    Plugin pluginObj(plugin);

    //Update cache if necessary.
    try {
        if (gh->activePlugins.HasChanged(*gh))
            gh->activePlugins.Load(*gh);
    } catch (error& e) {
        return c_error(e);
    }

    if (gh->activePlugins.find(pluginObj) == gh->activePlugins.end())
        *result = false;
    else
        *result = true;

    return LIBLO_OK;
}

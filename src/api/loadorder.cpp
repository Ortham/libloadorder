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

#include "libloadorder/loadorder.h"
#include "../backend/game.h"
#include "../backend/helpers.h"
#include "../backend/error.h"

using namespace std;
using namespace liblo;
namespace fs = boost::filesystem;

/*------------------------------
   Load Order Functions
   ------------------------------*/

/* Returns which method the game uses for the load order. */
LIBLO unsigned int lo_get_load_order_method(lo_game_handle gh, unsigned int * const method) {
    if (gh == nullptr || method == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    *method = gh->LoadOrderMethod();

    return LIBLO_OK;
}

/* Outputs a list of the plugins installed in the data path specified when the DB was
   created in load order, with the number of plugins given by numPlugins. */
LIBLO unsigned int lo_get_load_order(lo_game_handle gh, char *** const plugins, size_t * const numPlugins) {
    if (gh == nullptr || plugins == nullptr || numPlugins == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    unsigned int successRetCode = LIBLO_OK;

    //Free memory if in use.
    if (gh->extStringArray != nullptr) {
        for (size_t i = 0; i < gh->extStringArraySize; i++)
            delete[] gh->extStringArray[i];
        delete[] gh->extStringArray;
        gh->extStringArray = nullptr;
        gh->extStringArraySize = 0;
    }

    //Update cache if necessary.
    try {
        if (gh->loadOrder.HasChanged(*gh))
            gh->loadOrder.load(*gh);
    }
    catch (error& e) {
        return c_error(e);
    }

    //Exit now if load order is empty.
    vector<string> loadOrder(gh->loadOrder.getLoadOrder());
    if (loadOrder.empty())
        return LIBLO_OK;

    //Allocate memory.
    gh->extStringArraySize = loadOrder.size();
    try {
        gh->extStringArray = new char*[gh->extStringArraySize];
        for (size_t i = 0; i < gh->extStringArraySize; i++)
            gh->extStringArray[i] = ToNewCString(loadOrder[i]);
    }
    catch (bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
    }

    //Set outputs.
    *plugins = gh->extStringArray;
    *numPlugins = gh->extStringArraySize;

    return successRetCode;
}

/* Sets the load order to the given plugins list of length numPlugins.
   Then scans the Data directory and appends any other plugins not included in the
   array passed to the function. */
LIBLO unsigned int lo_set_load_order(lo_game_handle gh, const char * const * const plugins, const size_t numPlugins) {
    if (gh == nullptr || plugins == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");
    if (numPlugins == 0)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Zero-length plugin array passed.");

    //Put input into loadOrder object.
    vector<string> loadOrder;
    gh->loadOrder.clear();
    for (size_t i = 0; i < numPlugins; i++) {
        loadOrder.push_back(plugins[i]);
    }

    //Check to see if basic rules are being obeyed.
    try {
        gh->loadOrder.setLoadOrder(loadOrder, *gh);
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(LIBLO_ERROR_INVALID_ARGS, string("Invalid load order supplied. Details: ") + e.what());
    }

    //Now save changes.
    try {
        gh->loadOrder.save(*gh);
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

/* Gets the load order of the specified plugin, giving it as index. The first position
   in the load order is 0. */
LIBLO unsigned int lo_get_plugin_position(lo_game_handle gh, const char * const plugin, size_t * const index) {
    if (gh == nullptr || plugin == nullptr || index == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    unsigned int successRetCode = LIBLO_OK;

    //Update cache if necessary.
    try {
        if (gh->loadOrder.HasChanged(*gh))
            gh->loadOrder.load(*gh);
    }
    catch (error& e) {
        return c_error(e);
    }

    //Find plugin pos.
    size_t pos = gh->loadOrder.getPosition(plugin);
    if (pos == gh->loadOrder.getLoadOrder().size())
        return c_error(LIBLO_ERROR_FILE_NOT_FOUND, "\"" + string(plugin) + "\" cannot be found.");

    *index = pos;

    return successRetCode;
}

/* Sets the load order of the specified plugin, removing it from its current position
   if it has one. The first position in the load order is 0. If the index specified is
   greater than the number of plugins in the load order, the plugin will be inserted at
   the end of the load order. */
LIBLO unsigned int lo_set_plugin_position(lo_game_handle gh, const char * const plugin, const size_t index) {
    if (gh == nullptr || plugin == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    unsigned int successRetCode = LIBLO_OK;

    //Update cache if necessary.
    try {
        if (gh->loadOrder.HasChanged(*gh)) {
            gh->loadOrder.load(*gh);
        }
    }
    catch (error& e) {
        return c_error(e);
    }

    //Check that new load order is valid.
    try {
        gh->loadOrder.setPosition(plugin, index, *gh);
    }
    catch (error& e) {
        return c_error(LIBLO_ERROR_INVALID_ARGS, string("The operation results in an invalid load order. Details: ") + e.what());
    }

    //Now save changes.
    try {
        gh->loadOrder.save(*gh);
    }
    catch (error& e) {
        gh->loadOrder.clear();
        return c_error(e);
    }

    return LIBLO_OK;
}

/* Gets the plugin filename is at the specified load order position. The first position
   in the load order is 0. */
LIBLO unsigned int lo_get_indexed_plugin(lo_game_handle gh, const size_t index, char ** const plugin) {
    if (gh == nullptr || plugin == nullptr)
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Null pointer passed.");

    unsigned int successRetCode = LIBLO_OK;

    //Free memory if in use.
    delete[] gh->extString;
    gh->extString = nullptr;

    //Update cache if necessary.
    try {
        if (gh->loadOrder.HasChanged(*gh))
            gh->loadOrder.load(*gh);
    }
    catch (error& e) {
        return c_error(e);
    }

    //Allocate memory.
    try {
        gh->extString = ToNewCString(gh->loadOrder.getPluginAtPosition(index));
    }
    catch (bad_alloc& e) {
        return c_error(LIBLO_ERROR_NO_MEM, e.what());
    }
    catch (exception&) {
        return c_error(LIBLO_ERROR_INVALID_ARGS, "Index given is equal to or larger than the size of the load order.");
    }

    //Set output.
    *plugin = gh->extString;

    return successRetCode;
}

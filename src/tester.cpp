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

#include "api/libloadorder.h"
#include "backend/streams.h"

#include <iostream>
#include <boost/filesystem.hpp>

using std::endl;

int main() {
    unsigned int vMajor, vMinor, vPatch;

    lo_game_handle db;
    const char * gamePath = "C:/Program Files (x86)/Steam/steamapps/common/skyrim";
    unsigned int game = LIBLO_GAME_TES5;
    unsigned int ret;

    unsigned int loMethod;
    const char * master = "Skyrim.esm";
    const char * plugin = "Unofficial Skyrim Patch.esp";
    char ** loadOrder;
    size_t len;
    size_t index;
    char * outPlugin;
    const char * error;

    char ** activePlugins;
    bool active;

    liblo::ofstream out(boost::filesystem::path("libloadorder-tester.txt"));
    if (!out.good()) {
        std::cout << "File could not be opened for reading.";
        return 1;
    }

    out << "TESTING lo_is_compatible(...)" << endl;
    bool b = lo_is_compatible(5, 0, 0);
    if (b)
        out << '\t' << "library is compatible." << endl;
    else {
        out << '\t' << "library is incompatible." << endl;
        return 0;
    }

    out << "TESTING lo_get_version(...)" << endl;
    lo_get_version(&vMajor, &vMinor, &vPatch);
    out << '\t' << "Version: " << vMajor << '.' << vMinor << '.' << vPatch << endl;

    out << "TESTING lo_create_handle(...)" << endl;
    ret = lo_create_handle(&db, game, gamePath, NULL);
    if (ret != LIBLO_OK)
        out << '\t' << "lo_create_handle(...) failed. Error: " << ret << endl;
    else {
        out << "TESTING lo_set_game_master(...)" << endl;
        ret = lo_set_game_master(db, master);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_set_game_master(...) failed. Error: " << ret << endl;
        else
            out << '\t' << "lo_set_game_master(...) successful." << endl;

        out << "TESTING lo_get_load_order_method(...)" << endl;
        ret = lo_get_load_order_method(db, &loMethod);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_get_load_order_method(...) failed. Error: " << ret << endl;
        else
            out << '\t' << "Load Order Method: " << loMethod << endl;

        out << "TESTING lo_get_load_order(...)" << endl;
        ret = lo_get_load_order(db, &loadOrder, &len);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_get_load_order(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "List size: " << len << endl;
            for (size_t i = 0; i < len; i++) {
                out << '\t' << '\t' << i << " : " << loadOrder[i] << endl;
            }
        }

        out << "TESTING lo_set_load_order(...)" << endl;
        ret = lo_set_load_order(db, loadOrder, len);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_set_load_order(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "List size: " << len << endl;
            for (size_t i = 0; i < len; i++) {
                out << '\t' << '\t' << i << " : " << loadOrder[i] << endl;
            }
        }

        out << "TESTING lo_get_plugin_position(...)" << endl;
        ret = lo_get_plugin_position(db, plugin, &index);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_get_plugin_position(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "\"" << plugin << "\" position: " << index << endl;
        }

        out << "TESTING lo_set_plugin_position(...)" << endl;
        len = 1;
        ret = lo_set_plugin_position(db, plugin, index);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_set_plugin_position(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "\"" << plugin << "\" set position: " << index << endl;
        }

        index++;

        out << "TESTING lo_get_indexed_plugin(...)" << endl;
        len = 10;
        ret = lo_get_indexed_plugin(db, index, &outPlugin);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_get_indexed_plugin(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "Plugin at position " << index << " : " << outPlugin << endl;
        }

        out << "TESTING lo_get_active_plugins(...)" << endl;
        ret = lo_get_active_plugins(db, &activePlugins, &len);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_get_active_plugins(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "List size: " << len << endl;
            for (size_t i = 0; i < len; i++) {
                out << '\t' << '\t' << i << " : " << activePlugins[i] << endl;
            }
        }

        out << "TESTING lo_set_active_plugins(...)" << endl;
        ret = lo_set_active_plugins(db, activePlugins, len);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_set_active_plugins(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "List size: " << len << endl;
            for (size_t i = 0; i < len; i++) {
                out << '\t' << '\t' << i << " : " << activePlugins[i] << endl;
            }
        }

        out << "TESTING lo_get_plugin_active(...)" << endl;
        ret = lo_get_plugin_active(db, plugin, &active);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_get_plugin_active(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "\"" << plugin << "\" active status: " << active << endl;
        }

        out << "TESTING lo_set_plugin_active(...)" << endl;
        ret = lo_set_plugin_active(db, plugin, !active);
        if (ret != LIBLO_OK) {
            ret = lo_get_error_message(&error);
            if (ret != LIBLO_OK)
                out << '\t' << "lo_get_error_message(...) failed. Error: " << ret << endl;
            out << '\t' << "lo_set_plugin_active(...) failed. Error: " << error << endl;
        }
        else {
            out << '\t' << "\"" << plugin << "\" active status: " << active << endl;
        }

        out << "TESTING lo_get_error_message(...)" << endl;
        ret = lo_set_plugin_active(db, nullptr, !active);
        if (ret != LIBLO_OK) {
            ret = lo_get_error_message(&error);
            if (ret != LIBLO_OK)
                out << '\t' << "lo_get_error_message(...) failed. Error: " << ret << endl;
            else
                out << '\t' << "lo_set_plugin_active(...) failed. Error: " << error << endl;
        }
        else {
            out << '\t' << "\"" << plugin << "\" active status: " << active << endl;
        }
        lo_cleanup();

        out << "TESTING lo_destroy_handle(...)" << endl;
        lo_destroy_handle(db);
        out << "lo_destroy_handle(...) successful." << endl;
    }

    out.close();
    return 0;
}

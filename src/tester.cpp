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
#include "tester-interface.h"

#include <iostream>
#include <stdint.h>
#include <fstream>

using std::endl;

int main() {
    unsigned int vMajor, vMinor, vPatch;

    lo_game_handle db;
    const char * gamePath = reinterpret_cast<const char *>("C:/Program Files (x86)/Steam/steamapps/common/oblivion");
    unsigned int game = LIBLO_GAME_TES4;
    unsigned int ret;

    unsigned int loMethod;
    const char * master = reinterpret_cast<const char *>("Oblivion.esm");
    const char * plugin = reinterpret_cast<const char *>("Unofficial Oblivion Patch.esp");
    char ** loadOrder;
    size_t len;
    size_t index;
    char * outPlugin;

    char ** activePlugins;
    bool active;

    std::ofstream out("libloadorder-tester.txt");
    if (!out.good()){
        std::cout << "File could not be opened for reading.";
        return 1;
    }

    //First test the library interface directly.


    out << "TESTING C LIBRARY INTERFACE" << endl;

    out << "TESTING lo_is_compatible(...)" << endl;
    bool b = lo_is_compatible(2,0,0);
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
    ret = lo_create_handle(&db, game, gamePath);
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
            for (size_t i=0; i<len; i++) {
                out << '\t' << '\t' << i << " : " << loadOrder[i] << endl;
            }
        }

        out << "TESTING lo_set_load_order(...)" << endl;
        ret = lo_set_load_order(db, loadOrder, len);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_set_load_order(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "List size: " << len << endl;
            for (size_t i=0; i<len; i++) {
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
            for (size_t i=0; i<len; i++) {
                out << '\t' << '\t' << i << " : " << activePlugins[i] << endl;
            }
        }

        out << "TESTING lo_set_active_plugins(...)" << endl;
        ret = lo_set_active_plugins(db, activePlugins, len);
        if (ret != LIBLO_OK)
            out << '\t' << "lo_set_active_plugins(...) failed. Error: " << ret << endl;
        else {
            out << '\t' << "List size: " << len << endl;
            for (size_t i=0; i<len; i++) {
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
            ret = lo_get_error_message(&outPlugin);
            if (ret != LIBLO_OK)
                out << '\t' << "lo_get_error_message(...) failed. Error: " << ret << endl;
            out << '\t' << "lo_set_plugin_active(...) failed. Error: " << outPlugin << endl;
        } else {
            out << '\t' << "\"" << plugin << "\" active status: " << active << endl;
        }

        out << "TESTING GetLastErrorDetails(...)" << endl;
        ret = lo_set_plugin_active(db, NULL, !active);
        if (ret != LIBLO_OK) {
            ret = lo_get_error_message(&outPlugin);
            if (ret != LIBLO_OK)
                out << '\t' << "lo_get_error_message(...) failed. Error: " << ret << endl;
            else
                out << '\t' << "lo_set_plugin_active(...) failed. Error: " << outPlugin << endl;
        } else {
            out << '\t' << "\"" << plugin << "\" active status: " << active << endl;
        }
        lo_cleanup();

        out << "TESTING lo_destroy_handle(...)" << endl;
        lo_destroy_handle(db);
        out << "lo_destroy_handle(...) successful." << endl;
    }

    //Now let's test the C++ wrapper.
    std::string gamePathStr = "C:/Program Files (x86)/Steam/steamapps/common/oblivion";
    std::string masterStr = "Oblivion.esm";
    std::string pluginStr = "Unofficial Oblivion Patch.esp";
    std::vector<std::string> vec;
    std::set<std::string> unord_set;


    out << "TESTING C++ WRAPPER INTERFACE" << endl;

    out << "TESTING IsCompatible(...)" << endl;
    if (tester::liblo::IsCompatible(2, 0, 0))
        out << '\t' << "library is compatible." << endl;
    else
        out << '\t' << "library is incompatible." << endl;

    out << "TESTING GetVersionNums(...)" << endl;
    tester::liblo::GetVersionNums(vMajor, vMinor, vPatch);
    out << '\t' << "Version: " << vMajor << '.' << vMinor << '.' << vPatch << endl;

    try {
        out << "TESTING GameHandle(...)" << endl;
        tester::liblo::GameHandle gh(game, gamePathStr);
        out << '\t' << "~GameHandle(...) successful." << endl;

        out << "TESTING SetGameMaster(...)" << endl;
        gh.SetGameMaster(masterStr);
        out << '\t' << "~SetGameMaster(...) successful." << endl;

        out << "TESTING LoadOrderMethod(...)" << endl;
        loMethod = gh.LoadOrderMethod();
        out << '\t' << "Load Order Method: " << loMethod << endl;

        out << "TESTING LoadOrder(...) (getter)" << endl;
        vec = gh.LoadOrder();
        out << '\t' << "List size: " << vec.size() << endl;
        for (size_t i=0, max=vec.size(); i < max; i++)
            out << '\t' << '\t' << i << " : " << vec[i] << endl;

        out << "TESTING LoadOrder(...) (setter)" << endl;
        gh.LoadOrder(vec);
        out << '\t' << "List size: " << vec.size() << endl;
        for (size_t i=0, max=vec.size(); i < max; i++)
            out << '\t' << '\t' << i << " : " << vec[i] << endl;

        out << "TESTING PluginLoadOrder(...) (getter)" << endl;
        index = gh.PluginLoadOrder(pluginStr);
        out << '\t' << "Position of plugin \"" << pluginStr << "\": " << index << endl;

        out << "TESTING PluginLoadOrder(...) (setter)" << endl;
        gh.PluginLoadOrder(pluginStr, index);
        out << '\t' << "Position of plugin \"" << pluginStr << "\": " << index << endl;

        index++;
        out << "TESTING PluginAtIndex(...)" << endl;
        pluginStr = gh.PluginAtIndex(index);
        out << '\t' << "Plugin at position " << index << ": \"" << pluginStr << "\"" << endl;

        out << "TESTING ActivePlugins(...) (getter)" << endl;
        unord_set = gh.ActivePlugins();
        out << '\t' << "List size: " << unord_set.size() << endl;
        size_t i = 0;
        for (std::set<std::string>::iterator it=unord_set.begin(), endIt=unord_set.end(); it != endIt; ++it) {
            out << '\t' << '\t' << i << " : " << *it << endl;
            i++;
        }

        out << "TESTING ActivePlugins(...) (setter)" << endl;
        gh.ActivePlugins(unord_set);
        out << '\t' << "List size: " << unord_set.size() << endl;
        i = 0;
        for (std::set<std::string>::iterator it=unord_set.begin(), endIt=unord_set.end(); it != endIt; ++it) {
            out << '\t' << '\t' << i << " : " << *it << endl;
            i++;
        }

        out << "TESTING IsPluginActive(...)" << endl;
        active = gh.IsPluginActive(pluginStr);
        out << '\t' << "Plugin \"" << pluginStr << "\" is active: " << active << endl;

        active = !active;
        out << "TESTING SetPluginActiveStatus(...)" << endl;
        gh.SetPluginActiveStatus(pluginStr, active);
        out << '\t' << "Plugin \"" << pluginStr << "\" set to active status: " << active << endl;

        out << "TESTING ~GameHandle(...)" << endl;
        gh.~GameHandle();
        out << '\t' << "~GameHandle(...) successful." << endl;

    } catch (tester::liblo::exception& e) {
        out << '\t'<< "Exception thrown." << endl
            << '\t'<< '\t' << "Error code: " << e.code() << endl
            << '\t'<< '\t' << "Error message: " << e.what() << endl;
    }


    out.close();
    return 0;
}

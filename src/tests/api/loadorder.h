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

#ifndef __LIBLO_TEST_API_LOAD_ORDER__
#define __LIBLO_TEST_API_LOAD_ORDER__

#include "tests/fixtures.h"

#include <boost/filesystem.hpp>

TEST_F(OblivionOperationsTest, GetLoadOrderMethod) {
    unsigned int method;
    EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gh, &method));
    EXPECT_EQ(LIBLO_METHOD_TIMESTAMP, method);

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(gh, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, &method));
}

TEST_F(SkyrimOperationsTest, GetLoadOrderMethod) {
    unsigned int method;
    EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gh, &method));
    EXPECT_EQ(LIBLO_METHOD_TEXTFILE, method);

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(gh, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, &method));
}

TEST_F(OblivionOperationsTest, SetLoadOrder) {
    // Can't redistribute Oblivion.esm, but Nehrim.esm can be,
    // so use that for testing.
    char * plugins[1] = {
        "EnhancedWeather.esm"
    };
    size_t pluginsNum = 1;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, 0));

    // Test trying to set load order with non-Oblivion.esm without
    // first setting the game master.
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, 0));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, plugins, pluginsNum));

    // Now set game master and try again.
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));

    // Now test with more than one plugin.
    char * plugins2[] = {
        "EnhancedWeather.esm",
        "EnhancedWeather.esp"
    };
    pluginsNum = 2;
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins2, pluginsNum));

    // Now test with more than one plugin, where one doesn't exist.
    ASSERT_FALSE(boost::filesystem::exists("./game/Data/EnhancedWeather.esp.missing"));

    char * plugins3[] = {
        "EnhancedWeather.esm",
        "EnhancedWeather.esp.missing"
    };
    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_load_order(gh, plugins3, pluginsNum));
}

TEST_F(OblivionOperationsTest, GetLoadOrder) {
    char ** plugins;
    size_t pluginsNum;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, &plugins, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_load_order(gh, &plugins, &pluginsNum));
}

TEST_F(OblivionOperationsTest, SetPluginPosition) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "EnhancedWeather.esp", 1));
}

TEST_F(OblivionOperationsTest, GetPluginPosition) {
    size_t pos;
    EXPECT_EQ(LIBLO_OK, lo_get_plugin_position(gh, "EnhancedWeather.esm", &pos));
    EXPECT_EQ(0, pos);
}

TEST_F(OblivionOperationsTest, GetIndexedPlugin) {
    char * plugin;
    EXPECT_EQ(LIBLO_OK, lo_get_indexed_plugin(gh, 0, &plugin));
    EXPECT_STREQ("EnhancedWeather.esm", plugin);
}

#endif

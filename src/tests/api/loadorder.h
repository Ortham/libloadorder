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

#include <boost/algorithm/string.hpp>

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
    size_t pos;
    char * plugins[] = {
        "Blank.esm"
    };
    size_t pluginsNum = 1;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, 0));

    // Test trying to set load order with non-Oblivion.esm without
    // first setting the game master.
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, 0));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, plugins, pluginsNum));

    // Now set game master and try again.
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));

    // Check that load order was actually applied. Because a partial load order
    // was applied, the full load order may be invalid, so take that into
    // account.
    ASSERT_PRED1([](unsigned int i) {
        return i == LIBLO_OK || i == LIBLO_WARN_INVALID_LIST;
    }, lo_get_plugin_position(gh, "Blank.esm", &pos));
    EXPECT_EQ(0, pos);

    // Now test with more than one plugin.
    char * plugins2[] = {
        "Blank.esm",
        "Blank - Different.esm"
    };
    pluginsNum = 2;
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins2, pluginsNum));

    // Check that load order was actually applied. Because a partial load order
    // was applied, the full load order may be invalid, so take that into
    // account.
    ASSERT_PRED1([](unsigned int i) {
        return i == LIBLO_OK || i == LIBLO_WARN_INVALID_LIST;
    }, lo_get_plugin_position(gh, "Blank.esm", &pos));
    EXPECT_EQ(0, pos);
    ASSERT_PRED1([](unsigned int i) {
        return i == LIBLO_OK || i == LIBLO_WARN_INVALID_LIST;
    }, lo_get_plugin_position(gh, "Blank - Different.esm", &pos));
    EXPECT_EQ(1, pos);

    char * plugins3[] = {
        "Blank.esm",
        "Blank.esp.missing"
    };
    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_load_order(gh, plugins3, pluginsNum));
}

TEST_F(OblivionOperationsTest, GetLoadOrder) {
    char ** plugins;
    size_t pluginsNum;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, &plugins, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, NULL));

    EXPECT_EQ(LIBLO_WARN_INVALID_LIST, lo_get_load_order(gh, &plugins, &pluginsNum));

    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_get_load_order(gh, &plugins, &pluginsNum));
}

TEST_F(SkyrimOperationsTest, GetLoadOrder) {
    char ** plugins;
    size_t pluginsNum;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, &plugins, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_load_order(gh, &plugins, &pluginsNum));

    // Test that ghosted plugins get put into loadorder.txt correctly.
    std::vector<std::string> actualLines;
    std::string content;
    ASSERT_TRUE(boost::filesystem::exists(localPath / "loadorder.txt"));
    liblo::ifstream in(localPath / "loadorder.txt");
    while (in.good()) {
        std::string line;
        std::getline(in, line);
        actualLines.push_back(line);
    }
    in.close();

    EXPECT_EQ("Blank - Different.esm", actualLines[2]);
}

TEST_F(OblivionOperationsTest, SetPluginPosition) {
    // Load a plugin last.
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 100));
}

TEST_F(OblivionOperationsTest, GetPluginPosition) {
    size_t pos;
    EXPECT_EQ(LIBLO_WARN_INVALID_LIST, lo_get_plugin_position(gh, "Blank.esm", &pos));
    EXPECT_EQ(0, pos);

    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_get_plugin_position(gh, "Blank.esm", &pos));
    EXPECT_EQ(0, pos);
}

TEST_F(OblivionOperationsTest, GetIndexedPlugin) {
    char * plugin;
    EXPECT_EQ(LIBLO_WARN_INVALID_LIST, lo_get_indexed_plugin(gh, 0, &plugin));
    EXPECT_STREQ("Blank.esm", plugin);

    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_get_indexed_plugin(gh, 0, &plugin));
    EXPECT_STREQ("Blank.esm", plugin);
}

#endif

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

#ifndef __LIBLO_TEST_API_LOAD_ORDER__
#define __LIBLO_TEST_API_LOAD_ORDER__

#include "tests/fixtures.h"

#include <boost/algorithm/string.hpp>

TEST_F(OblivionOperationsTest, GetLoadOrderMethod) {
    unsigned int method = 0;
    EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gh, &method));
    EXPECT_EQ(LIBLO_METHOD_TIMESTAMP, method);

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(gh, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, &method));
}

TEST_F(SkyrimOperationsTest, GetLoadOrderMethod) {
    unsigned int method = 0;
    EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gh, &method));
    EXPECT_EQ(LIBLO_METHOD_TEXTFILE, method);

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(gh, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, &method));
}

TEST_F(OblivionOperationsTest, SetLoadOrder_MissingPlugin) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));

    // Try setting the load order with a missing plugin.
    const char * missingPlugins[] = {
        "Blank.esm",
        "Blank.missing.esp"
    };
    size_t pluginsNum = 2;

    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, missingPlugins, pluginsNum));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetLoadOrder_DuplicatePlugin) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));

    // Try setting the load order with a duplicate entry.
    const char * dupPlugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank.esp"
    };
    size_t pluginsNum = 3;

    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, dupPlugins, pluginsNum));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetLoadOrder_WrongGameMaster) {
    const char * plugins[] = {
        "Blank.esm"
    };
    size_t pluginsNum = 1;

    // Test trying to set load order with non-Oblivion.esm without
    // first setting the game master.
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));
}

TEST_F(OblivionOperationsTest, SetLoadOrder_BadMasterOrder) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));

    const char * badMasterOrderPlugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank - Different.esm"
    };
    size_t pluginsNum = 3;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, badMasterOrderPlugins, pluginsNum));
    AssertInitialState();

    const char * badMasterOrderPlugins2[] = {
        "Blank.esm",
        "Blank - Different Master Dependent.esm",
        "Blank - Different.esm",
    };
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, badMasterOrderPlugins2, pluginsNum));
}

TEST_F(OblivionOperationsTest, SetLoadOrder_NullInputs) {
    const char * plugins[] = {
        "Blank.esm"
    };
    size_t pluginsNum = 1;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(NULL, plugins, pluginsNum));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, pluginsNum));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, 0));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, plugins, 0));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetLoadOrder_NonPluginFile) {
    const char * plugins[] = {
        "Blank.esm",
        "NotAPlugin.esm"
    };
    size_t pluginsNum = 2;

    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetLoadOrder_Valid) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));

    const char * plugins[] = {
        "Blank.esm"
    };
    size_t pluginsNum = 1;

    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));
    EXPECT_EQ(0, CheckPluginPosition("Blank.esm"));

    // Now test with more than one plugin.
    const char * plugins2[] = {
        "Blank.esm",
        "Blank - Different.esm"
    };
    pluginsNum = 2;
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins2, pluginsNum));
    EXPECT_EQ(0, CheckPluginPosition("Blank.esm"));
    EXPECT_EQ(1, CheckPluginPosition("Blank - Different.esm"));
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_MissingPlugin) {
    const char * missingPlugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.missing.esp"
    };
    size_t pluginsNum = 3;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, missingPlugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_DuplicatePlugin) {
    // Try setting the load order with a duplicate entry.
    const char * dupPlugins[] = {
        "Skyrim.esm",
        "Blank.esp",
        "Blank.esp"
    };
    size_t pluginsNum = 3;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, dupPlugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_NoGameMaster) {
    const char * noGameMasterPlugins[] = {
        "Blank.esm",
        "Blank - Different.esm"
    };
    size_t pluginsNum = 2;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, noGameMasterPlugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_BadMasterOrder) {
    const char * badMasterOrderPlugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.esp",
        "Blank - Different.esm"
    };
    size_t pluginsNum = 4;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, badMasterOrderPlugins, pluginsNum));
    AssertInitialState();

    const char * badMasterOrderPlugins2[] = {
        "Skyrim.esm",
        "Blank - Master Dependent.esm",
        "Blank.esm",
    };
    pluginsNum = 3;
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, badMasterOrderPlugins2, pluginsNum));
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_NullInputs) {
    const char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank - Different.esm"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(NULL, plugins, pluginsNum));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, pluginsNum));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, NULL, 0));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_NonPluginFile) {
    const char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "NotAPlugin.esm"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetLoadOrder_Valid) {
    const char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank - Different.esm"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));
    EXPECT_EQ(1, CheckPluginPosition("Blank.esm"));
    EXPECT_EQ(2, CheckPluginPosition("Blank - Different.esm"));
}

TEST_F(OblivionOperationsTest, GetLoadOrder) {
    char ** plugins = {0};
    size_t pluginsNum;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(NULL, &plugins, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, &plugins, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_load_order(gh, &plugins, &pluginsNum));
}

TEST_F(SkyrimOperationsTest, GetLoadOrder) {
    char ** plugins = {0};
    size_t pluginsNum;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(NULL, &plugins, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, &pluginsNum));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, &plugins, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gh, NULL, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_load_order(gh, &plugins, &pluginsNum));

    // Test that ghosted plugins get put into loadorder.txt correctly.
    std::vector<std::string> actualLines;
    std::string content;
    ASSERT_TRUE(boost::filesystem::exists(localPath / "loadorder.txt"));
    boost::filesystem::ifstream in(localPath / "loadorder.txt");
    while (in.good()) {
        std::string line;
        std::getline(in, line);
        actualLines.push_back(line);
    }
    in.close();

    EXPECT_EQ("Blank - Different.esm", actualLines[2]);
}

TEST_F(OblivionOperationsTest, SetPluginPosition_NullInputs) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(NULL, "Blank - Plugin Dependent.esp", 100));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, NULL, 100));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetPluginPosition_PluginAmongstMasters) {
    // Try loading a plugin in the masters block.
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, "Blank.esp", 1));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetPluginPosition_PluginBeforeGameMaster) {
    // Try loading a plugin first.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank.esm", 0));
}

TEST_F(OblivionOperationsTest, SetPluginPosition_PluginBeforeItsMaster) {
    // Try loading a plugin before its master.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 4));
}

TEST_F(OblivionOperationsTest, SetPluginPosition_NonPluginFile) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, "NotAPlugin.esm", 100));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetPluginPosition_Valid) {
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));

    // Set a specific position.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 5));
    EXPECT_EQ(5, CheckPluginPosition("Blank - Plugin Dependent.esp"));

    // Load a plugin last.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 100));
    EXPECT_EQ(9, CheckPluginPosition("Blank - Plugin Dependent.esp"));
}

TEST_F(SkyrimOperationsTest, SetPluginPosition_NullInputs) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(NULL, "Blank - Plugin Dependent.esp", 100));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, NULL, 100));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetPluginPosition_PluginAmongstMasters) {
    // Try loading a plugin in the masters block.
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, "Blank.esp", 1));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetPluginPosition_PluginBeforeGameMaster) {
    // Try loading a plugin first.
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, "Blank.esm", 0));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetPluginPosition_PluginBeforeItsMaster) {
    // Try loading a plugin before its master.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 5));
}

TEST_F(SkyrimOperationsTest, SetPluginPosition_NonPluginFile) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gh, "NotAPlugin.esm", 100));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetPluginPosition_Valid) {
    // Set a specific position.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 6));
    EXPECT_EQ(6, CheckPluginPosition("Blank - Plugin Dependent.esp"));

    // Load a plugin last.
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "Blank - Plugin Dependent.esp", 100));
    EXPECT_EQ(10, CheckPluginPosition("Blank - Plugin Dependent.esp"));
}

TEST_F(OblivionOperationsTest, GetPluginPosition) {
    size_t pos = 0;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(NULL, "Blank.esp", &pos));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(gh, NULL, &pos));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(gh, "Blank.esp", NULL));
    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_get_plugin_position(gh, "NotAPlugin.esm", &pos));

    EXPECT_EQ(LIBLO_OK, lo_get_plugin_position(gh, "Blank.esp", &pos));
    EXPECT_EQ(4, pos);
}

TEST_F(SkyrimOperationsTest, GetPluginPosition) {
    size_t pos = 0;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(NULL, "Blank.esp", &pos));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(gh, NULL, &pos));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(gh, "Blank.esp", NULL));
    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_get_plugin_position(gh, "NotAPlugin.esm", &pos));

    EXPECT_EQ(LIBLO_OK, lo_get_plugin_position(gh, "Blank.esp", &pos));
    EXPECT_EQ(5, pos);
}

TEST_F(OblivionOperationsTest, GetIndexedPlugin) {
    char * plugin = nullptr;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(NULL, 0, &plugin));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(gh, 0, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_indexed_plugin(gh, 0, &plugin));
    EXPECT_STREQ("Blank.esm", plugin);

    plugin = nullptr;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(gh, 100, &plugin));
    EXPECT_EQ(nullptr, plugin);
}

TEST_F(SkyrimOperationsTest, GetIndexedPlugin) {
    char * plugin = nullptr;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(NULL, 0, &plugin));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(gh, 0, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_indexed_plugin(gh, 0, &plugin));
    EXPECT_STREQ("Skyrim.esm", plugin);

    plugin = nullptr;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(gh, 100, &plugin));
    EXPECT_EQ(nullptr, plugin);
}

#endif

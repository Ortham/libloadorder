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

#ifndef __LIBLO_TEST_API_ACTIVE_PLUGINS__
#define __LIBLO_TEST_API_ACTIVE_PLUGINS__

#include "tests/fixtures.h"

TEST_F(OblivionOperationsTest, GetActivePlugins) {
    char ** plugins = {0};
    size_t numPlugins = 0;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(NULL, &plugins, &numPlugins));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(gh, NULL, &numPlugins));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(gh, &plugins, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_active_plugins(gh, &plugins, &numPlugins));
    EXPECT_EQ(1, numPlugins);
    EXPECT_STREQ("Blank.esm", plugins[0]);
}

TEST_F(SkyrimOperationsTest, GetActivePlugins) {
    char ** plugins = {0};
    size_t numPlugins = 0;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(NULL, &plugins, &numPlugins));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(gh, NULL, &numPlugins));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(gh, &plugins, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_active_plugins(gh, &plugins, &numPlugins));
    EXPECT_EQ(2, numPlugins);

    auto pred = [numPlugins](const char * s, char ** a) {
        for (size_t i = 0; i < numPlugins; ++i) {
            if (strcmp(a[i], s) == 0)
                return true;
        }
        return false;
    };

    EXPECT_PRED2(pred, "Blank.esm", plugins);
    EXPECT_PRED2(pred, "Skyrim.esm", plugins);
}

TEST_F(OblivionOperationsTest, SetActivePlugins_NullInputs) {
    char * plugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(NULL, plugins, pluginsNum));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gh, NULL, pluginsNum));
    AssertInitialState();

    EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gh, plugins, 0));
    EXPECT_FALSE(CheckPluginActive("Blank.esm"));
}

TEST_F(OblivionOperationsTest, SetActivePlugins_NonPluginFile) {
    char * plugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp",
        "NotAPlugin.esm"
    };
    size_t pluginsNum = 4;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetActivePlugins_MissingPlugin) {
    char * plugins[] = {
        "Blank.esm",
        "Blank.missing.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_active_plugins(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetActivePlugins_Valid) {
    char * plugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gh, plugins, pluginsNum));
    EXPECT_TRUE(CheckPluginActive("Blank.esm"));
    EXPECT_TRUE(CheckPluginActive("Blank.esp"));
    EXPECT_TRUE(CheckPluginActive("Blank - Master Dependent.esp"));
}

TEST_F(SkyrimOperationsTest, SetActivePlugins_NullInputs) {
    char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 4;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(NULL, plugins, pluginsNum));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gh, NULL, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetActivePlugins_MissingPlugin) {
    char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.missing.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 4;

    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_active_plugins(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetActivePlugins_NonPluginFile) {
    char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp",
        "NotAPlugin.esm"
    };
    size_t pluginsNum = 5;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetActivePlugins_Valid) {
    char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 4;
    EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gh, plugins, pluginsNum));
    EXPECT_TRUE(CheckPluginActive("Blank.esm"));
    EXPECT_TRUE(CheckPluginActive("Blank.esp"));
    EXPECT_TRUE(CheckPluginActive("Blank - Master Dependent.esp"));
}

TEST_F(OblivionOperationsTest, GetPluginActive) {
    bool isActive = true;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(NULL, "Blank - Master Dependent.esp", &isActive));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(gh, NULL, &isActive));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(gh, "Blank - Master Dependent.esp", NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gh, "NotAPlugin.esm", &isActive));
    EXPECT_FALSE(isActive);

    isActive = true;

    EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gh, "Blank - Master Dependent.esp", &isActive));
    EXPECT_FALSE(isActive);
}

TEST_F(SkyrimOperationsTest, GetPluginActive) {
    bool isActive = true;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(NULL, "Blank - Master Dependent.esp", &isActive));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(gh, NULL, &isActive));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(gh, "Blank - Master Dependent.esp", NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gh, "NotAPlugin.esm", &isActive));
    EXPECT_FALSE(isActive);

    isActive = true;
    EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gh, "Blank - Master Dependent.esp", &isActive));
    EXPECT_FALSE(isActive);
}

TEST_F(OblivionOperationsTest, SetPluginActive) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(NULL, "Blank - Different Master Dependent.esp", true));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gh, NULL, true));
    AssertInitialState();

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gh, "NotAPlugin.esm", true));
    AssertInitialState();

    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_plugin_active(gh, "Blank.missing.esp", true));
    AssertInitialState();

    EXPECT_FALSE(CheckPluginActive("Blank - Different Master Dependent.esp"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank - Different Master Dependent.esp", true));
    EXPECT_TRUE(CheckPluginActive("Blank - Different Master Dependent.esp"));

    EXPECT_TRUE(CheckPluginActive("Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank.esm", false));
    EXPECT_FALSE(CheckPluginActive("Blank.esm"));
}

TEST_F(SkyrimOperationsTest, SetPluginActive) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(NULL, "Blank - Different Master Dependent.esp", true));
    AssertInitialState();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gh, NULL, true));
    AssertInitialState();

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gh, "NotAPlugin.esm", true));
    AssertInitialState();

    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_plugin_active(gh, "Blank.missing.esp", true));
    AssertInitialState();

    EXPECT_FALSE(CheckPluginActive("Blank - Different Master Dependent.esp"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank - Different Master Dependent.esp", true));
    EXPECT_TRUE(CheckPluginActive("Blank - Different Master Dependent.esp"));

    EXPECT_TRUE(CheckPluginActive("Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank.esm", false));
    EXPECT_FALSE(CheckPluginActive("Blank.esm"));
}

#endif

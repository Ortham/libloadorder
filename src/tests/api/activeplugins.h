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

#ifndef __LIBLO_TEST_API_ACTIVE_PLUGINS__
#define __LIBLO_TEST_API_ACTIVE_PLUGINS__

#include "tests/fixtures.h"
#include "tests/api/CApiGameOperationTest.h"

namespace liblo {
    namespace test {
        char ** begin(char ** cArray) {
            return cArray;
        }

        char ** end(char ** cArray, size_t cArraySize) {
            return cArray + cArraySize;
        }

        class lo_get_active_plugins_test : public CApiGameOperationTest {
        protected:
            lo_get_active_plugins_test() :
                plugins(nullptr),
                numPlugins(0) {
                // Write out an active plugins file.
                boost::filesystem::ofstream out(activePluginsFilePath);
                out << getActivePluginsFileLinePrefix() << blankEsm;
                out.close();
            }

            char ** plugins;
            size_t numPlugins;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_get_active_plugins_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_get_active_plugins_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(NULL, &plugins, &numPlugins));
        }

        TEST_P(lo_get_active_plugins_test, shouldFailIfPluginsPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(gameHandle, NULL, &numPlugins));
        }

        TEST_P(lo_get_active_plugins_test, shouldFailIfPluginsSizeIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_active_plugins(gameHandle, &plugins, NULL));
        }

        TEST_P(lo_get_active_plugins_test, outputShouldMatchExpectedActivePlugins) {
            EXPECT_EQ(LIBLO_OK, lo_get_active_plugins(gameHandle, &plugins, &numPlugins));

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE) {
                EXPECT_EQ(2, numPlugins);
                EXPECT_EQ(1, std::count(begin(plugins), end(plugins, numPlugins), masterFile));
                EXPECT_EQ(1, std::count(begin(plugins), end(plugins, numPlugins), blankEsm));
            }
            else {
                ASSERT_EQ(1, numPlugins);
            }
        }
    }
}

TEST_F(OblivionOperationsTest, SetActivePlugins_NullInputs) {
    const char * plugins[] = {
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
    const char * plugins[] = {
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
    const char * plugins[] = {
        "Blank.esm",
        "Blank.missing.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 3;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(OblivionOperationsTest, SetActivePlugins_Valid) {
    const char * plugins[] = {
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
    const char * plugins[] = {
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
    const char * plugins[] = {
        "Skyrim.esm",
        "Blank.esm",
        "Blank.missing.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 4;

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gh, plugins, pluginsNum));
    AssertInitialState();
}

TEST_F(SkyrimOperationsTest, SetActivePlugins_NonPluginFile) {
    const char * plugins[] = {
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
    const char * plugins[] = {
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

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gh, "Blank.missing.esp", true));
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

    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gh, "Blank.missing.esp", true));
    AssertInitialState();

    EXPECT_FALSE(CheckPluginActive("Blank - Different Master Dependent.esp"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank - Different Master Dependent.esp", true));
    EXPECT_TRUE(CheckPluginActive("Blank - Different Master Dependent.esp"));

    EXPECT_TRUE(CheckPluginActive("Blank.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank.esm", false));
    EXPECT_FALSE(CheckPluginActive("Blank.esm"));
}

#endif

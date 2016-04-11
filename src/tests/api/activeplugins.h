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

#ifndef LIBLO_TEST_API_ACTIVE_PLUGINS
#define LIBLO_TEST_API_ACTIVE_PLUGINS

#include "tests/api/CApiGameOperationTest.h"

#include <array>

namespace liblo {
    namespace test {
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

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                EXPECT_EQ(2, numPlugins);
                EXPECT_EQ(1, std::count(begin(plugins), end(plugins, numPlugins), masterFile));
                EXPECT_EQ(1, std::count(begin(plugins), end(plugins, numPlugins), blankEsm));
            }
            else {
                ASSERT_EQ(1, numPlugins);
            }
        }

        class lo_set_active_plugins_test : public CApiGameOperationTest {
        protected:
            lo_set_active_plugins_test() {
                plugins[0] = masterFile.c_str();
                plugins[1] = blankEsm.c_str();
            }

            std::array<const char *, 2> plugins;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_set_active_plugins_test,
                                ::testing::Values(
                                    LIBLO_GAME_TES3,
                                    LIBLO_GAME_TES4,
                                    LIBLO_GAME_TES5,
                                    LIBLO_GAME_FO3,
                                    LIBLO_GAME_FNV,
                                    LIBLO_GAME_FO4));

        TEST_P(lo_set_active_plugins_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(NULL, plugins.data(), plugins.size()));
        }

        TEST_P(lo_set_active_plugins_test, shouldFailIfPluginsPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gameHandle, NULL, plugins.size()));
        }

        TEST_P(lo_set_active_plugins_test, shouldFailIfPluginsSizeIsZeroForTimestampBasedGamesAndFailOtherwise) {
            if (loadOrderMethod == LIBLO_METHOD_TIMESTAMP)
                EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gameHandle, plugins.data(), 0));
            else
                EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gameHandle, plugins.data(), 0));
        }

        TEST_P(lo_set_active_plugins_test, shouldFailIfAPluginIsInvalid) {
            plugins[1] = invalidPlugin.c_str();
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_active_plugins(gameHandle, plugins.data(), plugins.size()));
        }

        TEST_P(lo_set_active_plugins_test, shouldSucceedIfPluginsSizeIsNonZero) {
            EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gameHandle, plugins.data(), plugins.size()));
        }

        class lo_get_plugin_active_test : public CApiGameOperationTest {
        protected:
            lo_get_plugin_active_test() : isActive(false) {}

            bool isActive;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_get_plugin_active_test,
                                ::testing::Values(
                                    LIBLO_GAME_TES3,
                                    LIBLO_GAME_TES4,
                                    LIBLO_GAME_TES5,
                                    LIBLO_GAME_FO3,
                                    LIBLO_GAME_FNV,
                                    LIBLO_GAME_FO4));

        TEST_P(lo_get_plugin_active_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(NULL, blankEsm.c_str(), &isActive));
        }

        TEST_P(lo_get_plugin_active_test, shouldFailIfPluginIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(gameHandle, NULL, &isActive));
        }

        TEST_P(lo_get_plugin_active_test, shouldFailIfOutputPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_active(gameHandle, blankEsm.c_str(), NULL));
        }

        TEST_P(lo_get_plugin_active_test, shouldOutputFalseForBlankEsm) {
            EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gameHandle, blankEsm.c_str(), &isActive));
            EXPECT_FALSE(isActive);
        }

        TEST_P(lo_get_plugin_active_test, shouldOutputTrueForBlankEsm) {
            // Write out an active plugins file.
            boost::filesystem::ofstream out(activePluginsFilePath);
            out << getActivePluginsFileLinePrefix() << masterFile;
            out.close();

            EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gameHandle, masterFile.c_str(), &isActive));
            EXPECT_TRUE(isActive);
        }

        class lo_set_plugin_active_test : public CApiGameOperationTest {};

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_set_plugin_active_test,
                                ::testing::Values(
                                    LIBLO_GAME_TES3,
                                    LIBLO_GAME_TES4,
                                    LIBLO_GAME_TES5,
                                    LIBLO_GAME_FO3,
                                    LIBLO_GAME_FNV,
                                    LIBLO_GAME_FO4));

        TEST_P(lo_set_plugin_active_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(NULL, blankEsm.c_str(), true));
        }

        TEST_P(lo_set_plugin_active_test, shouldFailIfPluginIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gameHandle, NULL, true));
        }

        TEST_P(lo_set_plugin_active_test, shouldSucceedIfActivatingAValidPlugin) {
            EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gameHandle, blankEsm.c_str(), true));
        }

        TEST_P(lo_set_plugin_active_test, shouldSucceedIfDeactivatingAValidPlugin) {
            EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gameHandle, blankEsm.c_str(), false));
        }

        TEST_P(lo_set_plugin_active_test, shouldFailIfActivatingAnInvalidPlugin) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_active(gameHandle, invalidPlugin.c_str(), true));
        }

        TEST_P(lo_set_plugin_active_test, shouldSucceedIfDeactivatingAnInvalidPlugin) {
            EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gameHandle, blankEsm.c_str(), false));
        }
    }
}

#endif

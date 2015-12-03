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

#ifndef LIBLO_TEST_API_LOAD_ORDER
#define LIBLO_TEST_API_LOAD_ORDER

#include "tests/api/CApiGameOperationTest.h"

#include <array>

namespace liblo {
    namespace test {
        class lo_get_load_order_method_test : public CApiGameOperationTest {
        protected:
            lo_get_load_order_method_test() : method(UINT_MAX) {}

            unsigned int method;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_get_load_order_method_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_get_load_order_method_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(NULL, &method));
        }

        TEST_P(lo_get_load_order_method_test, shouldFailIfOutputPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order_method(gameHandle, NULL));
        }

        TEST_P(lo_get_load_order_method_test, shouldOutputCorrectValue) {
            EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gameHandle, &method));
            EXPECT_EQ(loadOrderMethod, method);
        }

        class lo_set_load_order_test : public CApiGameOperationTest {
        protected:
            lo_set_load_order_test() {
                plugins[0] = masterFile.c_str();
                plugins[1] = blankEsm.c_str();
            }

            std::array<const char *, 2> plugins;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_set_load_order_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_set_load_order_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(NULL, plugins.data(), plugins.size()));
        }

        TEST_P(lo_set_load_order_test, shouldFailIfPluginsPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gameHandle, NULL, plugins.size()));
        }

        TEST_P(lo_set_load_order_test, shouldFailIfPluginsSizeIsZero) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gameHandle, plugins.data(), 0));
        }

        TEST_P(lo_set_load_order_test, shouldFailIfAPluginIsInvalid) {
            plugins[1] = invalidPlugin.c_str();
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_load_order(gameHandle, plugins.data(), plugins.size()));
        }

        TEST_P(lo_set_load_order_test, shouldSucceedIfPluginsSizeIsNonZero) {
            EXPECT_EQ(LIBLO_OK, lo_set_load_order(gameHandle, plugins.data(), plugins.size()));
        }

        class lo_get_load_order_test : public CApiGameOperationTest {
        protected:
            lo_get_load_order_test() :
                plugins(nullptr),
                numPlugins(0) {}

            virtual void SetUp() {
                CApiGameOperationTest::SetUp();

                const char * loadOrder[2] = {
                    masterFile.c_str(),
                    blankEsm.c_str(),
                };

                EXPECT_EQ(LIBLO_OK, lo_set_load_order(gameHandle, loadOrder, 2));
            }

            char ** plugins;
            size_t numPlugins;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_get_load_order_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_get_load_order_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(NULL, &plugins, &numPlugins));
        }

        TEST_P(lo_get_load_order_test, shouldFailIfPluginsPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gameHandle, NULL, &numPlugins));
        }

        TEST_P(lo_get_load_order_test, shouldFailIfPluginsSizeIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_load_order(gameHandle, &plugins, NULL));
        }

        TEST_P(lo_get_load_order_test, outputShouldMatchExpected) {
            EXPECT_EQ(LIBLO_OK, lo_get_load_order(gameHandle, &plugins, &numPlugins));

            ASSERT_EQ(11, numPlugins);
            EXPECT_EQ(masterFile, plugins[0]);
            EXPECT_EQ(blankEsm, plugins[1]);
        }

        class lo_set_plugin_position_test : public CApiGameOperationTest {};

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_set_plugin_position_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_set_plugin_position_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(NULL, masterFile.c_str(), 0));
        }

        TEST_P(lo_set_plugin_position_test, shouldFailIfPluginIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gameHandle, NULL, 1));
        }

        TEST_P(lo_set_plugin_position_test, shouldFailIfPluginIsInvalid) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_plugin_position(gameHandle, invalidPlugin.c_str(), 1));
        }

        TEST_P(lo_set_plugin_position_test, shouldSucceedWithValidInput) {
            EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gameHandle, masterFile.c_str(), 0));
        }

        class lo_get_plugin_position_test : public CApiGameOperationTest {
        protected:
            lo_get_plugin_position_test() : position(UINT_MAX) {}

            size_t position;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_get_plugin_position_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_get_plugin_position_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(NULL, masterFile.c_str(), &position));
        }

        TEST_P(lo_get_plugin_position_test, shouldFailIfPluginIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(gameHandle, NULL, &position));
        }

        TEST_P(lo_get_plugin_position_test, shouldFailIfPositionPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_plugin_position(gameHandle, masterFile.c_str(), NULL));
        }

        TEST_P(lo_get_plugin_position_test, shouldFailIfPluginIsNotFound) {
            EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_get_plugin_position(gameHandle, invalidPlugin.c_str(), &position));
        }

        TEST_P(lo_get_plugin_position_test, shouldSucceedWithValidInput) {
            ASSERT_EQ(LIBLO_OK, lo_set_plugin_position(gameHandle, masterFile.c_str(), 0));

            EXPECT_EQ(LIBLO_OK, lo_get_plugin_position(gameHandle, masterFile.c_str(), &position));
            EXPECT_EQ(0, position);
        }

        class lo_get_indexed_plugin_test : public CApiGameOperationTest {
        protected:
            lo_get_indexed_plugin_test() : plugin(nullptr) {}

            char * plugin;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_get_indexed_plugin_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_get_indexed_plugin_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(NULL, 0, &plugin));
        }

        TEST_P(lo_get_indexed_plugin_test, shouldFailIfPluginPointerIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(gameHandle, 0, NULL));
        }

        TEST_P(lo_get_indexed_plugin_test, shouldSucceedWithValidInput) {
            ASSERT_EQ(LIBLO_OK, lo_set_plugin_position(gameHandle, masterFile.c_str(), 0));

            EXPECT_EQ(LIBLO_OK, lo_get_indexed_plugin(gameHandle, 0, &plugin));
            EXPECT_EQ(masterFile, plugin);
        }

        TEST_P(lo_get_indexed_plugin_test, shouldFailIfIndexIsTooLarge) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_indexed_plugin(gameHandle, 100, &plugin));
        }
    }
}

#endif

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

#ifndef LIBLO_TEST_API_LIBLOADORDER
#define LIBLO_TEST_API_LIBLOADORDER

#include "tests/GameTest.h"
#include "tests/api/CApiGameOperationTest.h"
#include "tests/api/lo_create_handle_test.h"

#include <thread>

namespace liblo {
    namespace test {
        TEST(lo_get_version, shouldFailIfPassedNullMajorVersionParameter) {
            unsigned int vMinor = 0;
            unsigned int vPatch = 0;
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, &vMinor, &vPatch));

            ASSERT_NO_THROW(lo_cleanup());
        }

        TEST(lo_get_version, shouldFailIfPassedNullMinorVersionParameter) {
            unsigned int vMajor = 0;
            unsigned int vPatch = 0;
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(&vMajor, NULL, &vPatch));

            ASSERT_NO_THROW(lo_cleanup());
        }

        TEST(lo_get_version, shouldFailIfPassedNullPatchVersionParameter) {
            unsigned int vMajor = 0;
            unsigned int vMinor = 0;
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(&vMajor, &vMinor, NULL));

            ASSERT_NO_THROW(lo_cleanup());
        }

        TEST(lo_get_version, shouldSucceedIfPassedNonNullParameters) {
            unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
            EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));
        }

        TEST(lo_is_compatible, shouldReturnTrueIfMajorVersionIsEqual) {
            unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
            ASSERT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));

            EXPECT_TRUE(lo_is_compatible(vMajor, vMinor + 1, vPatch + 1));
        }

        TEST(lo_is_compatible, shouldReturnFalseIfMajorVersionIsNotEqual) {
            unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
            ASSERT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));

            EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor, vPatch));
        }

        TEST(lo_get_error_message, shouldFailIfPassedNullPointer) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

            ASSERT_NO_THROW(lo_cleanup());
        }

        TEST(lo_get_error_message, shouldOutputNullPointerIfNoErrorHasOccurred) {
            const char * error = nullptr;
            EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
            EXPECT_EQ(nullptr, error);
        }

        TEST(lo_get_error_message, shouldOutputCorrectErrorMessageIfErrorHasOccurred) {
            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

            const char * error = nullptr;
            EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
            EXPECT_STREQ("Null pointer passed.", error);

            ASSERT_NO_THROW(lo_cleanup());
        }

        TEST(lo_get_error_message, shouldOutputLastErrorMessageIfSuccessHasOccurredSinceLastError) {
            const char * error = nullptr;
            const char * lastError = nullptr;

            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));
            ASSERT_EQ(LIBLO_OK, lo_get_error_message(&error));
            ASSERT_NE(nullptr, error);
            lastError = error;

            EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
            ASSERT_EQ(lastError, error);
        }

        TEST(lo_get_error_message, errorMessagesShouldBeLocalToTheThreadTheErrorWasCausedIn) {
            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

            const char * error = nullptr;
            EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
            EXPECT_STREQ("Null pointer passed.", error);

            std::thread otherThread([]() {
                const char * error = nullptr;
                EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
                EXPECT_EQ(nullptr, error);
            });
            otherThread.join();

            EXPECT_NO_THROW(lo_cleanup());
        }

        TEST(lo_cleanup, shouldNotThrowIfNoErrorMessageToCleanUp) {
            EXPECT_NO_THROW(lo_cleanup());
        }

        TEST(lo_cleanup, shouldNotThrowIfThereIsAnErrorMessageToCleanUp) {
            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

            EXPECT_NO_THROW(lo_cleanup());
        }

        TEST(lo_cleanup, shouldFreeErrorMessageIfOneExists) {
            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

            EXPECT_NO_THROW(lo_cleanup());

            const char * error = nullptr;
            EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
            EXPECT_EQ(nullptr, error);
        }

        TEST(lo_cleanup, shouldNotCleanupErrorMessageInAnotherThread) {
            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

            std::thread otherThread([]() {
                EXPECT_NO_THROW(lo_cleanup());
            });
            otherThread.join();

            const char * error = nullptr;
            EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
            EXPECT_STREQ("Null pointer passed.", error);

            EXPECT_NO_THROW(lo_cleanup());
        }

        TEST(lo_destroy_handle, shouldNotThrowIfGameHandleIsNotCreated) {
            lo_game_handle gameHandle = nullptr;
            EXPECT_NO_THROW(lo_destroy_handle(gameHandle));
        }

        TEST(lo_destroy_handle, shouldNotThrowIfPassedNullPointer) {
            EXPECT_NO_THROW(lo_destroy_handle(NULL));
        }

        class lo_set_game_master_test : public CApiGameOperationTest {};

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
            lo_set_game_master_test,
            ::testing::Values(
                LIBLO_GAME_TES3,
                LIBLO_GAME_TES4,
                LIBLO_GAME_TES5,
                LIBLO_GAME_FO3,
                LIBLO_GAME_FNV,
                LIBLO_GAME_FO4));

        TEST_P(lo_set_game_master_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(NULL, blankEsm.c_str()));
        }

        TEST_P(lo_set_game_master_test, shouldFailIfPluginIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gameHandle, NULL));
        }

        TEST_P(lo_set_game_master_test, shouldSucceedIfPluginIsAnEmptyStringForTimestampBasedGamesAndFailOtherwise) {
            if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_FO4)
                EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gameHandle, ""));
            else
                EXPECT_EQ(LIBLO_OK, lo_set_game_master(gameHandle, ""));
        }

        TEST_P(lo_set_game_master_test, shouldSucceedIfPluginIsInvalidForTimestampBasedGamesAndFailOtherwise) {
            if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_FO4)
                EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gameHandle, invalidPlugin.c_str()));
            else
                EXPECT_EQ(LIBLO_OK, lo_set_game_master(gameHandle, invalidPlugin.c_str()));
        }

        TEST_P(lo_set_game_master_test, shouldSucceedIfPluginIsValidForTimestampBasedGamesAndFailOtherwise) {
            if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_FO4)
                EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gameHandle, blankEsm.c_str()));
            else
                EXPECT_EQ(LIBLO_OK, lo_set_game_master(gameHandle, blankEsm.c_str()));
        }

        TEST_P(lo_set_game_master_test, shouldSucceedIfPluginIsDefaultGameMasterForTimestampBasedGamesAndFailOtherwise) {
            if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_FO4)
                EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gameHandle, masterFile.c_str()));
            else
                EXPECT_EQ(LIBLO_OK, lo_set_game_master(gameHandle, masterFile.c_str()));
        }

        class lo_fix_plugin_lists_test : public CApiGameOperationTest {};

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
            lo_fix_plugin_lists_test,
            ::testing::Values(
                LIBLO_GAME_TES3,
                LIBLO_GAME_TES4,
                LIBLO_GAME_TES5,
                LIBLO_GAME_FO3,
                LIBLO_GAME_FNV,
                LIBLO_GAME_FO4));

        TEST_P(lo_fix_plugin_lists_test, shouldFailIfGameHandleIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_fix_plugin_lists(NULL));
        }

        TEST_P(lo_fix_plugin_lists_test, shouldSucceedIfGameHandleIsNotNull) {
            // Don't need to check its filesystem effects as that's handled by
            // lower-level tests.
            EXPECT_EQ(LIBLO_OK, lo_fix_plugin_lists(gameHandle));
        }
    }
}

#endif

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

#include "tests/fixtures.h"
#include "tests/GameTest.h"

#include <boost/algorithm/string.hpp>

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

        TEST(lo_destroy_handle, shouldNotThrowIfGameHandleIsNotCreated) {
            lo_game_handle gameHandle = nullptr;
            EXPECT_NO_THROW(lo_destroy_handle(gameHandle));
        }

        TEST(lo_destroy_handle, shouldNotThrowIfPassedNullPointer) {
            EXPECT_NO_THROW(lo_destroy_handle(NULL));
        }

        class lo_create_handle_test : public GameTest {
        protected:
            lo_create_handle_test() :
                invalidPath("./missing"),
                activePluginsFilePath(localPath / "plugins.txt"),
                loadOrderFilePath(localPath / "loadorder.txt"),
                blankEsm("Blank.esm"),
                blankDifferentEsm("Blank - Different.esm"),
                gameHandle(nullptr) {}

            inline virtual void SetUp() {
                GameTest::SetUp();

                ASSERT_FALSE(boost::filesystem::exists(invalidPath));

                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentEsm));

                // Make sure the game master file exists.
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / masterFile));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / masterFile));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / masterFile));
            }

            inline virtual void TearDown() {
                GameTest::TearDown();

                EXPECT_NO_THROW(lo_destroy_handle(gameHandle));

                ASSERT_NO_THROW(boost::filesystem::remove(activePluginsFilePath));
                ASSERT_NO_THROW(boost::filesystem::remove(loadOrderFilePath));

                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / masterFile));
            }

            const boost::filesystem::path invalidPath;

            const boost::filesystem::path activePluginsFilePath;
            const boost::filesystem::path loadOrderFilePath;

            const std::string blankEsm;
            const std::string blankDifferentEsm;

            lo_game_handle gameHandle;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                lo_create_handle_test,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(lo_create_handle_test, shouldFailIfHandleInputIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(NULL, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));
        }

        TEST_P(lo_create_handle_test, shouldFailIfGameTypeIsInvalid) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, UINT_MAX, gamePath.string().c_str(), localPath.string().c_str()));
        }

        TEST_P(lo_create_handle_test, shouldLeaveGameHandleUnchangedIfArgumentsAreInvalid) {
            ASSERT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, UINT_MAX, gamePath.string().c_str(), localPath.string().c_str()));
            EXPECT_EQ(nullptr, gameHandle);
        }

        TEST_P(lo_create_handle_test, shouldFailIfGamePathIsNull) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, GetParam(), NULL, localPath.string().c_str()));
        }

        TEST_P(lo_create_handle_test, shouldFailIfGamePathIsInvalid) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, GetParam(), invalidPath.string().c_str(), localPath.string().c_str()));
        }

        TEST_P(lo_create_handle_test, shouldFailIfLocalPathIsInvalid) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), invalidPath.string().c_str()));
        }

#ifdef _WIN32
        TEST_P(lo_create_handle_test, shouldNotFailDueToInvalidArgsWithNullLocalPathForWindowsOS) {
            // On Windows, passing a null local path causes libloadorder to
            // look up the game's local path in the Registry, and so its
            // success depends on external factors that should not be altered
            // for testing.
            EXPECT_NE(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), NULL));
        }
#else
        TEST_P(lo_create_handle_test, shouldFailWithNullLocalPathForNonWindowsOS) {
            EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), NULL));
        }
#endif

        TEST_P(lo_create_handle_test, shouldSucceedWithRelativePaths) {
            EXPECT_EQ(LIBLO_OK, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));
        }

        TEST_P(lo_create_handle_test, shouldSucceedWithAbsolutePaths) {
            boost::filesystem::path game = boost::filesystem::current_path() / gamePath;
            boost::filesystem::path local = boost::filesystem::current_path() / localPath;
            EXPECT_EQ(LIBLO_OK, lo_create_handle(&gameHandle, GetParam(), game.string().c_str(), local.string().c_str()));
        }

        TEST_P(lo_create_handle_test, shouldSetHandleToNonNullIfItSucceeds) {
            ASSERT_EQ(nullptr, gameHandle);
            ASSERT_EQ(LIBLO_OK, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));

            EXPECT_NE(nullptr, gameHandle);
        }

        TEST_P(lo_create_handle_test, shouldSucceedWithWarningIfFilesAreDesynchronisedForTextfileBasedGames) {
            if (GetParam() != LIBLO_GAME_TES5 && GetParam() != LIBLO_GAME_FO4)
                return;

            boost::filesystem::ofstream out(activePluginsFilePath);
            out << blankEsm;
            out.close();

            out.open(loadOrderFilePath);
            out << blankDifferentEsm;
            out.close();

            EXPECT_EQ(LIBLO_WARN_LO_MISMATCH, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));
        }

        class GameApiOperationTest : public lo_create_handle_test {
        protected:
            GameApiOperationTest() :
                invalidPlugin("NotAPlugin.esm") {}

            inline virtual void SetUp() {
                GameTest::SetUp();

                ASSERT_EQ(LIBLO_OK, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));
            }

            const std::string invalidPlugin;
        };

        class lo_set_game_master_test : public GameApiOperationTest {};

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

        class lo_fix_plugin_lists_test : public GameApiOperationTest {};

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

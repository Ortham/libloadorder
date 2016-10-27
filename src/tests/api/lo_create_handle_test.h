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

#ifndef LIBLO_TEST_API_LO_CREATE_HANDLE_TEST
#define LIBLO_TEST_API_LO_CREATE_HANDLE_TEST

#include "libloadorder/libloadorder.h"
#include "tests/GameTest.h"

namespace liblo {
    namespace test {
        class lo_create_handle_test : public GameTest {
        protected:
            lo_create_handle_test() :
                invalidPath("./missing"),
                gameHandle(nullptr) {}

            inline virtual void SetUp() {
                GameTest::SetUp();

                ASSERT_FALSE(boost::filesystem::exists(invalidPath));
            }

            inline virtual void TearDown() {
                GameTest::TearDown();

                EXPECT_NO_THROW(lo_destroy_handle(gameHandle));
            }

            const boost::filesystem::path invalidPath;

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
                                    LIBLO_GAME_FO4,
                                    LIBLO_GAME_TES5SE));

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
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            boost::filesystem::ofstream out(activePluginsFilePath);
            out << blankEsm;
            out.close();

            out.open(loadOrderFilePath);
            out << blankDifferentEsm;
            out.close();

            EXPECT_EQ(LIBLO_WARN_LO_MISMATCH, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));
        }
    }
}

#endif

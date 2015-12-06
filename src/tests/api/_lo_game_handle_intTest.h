/*  libloadorder

A library for reading and writing the load order of plugin files for
TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3,
Fallout: New Vegas and Fallout 4.

Copyright (C) 2015 Oliver Hamlet

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

#include "api/_lo_game_handle_int.h"
#include "libloadorder/constants.h"

#include <thread>

#include <gtest/gtest.h>

namespace liblo {
    namespace test {
        class _lo_game_handle_intTest : public ::testing::Test {
        protected:
            _lo_game_handle_intTest() :
                // Just use Skyrim game handle to test with as the
                // functionality tested isn't game-specific.
                gameHandle(LIBLO_GAME_TES5, "./Skyrim", "./local/Skyrim"),
                vectorOfStrings({"1", "2"}),
                otherVectorOfStrings({"3"}) {}

            _lo_game_handle_int gameHandle;

            std::vector<std::string> vectorOfStrings;
            std::vector<std::string> otherVectorOfStrings;
        };

        TEST_F(_lo_game_handle_intTest, settingExternalStringShouldCopyIt) {
            EXPECT_NO_THROW(gameHandle.setExternalString("test"));
            EXPECT_STREQ("test", gameHandle.getExternalString());
        }

        TEST_F(_lo_game_handle_intTest, settingExternalStringShouldSetItOnlyForTheThreadItWasSetIn) {
            EXPECT_NO_THROW(gameHandle.setExternalString("foo"));
            EXPECT_STREQ("foo", gameHandle.getExternalString());

            std::thread otherThread([&]() {
                EXPECT_STREQ("", gameHandle.getExternalString());

                EXPECT_NO_THROW(gameHandle.setExternalString("bar"));
                EXPECT_STREQ("bar", gameHandle.getExternalString());
            });
            otherThread.join();

            EXPECT_STREQ("foo", gameHandle.getExternalString());
        }

        TEST_F(_lo_game_handle_intTest, copyToStringArrayShouldCopyAContainersElementsToTheObjectsStringArray) {
            EXPECT_NO_THROW(gameHandle.setExternalStringArray(vectorOfStrings));

            EXPECT_EQ(vectorOfStrings.size(), gameHandle.getExternalStringArray().size());
            EXPECT_EQ(vectorOfStrings[0], gameHandle.getExternalStringArray()[0]);
            EXPECT_EQ(vectorOfStrings[1], gameHandle.getExternalStringArray()[1]);
        }

        TEST_F(_lo_game_handle_intTest, copyingToStringArrayTwiceShouldOverwriteTheFirstDataCopied) {
            ASSERT_NO_THROW(gameHandle.setExternalStringArray(vectorOfStrings));

            ASSERT_NO_THROW(gameHandle.setExternalStringArray(otherVectorOfStrings));
            ASSERT_NE(otherVectorOfStrings[0], vectorOfStrings[0]);

            EXPECT_EQ(otherVectorOfStrings.size(), gameHandle.getExternalStringArray().size());
            EXPECT_EQ(otherVectorOfStrings[0], gameHandle.getExternalStringArray()[0]);
        }

        TEST_F(_lo_game_handle_intTest, settingExternalStringArrayShouldSetItOnlyForTheThreadItWasSetIn) {
            ASSERT_NO_THROW(gameHandle.setExternalStringArray(vectorOfStrings));

            EXPECT_EQ(vectorOfStrings.size(), gameHandle.getExternalStringArray().size());
            EXPECT_EQ(vectorOfStrings[0], gameHandle.getExternalStringArray()[0]);
            EXPECT_EQ(vectorOfStrings[1], gameHandle.getExternalStringArray()[1]);

            std::thread otherThread([&]() {
                EXPECT_TRUE(gameHandle.getExternalStringArray().empty());

                ASSERT_NO_THROW(gameHandle.setExternalStringArray(otherVectorOfStrings));

                EXPECT_EQ(otherVectorOfStrings.size(), gameHandle.getExternalStringArray().size());
                EXPECT_EQ(otherVectorOfStrings[0], gameHandle.getExternalStringArray()[0]);
            });
            otherThread.join();

            EXPECT_EQ(vectorOfStrings.size(), gameHandle.getExternalStringArray().size());
            EXPECT_EQ(vectorOfStrings[0], gameHandle.getExternalStringArray()[0]);
            EXPECT_EQ(vectorOfStrings[1], gameHandle.getExternalStringArray()[1]);
        }
    }
}

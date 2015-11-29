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

#include <gtest/gtest.h>

#include "libloadorder/constants.h"
#include "api/_lo_game_handle_int.h"

namespace liblo {
    namespace test {
        class _lo_game_handle_intTest : public ::testing::Test {
        protected:
            _lo_game_handle_intTest() :
                // Just use Skyrim game handle to test with as the
                // functionality tested isn't game-specific.
                gameHandle(LIBLO_GAME_TES5, "./Skyrim", "./local/Skyrim"),
                vectorOfStrings({"1", "2"}) {}

            _lo_game_handle_int gameHandle;

            std::vector<std::string> vectorOfStrings;
        };

        TEST_F(_lo_game_handle_intTest, copyToStringArrayShouldCopyAContainersElementsToTheObjectsStringArray) {
            EXPECT_NO_THROW(gameHandle.copyToStringArray(vectorOfStrings));

            EXPECT_EQ(2, gameHandle.extStringArraySize);
            EXPECT_STREQ("1", gameHandle.extStringArray[0]);
            EXPECT_STREQ("2", gameHandle.extStringArray[1]);
        }

        TEST_F(_lo_game_handle_intTest, freeStringArrayShouldResetStringArraySize) {
            ASSERT_NO_THROW(gameHandle.copyToStringArray(vectorOfStrings));
            ASSERT_NE(0, gameHandle.extStringArraySize);

            EXPECT_NO_THROW(gameHandle.freeStringArray());
            EXPECT_EQ(0, gameHandle.extStringArraySize);
        }

        TEST_F(_lo_game_handle_intTest, freeStringArrayShouldResetStringArrayPointer) {
            ASSERT_NO_THROW(gameHandle.copyToStringArray(vectorOfStrings));
            ASSERT_NE(nullptr, gameHandle.extStringArray);

            EXPECT_NO_THROW(gameHandle.freeStringArray());
            EXPECT_EQ(nullptr, gameHandle.extStringArray);
        }

        TEST_F(_lo_game_handle_intTest, freeStringArrayShouldResetStringArrayElementPointers) {
            ASSERT_NO_THROW(gameHandle.copyToStringArray(vectorOfStrings));

            char ** stringArray = gameHandle.extStringArray;
            ASSERT_NE(nullptr, stringArray);
            ASSERT_NE(nullptr, stringArray[0]);
            ASSERT_NE(nullptr, stringArray[1]);

            EXPECT_NO_THROW(gameHandle.freeStringArray());
            EXPECT_EQ(nullptr, stringArray[0]);
            EXPECT_EQ(nullptr, stringArray[1]);
        }

        TEST_F(_lo_game_handle_intTest, copyToStringArrayShouldFreeAnyMemoryPreviouslyAllocated) {
            ASSERT_NO_THROW(gameHandle.copyToStringArray(vectorOfStrings));

            char ** firstStringArray = gameHandle.extStringArray;
            char * firstStringArrayElement = gameHandle.extStringArray[0];

            ASSERT_NO_THROW(gameHandle.copyToStringArray(vectorOfStrings));

            EXPECT_NE(firstStringArrayElement, firstStringArray[0]);
        }
    }
}

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
                gameHandle(LIBLO_GAME_TES5, "./Skyrim", "./local/Skyrim") {}

            _lo_game_handle_int gameHandle;
        };

        TEST_F(_lo_game_handle_intTest, copyToStringArrayShouldCopyAContainersElementsToTheObjectsStringArray) {
            std::vector<std::string> vectorOfStrings({
                "1",
                "2",
            });

            gameHandle.copyToStringArray(vectorOfStrings);

            EXPECT_EQ(2, gameHandle.extStringArraySize);
            EXPECT_STREQ("1", gameHandle.extStringArray[0]);
            EXPECT_STREQ("2", gameHandle.extStringArray[1]);
        }

        TEST_F(_lo_game_handle_intTest, freeStringArrayShouldResetStringArraySize) {
            std::vector<std::string> vectorOfStrings({
                "1",
                "2",
            });
            gameHandle.copyToStringArray(vectorOfStrings);

            gameHandle.freeStringArray();

            EXPECT_EQ(0, gameHandle.extStringArraySize);
        }

        TEST_F(_lo_game_handle_intTest, freeStringArrayShouldResetStringArrayPointer) {
            std::vector<std::string> vectorOfStrings({
                "1",
                "2",
            });
            gameHandle.copyToStringArray(vectorOfStrings);

            gameHandle.freeStringArray();

            EXPECT_EQ(nullptr, gameHandle.extStringArray);
        }

        TEST_F(_lo_game_handle_intTest, freeStringArrayShouldResetStringArrayElementPointers) {
            std::vector<std::string> vectorOfStrings({
                "1",
                "2",
            });
            gameHandle.copyToStringArray(vectorOfStrings);

            char ** stringArray = gameHandle.extStringArray;

            gameHandle.freeStringArray();

            EXPECT_EQ(nullptr, stringArray[0]);
            EXPECT_EQ(nullptr, stringArray[1]);
        }

        TEST_F(_lo_game_handle_intTest, copyToStringArrayShouldFreeAnyMemoryPreviouslyAllocated) {
            std::vector<std::string> vectorOfStrings({
                "1",
                "2",
            });
            gameHandle.copyToStringArray(vectorOfStrings);

            char ** firstStringArray = gameHandle.extStringArray;
            char * firstStringArrayElement = firstStringArray[0];

            gameHandle.copyToStringArray(vectorOfStrings);

            EXPECT_NE(firstStringArrayElement, firstStringArray[0]);
        }
    }
}

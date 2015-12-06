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

#include "backend/error.h"
#include "api/c_helpers.h"

namespace liblo {
    namespace test {
        TEST(copyToContainer, shouldCopyAStringArrayToAContainerOfTheGivenType) {
            const char * stringArray[2] = {
                "1",
                "2",
            };
            std::vector<std::string> expectedStringVector({
                "1",
                "2",
            });

            EXPECT_EQ(expectedStringVector, copyToContainer<std::vector<std::string>>(stringArray, 2));
        }

        TEST(c_error, shouldReturnTheCodeWhenPassedACodeAndString) {
            EXPECT_EQ(1, c_error(1, "what string"));
        }

        TEST(c_error, shouldReturnTheCodeWhenPassedAnErrorObject) {
            EXPECT_EQ(1, c_error(error(1, "what string")));
        }

        TEST(c_error, shouldSetTheGlobalErrorStringWhenPassedACodeAndString) {
            c_error(1, "what string");

            EXPECT_EQ("what string", extErrorString);
        }

        TEST(c_error, shouldSetTheGlobalErrorStringWhenPassedAnErrorObject) {
            c_error(error(1, "what string"));

            EXPECT_EQ("what string", extErrorString);
        }
    }
}

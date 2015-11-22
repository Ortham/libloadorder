/*  libloadorder

A library for reading and writing the load order of plugin files for
TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 and
Fallout: New Vegas.

Copyright (C) 2015    WrinklyNinja

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

namespace liblo {
    namespace test {
        TEST(error, whatShouldReturnStringPassedInConstructor) {
            error e(1, "what string");

            EXPECT_EQ(1, e.code());
        }

        TEST(error, codeShouldReturnTheCodePassedInConstructor) {
            error e(1, "what string");

            EXPECT_STREQ("what string", e.what());
        }

        TEST(c_error, shouldReturnTheCodeWhenPassedACodeAndString) {
            EXPECT_EQ(1, c_error(1, "what string"));

            delete[] extErrorString;
            extErrorString = nullptr;
        }

        TEST(c_error, shouldReturnTheCodeWhenPassedAnErrorObject) {
            EXPECT_EQ(1, c_error(error(1, "what string")));

            delete[] extErrorString;
            extErrorString = nullptr;
        }

        TEST(c_error, shouldSetTheGlobalErrorStringWhenPassedACodeAndString) {
            c_error(1, "what string");

            EXPECT_STREQ("what string", extErrorString);

            delete[] extErrorString;
            extErrorString = nullptr;
        }

        TEST(c_error, shouldSetTheGlobalErrorStringWhenPassedAnErrorObject) {
            c_error(error(1, "what string"));

            EXPECT_STREQ("what string", extErrorString);

            delete[] extErrorString;
            extErrorString = nullptr;
        }
    }
}
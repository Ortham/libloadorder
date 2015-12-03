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

#ifndef LIBLO_TEST_API_C_API_GAME_OPERATION_TEST
#define LIBLO_TEST_API_C_API_GAME_OPERATION_TEST

#include "tests/api/lo_create_handle_test.h"

namespace liblo {
    namespace test {
        class CApiGameOperationTest : public lo_create_handle_test {
        protected:
            inline virtual void SetUp() {
                GameTest::SetUp();

                ASSERT_EQ(LIBLO_OK, lo_create_handle(&gameHandle, GetParam(), gamePath.string().c_str(), localPath.string().c_str()));
            }
        };

        // A couple of helpers for using C arrays with standard library
        // algorithms.
        char ** begin(char ** cArray) {
            return cArray;
        }

        char ** end(char ** cArray, size_t cArraySize) {
            return cArray + cArraySize;
        }
    }
}

#endif

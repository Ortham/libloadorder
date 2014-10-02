/*  libloadorder

A library for reading and writing the load order of plugin files for
TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 and
Fallout: New Vegas.

Copyright (C) 2012    WrinklyNinja

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

#ifndef __LIBLO_TEST_FIXTURES__
#define __LIBLO_TEST_FIXTURES__

#include "../api/libloadorder.h"
#include <gtest/gtest.h>

class GameHandleCreationTest : public ::testing::Test {
protected:
    inline GameHandleCreationTest() : gh(NULL) {}

    inline virtual void TearDown() {
        ASSERT_NO_THROW(lo_destroy_handle(gh));
    };

    lo_game_handle gh;
};

class OblivionOperationsTest : public ::testing::Test {
protected:
    inline virtual void SetUp() {
        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, "./game", "./local"));
    }

    inline virtual void TearDown() {
        lo_destroy_handle(gh);
    };

    lo_game_handle gh;
};

class SkyrimOperationsTest : public ::testing::Test {
protected:
    inline virtual void SetUp() {
        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, "./game", "./local"));
    }

    inline virtual void TearDown() {
        lo_destroy_handle(gh);
    };

    lo_game_handle gh;
};

#endif

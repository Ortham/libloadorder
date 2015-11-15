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

#include "libloadorder/constants.h"
#include "backend/game.h"

#include <libespm/GameId.h>

namespace liblo {
    namespace test {
        class GameHandleTest : public ::testing::Test {
        protected:
            boost::filesystem::path gamePath;
        };

        TEST_F(GameHandleTest, morrowindIdShouldMapToLibespmsMorrowindId) {
            _lo_game_handle_int gameHandle(LIBLO_GAME_TES3, gamePath.string());

            EXPECT_EQ(libespm::GameId::MORROWIND, gameHandle.getLibespmId());
        }

        TEST_F(GameHandleTest, oblivionIdShouldMapToLibespmsOblivionId) {
            _lo_game_handle_int gameHandle(LIBLO_GAME_TES4, gamePath.string());

            EXPECT_EQ(libespm::GameId::OBLIVION, gameHandle.getLibespmId());
        }

        TEST_F(GameHandleTest, skyrimIdShouldMapToLibespmsSkyrimId) {
            _lo_game_handle_int gameHandle(LIBLO_GAME_TES5, gamePath.string());

            EXPECT_EQ(libespm::GameId::SKYRIM, gameHandle.getLibespmId());
        }

        TEST_F(GameHandleTest, fallout3IdShouldMapToLibespmsFallout3Id) {
            _lo_game_handle_int gameHandle(LIBLO_GAME_FO3, gamePath.string());

            EXPECT_EQ(libespm::GameId::FALLOUT3, gameHandle.getLibespmId());
        }

        TEST_F(GameHandleTest, falloutnvIdShouldMapToLibespmsFalloutnvId) {
            _lo_game_handle_int gameHandle(LIBLO_GAME_FNV, gamePath.string());

            EXPECT_EQ(libespm::GameId::FALLOUTNV, gameHandle.getLibespmId());
        }
    }
}

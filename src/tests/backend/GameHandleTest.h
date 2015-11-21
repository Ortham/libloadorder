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
        class GameHandleTest : public ::testing::TestWithParam<unsigned int> {
        protected:
            GameHandleTest() : gameHandle(GetParam(), "") {
                gameHandle.SetLocalAppData(getLocalPath(GetParam()));
            }

            inline libespm::GameId getExpectedLibespmId() {
                if (GetParam() == LIBLO_GAME_TES3)
                    return libespm::GameId::MORROWIND;
                else if (GetParam() == LIBLO_GAME_TES4)
                    return libespm::GameId::OBLIVION;
                else if (GetParam() == LIBLO_GAME_TES5)
                    return libespm::GameId::SKYRIM;
                else if (GetParam() == LIBLO_GAME_FO3)
                    return libespm::GameId::FALLOUT3;
                else
                    return libespm::GameId::FALLOUTNV;
            }

            inline boost::filesystem::path getLocalPath(unsigned int gameId) const {
                if (gameId == LIBLO_GAME_TES3)
                    return "./local/Morrowind";
                else if (gameId == LIBLO_GAME_TES4)
                    return "./local/Oblivion";
                else
                    return "./local/Skyrim";
            }

            _lo_game_handle_int gameHandle;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                GameHandleTest,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV));

        TEST_P(GameHandleTest, gettingLibespmIdShouldReturnExpectedValueForGame) {
            EXPECT_EQ(getExpectedLibespmId(), gameHandle.getLibespmId());
        }

        TEST_P(GameHandleTest, gettingLoadOrderFilePathShouldThrowForTimestampBasedGamesAndNotOtherwise) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                EXPECT_ANY_THROW(gameHandle.LoadOrderFile());
            else
                EXPECT_NO_THROW(gameHandle.LoadOrderFile());
        }
    }
}

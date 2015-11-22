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
#include "backend/GameSettings.h"

namespace liblo {
    namespace test {
        class GameSettingsTest : public ::testing::TestWithParam<unsigned int> {
        protected:
            GameSettingsTest() : gameSettings(GetParam(), "", getLocalPath(GetParam())) {}

            inline libespm::GameId getExpectedLibespmId() {
                if (GetParam() == LIBLO_GAME_TES3)
                    return libespm::GameId::MORROWIND;
                else if (GetParam() == LIBLO_GAME_TES4)
                    return libespm::GameId::OBLIVION;
                else if (GetParam() == LIBLO_GAME_TES5)
                    return libespm::GameId::SKYRIM;
                else if (GetParam() == LIBLO_GAME_FO3)
                    return libespm::GameId::FALLOUT3;
                else if (GetParam() == LIBLO_GAME_FNV)
                    return libespm::GameId::FALLOUTNV;
                else
                    return libespm::GameId::FALLOUT4;
            }

            inline boost::filesystem::path getLocalPath(unsigned int gameId) const {
                if (gameId == LIBLO_GAME_TES3)
                    return "./local/Morrowind";
                else if (gameId == LIBLO_GAME_TES4)
                    return "./local/Oblivion";
                else
                    return "./local/Skyrim";
            }

            GameSettings gameSettings;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                GameSettingsTest,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(GameSettingsTest, gettingIdShouldReturnTheTestParameter) {
            EXPECT_EQ(GetParam(), gameSettings.getId());
        }

        TEST_P(GameSettingsTest, gettingLibespmIdShouldReturnExpectedValueForGame) {
            EXPECT_EQ(getExpectedLibespmId(), gameSettings.getLibespmId());
        }

        TEST_P(GameSettingsTest, gettingMasterFileShouldReturnTheCorrectFilenameForEachGame) {
            if (GetParam() == LIBLO_GAME_TES3)
                EXPECT_EQ("Morrowind.esm", gameSettings.getMasterFile());
            else if (GetParam() == LIBLO_GAME_TES4)
                EXPECT_EQ("Oblivion.esm", gameSettings.getMasterFile());
            else if (GetParam() == LIBLO_GAME_TES5)
                EXPECT_EQ("Skyrim.esm", gameSettings.getMasterFile());
            else if (GetParam() == LIBLO_GAME_FO3)
                EXPECT_EQ("Fallout3.esm", gameSettings.getMasterFile());
            else if (GetParam() == LIBLO_GAME_FNV)
                EXPECT_EQ("FalloutNV.esm", gameSettings.getMasterFile());
            else
                EXPECT_EQ("Fallout4.esm", gameSettings.getMasterFile());
        }

        TEST_P(GameSettingsTest, gettingLoadOrderMethodShouldReturnTextfileForSkyrimAndFallout4AndTimestampOtherwise) {
            if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_FO4)
                EXPECT_EQ(LIBLO_METHOD_TEXTFILE, gameSettings.getLoadOrderMethod());
            else
                EXPECT_EQ(LIBLO_METHOD_TIMESTAMP, gameSettings.getLoadOrderMethod());
        }

        TEST_P(GameSettingsTest, pluginsFolderShouldBeDataFilesForMorrowindAndDataOtherwise) {
            if (GetParam() == LIBLO_GAME_TES3)
                EXPECT_EQ("Data Files", gameSettings.getPluginsFolder());
            else
                EXPECT_EQ("Data", gameSettings.getPluginsFolder());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeMorrowindIniForMorrowindAndPluginsTxtOtherwise) {
            if (GetParam() == LIBLO_GAME_TES3)
                EXPECT_EQ("Morrowind.ini", gameSettings.getActivePluginsFile());
            else
                EXPECT_EQ(getLocalPath(GetParam()) / "plugins.txt", gameSettings.getActivePluginsFile());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeInGameFolderForOblivionIfItIsSetToUseTheGameFolder) {
            if (GetParam() != LIBLO_GAME_TES4)
                return;

            // Set ini setting.
            boost::filesystem::ofstream out("Oblivion.ini");
            out << "bUseMyGamesDirectory=0";
            out.close();

            // Now reinitialise game settings.
            gameSettings = GameSettings(GetParam(), "", getLocalPath(GetParam()));

            EXPECT_EQ("plugins.txt", gameSettings.getActivePluginsFile());

            EXPECT_NO_THROW(boost::filesystem::remove("Oblivion.ini"));
        }

        TEST_P(GameSettingsTest, gettingLoadOrderFilePathShouldThrowForTimestampBasedGamesAndNotOtherwise) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                EXPECT_ANY_THROW(gameSettings.getLoadOrderFile());
            else
                EXPECT_NO_THROW(gameSettings.getLoadOrderFile());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeLoadOrderTxtForTimestampBasedGames) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_EQ(getLocalPath(GetParam()) / "loadorder.txt", gameSettings.getLoadOrderFile());
        }
    }
}

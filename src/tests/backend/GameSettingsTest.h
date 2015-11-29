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

#include "tests/GameTest.h"
#include "libloadorder/constants.h"
#include "backend/GameSettings.h"

namespace liblo {
    namespace test {
        class GameSettingsTest : public GameTest {
        protected:
            GameSettingsTest() : gameSettings(GetParam(), gamePath, localPath) {}

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

            const GameSettings gameSettings;
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
            EXPECT_EQ(masterFile, gameSettings.getMasterFile());
        }

        TEST_P(GameSettingsTest, gettingLoadOrderMethodShouldReturnTextfileForSkyrimAndFallout4AndTimestampOtherwise) {
            if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_FO4)
                EXPECT_EQ(LIBLO_METHOD_TEXTFILE, gameSettings.getLoadOrderMethod());
            else
                EXPECT_EQ(LIBLO_METHOD_TIMESTAMP, gameSettings.getLoadOrderMethod());
        }

        TEST_P(GameSettingsTest, pluginsFolderShouldBeDataFilesForMorrowindAndDataOtherwise) {
            EXPECT_EQ(pluginsPath, gameSettings.getPluginsFolder());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeMorrowindIniForMorrowindAndPluginsTxtOtherwise) {
            if (GetParam() == LIBLO_GAME_TES3)
                EXPECT_EQ(gamePath / "Morrowind.ini", gameSettings.getActivePluginsFile());
            else
                EXPECT_EQ(localPath / "plugins.txt", gameSettings.getActivePluginsFile());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeInGameFolderForOblivionIfItIsSetToUseTheGameFolder) {
            if (GetParam() != LIBLO_GAME_TES4)
                return;

            // Set ini setting.
            boost::filesystem::ofstream out("Oblivion.ini");
            out << "bUseMyGamesDirectory=0";
            out.close();

            // The active plugins folder for existing game settings should be
            // unchanged, but new objects should use the game folder.
            EXPECT_NE(gamePath / "plugins.txt", gameSettings.getActivePluginsFile());
            EXPECT_EQ("plugins.txt", GameSettings(GetParam(), "", localPath).getActivePluginsFile());

            EXPECT_NO_THROW(boost::filesystem::remove("Oblivion.ini"));
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeInLocalFolderForOblivionIfItIsSetNotToUseTheGameFolder) {
            if (GetParam() != LIBLO_GAME_TES4)
                return;

            // Set ini setting.
            boost::filesystem::ofstream out("Oblivion.ini");
            out << "bUseMyGamesDirectory=1";
            out.close();

            // The active plugins folder for existing game settings should be
            // unchanged, but new objects should use the game folder.
            EXPECT_EQ(localPath / "plugins.txt", gameSettings.getActivePluginsFile());
            EXPECT_EQ(localPath / "plugins.txt", GameSettings(GetParam(), gamePath, localPath).getActivePluginsFile());

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
                EXPECT_EQ(localPath / "loadorder.txt", gameSettings.getLoadOrderFile());
        }
    }
}

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
            GameSettingsTest() :
                gameSettings(GetParam(), gamePath, localPath),
                oblivionIni(gamePath / "Oblivion.ini") {}

            inline virtual void TearDown() {
                GameTest::TearDown();

                try {
                  EXPECT_NO_THROW(boost::filesystem::remove(oblivionIni));
                } catch (std::exception& e) {
                  std::cout << e.what() << std::endl;
                }
            }

            inline libespm::GameId getExpectedLibespmId() const {
                if (GetParam() == LIBLO_GAME_TES3)
                    return libespm::GameId::MORROWIND;
                else if (GetParam() == LIBLO_GAME_TES4)
                    return libespm::GameId::OBLIVION;
                else if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_TES5SE)
                    return libespm::GameId::SKYRIM;
                else if (GetParam() == LIBLO_GAME_FO3)
                    return libespm::GameId::FALLOUT3;
                else if (GetParam() == LIBLO_GAME_FNV)
                    return libespm::GameId::FALLOUTNV;
                else
                    return libespm::GameId::FALLOUT4;
            }

            inline std::vector<std::string> getExpectedImplicitlyActivePlugins() const {
                if (GetParam() == LIBLO_GAME_TES5) {
                    return std::vector<std::string>({
                        masterFile,
                        "Update.esm",
                    });
                }
                else if (GetParam() == LIBLO_GAME_FO4) {
                    return std::vector<std::string>({
                        masterFile,
                        "DLCRobot.esm",
                        "DLCworkshop01.esm",
                        "DLCCoast.esm",
                        "DLCworkshop02.esm",
                        "DLCworkshop03.esm",
                        "DLCNukaWorld.esm",
                    });
                } else if (GetParam() == LIBLO_GAME_TES5SE) {
                  return std::vector<std::string>({
                      masterFile,
                      "Update.esm",
                      "Dawnguard.esm",
                      "Hearthfires.esm",
                      "Dragonborn.esm",
                  });
                }

                return std::vector<std::string>();
            }

            const GameSettings gameSettings;

            const boost::filesystem::path oblivionIni;
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
                                    LIBLO_GAME_FO4,
                                    LIBLO_GAME_TES5SE));

        TEST_P(GameSettingsTest, gettingIdShouldReturnTheTestParameter) {
            EXPECT_EQ(GetParam(), gameSettings.getId());
        }

        TEST_P(GameSettingsTest, gettingLibespmIdShouldReturnExpectedValueForGame) {
            EXPECT_EQ(getExpectedLibespmId(), gameSettings.getLibespmId());
        }

        TEST_P(GameSettingsTest, gettingMasterFileShouldReturnTheCorrectFilenameForEachGame) {
            EXPECT_EQ(masterFile, gameSettings.getMasterFile());
        }

        TEST_P(GameSettingsTest, gettingLoadOrderMethodShouldReturnTextfileForSkyrimAndAsteriskForFallout4AndTimestampOtherwise) {
            EXPECT_EQ(loadOrderMethod, gameSettings.getLoadOrderMethod());
        }

        TEST_P(GameSettingsTest, gettingImplicitlyActivePluginsShouldReturnCorrectPluginNames) {
            EXPECT_EQ(getExpectedImplicitlyActivePlugins(), gameSettings.getImplicitlyActivePlugins());
        }

        TEST_P(GameSettingsTest, isImplicitlyActiveShouldReturnTrueForAllImplicitlyActivePlugins) {
            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins())
                EXPECT_TRUE(gameSettings.isImplicitlyActive(plugin));
        }

        TEST_P(GameSettingsTest, isImplicitlyActiveShouldReturnFalseForAPluginThatIsNotImplicitlyActive) {
            EXPECT_FALSE(gameSettings.isImplicitlyActive(blankEsm));
        }

        TEST_P(GameSettingsTest, pluginsFolderShouldBeDataFilesForMorrowindAndDataOtherwise) {
            EXPECT_EQ(pluginsPath, gameSettings.getPluginsFolder());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeMorrowindIniForMorrowindAndPluginsTxtOtherwise) {
            EXPECT_EQ(activePluginsFilePath, gameSettings.getActivePluginsFile());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeInGameFolderForOblivionIfItIsSetToUseTheGameFolder) {
            if (GetParam() != LIBLO_GAME_TES4)
                return;

            // Set ini setting.
            boost::filesystem::ofstream out(oblivionIni);
            out << "bUseMyGamesDirectory=0";
            out.close();

            // The active plugins folder for existing game settings should be
            // unchanged, but new objects should use the game folder.
            ASSERT_NE(activePluginsFilePath, activePluginsFilePath.filename());
            EXPECT_EQ(activePluginsFilePath, gameSettings.getActivePluginsFile());
            GameSettings settings = GameSettings(GetParam(), gamePath, localPath);
            EXPECT_EQ(gamePath / activePluginsFilePath.filename(), settings.getActivePluginsFile());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeInLocalFolderForOblivionIfItIsSetNotToUseTheGameFolder) {
            if (GetParam() != LIBLO_GAME_TES4)
                return;

            // Set ini setting.
            boost::filesystem::ofstream out(oblivionIni);
            out << "bUseMyGamesDirectory=1";
            out.close();

            // The active plugins folder for existing game settings should be
            // unchanged, but new objects should use the game folder.
            EXPECT_EQ(activePluginsFilePath, gameSettings.getActivePluginsFile());
            EXPECT_EQ(activePluginsFilePath, GameSettings(GetParam(), gamePath, localPath).getActivePluginsFile());
        }

        TEST_P(GameSettingsTest, gettingLoadOrderFilePathShouldNotThrowForTextfileBasedGamesAndThrowOtherwise) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_NO_THROW(gameSettings.getLoadOrderFile());
            else
                EXPECT_ANY_THROW(gameSettings.getLoadOrderFile());
        }

        TEST_P(GameSettingsTest, activePluginsFileShouldBeLoadOrderTxtForTextfileBasedGames) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_EQ(loadOrderFilePath, gameSettings.getLoadOrderFile());
        }
    }
}

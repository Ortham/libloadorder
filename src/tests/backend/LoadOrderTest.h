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
#include "backend/LoadOrder.h"

namespace liblo {
    namespace test {
        class LoadOrderTest : public ::testing::TestWithParam<unsigned int> {
        protected:
            inline LoadOrderTest() :
                updateEsm("Update.esm"),
                blankEsm("Blank.esm"),
                blankDifferentEsm("Blank - Different.esm"),
                blankEsp("Blank.esp"),
                invalidPlugin("NotAPlugin.esm"),
                missingPlugin("missing.esm"),
                loadOrderWithDuplicatesFile("duplicates.txt"),
                morrowindLoadOrderWithDuplicatesFile("mwduplicates.ini"),
                loadOrderWithPluginBeforeMasterFile("unpartitioned.txt"),
                morrowindLoadOrderWithPluginBeforeMasterFile("mwunpartitioned.ini"),
                gameHandle(GetParam(), getGamePath(GetParam())) {
                gameHandle.SetLocalAppData(getLocalPath(GetParam()));
            }

            inline virtual void SetUp() {
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / blankEsm));
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / blankDifferentEsm));
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / blankEsp));
                ASSERT_FALSE(boost::filesystem::exists(gameHandle.PluginsFolder() / missingPlugin));

                // Write out an non-empty, non-plugin file.
                boost::filesystem::ofstream out(gameHandle.PluginsFolder() / invalidPlugin);
                out << "This isn't a valid plugin file.";
                out.close();
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / invalidPlugin));

                // Make sure the game master file exists.
                ASSERT_FALSE(boost::filesystem::exists(gameHandle.PluginsFolder() / gameHandle.MasterFile()));
                ASSERT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsm, gameHandle.PluginsFolder() / gameHandle.MasterFile()));
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / gameHandle.MasterFile()));

                // Make sure Update.esm exists.
                ASSERT_FALSE(boost::filesystem::exists(gameHandle.PluginsFolder() / updateEsm));
                ASSERT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsm, gameHandle.PluginsFolder() / updateEsm));
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / updateEsm));

                // Write out a load order file containing duplicates.
                out.open(loadOrderWithDuplicatesFile);
                out << gameHandle.MasterFile() << std::endl
                    << blankEsm << std::endl
                    << blankDifferentEsm << std::endl
                    << blankEsm << std::endl
                    << invalidPlugin << std::endl;
                out.close();

                // Do the same again, but for Morrowind's load order file format.
                out.open(morrowindLoadOrderWithDuplicatesFile);
                out << "GameFile0=" << gameHandle.MasterFile() << std::endl
                    << "GameFile1=" << blankEsm << std::endl
                    << "GameFile2=" << blankDifferentEsm << std::endl
                    << "GameFile3=" << blankEsm << std::endl
                    << "GameFile4=" << invalidPlugin << std::endl;
                out.close();

                // Write out a load order file containing a plugin before a master
                out.open(loadOrderWithPluginBeforeMasterFile);
                out << gameHandle.MasterFile() << std::endl
                    << blankEsp << std::endl
                    << blankDifferentEsm << std::endl;
                out.close();

                // Do the same again, but for Morrowind's load order file format.
                out.open(morrowindLoadOrderWithPluginBeforeMasterFile);
                out << "GameFile0=" << gameHandle.MasterFile() << std::endl
                    << "GameFile1=" << blankEsp << std::endl
                    << "GameFile2=" << blankDifferentEsm << std::endl;
                out.close();
            }

            inline virtual void TearDown() {
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / invalidPlugin));
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / gameHandle.MasterFile()));
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / updateEsm));

                ASSERT_NO_THROW(boost::filesystem::remove(loadOrderWithDuplicatesFile));
                ASSERT_NO_THROW(boost::filesystem::remove(loadOrderWithPluginBeforeMasterFile));
                ASSERT_NO_THROW(boost::filesystem::remove(morrowindLoadOrderWithDuplicatesFile));
                ASSERT_NO_THROW(boost::filesystem::remove(morrowindLoadOrderWithPluginBeforeMasterFile));
            }

            inline std::string getGamePath(unsigned int gameId) const {
                if (gameId == LIBLO_GAME_TES3)
                    return "./Morrowind";
                else if (gameId == LIBLO_GAME_TES4)
                    return "./Oblivion";
                else
                    return "./Skyrim";
            }

            inline boost::filesystem::path getLocalPath(unsigned int gameId) const {
                if (gameId == LIBLO_GAME_TES3)
                    return "./local/Morrowind";
                else if (gameId == LIBLO_GAME_TES4)
                    return "./local/Oblivion";
                else
                    return "./local/Skyrim";
            }

            inline void loadDuplicatesFromFile() {
                if (gameHandle.Id() == LIBLO_GAME_TES3)
                    loadOrder.LoadFromFile(gameHandle, morrowindLoadOrderWithDuplicatesFile);
                else
                    loadOrder.LoadFromFile(gameHandle, loadOrderWithDuplicatesFile);
            }

            inline void loadPluginBeforeMasterFromFile() {
                if (gameHandle.Id() == LIBLO_GAME_TES3)
                    loadOrder.LoadFromFile(gameHandle, morrowindLoadOrderWithPluginBeforeMasterFile);
                else
                    loadOrder.LoadFromFile(gameHandle, loadOrderWithPluginBeforeMasterFile);
            }

            LoadOrder loadOrder;
            _lo_game_handle_int gameHandle;

            std::string updateEsm;
            std::string blankEsm;
            std::string blankDifferentEsm;
            std::string blankEsp;
            std::string invalidPlugin;
            std::string missingPlugin;

            boost::filesystem::path loadOrderWithDuplicatesFile;
            boost::filesystem::path loadOrderWithPluginBeforeMasterFile;
            boost::filesystem::path morrowindLoadOrderWithDuplicatesFile;
            boost::filesystem::path morrowindLoadOrderWithPluginBeforeMasterFile;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                LoadOrderTest,
                                ::testing::Values(
                                LIBLO_GAME_TES3,
                                LIBLO_GAME_TES4,
                                LIBLO_GAME_TES5,
                                LIBLO_GAME_FO3,
                                LIBLO_GAME_FNV));

        TEST_P(LoadOrderTest, settingAValidLoadOrderShouldNotThrow) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            EXPECT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithPluginsBeforeMastersShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                blankEsp,
                blankDifferentEsm,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithPluginsBeforeMastersShouldMakeNoChanges) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                blankEsp,
                blankDifferentEsm,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithAnInvalidPluginShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithAnInvalidPluginShouldMakeNoChanges) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithACaseInsensitiveDuplicatePluginShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithACaseInsensitiveDuplicatePluginShouldMakeNoChanges) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, settingThenGettingLoadOrderShouldReturnTheSetLoadOrder) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingTheLoadOrderTwiceShouldReplaceTheFirstLoadOrder) {
            std::vector<std::string> firstLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            std::vector<std::string> secondLoadOrder({
                gameHandle.MasterFile(),
                blankDifferentEsm,
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder, gameHandle));
            ASSERT_NO_THROW(loadOrder.setLoadOrder(secondLoadOrder, gameHandle));

            EXPECT_EQ(secondLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingAnInvalidLoadOrderShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                blankEsp,
                blankDifferentEsm,
            });

            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));

            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithTheGameMasterNotAtTheBeginningShouldFailForTextfileLoadOrderGamesAndSucceedOtherwise) {
            std::vector<std::string> plugins({
                blankEsm,
                gameHandle.MasterFile(),
            });
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setLoadOrder(plugins, gameHandle));
            else
                EXPECT_NO_THROW(loadOrder.setLoadOrder(plugins, gameHandle));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithTheGameMasterNotAtTheBeginningShouldMakeNoChangesForTextfileLoadOrderGames) {
            std::vector<std::string> plugins({
                blankEsm,
                gameHandle.MasterFile(),
            });
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setLoadOrder(plugins, gameHandle));
                EXPECT_TRUE(loadOrder.getLoadOrder().empty());
            }
        }

        TEST_P(LoadOrderTest, positionOfAMissingPluginShouldEqualTheLoadOrderSize) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_EQ(validLoadOrder.size(), loadOrder.getPosition(missingPlugin));
        }

        TEST_P(LoadOrderTest, positionOfAPluginShouldBeEqualToItsLoadOrderIndex) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_EQ(1, loadOrder.getPosition(blankEsm));
        }

        TEST_P(LoadOrderTest, gettingAPluginsPositionShouldBeCaseInsensitive) {
          std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_EQ(1, loadOrder.getPosition(boost::to_lower_copy(blankEsm)));
        }

        TEST_P(LoadOrderTest, gettingPluginAtAPositionGreaterThanTheHighestIndexShouldThrow) {
            EXPECT_ANY_THROW(loadOrder.getPluginAtPosition(0));
        }

        TEST_P(LoadOrderTest, gettingPluginAtAValidPositionShouldReturnItsLoadOrderIndex) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_EQ(blankEsm, loadOrder.getPluginAtPosition(1));
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstShouldThrowForTextfileLoadOrderGamesAndNotOtherwise) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 0, gameHandle));
            else {
                EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 0, gameHandle));
            }
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstForATextfileBasedGameShouldMakeNoChanges) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 0, gameHandle));
                EXPECT_TRUE(loadOrder.getLoadOrder().empty());
            }
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstForATimestampBasedGameShouldSucceed) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 0, gameHandle));
                EXPECT_FALSE(loadOrder.getLoadOrder().empty());
                EXPECT_EQ(0, loadOrder.getPosition(blankEsm));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginShouldThrowForTextfileLoadOrderGamesAndNotOtherwise) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 1, gameHandle));
            else
                EXPECT_NO_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 1, gameHandle));
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginShouldMakeNoChangesForTextfileLoadOrderGames) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 1, gameHandle));
                EXPECT_EQ(0, loadOrder.getPosition(gameHandle.MasterFile()));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginForATextfileBasedGameShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                ASSERT_ANY_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 1, gameHandle));
                EXPECT_EQ(blankEsm, loadOrder.getPluginAtPosition(1));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginForATimestampBasedGameShouldSucceed) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                ASSERT_NO_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 1, gameHandle));
                EXPECT_EQ(blankEsm, loadOrder.getPluginAtPosition(0));
                EXPECT_EQ(gameHandle.MasterFile(), loadOrder.getPluginAtPosition(1));
            }
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnInvalidPluginShouldThrow) {
            ASSERT_NO_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 0, gameHandle));

            EXPECT_ANY_THROW(loadOrder.setPosition(invalidPlugin, 1, gameHandle));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnInvalidPluginShouldMakeNoChanges) {
            ASSERT_NO_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 0, gameHandle));

            ASSERT_ANY_THROW(loadOrder.setPosition(invalidPlugin, 1, gameHandle));
            EXPECT_EQ(1, loadOrder.getLoadOrder().size());
        }

        TEST_P(LoadOrderTest, settingThePositionOfAPluginToGreaterThanTheLoadOrderSizeShouldPutThePluginAtTheEnd) {
            ASSERT_NO_THROW(loadOrder.setPosition(gameHandle.MasterFile(), 0, gameHandle));

            EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 2, gameHandle));
            EXPECT_EQ(2, loadOrder.getLoadOrder().size());
            EXPECT_EQ(1, loadOrder.getPosition(blankEsm));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAPluginShouldBeCaseInsensitive) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_NO_THROW(loadOrder.setPosition(boost::to_lower_copy(blankEsm), 2, gameHandle));

            std::vector<std::string> expectedLoadOrder({
                gameHandle.MasterFile(),
                blankDifferentEsm,
                blankEsm,
            });
            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingANonMasterPluginToLoadBeforeAMasterPluginShouldThrow) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsp, 1, gameHandle));
        }

        TEST_P(LoadOrderTest, settingANonMasterPluginToLoadBeforeAMasterPluginShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsp, 1, gameHandle));
            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingAMasterToLoadAfterAPluginShouldThrow) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 2, gameHandle));
        }

        TEST_P(LoadOrderTest, settingAMasterToLoadAfterAPluginShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 2, gameHandle));
            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, loadingFromFileShouldLoadAllEntriesForValidPlugins) {
            std::vector<std::string> expectedLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
                blankEsm,
            });
            // Loading from file for Skyrim will also insert Update.esm
            // after other masters if it's not in the file.
            if (gameHandle.Id() == LIBLO_GAME_TES5)
                expectedLoadOrder.push_back(updateEsm);

            ASSERT_NO_THROW(loadDuplicatesFromFile());

            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, loadingFromFileShouldActivateTheGameMasterForTextfileBasedGamesAndNotOtherwise) {
            ASSERT_NO_THROW(loadDuplicatesFromFile());
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_TRUE(loadOrder.isActive(gameHandle.MasterFile()));
            else
                EXPECT_FALSE(loadOrder.isActive(gameHandle.MasterFile()));
        }

        TEST_P(LoadOrderTest, removingDuplicatePluginsShouldKeepTheLastOfTheDuplicates) {
            std::vector<std::string> expectedLoadOrder({
                gameHandle.MasterFile(),
                blankDifferentEsm,
                blankEsm,
            });
            // Loading from file for Skyrim will also insert Update.esm
            // after other masters if it's not in the file.
            if (gameHandle.Id() == LIBLO_GAME_TES5)
                expectedLoadOrder.push_back(updateEsm);

            ASSERT_NO_THROW(loadDuplicatesFromFile());

            EXPECT_NO_THROW(loadOrder.unique());
            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, partitioningMastersShouldMoveAllMastersBeforeAllNonMasters) {
            std::vector<std::string> expectedLoadOrder({
                gameHandle.MasterFile(),
                blankDifferentEsm,
                blankEsp,
            });
            // Loading from file for Skyrim will also insert Update.esm
            // after other masters if it's not in the file.
            if (gameHandle.Id() == LIBLO_GAME_TES5)
                expectedLoadOrder.insert(std::next(std::begin(expectedLoadOrder), 1), updateEsm);

            ASSERT_NO_THROW(loadPluginBeforeMasterFromFile());

            EXPECT_NO_THROW(loadOrder.partitionMasters(gameHandle));
            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, clearingLoadOrderShouldRemoveAllPluginsFromTheLoadOrder) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_NO_THROW(loadOrder.clear());
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, checkingIfAnInactivePluginIsActiveShouldReturnFalse) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_FALSE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, checkingIfAPluginNotInTheLoadOrderIsActiveShouldReturnFalse) {
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, activatingAnInvalidPluginShouldThrow) {
            EXPECT_ANY_THROW(loadOrder.activate(invalidPlugin, gameHandle));
        }

        TEST_P(LoadOrderTest, activatingANonMasterPluginNotInTheLoadOrderShouldAppendItToTheLoadOrder) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_EQ(3, loadOrder.getLoadOrder().size());

            EXPECT_NO_THROW(loadOrder.activate(blankEsp, gameHandle));
            EXPECT_EQ(3, loadOrder.getPosition(blankEsp));
            EXPECT_TRUE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, activatingAMasterPluginNotInTheLoadOrderShouldInsertItAfterAllOtherMasters) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_EQ(3, loadOrder.getLoadOrder().size());

            EXPECT_NO_THROW(loadOrder.activate(blankDifferentEsm, gameHandle));
            ASSERT_EQ(4, loadOrder.getLoadOrder().size());
            EXPECT_EQ(2, loadOrder.getPosition(blankDifferentEsm));
            EXPECT_TRUE(loadOrder.isActive(blankDifferentEsm));
        }

        TEST_P(LoadOrderTest, activatingTheGameMasterFileNotInTheLoadOrderShouldInsertItAtTheBeginningForTextfileBasedGamesAndAfterAllOtherMastersOtherwise) {
            ASSERT_NO_THROW(loadOrder.activate(blankEsm, gameHandle));

            EXPECT_NO_THROW(loadOrder.activate(gameHandle.MasterFile(), gameHandle));
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_EQ(0, loadOrder.getPosition(gameHandle.MasterFile()));
            else
                EXPECT_EQ(1, loadOrder.getPosition(gameHandle.MasterFile()));
        }

        TEST_P(LoadOrderTest, activatingAPluginInTheLoadOrderShouldSetItToActive) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_FALSE(loadOrder.isActive(blankDifferentEsm));

            EXPECT_NO_THROW(loadOrder.activate(blankDifferentEsm, gameHandle));
            EXPECT_TRUE(loadOrder.isActive(blankDifferentEsm));
        }

        TEST_P(LoadOrderTest, checkingIfAPluginIsActiveShouldBeCaseInsensitive) {
            EXPECT_NO_THROW(loadOrder.activate(blankEsm, gameHandle));
            EXPECT_TRUE(loadOrder.isActive(boost::to_lower_copy(blankEsm)));
        }

        TEST_P(LoadOrderTest, activatingAPluginShouldBeCaseInsensitive) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            EXPECT_NO_THROW(loadOrder.activate(boost::to_lower_copy(blankEsm), gameHandle));

            EXPECT_TRUE(loadOrder.isActive(blankEsm));
            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, activatingAPluginWhen255AreAlreadyActiveShouldThrow) {
            // Create 255 plugins to test active plugins limit with. Do it here
            // because it's too expensive to do for every test.
            for (size_t i = 0; i < 255; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsp, gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
                EXPECT_NO_THROW(loadOrder.activate(std::to_string(i) + ".esp", gameHandle));
            }

            EXPECT_ANY_THROW(loadOrder.activate(blankEsm, gameHandle));

            for (size_t i = 0; i < 255; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, activatingAPluginWhen255AreAlreadyActiveShouldMakeNoChanges) {
            // Create 255 plugins to test active plugins limit with. Do it here
            // because it's too expensive to do for every test.
            for (size_t i = 0; i < 255; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsp, gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
                EXPECT_NO_THROW(loadOrder.activate(std::to_string(i) + ".esp", gameHandle));
            }

            EXPECT_ANY_THROW(loadOrder.activate(blankEsm, gameHandle));
            EXPECT_FALSE(loadOrder.isActive(blankEsm));

            for (size_t i = 0; i < 255; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, deactivatingAPluginNotInTheLoadOrderShouldDoNothing) {
            EXPECT_NO_THROW(loadOrder.deactivate(blankEsp, gameHandle));
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, deactivatingTheGameMasterFileShouldThrowForTextfileLoadOrderGamesAndNotOtherwise) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.deactivate(gameHandle.MasterFile(), gameHandle));
            else
                EXPECT_NO_THROW(loadOrder.deactivate(gameHandle.MasterFile(), gameHandle));
        }

        TEST_P(LoadOrderTest, deactivatingTheGameMasterFileShouldForTextfileLoadOrderGamesShouldMakeNoChanges) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.deactivate(gameHandle.MasterFile(), gameHandle));
                EXPECT_FALSE(loadOrder.isActive(gameHandle.MasterFile()));
            }
        }

        TEST_P(LoadOrderTest, forSkyrimDeactivatingUpdateEsmShouldThrow) {
            if (gameHandle.Id() == LIBLO_GAME_TES5)
                EXPECT_ANY_THROW(loadOrder.deactivate(updateEsm, gameHandle));
        }

        TEST_P(LoadOrderTest, forSkyrimDeactivatingUpdateEsmShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                updateEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_NO_THROW(loadOrder.activate(updateEsm, gameHandle));

            if (gameHandle.Id() == LIBLO_GAME_TES5) {
                EXPECT_ANY_THROW(loadOrder.deactivate(updateEsm, gameHandle));
                EXPECT_TRUE(loadOrder.isActive(updateEsm));
            }
        }

        TEST_P(LoadOrderTest, deactivatingAnInactivePluginShouldHaveNoEffect) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_FALSE(loadOrder.isActive(blankEsm));

            EXPECT_NO_THROW(loadOrder.deactivate(blankEsm, gameHandle));
            EXPECT_FALSE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, deactivatingAnActivePluginShouldMakeItInactive) {
            ASSERT_NO_THROW(loadOrder.activate(blankEsp, gameHandle));
            ASSERT_TRUE(loadOrder.isActive(blankEsp));

            EXPECT_NO_THROW(loadOrder.deactivate(blankEsp, gameHandle));
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnActivePluginShouldKeepItActive) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_NO_THROW(loadOrder.activate(blankEsm, gameHandle));

            loadOrder.setPosition(blankEsm, 2, gameHandle);
            EXPECT_TRUE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnInactivePluginShouldKeepItInactive) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));

            loadOrder.setPosition(blankEsm, 2, gameHandle);
            EXPECT_FALSE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, settingLoadOrderShouldActivateTheGameMasterForTextfileBasedGamesAndNotOtherwise) {
            std::vector<std::string> firstLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder, gameHandle));
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_TRUE(loadOrder.isActive(gameHandle.MasterFile()));
            else
                EXPECT_FALSE(loadOrder.isActive(gameHandle.MasterFile()));
        }

        TEST_P(LoadOrderTest, settingANewLoadOrderShouldRetainTheActiveStateOfPluginsInTheOldLoadOrder) {
            std::vector<std::string> firstLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder, gameHandle));
            ASSERT_NO_THROW(loadOrder.activate(blankEsm, gameHandle));

            std::vector<std::string> secondLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(secondLoadOrder, gameHandle));

            EXPECT_TRUE(loadOrder.isActive(blankEsm));
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
        }
    }
}

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
#include "backend/helpers.h"

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
                nonAsciiEsm("Blàñk.esm"),
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

                // Make sure the non-ASCII plugin exists.
                ASSERT_FALSE(boost::filesystem::exists(gameHandle.PluginsFolder() / nonAsciiEsm));
                ASSERT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsm, gameHandle.PluginsFolder() / nonAsciiEsm));
                ASSERT_TRUE(boost::filesystem::exists(gameHandle.PluginsFolder() / nonAsciiEsm));

                // Morrowind load order files have a slightly different
                // format and a prefix is necessary.
                std::string linePrefix = getActivePluginsFileLinePrefix(GetParam());

                // Write out an active plugins file, making it as invalid as
                // possible for the game to still fix.
                out.open(gameHandle.ActivePluginsFile());
                out << std::endl
                    << '#' << FromUTF8(blankDifferentEsm) << std::endl
                    << linePrefix << FromUTF8(blankEsm) << std::endl
                    << linePrefix << FromUTF8(blankEsp) << std::endl
                    << linePrefix << FromUTF8(nonAsciiEsm) << std::endl
                    << linePrefix << FromUTF8(blankEsm) << std::endl
                    << linePrefix << FromUTF8(invalidPlugin) << std::endl;
                out.close();

                if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                    // Write out the game's load order file, using the valid
                    // version of what's in the active plugins file, plus
                    // additional plugins.
                    out.open(gameHandle.LoadOrderFile());
                    out << gameHandle.MasterFile() << std::endl
                        << nonAsciiEsm << std::endl
                        << blankEsm << std::endl
                        << updateEsm << std::endl
                        << blankDifferentEsm << std::endl
                        << blankEsp << std::endl;
                    out.close();
                }
            }

            inline virtual void TearDown() {
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / invalidPlugin));
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / gameHandle.MasterFile()));
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / updateEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / nonAsciiEsm));

                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.ActivePluginsFile()));
                if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                    ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.LoadOrderFile()));
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

            inline std::string getActivePluginsFileLinePrefix(unsigned int gameId) {
                if (gameId == LIBLO_GAME_TES3)
                    return "GameFile0=";
                else
                    return "";
            }

            LoadOrder loadOrder;
            _lo_game_handle_int gameHandle;

            std::string updateEsm;
            std::string blankEsm;
            std::string blankDifferentEsm;
            std::string blankEsp;
            std::string invalidPlugin;
            std::string missingPlugin;
            std::string nonAsciiEsm;
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

        TEST_P(LoadOrderTest, activatingAPluginWhenMaxNumberAreAlreadyActiveShouldThrow) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsp, gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
                EXPECT_NO_THROW(loadOrder.activate(std::to_string(i) + ".esp", gameHandle));
            }

            EXPECT_ANY_THROW(loadOrder.activate(blankEsm, gameHandle));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, activatingAPluginWhenMaxNumberAreAlreadyActiveShouldMakeNoChanges) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsp, gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
                EXPECT_NO_THROW(loadOrder.activate(std::to_string(i) + ".esp", gameHandle));
            }

            EXPECT_ANY_THROW(loadOrder.activate(blankEsm, gameHandle));
            EXPECT_FALSE(loadOrder.isActive(blankEsm));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
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

        TEST_P(LoadOrderTest, settingInvalidActivePluginsShouldThrow) {
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                updateEsm,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
        }

        TEST_P(LoadOrderTest, settingInvalidActivePluginsShouldMakeNoChanges) {
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                updateEsm,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
            EXPECT_TRUE(loadOrder.getActivePlugins().empty());
        }

        TEST_P(LoadOrderTest, settingMoreThanMaxNumberActivePluginsShouldThrow) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                updateEsm,
            });
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsp, gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
                activePlugins.insert(std::to_string(i) + ".esp");
            }

            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, settingMoreThanMaxNumberActivePluginsShouldMakeNoChanges) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                updateEsm,
            });
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(gameHandle.PluginsFolder() / blankEsp, gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
                activePlugins.insert(std::to_string(i) + ".esp");
            }

            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
            EXPECT_TRUE(loadOrder.getActivePlugins().empty());

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutGameMasterShouldThrowForTextfileBasedGamesAndNotOtherwise) {
            std::unordered_set<std::string> activePlugins({
                updateEsm,
                blankEsm,
            });
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
            else
                EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutGameMasterShouldMakeNoChangesForTextfileBasedGames) {
            std::unordered_set<std::string> activePlugins({
                updateEsm,
                blankEsm,
            });
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
            }
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutUpdateEsmWhenItExistsShouldThrowForSkyrimAndNotOtherwise) {
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                blankEsm,
            });
            if (gameHandle.Id() == LIBLO_GAME_TES5)
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
            else
                EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutUpdateEsmWhenItExistsShouldMakeNoChangesForSkyrim) {
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                blankEsm,
            });
            if (gameHandle.Id() == LIBLO_GAME_TES5) {
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
            }
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutUpdateEsmWhenItDoesNotExistShouldNotThrow) {
            ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / updateEsm));

            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                blankEsm,
            });
            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));
        }

        TEST_P(LoadOrderTest, settingActivePluginsShouldDeactivateAnyOthersInLoadOrderCaseInsensitively) {
            std::vector<std::string> validLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder, gameHandle));
            ASSERT_NO_THROW(loadOrder.activate(blankEsp, gameHandle));

            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                updateEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));

            std::unordered_set<std::string> expectedActivePlugins({
                gameHandle.MasterFile(),
                updateEsm,
                blankEsm,
            });
            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, settingActivePluginsNotInLoadOrderShouldAddThem) {
            std::unordered_set<std::string> activePlugins({
                gameHandle.MasterFile(),
                updateEsm,
                blankEsm,
            });
            std::vector<std::string> expectedLoadOrder({
                gameHandle.MasterFile(),
                updateEsm,
                blankEsm,
            });
            ASSERT_TRUE(loadOrder.getLoadOrder().empty());

            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins, gameHandle));

            std::vector<std::string> newLoadOrder(loadOrder.getLoadOrder());
            EXPECT_EQ(3, newLoadOrder.size());
            EXPECT_EQ(1, count(std::begin(newLoadOrder), std::end(newLoadOrder), gameHandle.MasterFile()));
            EXPECT_EQ(1, count(std::begin(newLoadOrder), std::end(newLoadOrder), updateEsm));
            EXPECT_EQ(1, count(std::begin(newLoadOrder), std::end(newLoadOrder), blankEsm));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTimestampBasedGames) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                EXPECT_TRUE(LoadOrder::isSynchronised(gameHandle));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesIfLoadOrderFileDoesNotExist) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.LoadOrderFile()));

            EXPECT_TRUE(LoadOrder::isSynchronised(gameHandle));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesIfActivePluginsFileDoesNotExist) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.ActivePluginsFile()));

            EXPECT_TRUE(LoadOrder::isSynchronised(gameHandle));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesWhenLoadOrderAndActivePluginsFileContentsAreEquivalent) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            EXPECT_TRUE(LoadOrder::isSynchronised(gameHandle));
        }

        TEST_P(LoadOrderTest, isNotSynchronisedForTextfileBasedGamesWhenLoadOrderAndActivePluginsFileContentsAreNotEquivalent) {
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            boost::filesystem::ofstream out(gameHandle.LoadOrderFile(), std::ios_base::app);
            out << blankEsm << std::endl;

            EXPECT_FALSE(LoadOrder::isSynchronised(gameHandle));
        }
    }
}

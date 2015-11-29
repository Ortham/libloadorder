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
#include "backend/LoadOrder.h"
#include "backend/helpers.h"

#include <thread>
#include <chrono>

namespace liblo {
    namespace test {
        class LoadOrderTest : public GameTest {
        protected:
            inline LoadOrderTest() :
                blankEsm("Blank.esm"),
                blankDifferentEsm("Blank - Different.esm"),
                blankMasterDependentEsm("Blank - Master Dependent.esm"),
                blankDifferentMasterDependentEsm("Blank - Different Master Dependent.esm"),
                blankEsp("Blank.esp"),
                blankDifferentEsp("Blank - Different.esp"),
                blankMasterDependentEsp("Blank - Master Dependent.esp"),
                blankDifferentMasterDependentEsp("Blank - Different Master Dependent.esp"),
                blankPluginDependentEsp("Blank - Plugin Dependent.esp"),
                blankDifferentPluginDependentEsp("Blank - Different Plugin Dependent.esp"),
                invalidPlugin("NotAPlugin.esm"),
                missingPlugin("missing.esm"),
                updateEsm("Update.esm"),
                nonAsciiEsm("Blàñk.esm"),
                gameSettings(GetParam(), gamePath, localPath),
                loadOrder(gameSettings) {}

            inline virtual void SetUp() {
                GameTest::SetUp();

                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankMasterDependentEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentMasterDependentEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankMasterDependentEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentMasterDependentEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentPluginDependentEsp));
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / missingPlugin));

                // Write out an non-empty, non-plugin file.
                boost::filesystem::ofstream out(pluginsPath / invalidPlugin);
                out << "This isn't a valid plugin file.";
                out.close();
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / invalidPlugin));

                // Make sure the game master file exists.
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / masterFile));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / masterFile));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / masterFile));

                // Make sure Update.esm exists.
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / updateEsm));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / updateEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / updateEsm));

                // Make sure the non-ASCII plugin exists.
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / nonAsciiEsm));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / nonAsciiEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / nonAsciiEsm));

                // Morrowind load order files have a slightly different
                // format and a prefix is necessary.
                std::string linePrefix = getActivePluginsFileLinePrefix();

                // Write out an active plugins file, making it as invalid as
                // possible for the game to still fix.
                out.open(gameSettings.getActivePluginsFile());
                out << std::endl
                    << '#' << utf8ToWindows1252(blankDifferentEsm) << std::endl
                    << linePrefix << utf8ToWindows1252(blankEsm) << std::endl
                    << linePrefix << utf8ToWindows1252(blankEsp) << std::endl
                    << linePrefix << utf8ToWindows1252(nonAsciiEsm) << std::endl
                    << linePrefix << utf8ToWindows1252(blankEsm) << std::endl
                    << linePrefix << utf8ToWindows1252(invalidPlugin) << std::endl;
                out.close();

                if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                    // Write out the game's load order file, using the valid
                    // version of what's in the active plugins file, plus
                    // additional plugins.
                    out.open(gameSettings.getLoadOrderFile());
                    out << nonAsciiEsm << std::endl
                        << masterFile << std::endl
                        << blankDifferentEsm << std::endl
                        << blankEsm << std::endl
                        << updateEsm << std::endl
                        << blankEsp << std::endl;
                    out.close();
                }
                else {
                    // Set load order using timestamps
                    std::vector<std::string> plugins({
                        masterFile,
                        blankEsm,
                        blankDifferentEsm,
                        blankMasterDependentEsm,
                        blankDifferentMasterDependentEsm,
                        nonAsciiEsm,
                        blankEsp,  // Put a plugin before master to test fixup.
                        updateEsm,
                        blankDifferentEsp,
                        blankMasterDependentEsp,
                        blankDifferentMasterDependentEsp,
                        blankPluginDependentEsp,
                        blankDifferentPluginDependentEsp,
                    });
                    time_t modificationTime = time(NULL);  // Current time.
                    for (const auto &plugin : plugins) {
                        boost::filesystem::last_write_time(pluginsPath / plugin, modificationTime);
                        modificationTime += 60;
                    }
                }
            }

            inline virtual void TearDown() {
                GameTest::TearDown();

                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / invalidPlugin));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / masterFile));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / updateEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / nonAsciiEsm));

                ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getActivePluginsFile()));
                if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                    ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getLoadOrderFile()));
            }

            inline std::string getActivePluginsFileLinePrefix() {
                if (GetParam() == LIBLO_GAME_TES3)
                    return "GameFile0=";
                else
                    return "";
            }

            const GameSettings gameSettings;
            LoadOrder loadOrder;

            std::string blankEsm;
            std::string blankDifferentEsm;
            std::string blankMasterDependentEsm;
            std::string blankDifferentMasterDependentEsm;
            std::string blankEsp;
            std::string blankDifferentEsp;
            std::string blankMasterDependentEsp;
            std::string blankDifferentMasterDependentEsp;
            std::string blankPluginDependentEsp;
            std::string blankDifferentPluginDependentEsp;

            std::string invalidPlugin;
            std::string missingPlugin;
            std::string updateEsm;
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
                                LIBLO_GAME_FNV,
                                LIBLO_GAME_FO4));

        TEST_P(LoadOrderTest, settingAValidLoadOrderShouldNotThrow) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            EXPECT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithPluginsBeforeMastersShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                blankEsp,
                blankDifferentEsm,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithPluginsBeforeMastersShouldMakeNoChanges) {
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                blankEsp,
                blankDifferentEsm,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithAnInvalidPluginShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithAnInvalidPluginShouldMakeNoChanges) {
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithACaseInsensitiveDuplicatePluginShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                blankEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithACaseInsensitiveDuplicatePluginShouldMakeNoChanges) {
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                blankEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, settingThenGettingLoadOrderShouldReturnTheSetLoadOrder) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingTheLoadOrderTwiceShouldReplaceTheFirstLoadOrder) {
            std::vector<std::string> firstLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            std::vector<std::string> secondLoadOrder({
                masterFile,
                blankDifferentEsm,
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder));
            ASSERT_NO_THROW(loadOrder.setLoadOrder(secondLoadOrder));

            EXPECT_EQ(secondLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingAnInvalidLoadOrderShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            std::vector<std::string> invalidLoadOrder({
                masterFile,
                blankEsp,
                blankDifferentEsm,
            });

            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder));

            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithTheGameMasterNotAtTheBeginningShouldFailForTextfileLoadOrderGamesAndSucceedOtherwise) {
            std::vector<std::string> plugins({
                blankEsm,
                masterFile,
            });
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setLoadOrder(plugins));
            else
                EXPECT_NO_THROW(loadOrder.setLoadOrder(plugins));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithTheGameMasterNotAtTheBeginningShouldMakeNoChangesForTextfileLoadOrderGames) {
            std::vector<std::string> plugins({
                blankEsm,
                masterFile,
            });
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setLoadOrder(plugins));
                EXPECT_TRUE(loadOrder.getLoadOrder().empty());
            }
        }

        TEST_P(LoadOrderTest, positionOfAMissingPluginShouldEqualTheLoadOrderSize) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_EQ(validLoadOrder.size(), loadOrder.getPosition(missingPlugin));
        }

        TEST_P(LoadOrderTest, positionOfAPluginShouldBeEqualToItsLoadOrderIndex) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_EQ(1, loadOrder.getPosition(blankEsm));
        }

        TEST_P(LoadOrderTest, gettingAPluginsPositionShouldBeCaseInsensitive) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_EQ(1, loadOrder.getPosition(boost::to_lower_copy(blankEsm)));
        }

        TEST_P(LoadOrderTest, gettingPluginAtAPositionGreaterThanTheHighestIndexShouldThrow) {
            EXPECT_ANY_THROW(loadOrder.getPluginAtPosition(0));
        }

        TEST_P(LoadOrderTest, gettingPluginAtAValidPositionShouldReturnItsLoadOrderIndex) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_EQ(blankEsm, loadOrder.getPluginAtPosition(1));
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstShouldThrowForTextfileLoadOrderGamesAndNotOtherwise) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 0));
            else {
                EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 0));
            }
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstForATextfileBasedGameShouldMakeNoChanges) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 0));
                EXPECT_TRUE(loadOrder.getLoadOrder().empty());
            }
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstForATimestampBasedGameShouldSucceed) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 0));
                EXPECT_FALSE(loadOrder.getLoadOrder().empty());
                EXPECT_EQ(0, loadOrder.getPosition(blankEsm));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginShouldThrowForTextfileLoadOrderGamesAndNotOtherwise) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setPosition(masterFile, 1));
            else
                EXPECT_NO_THROW(loadOrder.setPosition(masterFile, 1));
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginShouldMakeNoChangesForTextfileLoadOrderGames) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setPosition(masterFile, 1));
                EXPECT_EQ(0, loadOrder.getPosition(masterFile));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginForATextfileBasedGameShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                ASSERT_ANY_THROW(loadOrder.setPosition(masterFile, 1));
                EXPECT_EQ(blankEsm, loadOrder.getPluginAtPosition(1));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginForATimestampBasedGameShouldSucceed) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                ASSERT_NO_THROW(loadOrder.setPosition(masterFile, 1));
                EXPECT_EQ(blankEsm, loadOrder.getPluginAtPosition(0));
                EXPECT_EQ(masterFile, loadOrder.getPluginAtPosition(1));
            }
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnInvalidPluginShouldThrow) {
            ASSERT_NO_THROW(loadOrder.setPosition(masterFile, 0));

            EXPECT_ANY_THROW(loadOrder.setPosition(invalidPlugin, 1));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnInvalidPluginShouldMakeNoChanges) {
            ASSERT_NO_THROW(loadOrder.setPosition(masterFile, 0));

            ASSERT_ANY_THROW(loadOrder.setPosition(invalidPlugin, 1));
            EXPECT_EQ(1, loadOrder.getLoadOrder().size());
        }

        TEST_P(LoadOrderTest, settingThePositionOfAPluginToGreaterThanTheLoadOrderSizeShouldPutThePluginAtTheEnd) {
            ASSERT_NO_THROW(loadOrder.setPosition(masterFile, 0));

            EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 2));
            EXPECT_EQ(2, loadOrder.getLoadOrder().size());
            EXPECT_EQ(1, loadOrder.getPosition(blankEsm));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAPluginShouldBeCaseInsensitive) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_NO_THROW(loadOrder.setPosition(boost::to_lower_copy(blankEsm), 2));

            std::vector<std::string> expectedLoadOrder({
                masterFile,
                blankDifferentEsm,
                blankEsm,
            });
            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingANonMasterPluginToLoadBeforeAMasterPluginShouldThrow) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsp, 1));
        }

        TEST_P(LoadOrderTest, settingANonMasterPluginToLoadBeforeAMasterPluginShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsp, 1));
            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, settingAMasterToLoadAfterAPluginShouldThrow) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 2));
        }

        TEST_P(LoadOrderTest, settingAMasterToLoadAfterAPluginShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 2));
            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, clearingLoadOrderShouldRemoveAllPluginsFromTheLoadOrder) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_NO_THROW(loadOrder.clear());
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, checkingIfAnInactivePluginIsActiveShouldReturnFalse) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_FALSE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, checkingIfAPluginNotInTheLoadOrderIsActiveShouldReturnFalse) {
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, activatingAnInvalidPluginShouldThrow) {
            EXPECT_ANY_THROW(loadOrder.activate(invalidPlugin));
        }

        TEST_P(LoadOrderTest, activatingANonMasterPluginNotInTheLoadOrderShouldAppendItToTheLoadOrder) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_EQ(3, loadOrder.getLoadOrder().size());

            EXPECT_NO_THROW(loadOrder.activate(blankEsp));
            EXPECT_EQ(3, loadOrder.getPosition(blankEsp));
            EXPECT_TRUE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, activatingAMasterPluginNotInTheLoadOrderShouldInsertItAfterAllOtherMasters) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_EQ(3, loadOrder.getLoadOrder().size());

            EXPECT_NO_THROW(loadOrder.activate(blankDifferentEsm));
            ASSERT_EQ(4, loadOrder.getLoadOrder().size());
            EXPECT_EQ(2, loadOrder.getPosition(blankDifferentEsm));
            EXPECT_TRUE(loadOrder.isActive(blankDifferentEsm));
        }

        TEST_P(LoadOrderTest, activatingTheGameMasterFileNotInTheLoadOrderShouldInsertItAtTheBeginningForTextfileBasedGamesAndAfterAllOtherMastersOtherwise) {
            ASSERT_NO_THROW(loadOrder.activate(blankEsm));

            EXPECT_NO_THROW(loadOrder.activate(masterFile));
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_EQ(0, loadOrder.getPosition(masterFile));
            else
                EXPECT_EQ(1, loadOrder.getPosition(masterFile));
        }

        TEST_P(LoadOrderTest, activatingAPluginInTheLoadOrderShouldSetItToActive) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_FALSE(loadOrder.isActive(blankDifferentEsm));

            EXPECT_NO_THROW(loadOrder.activate(blankDifferentEsm));
            EXPECT_TRUE(loadOrder.isActive(blankDifferentEsm));
        }

        TEST_P(LoadOrderTest, checkingIfAPluginIsActiveShouldBeCaseInsensitive) {
            EXPECT_NO_THROW(loadOrder.activate(blankEsm));
            EXPECT_TRUE(loadOrder.isActive(boost::to_lower_copy(blankEsm)));
        }

        TEST_P(LoadOrderTest, activatingAPluginShouldBeCaseInsensitive) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_NO_THROW(loadOrder.activate(boost::to_lower_copy(blankEsm)));

            EXPECT_TRUE(loadOrder.isActive(blankEsm));
            EXPECT_EQ(validLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, activatingAPluginWhenMaxNumberAreAlreadyActiveShouldThrow) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / (std::to_string(i) + ".esp")));
                EXPECT_NO_THROW(loadOrder.activate(std::to_string(i) + ".esp"));
            }

            EXPECT_ANY_THROW(loadOrder.activate(blankEsm));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, activatingAPluginWhenMaxNumberAreAlreadyActiveShouldMakeNoChanges) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / (std::to_string(i) + ".esp")));
                EXPECT_NO_THROW(loadOrder.activate(std::to_string(i) + ".esp"));
            }

            EXPECT_ANY_THROW(loadOrder.activate(blankEsm));
            EXPECT_FALSE(loadOrder.isActive(blankEsm));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, deactivatingAPluginNotInTheLoadOrderShouldDoNothing) {
            EXPECT_NO_THROW(loadOrder.deactivate(blankEsp));
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
            EXPECT_TRUE(loadOrder.getLoadOrder().empty());
        }

        TEST_P(LoadOrderTest, deactivatingTheGameMasterFileShouldThrowForTextfileLoadOrderGamesAndNotOtherwise) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.deactivate(masterFile));
            else
                EXPECT_NO_THROW(loadOrder.deactivate(masterFile));
        }

        TEST_P(LoadOrderTest, deactivatingTheGameMasterFileShouldForTextfileLoadOrderGamesShouldMakeNoChanges) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.deactivate(masterFile));
                EXPECT_FALSE(loadOrder.isActive(masterFile));
            }
        }

        TEST_P(LoadOrderTest, forSkyrimDeactivatingUpdateEsmShouldThrow) {
            if (GetParam() == LIBLO_GAME_TES5)
                EXPECT_ANY_THROW(loadOrder.deactivate(updateEsm));
        }

        TEST_P(LoadOrderTest, forSkyrimDeactivatingUpdateEsmShouldMakeNoChanges) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                updateEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_NO_THROW(loadOrder.activate(updateEsm));

            if (GetParam() == LIBLO_GAME_TES5) {
                EXPECT_ANY_THROW(loadOrder.deactivate(updateEsm));
                EXPECT_TRUE(loadOrder.isActive(updateEsm));
            }
        }

        TEST_P(LoadOrderTest, deactivatingAnInactivePluginShouldHaveNoEffect) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_FALSE(loadOrder.isActive(blankEsm));

            EXPECT_NO_THROW(loadOrder.deactivate(blankEsm));
            EXPECT_FALSE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, deactivatingAnActivePluginShouldMakeItInactive) {
            ASSERT_NO_THROW(loadOrder.activate(blankEsp));
            ASSERT_TRUE(loadOrder.isActive(blankEsp));

            EXPECT_NO_THROW(loadOrder.deactivate(blankEsp));
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnActivePluginShouldKeepItActive) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_NO_THROW(loadOrder.activate(blankEsm));

            loadOrder.setPosition(blankEsm, 2);
            EXPECT_TRUE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, settingThePositionOfAnInactivePluginShouldKeepItInactive) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            loadOrder.setPosition(blankEsm, 2);
            EXPECT_FALSE(loadOrder.isActive(blankEsm));
        }

        TEST_P(LoadOrderTest, settingLoadOrderShouldActivateTheGameMasterForTextfileBasedGamesAndNotOtherwise) {
            std::vector<std::string> firstLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder));
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_TRUE(loadOrder.isActive(masterFile));
            else
                EXPECT_FALSE(loadOrder.isActive(masterFile));
        }

        TEST_P(LoadOrderTest, settingANewLoadOrderShouldRetainTheActiveStateOfPluginsInTheOldLoadOrder) {
            std::vector<std::string> firstLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder));
            ASSERT_NO_THROW(loadOrder.activate(blankEsm));

            std::vector<std::string> secondLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(secondLoadOrder));

            EXPECT_TRUE(loadOrder.isActive(blankEsm));
            EXPECT_FALSE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, settingInvalidActivePluginsShouldThrow) {
            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingInvalidActivePluginsShouldMakeNoChanges) {
            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
            EXPECT_TRUE(loadOrder.getActivePlugins().empty());
        }

        TEST_P(LoadOrderTest, settingMoreThanMaxNumberActivePluginsShouldThrow) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
            });
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / (std::to_string(i) + ".esp")));
                activePlugins.insert(std::to_string(i) + ".esp");
            }

            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, settingMoreThanMaxNumberActivePluginsShouldMakeNoChanges) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
            });
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / (std::to_string(i) + ".esp")));
                activePlugins.insert(std::to_string(i) + ".esp");
            }

            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
            EXPECT_TRUE(loadOrder.getActivePlugins().empty());

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutGameMasterShouldThrowForTextfileBasedGamesAndNotOtherwise) {
            std::unordered_set<std::string> activePlugins({
                updateEsm,
                blankEsm,
            });
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
            else
                EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutGameMasterShouldMakeNoChangesForTextfileBasedGames) {
            std::unordered_set<std::string> activePlugins({
                updateEsm,
                blankEsm,
            });
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
            }
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutUpdateEsmWhenItExistsShouldThrowForSkyrimAndNotOtherwise) {
            std::unordered_set<std::string> activePlugins({
                masterFile,
                blankEsm,
            });
            if (GetParam() == LIBLO_GAME_TES5)
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
            else
                EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutUpdateEsmWhenItExistsShouldMakeNoChangesForSkyrim) {
            std::unordered_set<std::string> activePlugins({
                masterFile,
                blankEsm,
            });
            if (GetParam() == LIBLO_GAME_TES5) {
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
            }
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutUpdateEsmWhenItDoesNotExistShouldNotThrow) {
            ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / updateEsm));

            std::unordered_set<std::string> activePlugins({
                masterFile,
                blankEsm,
            });
            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingActivePluginsShouldDeactivateAnyOthersInLoadOrderCaseInsensitively) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));
            ASSERT_NO_THROW(loadOrder.activate(blankEsp));

            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));

            std::unordered_set<std::string> expectedActivePlugins({
                masterFile,
                updateEsm,
                blankEsm,
            });
            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, settingActivePluginsNotInLoadOrderShouldAddThem) {
            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
                blankEsm,
            });
            std::vector<std::string> expectedLoadOrder({
                masterFile,
                updateEsm,
                blankEsm,
            });
            ASSERT_TRUE(loadOrder.getLoadOrder().empty());

            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));

            std::vector<std::string> newLoadOrder(loadOrder.getLoadOrder());
            EXPECT_EQ(3, newLoadOrder.size());
            EXPECT_EQ(1, count(std::begin(newLoadOrder), std::end(newLoadOrder), masterFile));
            EXPECT_EQ(1, count(std::begin(newLoadOrder), std::end(newLoadOrder), updateEsm));
            EXPECT_EQ(1, count(std::begin(newLoadOrder), std::end(newLoadOrder), blankEsm));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTimestampBasedGames) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesIfLoadOrderFileDoesNotExist) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getLoadOrderFile()));

            EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesIfActivePluginsFileDoesNotExist) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getActivePluginsFile()));

            EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesWhenLoadOrderAndActivePluginsFileContentsAreEquivalent) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isNotSynchronisedForTextfileBasedGamesWhenLoadOrderAndActivePluginsFileContentsAreNotEquivalent) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
                return;

            boost::filesystem::ofstream out(gameSettings.getLoadOrderFile(), std::ios_base::trunc);
            out << blankEsm << std::endl;

            EXPECT_FALSE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, loadingDataShouldNotThrowIfActivePluginsFileDoesNotExist) {
            ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getActivePluginsFile()));

            EXPECT_NO_THROW(loadOrder.load());
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateNoPluginsIfActivePluginsFileDoesNotExist) {
            ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getActivePluginsFile()));

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_TRUE(loadOrder.getActivePlugins().empty());
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateTheGameMasterForTextfileBasedGamesAndNotOtherwise) {
            EXPECT_NO_THROW(loadOrder.load());

            int count = loadOrder.getActivePlugins().count(masterFile);
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                EXPECT_EQ(1, count);
            else
                EXPECT_EQ(0, count);
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateUpdateEsmWhenItExistsForSkyrimAndNotOtherwise) {
            EXPECT_NO_THROW(loadOrder.load());

            int count = loadOrder.getActivePlugins().count(updateEsm);
            if (GetParam() == LIBLO_GAME_TES5)
                EXPECT_EQ(1, count);
            else
                EXPECT_EQ(0, count);
        }

        TEST_P(LoadOrderTest, loadingDataShouldNotActivateUpdateEsmWhenItDoesNotExist) {
            ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / updateEsm));

            EXPECT_NO_THROW(loadOrder.load());

            EXPECT_EQ(0, loadOrder.getActivePlugins().count(updateEsm));
        }

        TEST_P(LoadOrderTest, loadingDataWithMoreThanMaxNumberActivePluginsShouldStopWhenMaxIsReached) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::unordered_set<std::string> expectedActivePlugins;

            std::string linePrefix = getActivePluginsFileLinePrefix();
            boost::filesystem::ofstream out(gameSettings.getActivePluginsFile());

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                out << linePrefix << utf8ToWindows1252(masterFile) << std::endl;
                expectedActivePlugins.insert(masterFile);

                if (GetParam() == LIBLO_GAME_TES5) {
                    out << linePrefix << utf8ToWindows1252(updateEsm) << std::endl;
                    expectedActivePlugins.insert(updateEsm);
                }
            }

            for (size_t i = 0; i < LoadOrder::maxActivePlugins - expectedActivePlugins.size(); ++i) {
                std::string filename = std::to_string(i) + ".esp";
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / filename));
                out << linePrefix << filename << std::endl;
                expectedActivePlugins.insert(filename);
            }
            out.close();

            EXPECT_NO_THROW(loadOrder.load());

            EXPECT_EQ(expectedActivePlugins.size(), loadOrder.getActivePlugins().size());
            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, loadingDataShouldFixInvalidDataWhenReadingActivePluginsFile) {
            EXPECT_NO_THROW(loadOrder.load());

            std::unordered_set<std::string> expectedActivePlugins({
                nonAsciiEsm,
                blankEsm,
                blankEsp,
            });
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                expectedActivePlugins.insert(masterFile);

                if (GetParam() == LIBLO_GAME_TES5)
                    expectedActivePlugins.insert(updateEsm);
            }
            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, loadingDataShouldPreferLoadOrderFileForTextfileBasedGamesOtherwiseUseTimestamps) {
            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder;
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                expectedLoadOrder = std::vector<std::string>({
                    masterFile,
                    nonAsciiEsm,
                    blankDifferentEsm,
                    blankEsm,
                    updateEsm,
                });
                EXPECT_TRUE(equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
            }
            else {
                expectedLoadOrder = std::vector<std::string>({
                    masterFile,
                    blankEsm,
                    blankDifferentEsm,
                    blankMasterDependentEsm,
                    blankDifferentMasterDependentEsm,
                    nonAsciiEsm,
                    updateEsm,
                    blankEsp,
                    blankDifferentEsp,
                    blankMasterDependentEsp,
                    blankDifferentMasterDependentEsp,
                    blankPluginDependentEsp,
                    blankDifferentPluginDependentEsp,
                });
                EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
            }
        }

        TEST_P(LoadOrderTest, loadingDataShouldFallBackToActivePluginsFileForTextfileBasedGamesOtherwiseUseTimestamps) {
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
                ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getLoadOrderFile()));

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder;
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                expectedLoadOrder = std::vector<std::string>({
                    masterFile,
                    nonAsciiEsm,
                    blankEsm,
                });
                if (GetParam() == LIBLO_GAME_TES5)
                    expectedLoadOrder.push_back(updateEsm);

                EXPECT_TRUE(equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
            }
            else {
                expectedLoadOrder = std::vector<std::string>({
                    masterFile,
                    blankEsm,
                    blankDifferentEsm,
                    blankMasterDependentEsm,
                    blankDifferentMasterDependentEsm,
                    nonAsciiEsm,
                    updateEsm,
                    blankEsp,
                    blankDifferentEsp,
                    blankMasterDependentEsp,
                    blankDifferentMasterDependentEsp,
                    blankPluginDependentEsp,
                    blankDifferentPluginDependentEsp,
                });
                EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
            }
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldDiscardTheDataRead) {
            ASSERT_NO_THROW(loadOrder.load());

            std::string linePrefix = getActivePluginsFileLinePrefix();
            boost::filesystem::ofstream out(gameSettings.getActivePluginsFile(), std::ios_base::trunc);
            out << linePrefix << utf8ToWindows1252(blankEsp) << std::endl;
            out.close();

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                out.open(gameSettings.getLoadOrderFile(), std::ios_base::trunc);
                out << blankDifferentEsm << std::endl;
                out.close();
            }

            EXPECT_NO_THROW(loadOrder.load());

            std::unordered_set<std::string> expectedActivePlugins({
                blankEsp,
            });
            std::vector<std::string> expectedLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
                blankMasterDependentEsm,
                blankDifferentMasterDependentEsm,
                nonAsciiEsm,
                updateEsm,
                blankEsp,
                blankDifferentEsp,
                blankMasterDependentEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
                blankDifferentPluginDependentEsp,
            });
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
                EXPECT_NE(expectedLoadOrder, loadOrder.getLoadOrder());
                EXPECT_TRUE(is_permutation(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));

                expectedActivePlugins.insert(masterFile);

                if (GetParam() == LIBLO_GAME_TES5)
                    expectedActivePlugins.insert(updateEsm);
            }
            else {
                EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
            }

            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, savingShouldSetTimestampsForTimestampBasedGamesAndWriteToLoadOrderAndActivePluginsFilesOtherwise) {
            std::vector<std::string> plugins({
                masterFile,
                blankEsm,
                blankMasterDependentEsm,
                blankDifferentEsm,
                blankDifferentMasterDependentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(plugins));

            EXPECT_NO_THROW(loadOrder.save());

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_TRUE(equal(begin(plugins), end(plugins), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, savingShouldWriteActivePluginsToActivePluginsFile) {
            std::unordered_set<std::string> activePlugins({
                masterFile,
                updateEsm,
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setActivePlugins(activePlugins));

            EXPECT_NO_THROW(loadOrder.save());

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_EQ(activePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadOrderIsEmpty) {
            ASSERT_TRUE(loadOrder.getLoadOrder().empty());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldNotDetectFilesystemChangesIfLoadedAndPluginsFolderIsUnchanged) {
            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            EXPECT_FALSE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadedAndPluginsFolderIsChanged) {
            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));
            ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / updateEsm));

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadedAndPluginsFolderTimetampIsSetToOlderTime) {
            ASSERT_NO_THROW(loadOrder.load());

            time_t currentModTime = boost::filesystem::last_write_time(pluginsPath);
            EXPECT_NO_THROW(boost::filesystem::last_write_time(pluginsPath, currentModTime - 1));

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldNotDetectFilesystemChangesIfLoadedAndActivePluginsFileIsUnchanged) {
            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            EXPECT_FALSE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadedAndActivePluginsFileIsChanged) {
            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            boost::filesystem::ofstream out(gameSettings.getActivePluginsFile());
            out << std::endl;
            out.close();

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadedAndActivePluginsTimetampIsSetToOlderTime) {
            ASSERT_NO_THROW(loadOrder.load());

            time_t currentModTime = boost::filesystem::last_write_time(gameSettings.getActivePluginsFile());
            EXPECT_NO_THROW(boost::filesystem::last_write_time(gameSettings.getActivePluginsFile(), currentModTime - 1));

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldNotDetectFilesystemChangesIfLoadedAndLoadOrderFileIsUnchangedForTextfileBasedGames) {
            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            EXPECT_FALSE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadedAndLoadOrderFileIsChangedForTextfileBasedGames) {
            if (gameSettings.getLoadOrderMethod() != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            boost::filesystem::ofstream out(gameSettings.getLoadOrderFile());
            out << std::endl;
            out.close();

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfLoadedAndLoadOrderFileTimetampIsSetToOlderTimeForTextfileBasedGames) {
            if (gameSettings.getLoadOrderMethod() != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(loadOrder.load());

            time_t currentModTime = boost::filesystem::last_write_time(gameSettings.getLoadOrderFile());
            EXPECT_NO_THROW(boost::filesystem::last_write_time(gameSettings.getLoadOrderFile(), currentModTime - 1));

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfAPluginIsEdited) {
            ASSERT_NO_THROW(loadOrder.load());
            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            boost::filesystem::ofstream out(pluginsPath / updateEsm);
            out << std::endl;
            out.close();

            EXPECT_TRUE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfNoChangesAreMadeButActivePluginsFileAndPluginsFolderTimestampsAreDifferent) {
            time_t currentModTime = boost::filesystem::last_write_time(pluginsPath);
            EXPECT_NO_THROW(boost::filesystem::last_write_time(gameSettings.getActivePluginsFile(), currentModTime + 2));

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_FALSE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfNoChangesAreMadeButLoadOrderFileAndPluginsFolderTimestampsAreDifferent) {
            if (gameSettings.getLoadOrderMethod() != LIBLO_METHOD_TEXTFILE)
                return;

            time_t currentModTime = boost::filesystem::last_write_time(pluginsPath);
            EXPECT_NO_THROW(boost::filesystem::last_write_time(gameSettings.getLoadOrderFile(), currentModTime + 2));

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_FALSE(loadOrder.hasFilesystemChanged());
        }

        TEST_P(LoadOrderTest, shouldDetectFilesystemChangesIfNoChangesAreMadeButLoadOrderAndActivePluginsFilesAreDifferent) {
            if (gameSettings.getLoadOrderMethod() != LIBLO_METHOD_TEXTFILE)
                return;

            time_t currentModTime = boost::filesystem::last_write_time(gameSettings.getActivePluginsFile());
            EXPECT_NO_THROW(boost::filesystem::last_write_time(gameSettings.getLoadOrderFile(), currentModTime + 2));

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_FALSE(loadOrder.hasFilesystemChanged());
        }
    }
}

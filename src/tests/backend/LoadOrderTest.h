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

#include "backend/GameSettings.h"
#include "backend/LoadOrder.h"
#include "backend/helpers.h"
#include "libloadorder/constants.h"
#include "tests/GameTest.h"

#include <thread>
#include <chrono>

#include <boost/algorithm/string.hpp>

namespace liblo {
    namespace test {
        class LoadOrderTest : public GameTest {
        protected:
            inline LoadOrderTest() :
                blankMasterDependentEsm("Blank - Master Dependent.esm"),
                blankDifferentMasterDependentEsm("Blank - Different Master Dependent.esm"),
                blankEsp("Blank.esp"),
                blankDifferentEsp("Blank - Different.esp"),
                blankMasterDependentEsp("Blank - Master Dependent.esp"),
                blankDifferentMasterDependentEsp("Blank - Different Master Dependent.esp"),
                blankPluginDependentEsp("Blank - Plugin Dependent.esp"),
                blankDifferentPluginDependentEsp("Blank - Different Plugin Dependent.esp"),
                hearthfiresEsm("Hearthfires.esm"),
                dawnguardEsm("Dawnguard.esm"),
                dragonbornEsm("Dragonborn.esm"),
                missingPlugin("missing.esm"),
                updateEsm("Update.esm"),
                nonAsciiEsm("Blàñk.esm"),
                gameSettings(GetParam(), gamePath, localPath),
                loadOrder(gameSettings) {}

            inline virtual void SetUp() {
                GameTest::SetUp();

                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankMasterDependentEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentMasterDependentEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankMasterDependentEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentMasterDependentEsp));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentPluginDependentEsp));
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / missingPlugin));

                // Make sure the non-ASCII plugin exists.
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / nonAsciiEsm));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / nonAsciiEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / nonAsciiEsm));

                // Morrowind load order files have a slightly different
                // format and a prefix is necessary.
                std::string linePrefix = getActivePluginsFileLinePrefix();

                // Write out a load order, making it as invalid as possible
                // for the game to still fix.
                std::vector<std::pair<std::string, bool>> plugins({
                    {nonAsciiEsm, true},
                    {masterFile, false},
                    {blankDifferentEsm, false},
                    {blankEsm, true},
                    {blankMasterDependentEsm, false},
                    {blankDifferentMasterDependentEsm, false},
                    {blankEsp, true},  // Put a plugin before master to test fixup.
                    {blankDifferentEsp, false},
                    {blankMasterDependentEsp, false},
                    {blankDifferentMasterDependentEsp, false},
                    {blankPluginDependentEsp, false},
                    {blankDifferentPluginDependentEsp, false},
                    {invalidPlugin, false},
                });
                writeLoadOrder(plugins);
            }

            inline virtual void TearDown() {
                GameTest::TearDown();

                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / updateEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / nonAsciiEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / automatronDlcEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / wastelandWorkshopDlcEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / farHarborDlcEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / contraptionsWorkshopDlcEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / vaultTecWorkshopDlcEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / nukaWorldDlcEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / hearthfiresEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / dawnguardEsm));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / dragonbornEsm));
            }

            inline void writeLoadOrder(std::vector<std::pair<std::string, bool>> loadOrder) const {
                std::string linePrefix = getActivePluginsFileLinePrefix();

                if (loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                    boost::filesystem::ofstream out(activePluginsFilePath);
                    for (const auto& plugin : loadOrder) {
                        if (plugin.second)
                            out << linePrefix;

                        out << utf8ToWindows1252(plugin.first) << std::endl;
                    }
                }
                else {
                    boost::filesystem::ofstream out(activePluginsFilePath);
                    for (const auto& plugin : loadOrder) {
                        if (plugin.second)
                            out << linePrefix << utf8ToWindows1252(plugin.first) << std::endl;
                    }
                    out.close();

                    if (loadOrderMethod == LIBLO_METHOD_TEXTFILE) {
                        boost::filesystem::ofstream out(loadOrderFilePath);
                        for (const auto& plugin : loadOrder)
                            out << plugin.first << std::endl;
                    }
                    else {
                        time_t modificationTime = time(NULL);  // Current time.
                        for (const auto& plugin : loadOrder) {
                            boost::filesystem::last_write_time(pluginsPath / plugin.first, modificationTime);
                            modificationTime += 60;
                        }
                    }
                }
            }

            void incrementModTime(const boost::filesystem::path& file) {
                time_t currentModTime = boost::filesystem::last_write_time(file);
                boost::filesystem::last_write_time(file, currentModTime + 1);
            }

            void decrementModTime(const boost::filesystem::path& file) {
                time_t currentModTime = boost::filesystem::last_write_time(file);
                boost::filesystem::last_write_time(file, currentModTime - 1);
            }

            inline void create(const std::string& plugin) const {
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / plugin));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / plugin));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / plugin));
            }

            inline void createImplicitlyActivePlugins() const {
                for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                    if (!boost::filesystem::exists(pluginsPath / plugin))
                        create(plugin);
                }
            }

            std::vector<std::string> readTextFile(const boost::filesystem::path& path) {
              boost::filesystem::ifstream in(path);
              std::vector<std::string> lines;
              while (in) {
                std::string line;
                std::getline(in, line);

                if (!line.empty())
                  lines.push_back(windows1252toUtf8(line));
              }

              return lines;
            }

            const GameSettings gameSettings;
            LoadOrder loadOrder;

            const std::string blankMasterDependentEsm;
            const std::string blankDifferentMasterDependentEsm;
            const std::string blankEsp;
            const std::string blankDifferentEsp;
            const std::string blankMasterDependentEsp;
            const std::string blankDifferentMasterDependentEsp;
            const std::string blankPluginDependentEsp;
            const std::string blankDifferentPluginDependentEsp;

            const std::string hearthfiresEsm;
            const std::string dawnguardEsm;
            const std::string dragonbornEsm;

            const std::string missingPlugin;
            const std::string updateEsm;
            const std::string nonAsciiEsm;
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
                                    LIBLO_GAME_FO4,
                                    LIBLO_GAME_TES5SE));

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

            EXPECT_TRUE(std::equal(begin(validLoadOrder), end(validLoadOrder), begin(loadOrder.getLoadOrder())));
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

            EXPECT_TRUE(std::equal(begin(secondLoadOrder), end(secondLoadOrder), begin(loadOrder.getLoadOrder())));
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

            EXPECT_TRUE(std::equal(begin(validLoadOrder), end(validLoadOrder), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithTheGameMasterNotAtTheBeginningShouldFailForTextfileAndAsteriskLoadOrderGamesAndSucceedOtherwise) {
            std::vector<std::string> plugins({
                blankEsm,
                masterFile,
            });
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK)
                EXPECT_ANY_THROW(loadOrder.setLoadOrder(plugins));
            else
                EXPECT_NO_THROW(loadOrder.setLoadOrder(plugins));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithTheGameMasterNotAtTheBeginningShouldMakeNoChangesForTextfileAndAsteriskLoadOrderGames) {
            std::vector<std::string> plugins({
                blankEsm,
                masterFile,
            });
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
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

            EXPECT_EQ(12, loadOrder.getPosition(missingPlugin));
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

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstShouldThrowForTextfileAndAsteriskLoadOrderGamesAndNotOtherwise) {
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK)
                EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 0));
            else {
                EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 0));
            }
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstForATextfileOrAsteriskBasedGameShouldMakeNoChanges) {
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 0));
                EXPECT_TRUE(loadOrder.getLoadOrder().empty());
            }
        }

        TEST_P(LoadOrderTest, settingAPluginThatIsNotTheGameMasterFileToLoadFirstForATimestampOrAsteriskBasedGameShouldSucceed) {
            if (loadOrderMethod == LIBLO_METHOD_TIMESTAMP) {
                EXPECT_NO_THROW(loadOrder.setPosition(blankEsm, 0));
                EXPECT_FALSE(loadOrder.getLoadOrder().empty());
                EXPECT_EQ(0, loadOrder.getPosition(blankEsm));
            }
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginShouldThrowForTextfileAndAsteriskLoadOrderGamesAndNotOtherwise) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK)
                EXPECT_ANY_THROW(loadOrder.setPosition(masterFile, 1));
            else
                EXPECT_NO_THROW(loadOrder.setPosition(masterFile, 1));
        }

        TEST_P(LoadOrderTest, settingTheGameMasterFileToLoadAfterAnotherPluginShouldMakeNoChangesForTextfileOrAsteriskLoadOrderGames) {
            std::vector<std::string> validLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
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

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE) {
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

            if (loadOrderMethod == LIBLO_METHOD_TIMESTAMP) {
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

            EXPECT_TRUE(std::equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, settingANonMasterPluginToLoadBeforeAMasterPluginShouldThrow) {
            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsp, 1));
        }

        TEST_P(LoadOrderTest, settingANonMasterPluginToLoadBeforeAMasterPluginShouldMakeNoChanges) {
            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsp, 1));
            EXPECT_NE(1, loadOrder.getPosition(blankEsp));
        }

        TEST_P(LoadOrderTest, settingAMasterToLoadAfterAPluginShouldThrow) {
            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 10));
        }

        TEST_P(LoadOrderTest, settingAMasterToLoadAfterAPluginShouldMakeNoChanges) {
            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_ANY_THROW(loadOrder.setPosition(blankEsm, 10));
            EXPECT_NE(10, loadOrder.getPosition(blankEsm));
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

        TEST_P(LoadOrderTest, clearingLoadOrderShouldResetTimestamps) {
            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_NO_THROW(loadOrder.clear());
            ASSERT_NO_THROW(loadOrder.load());
            EXPECT_FALSE(loadOrder.getLoadOrder().empty());
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
            ASSERT_NO_THROW(loadOrder.setPosition(masterFile, 0));

            EXPECT_NO_THROW(loadOrder.activate(blankEsp));
            EXPECT_EQ(1, loadOrder.getPosition(blankEsp));
            EXPECT_TRUE(loadOrder.isActive(blankEsp));
        }

        TEST_P(LoadOrderTest, activatingAMasterPluginNotInTheLoadOrderShouldInsertItAfterAllOtherMasters) {
            ASSERT_NO_THROW(loadOrder.setPosition(masterFile, 0));
            ASSERT_NO_THROW(loadOrder.setPosition(blankEsp, 1));

            EXPECT_NO_THROW(loadOrder.activate(blankDifferentEsm));
            EXPECT_EQ(1, loadOrder.getPosition(blankDifferentEsm));
            EXPECT_TRUE(loadOrder.isActive(blankDifferentEsm));
        }

        TEST_P(LoadOrderTest, activatingTheGameMasterFileNotInTheLoadOrderShouldInsertItAfterAllOtherMastersForTimestampBasedGamesAndAtTheBeginningOtherwise) {
            ASSERT_NO_THROW(loadOrder.activate(blankEsm));

            EXPECT_NO_THROW(loadOrder.activate(masterFile));
            if (loadOrderMethod == LIBLO_METHOD_TIMESTAMP)
                EXPECT_EQ(1, loadOrder.getPosition(masterFile));
            else
                EXPECT_EQ(0, loadOrder.getPosition(masterFile));
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
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(validLoadOrder));

            EXPECT_NO_THROW(loadOrder.activate(boost::to_lower_copy(blankEsm)));

            EXPECT_TRUE(loadOrder.isActive(blankEsm));

            EXPECT_TRUE(std::equal(begin(validLoadOrder), end(validLoadOrder), begin(loadOrder.getLoadOrder())));
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

        TEST_P(LoadOrderTest, deactivatingAnImplicitlyActivePluginThatIsInstalledShouldThrow) {
            createImplicitlyActivePlugins();
            loadOrder.load();

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                EXPECT_ANY_THROW(loadOrder.deactivate(plugin));
                EXPECT_TRUE(loadOrder.isActive(plugin));
            }
        }

        TEST_P(LoadOrderTest, deactivatingAnImplicitlyActivePluginThatIsNotInstalledShouldDoNothing) {
            loadOrder.load();

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                if (boost::filesystem::exists(pluginsPath / plugin))
                    continue;

                EXPECT_NO_THROW(loadOrder.deactivate(plugin));
                EXPECT_FALSE(loadOrder.isActive(plugin));
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

        TEST_P(LoadOrderTest, settingLoadOrderShouldActivateTheGameMasterForTextfileAndAsteriskBasedGamesAndNotOtherwise) {
            std::vector<std::string> firstLoadOrder({
                masterFile,
                blankEsm,
                blankDifferentEsm,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(firstLoadOrder));
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK)
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
            std::vector<std::string> activePlugins({
                masterFile,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingInvalidActivePluginsShouldMakeNoChanges) {
            std::vector<std::string> activePlugins({
                masterFile,
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
            EXPECT_TRUE(loadOrder.getActivePlugins().empty());
        }

        TEST_P(LoadOrderTest, settingMoreThanMaxNumberActivePluginsShouldThrow) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::vector<std::string> activePlugins({
                masterFile,
            });
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / (std::to_string(i) + ".esp")));
                activePlugins.push_back(std::to_string(i) + ".esp");
            }

            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, settingMoreThanMaxNumberActivePluginsShouldMakeNoChanges) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::vector<std::string> activePlugins({
                masterFile,
            });
            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / (std::to_string(i) + ".esp")));
                activePlugins.push_back(std::to_string(i) + ".esp");
            }

            EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
            EXPECT_TRUE(loadOrder.getActivePlugins().empty());

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutGameMasterShouldThrowForTextfileAndAsteriskBasedGamesAndNotOtherwise) {
            std::vector<std::string> activePlugins({
                blankEsm,
            });
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
            }
            else
                EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutAnInstalledImplicitlyActivePluginShouldThrow) {
            createImplicitlyActivePlugins();

            std::vector<std::string> activePlugins({
                masterFile,
                blankEsm,
            });

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                EXPECT_ANY_THROW(loadOrder.setActivePlugins(activePlugins));
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
            }
        }

        TEST_P(LoadOrderTest, settingActivePluginsWithoutAnImplicitlyActivePluginThatIsNotInstalledShouldNotThrow) {
            std::vector<std::string> activePlugins({
                masterFile,
                blankEsm,
            });
            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
        }

        TEST_P(LoadOrderTest, settingActivePluginsShouldDeactivateAnyOthersInLoadOrderCaseInsensitively) {
            ASSERT_NO_THROW(loadOrder.load());
            ASSERT_TRUE(loadOrder.isActive(blankEsp));

            std::vector<std::string> activePlugins({
                masterFile,
                boost::to_lower_copy(blankEsm),
            });

            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
            activePlugins[1] = blankEsm;

            EXPECT_EQ(activePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, settingActivePluginsNotInLoadOrderShouldAddThemInTheOrderTheyAreGiven) {
            std::vector<std::string> activePlugins({
                masterFile,
                blankEsm,
            });
            ASSERT_TRUE(loadOrder.getLoadOrder().empty());

            EXPECT_NO_THROW(loadOrder.setActivePlugins(activePlugins));

            EXPECT_EQ(activePlugins, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, isSynchronisedForTimestampAndAsteriskBasedGames) {
            if (loadOrderMethod == LIBLO_METHOD_TIMESTAMP || loadOrderMethod == LIBLO_METHOD_ASTERISK)
                EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesIfLoadOrderFileDoesNotExist) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(loadOrderFilePath));

            EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesIfActivePluginsFileDoesNotExist) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(activePluginsFilePath));

            EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isSynchronisedForTextfileBasedGamesWhenLoadOrderAndActivePluginsFileContentsAreEquivalent) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            EXPECT_TRUE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, isNotSynchronisedForTextfileBasedGamesWhenLoadOrderAndActivePluginsFileContentsAreNotEquivalent) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            boost::filesystem::ofstream out(loadOrderFilePath, std::ios_base::trunc);
            out << blankEsm << std::endl;

            EXPECT_FALSE(LoadOrder::isSynchronised(gameSettings));
        }

        TEST_P(LoadOrderTest, loadingDataShouldNotThrowIfActivePluginsFileDoesNotExist) {
            ASSERT_NO_THROW(boost::filesystem::remove(activePluginsFilePath));

            EXPECT_NO_THROW(loadOrder.load());
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateNoPluginsIfActivePluginsFileDoesNotExistAndTheGameHasNoImplicitlyActivePlugins) {
            ASSERT_NO_THROW(boost::filesystem::remove(activePluginsFilePath));

            ASSERT_NO_THROW(loadOrder.load());

            if (gameSettings.getImplicitlyActivePlugins().empty())
                EXPECT_TRUE(loadOrder.getActivePlugins().empty());
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateTheGameMasterForTextfileAndAsteriskBasedGamesAndNotOtherwise) {
            EXPECT_NO_THROW(loadOrder.load());

            bool ismasterFileActive = loadOrder.isActive(masterFile);
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK)
                EXPECT_TRUE(ismasterFileActive);
            else
                EXPECT_FALSE(ismasterFileActive);
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateInstalledImplicitlyActivePlugins) {
            createImplicitlyActivePlugins();
            EXPECT_NO_THROW(loadOrder.load());

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                EXPECT_TRUE(loadOrder.isActive(plugin));
            }
        }

        TEST_P(LoadOrderTest, loadingDataShouldActivateInstalledImplicitlyActivePluginsIfActivePluginsFileIsMissing) {
            boost::filesystem::remove(activePluginsFilePath);
            
            createImplicitlyActivePlugins();
            EXPECT_NO_THROW(loadOrder.load());

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                EXPECT_TRUE(loadOrder.isActive(plugin));
            }
        }

        TEST_P(LoadOrderTest, loadingDataShouldNotActivateInstalledImplicitlyActivePluginsThatAreNotInstalled) {
            EXPECT_NO_THROW(loadOrder.load());

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
                if (!boost::filesystem::exists(pluginsPath / plugin))
                    EXPECT_FALSE(loadOrder.isActive(plugin));
            }
        }

        TEST_P(LoadOrderTest, loadingDataWithMoreThanMaxNumberActivePluginsShouldStopWhenMaxIsReached) {
            // Create plugins to test active plugins limit with. Do it
            // here because it's too expensive to do for every test.
            std::vector<std::string> expectedActivePlugins;

            std::string linePrefix = getActivePluginsFileLinePrefix();
            boost::filesystem::ofstream out(activePluginsFilePath);

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK)
                expectedActivePlugins.push_back(masterFile);

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i) {
                std::string filename = std::to_string(i) + ".esp";
                EXPECT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsp, pluginsPath / filename));
                out << linePrefix << filename << std::endl;
                expectedActivePlugins.push_back(filename);
            }
            out.close();

            expectedActivePlugins.erase(prev(end(expectedActivePlugins), expectedActivePlugins.size() - LoadOrder::maxActivePlugins), end(expectedActivePlugins));

            EXPECT_NO_THROW(loadOrder.setLoadOrder(expectedActivePlugins));
            EXPECT_NO_THROW(loadOrder.load());

            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());

            for (size_t i = 0; i < LoadOrder::maxActivePlugins; ++i)
                EXPECT_NO_THROW(boost::filesystem::remove(pluginsPath / (std::to_string(i) + ".esp")));
        }

        TEST_P(LoadOrderTest, loadingDataShouldFixInvalidDataWhenReadingActivePluginsFile) {
            createImplicitlyActivePlugins();
            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedActivePlugins({
                nonAsciiEsm,
                blankEsm,
                blankEsp,
            });

            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                expectedActivePlugins.insert(begin(expectedActivePlugins), masterFile);

                if (GetParam() == LIBLO_GAME_TES5)
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins)), updateEsm);
                else if (GetParam() == LIBLO_GAME_TES5SE) {
                  expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), updateEsm);
                  expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), "Dawnguard.esm");
                  expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), "Hearthfires.esm");
                  expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), "Dragonborn.esm");
                }
                else if (GetParam() == LIBLO_GAME_FO4) {
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), automatronDlcEsm);
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), wastelandWorkshopDlcEsm);
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), farHarborDlcEsm);
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), contraptionsWorkshopDlcEsm);
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), vaultTecWorkshopDlcEsm);
                    expectedActivePlugins.insert(prev(end(expectedActivePlugins), 3), nukaWorldDlcEsm);
                }
            }
            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, loadingDataShouldPreferLoadOrderFileForTextfileBasedGamesOtherwiseUseTimestamps) {
            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder;
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE) {
                expectedLoadOrder = std::vector<std::string>({
                    masterFile,
                    nonAsciiEsm,
                    blankDifferentEsm,
                    blankEsm,
                    blankMasterDependentEsm,
                    blankDifferentMasterDependentEsm,
                });
                EXPECT_TRUE(equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
            }
            else {
                expectedLoadOrder = std::vector<std::string>({
                    nonAsciiEsm,
                    masterFile,
                    blankDifferentEsm,
                    blankEsm,
                    blankMasterDependentEsm,
                    blankDifferentMasterDependentEsm,
                    blankEsp,
                    blankDifferentEsp,
                    blankMasterDependentEsp,
                    blankDifferentMasterDependentEsp,
                    blankPluginDependentEsp,
                    blankDifferentPluginDependentEsp,
                });

                // Asterisk-based games load their master file first.
                if (loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                    expectedLoadOrder.erase(next(begin(expectedLoadOrder)));
                    expectedLoadOrder.insert(begin(expectedLoadOrder), masterFile);
                }

                EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
            }
        }

        TEST_P(LoadOrderTest, loadingDataShouldFallBackToActivePluginsFileForTextfileBasedGames) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(boost::filesystem::remove(loadOrderFilePath));

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder;
            expectedLoadOrder = std::vector<std::string>({
                masterFile,
                nonAsciiEsm,
                blankEsm,
            });

            EXPECT_TRUE(equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, loadingDataShouldUseHardcodedPositionsForImplicitlyActivePluginsIfGameIsFallout4OrSkyrimSE) {
          createImplicitlyActivePlugins();

          EXPECT_NO_THROW(loadOrder.load());

          std::vector<std::string> expectedLoadOrder;
          if (gameSettings.getId() == LIBLO_GAME_FO4 || gameSettings.getId() == LIBLO_GAME_TES5SE) {
            expectedLoadOrder = gameSettings.getImplicitlyActivePlugins();
            expectedLoadOrder.push_back(nonAsciiEsm);
            expectedLoadOrder.push_back(blankDifferentEsm);
            expectedLoadOrder.push_back(blankEsm);
            expectedLoadOrder.push_back(blankMasterDependentEsm);
            expectedLoadOrder.push_back(blankDifferentMasterDependentEsm);
          } else {
            expectedLoadOrder = std::vector<std::string>({
              nonAsciiEsm,
              masterFile,
              blankDifferentEsm,
              blankEsm,
              blankMasterDependentEsm,
              blankDifferentMasterDependentEsm,
            });

            if (gameSettings.getId() == LIBLO_GAME_TES5) {
              expectedLoadOrder[0] = masterFile;
              expectedLoadOrder[1] = nonAsciiEsm;
            }

            for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
              if (plugin != masterFile)
                expectedLoadOrder.push_back(plugin);
            }
          }

          EXPECT_TRUE(equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, loadingDataShouldIgnoreThePositionsOfImplicitlyActivePluginsInPluginsDotTxtIfTheGameIsFallout4OrSkyrimSE) {
          if (loadOrderMethod != LIBLO_METHOD_ASTERISK)
            return;

          createImplicitlyActivePlugins();

          std::vector<std::pair<std::string, bool>> plugins = {
            {blankEsm, true},
          };
          for (const auto& plugin : gameSettings.getImplicitlyActivePlugins()) {
            if (plugin != masterFile)
              plugins.push_back(std::pair<std::string, bool>(plugin, true));
          }

          writeLoadOrder(plugins);

          EXPECT_NO_THROW(loadOrder.load());

          size_t blankEsmPos = loadOrder.getPosition(blankEsm);
          size_t implicitlyActivePluginPos = loadOrder.getPosition(plugins[1].first);

          EXPECT_GT(blankEsmPos, implicitlyActivePluginPos);
        }

        TEST_P(LoadOrderTest, loadingDataShouldUseHardcodedPositionsForImplicitlyActivePluginsEvenIfPrecedingPluginsAreMissing) {
          if (gameSettings.getImplicitlyActivePlugins().size() < 3)
            return;

          createImplicitlyActivePlugins();

          const std::string removedPlugin = gameSettings.getImplicitlyActivePlugins()[1];
          boost::filesystem::remove(pluginsPath / removedPlugin);

          std::vector<std::string> expectedLoadOrder = gameSettings.getImplicitlyActivePlugins();
          expectedLoadOrder.erase(next(begin(expectedLoadOrder)));
          expectedLoadOrder.push_back(nonAsciiEsm);
          expectedLoadOrder.push_back(blankDifferentEsm);
          expectedLoadOrder.push_back(blankEsm);
          expectedLoadOrder.push_back(blankMasterDependentEsm);
          expectedLoadOrder.push_back(blankDifferentMasterDependentEsm);

          EXPECT_NO_THROW(loadOrder.load());

          EXPECT_TRUE(equal(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadTheActivePluginsIfTheyHaveBeenChanged) {
            ASSERT_NO_THROW(loadOrder.load());

            writeLoadOrder({{blankEsp, true}});
            incrementModTime(activePluginsFilePath);

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedActivePlugins({
                blankEsp,
            });
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                expectedActivePlugins.insert(begin(expectedActivePlugins), masterFile);
            }

            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadTheActivePluginsIfTheyHaveBeenChangedAndFileHasOlderTimestamp) {
            ASSERT_NO_THROW(loadOrder.load());

            writeLoadOrder({{blankEsp, true}});
            decrementModTime(activePluginsFilePath);

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedActivePlugins({
                blankEsp,
            });
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE || loadOrderMethod == LIBLO_METHOD_ASTERISK) {
                expectedActivePlugins.insert(begin(expectedActivePlugins), masterFile);
            }

            EXPECT_EQ(expectedActivePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadTheLoadOrderIfItHasBeenChangedForTextfileBasedGames) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(loadOrder.load());

            writeLoadOrder({{blankDifferentEsm, false}});
            incrementModTime(loadOrderFilePath);

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder({
                nonAsciiEsm,
                masterFile,
                blankDifferentEsm,
                blankEsm,
                blankMasterDependentEsm,
                blankDifferentMasterDependentEsm,
                blankEsp,
                blankDifferentEsp,
                blankMasterDependentEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
                blankDifferentPluginDependentEsp,
            });
            if (loadOrderMethod == LIBLO_METHOD_TEXTFILE) {
                EXPECT_NE(expectedLoadOrder, loadOrder.getLoadOrder());
                EXPECT_TRUE(is_permutation(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
            }
            else
                EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadTheLoadOrderIfItHasBeenChangedForTextfileBasedGamesAndFileHasOlderTimestamp) {
            if (loadOrderMethod != LIBLO_METHOD_TEXTFILE)
                return;

            ASSERT_NO_THROW(loadOrder.load());

            writeLoadOrder({{blankDifferentEsm, false}});
            decrementModTime(loadOrderFilePath);

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder({
                nonAsciiEsm,
                masterFile,
                blankDifferentEsm,
                blankEsm,
                blankMasterDependentEsm,
                blankDifferentMasterDependentEsm,
                blankEsp,
                blankDifferentEsp,
                blankMasterDependentEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
                blankDifferentPluginDependentEsp,
            });
            EXPECT_NE(expectedLoadOrder, loadOrder.getLoadOrder());
            EXPECT_TRUE(is_permutation(begin(expectedLoadOrder), end(expectedLoadOrder), begin(loadOrder.getLoadOrder())));
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadFromThePluginsFolderIfItHasBeenChanged) {
            ASSERT_NO_THROW(loadOrder.load());

            ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / nonAsciiEsm));
            incrementModTime(pluginsPath);

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder({
                masterFile,
                blankDifferentEsm,
                blankEsm,
                blankMasterDependentEsm,
                blankDifferentMasterDependentEsm,
                blankEsp,
                blankDifferentEsp,
                blankMasterDependentEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
                blankDifferentPluginDependentEsp,
            });
            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadFromThePluginsFolderIfItHasBeenChangedAndFolderHasOlderTimestamp) {
            ASSERT_NO_THROW(loadOrder.load());

            ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / nonAsciiEsm));
            decrementModTime(pluginsPath);

            EXPECT_NO_THROW(loadOrder.load());

            std::vector<std::string> expectedLoadOrder({
                masterFile,
                blankDifferentEsm,
                blankEsm,
                blankMasterDependentEsm,
                blankDifferentMasterDependentEsm,
                blankEsp,
                blankDifferentEsp,
                blankMasterDependentEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
                blankDifferentPluginDependentEsp,
            });
            EXPECT_EQ(expectedLoadOrder, loadOrder.getLoadOrder());
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadAPluginIfItHasBeenEdited) {
            create(updateEsm);
            ASSERT_NO_THROW(loadOrder.load());

            boost::filesystem::ofstream out(pluginsPath / updateEsm);
            out << std::endl;
            out.close();
            incrementModTime(pluginsPath / updateEsm);

            EXPECT_NO_THROW(loadOrder.load());

            EXPECT_EQ(loadOrder.getLoadOrder().size(), loadOrder.getPosition(updateEsm));
        }

        TEST_P(LoadOrderTest, loadingDataTwiceShouldReloadAPluginIfItHasBeenEditedAndFileHasOlderTimestamp) {
            create(updateEsm);
            ASSERT_NO_THROW(loadOrder.load());

            boost::filesystem::ofstream out(pluginsPath / updateEsm);
            out << std::endl;
            out.close();
            decrementModTime(pluginsPath / updateEsm);

            EXPECT_NO_THROW(loadOrder.load());

            EXPECT_EQ(loadOrder.getLoadOrder().size(), loadOrder.getPosition(updateEsm));
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
            std::vector<std::string> activePlugins({
                masterFile,
                blankEsm,
            });
            ASSERT_NO_THROW(loadOrder.setActivePlugins(activePlugins));

            EXPECT_NO_THROW(loadOrder.save());

            ASSERT_NO_THROW(loadOrder.load());

            EXPECT_EQ(activePlugins, loadOrder.getActivePlugins());
        }

        TEST_P(LoadOrderTest, savingShouldWriteWholeLoadOrderToActivePluginsFileWithAsteriskPrefixesForActivePluginsForAsteriskBasedGames) {
            if (loadOrderMethod != LIBLO_METHOD_ASTERISK)
                return;

            std::vector<std::string> plugins({
                masterFile,
                blankEsm,
                blankMasterDependentEsm,
                blankDifferentEsm,
                nonAsciiEsm,
                blankDifferentMasterDependentEsm,
                blankMasterDependentEsp,
                blankDifferentEsp,
                blankDifferentPluginDependentEsp,
                blankEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
            });
            std::vector<std::string> activePlugins({
                masterFile,
                blankEsm,
                blankDifferentEsp,
            });
            ASSERT_NO_THROW(loadOrder.setLoadOrder(plugins));
            ASSERT_NO_THROW(loadOrder.setActivePlugins(activePlugins));
            EXPECT_NO_THROW(loadOrder.save());

            boost::filesystem::ifstream in(activePluginsFilePath);
            std::vector<std::string> lines = readTextFile(activePluginsFilePath);

            std::vector<std::string> expectedLines({
                '*' + blankEsm,
                blankMasterDependentEsm,
                blankDifferentEsm,
                nonAsciiEsm,
                blankDifferentMasterDependentEsm,
                blankMasterDependentEsp,
                '*' + blankDifferentEsp,
                blankDifferentPluginDependentEsp,
                blankEsp,
                blankDifferentMasterDependentEsp,
                blankPluginDependentEsp,
            });

            EXPECT_EQ(expectedLines, lines);
        }

        TEST_P(LoadOrderTest, savingShouldNotWriteImplicitlyActivePluginsToPluginsDotTxtForFallout4AndSkyrimSE) {
          createImplicitlyActivePlugins();

          ASSERT_NO_THROW(loadOrder.load());
          ASSERT_NO_THROW(loadOrder.save());

          std::string linePrefix = getActivePluginsFileLinePrefix();
          std::vector<std::string> lines = readTextFile(activePluginsFilePath);

          std::vector<std::string> expectedLines({
            nonAsciiEsm,
            blankEsm,
            blankEsp,
          });

          if (gameSettings.getId() == LIBLO_GAME_TES3) {
            for (size_t i = 0; i < expectedLines.size(); ++i)
              expectedLines[i] = "GameFile" + std::to_string(i) + "=" + expectedLines[i];
          } else if (gameSettings.getId() == LIBLO_GAME_TES5) {
            expectedLines.insert(prev(end(expectedLines)), updateEsm);
          } else if (loadOrderMethod == LIBLO_METHOD_ASTERISK) {
            expectedLines = {
              '*' + nonAsciiEsm,
              blankDifferentEsm,
              '*' + blankEsm,
              blankMasterDependentEsm,
              blankDifferentMasterDependentEsm,
              '*' + blankEsp,
              blankDifferentEsp,
              blankMasterDependentEsp,
              blankDifferentMasterDependentEsp,
              blankPluginDependentEsp,
              blankDifferentPluginDependentEsp,
            };
          }

          EXPECT_EQ(expectedLines, lines);
        }
    }
}

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
#include "backend/plugins.h"

namespace liblo {
    namespace test {
        class LoadOrderTest : public ::testing::TestWithParam<unsigned int> {
        protected:
            inline LoadOrderTest() :
                blankEsm("Blank.esm"),
                blankDifferentEsm("Blank - Different.esm"),
                blankEsp("Blank.esp"),
                invalidPlugin("NotAPlugin.esm"),
                missingPlugin("missing.esm"),
                gameHandle(GetParam(), getGamePath(GetParam())) {}

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
            }

            inline virtual void TearDown() {
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / invalidPlugin));
                ASSERT_NO_THROW(boost::filesystem::remove(gameHandle.PluginsFolder() / gameHandle.MasterFile()));
            }

            inline std::string getGamePath(unsigned int gameId) const {
                if (gameId == LIBLO_GAME_TES3)
                    return "./Morrowind";
                else if (gameId == LIBLO_GAME_TES4)
                    return "./Oblivion";
                else
                    return "./Skyrim";
            }

            LoadOrder loadOrder;
            _lo_game_handle_int gameHandle;

            std::string blankEsm;
            std::string blankDifferentEsm;
            std::string blankEsp;
            std::string invalidPlugin;
            std::string missingPlugin;
        };

        // Pass an empty first argument, as it's a prefix for the test instantation,
        // but we only have the one so no prefix is necessary.
        INSTANTIATE_TEST_CASE_P(,
                                LoadOrderTest,
                                ::testing::Values(
                                //LIBLO_GAME_TES3,
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

        TEST_P(LoadOrderTest, settingALoadOrderWithAnInvalidPluginShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                invalidPlugin,
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
        }

        TEST_P(LoadOrderTest, settingALoadOrderWithACaseInsensitiveDuplicatePluginShouldThrow) {
            std::vector<std::string> invalidLoadOrder({
                gameHandle.MasterFile(),
                blankEsm,
                boost::to_lower_copy(blankEsm),
            });
            EXPECT_ANY_THROW(loadOrder.setLoadOrder(invalidLoadOrder, gameHandle));
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
    }
}

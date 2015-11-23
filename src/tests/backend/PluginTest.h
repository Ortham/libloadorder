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

#include <gtest/gtest.h>

#include "backend/Plugin.h"
#include "backend/GameSettings.h"

namespace liblo {
    namespace test {
        class PluginTest : public ::testing::Test {
        protected:
            PluginTest() :
                blankEsm("Blank.esm"),
                blankDifferentEsm("Blank - Different.esm"),
                blankEsmGhost(blankEsm + ".ghost"),
                updateEsm("Update.esm"),
                updateEsmGhost(updateEsm + ".ghost"),
                invalidPlugin("NotAPlugin.esm"),
                missingPlugin("missing.esm"),
                // Just use Skyrim game settings to test with. The
                // functionality only needs to get a plugins folder and libespm
                // game ID.
                gameSettings(LIBLO_GAME_TES5, "./Skyrim", "./local/Skyrim") {}

            inline virtual void SetUp() {
                ASSERT_TRUE(boost::filesystem::exists(gameSettings.getPluginsFolder() / blankEsm));
                ASSERT_TRUE(boost::filesystem::exists(gameSettings.getPluginsFolder() / blankDifferentEsm));
                ASSERT_FALSE(boost::filesystem::exists(gameSettings.getPluginsFolder() / missingPlugin));

                // Write out an non-empty, non-plugin file.
                boost::filesystem::ofstream out(gameSettings.getPluginsFolder() / invalidPlugin);
                out << "This isn't a valid plugin file.";
                out.close();
                ASSERT_TRUE(boost::filesystem::exists(gameSettings.getPluginsFolder() / invalidPlugin));

                // Create a ghosted copy of a plugin.
                EXPECT_NO_THROW(boost::filesystem::copy(gameSettings.getPluginsFolder() / blankEsm, gameSettings.getPluginsFolder() / blankEsmGhost));

                // Ghost a plugin.
                EXPECT_NO_THROW(boost::filesystem::copy(gameSettings.getPluginsFolder() / blankEsm, gameSettings.getPluginsFolder() / updateEsmGhost));
            }

            inline virtual void TearDown() {
                ASSERT_NO_THROW(boost::filesystem::remove(gameSettings.getPluginsFolder() / invalidPlugin));
                EXPECT_NO_THROW(boost::filesystem::remove(gameSettings.getPluginsFolder() / blankEsmGhost));
                EXPECT_NO_THROW(boost::filesystem::remove(gameSettings.getPluginsFolder() / updateEsmGhost));
            }

            GameSettings gameSettings;

            std::string blankEsm;
            std::string blankDifferentEsm;
            std::string blankEsmGhost;
            std::string updateEsm;
            std::string updateEsmGhost;
            std::string invalidPlugin;
            std::string missingPlugin;
        };

        TEST_F(PluginTest, getNameShouldReturnTheFilenameOfANonGhostedPlugin) {
            Plugin plugin(blankEsm, gameSettings);
            EXPECT_EQ(blankEsm, plugin.getName());
        }

        TEST_F(PluginTest, getNameShouldReturnTheUnghostedFilenameForAGhostedPlugin) {
            Plugin plugin(blankEsmGhost, gameSettings);

            EXPECT_EQ(blankEsm, plugin.getName());
        }

        TEST_F(PluginTest, constructingForAPluginShouldCacheItsModificationTime) {
            time_t modTime = boost::filesystem::last_write_time(gameSettings.getPluginsFolder() / blankEsm);

            Plugin plugin(blankEsm, gameSettings);
            EXPECT_EQ(modTime, plugin.getModTime());
        }

        TEST_F(PluginTest, aFileChangeShouldNotBeDetectedIfItIsNotEdited) {
            Plugin plugin(blankEsm, gameSettings);

            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));

            EXPECT_FALSE(plugin.hasFileChanged(gameSettings.getPluginsFolder()));
        }

        TEST_F(PluginTest, aFileChangeShouldBeDetectedIfItIsEdited) {
            Plugin plugin(blankEsmGhost, gameSettings);

            // Timestamps have 1 second precision, so wait to allow them
            // to change.
            std::this_thread::sleep_for(std::chrono::seconds(1));
            boost::filesystem::ofstream out(gameSettings.getPluginsFolder() / blankEsmGhost);
            out << std::endl;
            out.close();

            EXPECT_TRUE(plugin.hasFileChanged(gameSettings.getPluginsFolder()));
        }

        TEST_F(PluginTest, aFileChangeShouldBeDetectedIfItIsSetToAnOlderTimestamp) {
            Plugin plugin(blankEsmGhost, gameSettings);

            EXPECT_NO_THROW(boost::filesystem::last_write_time(gameSettings.getPluginsFolder() / blankEsmGhost, plugin.getModTime() - 1));

            EXPECT_TRUE(plugin.hasFileChanged(gameSettings.getPluginsFolder()));
        }

        TEST_F(PluginTest, settingTheModificationTimeOfAPluginShouldUpdateTheFilesystem) {
            Plugin plugin(blankEsm, gameSettings);
            time_t newModTime = plugin.getModTime() + 60;

            EXPECT_NO_THROW(plugin.setModTime(newModTime, gameSettings.getPluginsFolder()));
            EXPECT_EQ(newModTime, boost::filesystem::last_write_time(gameSettings.getPluginsFolder() / blankEsm));
        }

        TEST_F(PluginTest, settingTheModificationTimeOfAPluginShouldUpdateItsCachedModificationTime) {
            Plugin plugin(blankEsm, gameSettings);
            time_t newModTime = plugin.getModTime() + 60;

            EXPECT_NO_THROW(plugin.setModTime(newModTime, gameSettings.getPluginsFolder()));
            EXPECT_EQ(newModTime, plugin.getModTime());
        }

        TEST_F(PluginTest, activatingAPluginShouldSetItToActive) {
            Plugin plugin(blankEsm, gameSettings);
            ASSERT_FALSE(plugin.isActive());

            plugin.activate(gameSettings.getPluginsFolder());
            EXPECT_TRUE(plugin.isActive());
        }

        TEST_F(PluginTest, activatingAGhostedPluginShouldUnghostIt) {
            Plugin plugin(blankEsmGhost, gameSettings);
            ASSERT_FALSE(plugin.isActive());

            plugin.activate(gameSettings.getPluginsFolder());
            EXPECT_TRUE(plugin.isActive());
            EXPECT_TRUE(boost::filesystem::exists(gameSettings.getPluginsFolder() / blankEsm));
            EXPECT_FALSE(boost::filesystem::exists(gameSettings.getPluginsFolder() / blankEsmGhost));
        }

        TEST_F(PluginTest, activatingAGhostedPluginShouldUpdateItsCachedModificationTime) {
            Plugin plugin(blankEsmGhost, gameSettings);
            ASSERT_FALSE(plugin.isActive());

            plugin.activate(gameSettings.getPluginsFolder());
            time_t modTime = boost::filesystem::last_write_time(gameSettings.getPluginsFolder() / blankEsm);
            EXPECT_EQ(modTime, plugin.getModTime());
        }

        TEST_F(PluginTest, activatingAnActivePluginShouldHaveNoEffect) {
            Plugin plugin(blankEsm, gameSettings);
            plugin.activate(gameSettings.getPluginsFolder());
            time_t oldModTime = plugin.getModTime();
            ASSERT_TRUE(plugin.isActive());

            plugin.activate(gameSettings.getPluginsFolder());
            EXPECT_TRUE(plugin.isActive());
            EXPECT_EQ(oldModTime, plugin.getModTime());
        }

        TEST_F(PluginTest, deactivatingAnActivePluginShouldSetItToNotActive) {
            Plugin plugin(blankEsm, gameSettings);
            plugin.activate(gameSettings.getPluginsFolder());
            ASSERT_TRUE(plugin.isActive());

            plugin.deactivate();
            EXPECT_FALSE(plugin.isActive());
        }

        TEST_F(PluginTest, deactivatingAnInactivePluginShouldHaveNoEffect) {
            Plugin plugin(blankEsm, gameSettings);
            ASSERT_FALSE(plugin.isActive());

            plugin.deactivate();
            EXPECT_FALSE(plugin.isActive());
        }

        TEST_F(PluginTest, copiesOfTheSamePluginShouldBeEqual) {
            Plugin plugin1(blankEsm, gameSettings);
            Plugin plugin2(blankEsm, gameSettings);

            EXPECT_TRUE(plugin1 == plugin2);
            EXPECT_FALSE(plugin1 != plugin2);

            // Test symmetry.
            EXPECT_TRUE(plugin2 == plugin1);
            EXPECT_FALSE(plugin2 != plugin1);
        }

        TEST_F(PluginTest, ghostedAndNotGhostedCopiesOfTheSamePluginShouldBeEqual) {
            Plugin plugin1(blankEsm, gameSettings);

            Plugin plugin2(blankEsmGhost, gameSettings);

            EXPECT_TRUE(plugin1 == plugin2);
            EXPECT_FALSE(plugin1 != plugin2);

            // Test symmetry.
            EXPECT_TRUE(plugin2 == plugin1);
            EXPECT_FALSE(plugin2 != plugin1);
        }

        TEST_F(PluginTest, differentPluginsShouldNotBeEqual) {
            Plugin plugin1(blankEsm, gameSettings);
            Plugin plugin2(blankDifferentEsm, gameSettings);

            EXPECT_FALSE(plugin1 == plugin2);
            EXPECT_TRUE(plugin1 != plugin2);

            // Test symmetry.
            EXPECT_FALSE(plugin2 == plugin1);
            EXPECT_TRUE(plugin2 != plugin1);
        }

        TEST_F(PluginTest, aPluginShouldBeCaseInsensitivelyEqualToItsFilename) {
            Plugin plugin(blankEsm, gameSettings);

            EXPECT_TRUE(plugin == boost::to_lower_copy(blankEsm));
        }

        TEST_F(PluginTest, aGhostedPluginShouldBeEqualToItsNonGhostedFilename) {
            Plugin plugin(blankEsmGhost, gameSettings);

            EXPECT_TRUE(plugin == blankEsm);
        }

        TEST_F(PluginTest, aNonGhostedPluginShouldBeEqualToItsGhostedFilename) {
            Plugin plugin(blankEsm, gameSettings);

            EXPECT_TRUE(plugin == blankEsmGhost);
        }

        TEST_F(PluginTest, aPluginShouldNotBeEqualToAnotherPluginsFilename) {
            Plugin plugin(blankEsm, gameSettings);

            EXPECT_FALSE(plugin == blankDifferentEsm);
            EXPECT_TRUE(plugin != blankDifferentEsm);
        }

        TEST_F(PluginTest, blankEsmShouldBeValid) {
            EXPECT_TRUE(Plugin::isValid(blankEsm, gameSettings));
        }

        TEST_F(PluginTest, blankEsmShouldBeValidWhenGhostedAndUnghostedFilenameIsGiven) {
            EXPECT_TRUE(Plugin::isValid(updateEsmGhost, gameSettings));
        }

        TEST_F(PluginTest, blankEsmShouldBeValidWhenNotGhostedAndGhostedFilenameIsGiven) {
            EXPECT_TRUE(Plugin::isValid(blankEsmGhost, gameSettings));
        }

        TEST_F(PluginTest, invalidPluginShouldBeRecognisedAsSuch) {
            EXPECT_FALSE(Plugin::isValid(invalidPlugin, gameSettings));
        }

        TEST_F(PluginTest, missingPluginShouldBeTreatedAsInvalid) {
            EXPECT_FALSE(Plugin::isValid(missingPlugin, gameSettings));
        }
    }
}

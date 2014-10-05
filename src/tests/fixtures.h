/*  libloadorder

A library for reading and writing the load order of plugin files for
TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 and
Fallout: New Vegas.

Copyright (C) 2012    WrinklyNinja

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

#ifndef __LIBLO_TEST_FIXTURES__
#define __LIBLO_TEST_FIXTURES__

#include "../api/libloadorder.h"
#include "backend/streams.h"

#include <gtest/gtest.h>
#include <boost/filesystem.hpp>

class GameHandleCreationTest : public ::testing::Test {
protected:
    inline GameHandleCreationTest() : gh(NULL) {}

    inline virtual void TearDown() {
        ASSERT_NO_THROW(lo_destroy_handle(gh));
    };

    lo_game_handle gh;
};

class GameOperationsTest : public ::testing::Test {
protected:
    GameOperationsTest(const boost::filesystem::path& gameDataPath, const boost::filesystem::path& gameLocalPath)
        : dataPath(gameDataPath), localPath(gameLocalPath), gh(NULL) {}

    inline virtual void SetUp() {
        boost::filesystem::create_directories(localPath);
        ASSERT_TRUE(boost::filesystem::exists(localPath));

        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different Master Dependent.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different Master Dependent.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Plugin Dependent.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different Plugin Dependent.esp"));

        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank.esm.missing"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank.esp.missing"));

        // Ghost a plugin.
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));
        boost::filesystem::rename(dataPath / "Blank - Master Dependent.esm", dataPath / "Blank - Master Dependent.esm.ghost");
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));

        // Write out an empty file.
        liblo::ofstream out(dataPath / "EmptyFile.esm");
        out.close();
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "EmptyFile.esm"));

        // Write out an non-empty, non-plugin file.
        out.open(dataPath / "NotAPlugin.esm");
        out << "This isn't a valid plugin file.";
        out.close();
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "NotAPlugin.esm"));
    }

    inline virtual void TearDown() {
        // Unghost the ghosted plugin.
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));
        boost::filesystem::rename(dataPath / "Blank - Master Dependent.esm.ghost", dataPath / "Blank - Master Dependent.esm");
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));

        // Delete generated files.
        boost::filesystem::remove(dataPath / "EmptyFile.esm");
        boost::filesystem::remove(dataPath / "NotAPlugin.esm");
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "EmptyFile.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "NotAPlugin.esm"));

        lo_destroy_handle(gh);
    }

    const boost::filesystem::path dataPath;
    const boost::filesystem::path localPath;

    lo_game_handle gh;
};

class OblivionOperationsTest : public GameOperationsTest {
protected:
    OblivionOperationsTest() : GameOperationsTest("./Oblivion/Data", "./local/Oblivion") {}

    inline virtual void SetUp() {
        GameOperationsTest::SetUp();

        // Oblivion's load order is decided through timestamps, so reset them to a known order before each test.
        std::list<std::string> loadOrder = {
            "Blank.esm",
            "Blank - Different.esm",
            "Blank - Master Dependent.esm",  // Ghosted
            "Blank - Different Master Dependent.esm",
            "Blank.esp",
            "Blank - Different.esp",
            "Blank - Master Dependent.esp",
            "Blank - Different Master Dependent.esp",
            "Blank - Plugin Dependent.esp",
            "Blank - Different Plugin Dependent.esp"
        };
        time_t modificationTime = time(NULL);  // Current time.
        for (const auto &plugin : loadOrder) {
            if (boost::filesystem::exists(dataPath / boost::filesystem::path(plugin + ".ghost"))) {
                boost::filesystem::last_write_time(dataPath / boost::filesystem::path(plugin + ".ghost"), modificationTime);
            }
            else {
                boost::filesystem::last_write_time(dataPath / plugin, modificationTime);
            }
            modificationTime += 60;
        }

        // Set Oblivion's active plugins to a known list before running the test.
        liblo::ofstream activePlugins(localPath / "plugins.txt");
        activePlugins
            << "Blank.esm" << std::endl;
        activePlugins.close();

        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    }

    inline virtual void TearDown() {
        GameOperationsTest::TearDown();

        // Delete existing plugins.txt.
        boost::filesystem::remove(localPath / "plugins.txt");
    };
};

class SkyrimOperationsTest : public GameOperationsTest {
protected:
    SkyrimOperationsTest() : GameOperationsTest("./Skyrim/Data", "./local/Skyrim") {}

    inline virtual void SetUp() {
        GameOperationsTest::SetUp();

        // Can't change Skyrim's main master file, so mock it.
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Skyrim.esm"));
        boost::filesystem::copy_file(dataPath / "Blank.esm", dataPath / "Skyrim.esm");
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Skyrim.esm"));

        // Set Skyrim's load order to a known list before running the test.
        liblo::ofstream loadOrder(localPath / "loadorder.txt");
        loadOrder
            << "Skyrim.esm" << std::endl
            << "Blank.esm" << std::endl
            << "Blank - Different.esm" << std::endl
            << "Blank - Master Dependent.esm" << std::endl  // Ghosted
            << "Blank - Different Master Dependent.esm" << std::endl
            << "Blank.esp" << std::endl
            << "Blank - Different.esp" << std::endl
            << "Blank - Master Dependent.esp" << std::endl
            << "Blank - Different Master Dependent.esp" << std::endl
            << "Blank - Plugin Dependent.esp" << std::endl
            << "Blank - Different Plugin Dependent.esp" << std::endl;
        loadOrder.close();

        // Set Skyrim's active plugins to a known list before running the test.
        liblo::ofstream activePlugins(localPath / "plugins.txt");
        activePlugins
            << "Skyrim.esm" << std::endl
            << "Blank.esm" << std::endl;
        activePlugins.close();

        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    }

    inline virtual void TearDown() {
        GameOperationsTest::TearDown();

        // Delete the mock Skyrim.esm.
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Skyrim.esm"));
        boost::filesystem::remove(dataPath / "Skyrim.esm");
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Skyrim.esm"));

        // Delete existing plugins.txt and loadorder.txt.
        boost::filesystem::remove(localPath / "plugins.txt");
        boost::filesystem::remove(localPath / "loadorder.txt");
    };
};

#endif

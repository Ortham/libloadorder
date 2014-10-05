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

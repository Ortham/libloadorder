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

#ifndef __LIBLO_TEST_API__
#define __LIBLO_TEST_API__

#include "tests/fixtures.h"

#include <boost/algorithm/string.hpp>

TEST(GetVersion, HandlesNullInput) {
    unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(&vMajor, NULL, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, &vMinor, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, NULL, &vPatch));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, NULL, NULL));
}

TEST(GetVersion, HandlesValidInput) {
    unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
    EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));
}

TEST(IsCompatible, HandlesCompatibleVersion) {
    unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
    EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));

    EXPECT_TRUE(lo_is_compatible(vMajor, vMinor, vPatch));
    // Test somewhat arbitrary variations.
    EXPECT_TRUE(lo_is_compatible(vMajor, vMinor + 1, vPatch + 1));
    if (vMinor > 0 && vPatch > 0)
        EXPECT_TRUE(lo_is_compatible(vMajor, vMinor - 1, vPatch - 1));
}

TEST(IsCompatible, HandlesIncompatibleVersion) {
    unsigned int vMajor = 0, vMinor = 0, vPatch = 0;
    EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));

    EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor, vPatch));
    // Test somewhat arbitrary variations.
    EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor + 1, vPatch + 1));
    if (vMinor > 0 && vPatch > 0)
        EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor - 1, vPatch - 1));
}

TEST(GetErrorMessage, HandlesInputCorrectly) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

    const char * error = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    ASSERT_STREQ("Null pointer passed.", error);
}

TEST(Cleanup, CleansUpAfterError) {
    // First generate an error.
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

    // Check that the error message is non-null.
    const char * error = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    ASSERT_STREQ("Null pointer passed.", error);

    ASSERT_NO_THROW(lo_cleanup());

    // Now check that the error message pointer is null.
    error = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    EXPECT_EQ(nullptr, error);
}

TEST(Cleanup, HandlesNoError) {
    ASSERT_NO_THROW(lo_cleanup());

    const char * error = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    EXPECT_EQ(nullptr, error);
}

TEST_F(OblivionHandleCreationTest, HandlesValidInputs) {
    // Try all the game codes, it doesn't matter for lo_create_handle what game is actually at the given paths.
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES3, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    gh = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    gh = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    gh = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_FO3, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    gh = nullptr;
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_FNV, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    gh = nullptr;

    // Also test absolute paths.
    boost::filesystem::path game = boost::filesystem::current_path() / dataPath.parent_path();
    boost::filesystem::path local = boost::filesystem::current_path() / localPath;
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, game.string().c_str(), local.string().c_str()));
}

TEST_F(OblivionHandleCreationTest, HandlesInvalidHandleInput) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(NULL, LIBLO_GAME_TES4, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
}

TEST_F(OblivionHandleCreationTest, HandlesInvalidGameType) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, UINT_MAX, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
}

TEST_F(OblivionHandleCreationTest, HandlesInvalidGamePathInput) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, LIBLO_GAME_TES4, NULL, localPath.string().c_str()));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, LIBLO_GAME_TES4, missingPath.string().c_str(), localPath.string().c_str()));
}

TEST_F(OblivionHandleCreationTest, HandlesInvalidLocalPathInput) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, LIBLO_GAME_TES4, dataPath.parent_path().string().c_str(), missingPath.string().c_str()));
}

#ifdef _WIN32
TEST_F(OblivionHandleCreationTest, HandlesNullLocalPath) {
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, dataPath.parent_path().string().c_str(), NULL));
}
#endif

TEST(GameHandleDestroyTest, HandledNullInput) {
    ASSERT_NO_THROW(lo_destroy_handle(NULL));
}

TEST_F(OblivionOperationsTest, SetGameMaster) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gh, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gh, "EmptyFile.esm"));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gh, "NotAPlugin.esm"));

    EXPECT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));

    // Try setting to a master that doesn't exist.
    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_game_master(gh, "Blank.missing.esm"));
}

TEST_F(OblivionOperationsTest, FixPluginLists) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_fix_plugin_lists(NULL));

    // Write a broken plugins.txt.
    liblo::ofstream active(localPath / "plugins.txt");
    active << "Blank - Master Dependent.esp" << std::endl
        << "Blank - Plugin Dependent.esp" << std::endl
        << "Blank - Different Master Dependent.esp" << std::endl
        << "Blank - Master Dependent.esp" << std::endl  // Duplicate, should be removed.
        << "Blank.missing.esm" << std::endl  // Missing, should be removed.
        << "Blank.esp" << std::endl;
    active.close();

    // Set the load order.
    char * plugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank - Different Plugin Dependent.esp",
        "Blank - Master Dependent.esp",
        "Blank - Different Master Dependent.esp",
        "Blank - Plugin Dependent.esp"
    };
    size_t pluginsNum = 6;
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "Blank.esm"));
    ASSERT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));

    // Now fix plugins.txt
    ASSERT_PRED1([](unsigned int i) {
        return i == LIBLO_OK || i == LIBLO_WARN_INVALID_LIST;
    }, lo_fix_plugin_lists(gh));

    // Read plugins.txt. Order doesn't matter, so just check content using a sorted list.
    std::list<std::string> expectedLines = {
        "Blank - Different Master Dependent.esp",
        "Blank - Master Dependent.esp",
        "Blank - Plugin Dependent.esp",
        "Blank.esp"
    };
    std::list<std::string> actualLines;
    std::string content;
    liblo::ifstream in(localPath / "plugins.txt");
    while (in.good()) {
        std::string line;
        std::getline(in, line);
        actualLines.push_back(line);
    }
    in.close();
    actualLines.pop_back();  // Remove the trailing newline.
    actualLines.sort();

    EXPECT_EQ(expectedLines, actualLines);
}

#endif

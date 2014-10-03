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
#include "backend/streams.h"
#include "backend/helpers.h"

#include <boost/filesystem.hpp>
#include <boost/algorithm/string.hpp>

TEST(GetVersion, HandlesNullInput) {
    unsigned int vMajor, vMinor, vPatch;
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(&vMajor, NULL, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, &vMinor, NULL));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, NULL, &vPatch));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_version(NULL, NULL, NULL));
}

TEST(GetVersion, HandlesValidInput) {
    unsigned int vMajor, vMinor, vPatch;
    EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));
}

TEST(IsCompatible, HandlesCompatibleVersion) {
    unsigned int vMajor, vMinor, vPatch;
    EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));

    EXPECT_TRUE(lo_is_compatible(vMajor, vMinor, vPatch));
    // Test somewhat arbitrary variations.
    EXPECT_TRUE(lo_is_compatible(vMajor, vMinor + 1, vPatch + 1));
    if (vMinor > 0 && vPatch > 0)
        EXPECT_TRUE(lo_is_compatible(vMajor, vMinor - 1, vPatch - 1));
}

TEST(IsCompatible, HandlesIncompatibleVersion) {
    unsigned int vMajor, vMinor, vPatch;
    EXPECT_EQ(LIBLO_OK, lo_get_version(&vMajor, &vMinor, &vPatch));

    EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor, vPatch));
    // Test somewhat arbitrary variations.
    EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor + 1, vPatch + 1));
    if (vMinor > 0 && vPatch > 0)
        EXPECT_FALSE(lo_is_compatible(vMajor + 1, vMinor - 1, vPatch - 1));
}

TEST(GetErrorMessage, HandlesInputCorrectly) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

    const char * error;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    ASSERT_STREQ("Null pointer passed.", error);
}

TEST(Cleanup, CleansUpAfterError) {
    // First generate an error.
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_get_error_message(NULL));

    // Check that the error message is non-null.
    const char * error;
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

TEST_F(GameHandleCreationTest, HandlesValidInputs) {
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES3, "./game", "./local"));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, "./game", "./local"));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, "./game", "./local"));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_FO3, "./game", "./local"));
    ASSERT_NO_THROW(lo_destroy_handle(gh));
    EXPECT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_FNV, "./game", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidHandleInput) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(NULL, LIBLO_GAME_TES4, "./game", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidGameType) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, UINT_MAX, "./game", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidGamePathInput) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, LIBLO_GAME_TES4, NULL, "./local"));
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, LIBLO_GAME_TES4, "/\0", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidLocalPathInput) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_create_handle(&gh, LIBLO_GAME_TES4, "./game", "/\0"));
}

TEST(GameHandleDestroyTest, HandledNullInput) {
    ASSERT_NO_THROW(lo_destroy_handle(NULL));
}

TEST_F(OblivionOperationsTest, SetGameMaster) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gh, NULL));

    // Write out an empty file.
    liblo::ofstream out("./game/Data/Not a plugin.esm");
    out.close();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gh, "Not a plugin.esm"));
    boost::filesystem::remove("./game/Data/Not a plugin.esm");

    // Write out an non-empty, non-plugin file.
    out.open("./game/Data/Not a plugin.esm");
    out << "This isn't a valid plugin file.";
    out.close();
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_set_game_master(gh, "Not a plugin.esm"));
    boost::filesystem::remove("./game/Data/Not a plugin.esm");

    EXPECT_EQ(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm"));

    // Try setting to a master that doesn't exist.
    ASSERT_FALSE(boost::filesystem::exists("./game/Data/EnhancedWeather.esm.missing"));
    EXPECT_EQ(LIBLO_ERROR_FILE_NOT_FOUND, lo_set_game_master(gh, "EnhancedWeather.esm.missing"));
}

TEST_F(OblivionOperationsTest, FixPluginLists) {
    EXPECT_EQ(LIBLO_ERROR_INVALID_ARGS, lo_fix_plugin_lists(NULL));

    // Write a broken plugins.txt.
    ASSERT_FALSE(boost::filesystem::exists("./game/Data/EnhancedWeather.esm.missing"));
    boost::filesystem::create_directory("./local");
    ASSERT_TRUE(boost::filesystem::exists("./local"));
    liblo::ofstream active("./local/plugins.txt");
    active << "Cava Obscura - Cyrodiil.esp" << std::endl
        << "Cava Obscura - Filter Patch For Mods.esp" << std::endl
        << "Cava Obscura - SI.esp" << std::endl
        << "Cava Obscura - Cyrodiil.esp" << std::endl  // Duplicate, should be removed.
        << "EnhancedWeather - Darker Nights, 80.esp" << std::endl
        << "EnhancedWeather.esm.missing" << std::endl  // Missing, should be removed.
        << "EnhancedWeather.esp" << std::endl;
    active.close();

    // Set the load order.
    char * plugins[] = {
        "EnhancedWeather.esm",
        "EnhancedWeather.esp",
        "EnhancedWeather - Darker Nights, 80.esp",
        "Cava Obscura - Cyrodiil.esp",
        "Cava Obscura - SI.esp",
        "Cava Obscura - Filter Patch For Mods.esp"
    };
    size_t pluginsNum = 6;
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm"));
    ASSERT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));

    // Now fix plugins.txt
    ASSERT_PRED1([](unsigned int i) {
        return i == LIBLO_OK || i == LIBLO_WARN_INVALID_LIST;
    }, lo_fix_plugin_lists(gh));

    // Read plugins.txt. Order doesn't matter, so just check content using a sorted list.
    std::list<std::string> expectedLines = {
        "Cava Obscura - Cyrodiil.esp",
        "Cava Obscura - Filter Patch For Mods.esp",
        "Cava Obscura - SI.esp",
        "EnhancedWeather - Darker Nights, 80.esp",
        "EnhancedWeather.esp"
    };
    std::list<std::string> actualLines;
    std::string content;
    liblo::fileToBuffer("./local/plugins.txt", content);
    boost::split(actualLines, content, [](char c) {
        return c == '\n';
    });
    actualLines.pop_back();  // Remove the trailing newline.
    actualLines.sort();

    EXPECT_EQ(expectedLines, actualLines);
}

#endif

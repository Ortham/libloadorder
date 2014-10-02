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

#include "api/libloadorder.h"
#include "backend/streams.h"

#include <iostream>
#include <boost/filesystem.hpp>
#include <gtest/gtest.h>

using std::endl;

namespace fs = boost::filesystem;

TEST(GetVersion, HandlesNullInput) {
    unsigned int vMajor, vMinor, vPatch;
    EXPECT_NE(LIBLO_OK, lo_get_version(&vMajor, NULL, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_version(NULL, &vMinor, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_version(NULL, NULL, &vPatch));
    EXPECT_NE(LIBLO_OK, lo_get_version(NULL, NULL, NULL));
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
    EXPECT_NE(LIBLO_OK, lo_get_error_message(NULL));

    const char * error;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    ASSERT_STREQ("Null pointer passed.", error);
}

TEST(Cleanup, CleansUpAfterError) {
    // First generate an error.
    EXPECT_NE(LIBLO_OK, lo_get_error_message(NULL));

    // Check that the error message is non-null.
    const char * error;
    EXPECT_EQ(LIBLO_OK, lo_get_error_message(&error));
    ASSERT_STREQ("Null pointer passed.", error);

    ASSERT_NO_THROW(lo_cleanup());

    // Now check that the error message pointer is null.
    const char * error = nullptr;
    EXPECT_NE(LIBLO_OK, lo_get_error_message(&error));
    EXPECT_EQ(nullptr, error);
}

TEST(Cleanup, HandlesNoError) {
    ASSERT_NO_THROW(lo_cleanup());

    const char * error = nullptr;
    EXPECT_NE(LIBLO_OK, lo_get_error_message(&error));
    EXPECT_EQ(nullptr, error);
}

class GameHandleCreationTest : public ::testing::Test {
protected:
    GameHandleCreationTest() : gh(NULL) {}

    virtual void TearDown() {
        ASSERT_NO_THROW(lo_destroy_handle(gh));
    };

    lo_game_handle gh;
};

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
    EXPECT_NE(LIBLO_OK, lo_create_handle(NULL, LIBLO_GAME_TES4, "./game", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidGameType) {
    EXPECT_NE(LIBLO_OK, lo_create_handle(&gh, UINT_MAX, "./game", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidGamePathInput) {
    EXPECT_NE(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, NULL, "./local"));
    EXPECT_NE(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, "/\0", "./local"));
}

TEST_F(GameHandleCreationTest, HandlesInvalidLocalPathInput) {
    EXPECT_NE(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, "./game", "/\0"));
}

TEST(GameHandleDestroyTest, HandledNullInput) {
    ASSERT_NO_THROW(lo_destroy_handle(NULL));
}

class OblivionOperationsTest : public ::testing::Test {
protected:
    virtual void SetUp() {
        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, "./game", "./local"));
    }

    virtual void TearDown() {
        lo_destroy_handle(gh);
    };

    lo_game_handle gh;
};

TEST_F(OblivionOperationsTest, SetGameMaster) {
    EXPECT_NE(LIBLO_OK, lo_set_game_master(gh, NULL));
    EXPECT_NE(LIBLO_OK, lo_set_game_master(gh, "Missing Plugin.esp"));
    EXPECT_NE(LIBLO_OK, lo_set_game_master(gh, "Can't be a plugin"));

    EXPECT_EQ(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm"));

    // Try setting to a master that doesn't exist.
    ASSERT_FALSE(fs::exists("./game/Data/EnhancedWeather.esm.missing"));
    EXPECT_NE(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm.missing"));
}

TEST_F(OblivionOperationsTest, GetLoadOrderMethod) {
    unsigned int method;
    EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gh, &method));
    EXPECT_EQ(LIBLO_METHOD_TIMESTAMP, method);

    EXPECT_NE(LIBLO_OK, lo_get_load_order_method(NULL, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_load_order_method(gh, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_load_order_method(NULL, &method));
}

TEST_F(OblivionOperationsTest, SetLoadOrder) {
    // Can't redistribute Oblivion.esm, but Nehrim.esm can be,
    // so use that for testing.
    char * plugins[1] = {
        "EnhancedWeather.esm"
    };
    size_t pluginsNum = 1;

    EXPECT_NE(LIBLO_OK, lo_set_load_order(gh, NULL, pluginsNum));
    EXPECT_NE(LIBLO_OK, lo_set_load_order(gh, NULL, 0));

    // Test trying to set load order with non-Oblivion.esm without
    // first setting the game master.
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, 0));
    EXPECT_NE(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));

    // Now set game master and try again.
    ASSERT_EQ(LIBLO_OK, lo_set_game_master(gh, "EnhancedWeather.esm"));
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins, pluginsNum));

    // Now test with more than one plugin.
    char * plugins2[] = {
        "EnhancedWeather.esm",
        "EnhancedWeather.esp"
    };
    pluginsNum = 2;
    EXPECT_EQ(LIBLO_OK, lo_set_load_order(gh, plugins2, pluginsNum));

    // Now test with more than one plugin, where one doesn't exist.
    ASSERT_FALSE(fs::exists("./game/Data/EnhancedWeather.esp.missing"));

    char * plugins3[] = {
        "EnhancedWeather.esm",
        "EnhancedWeather.esp.missing"
    };
    EXPECT_NE(LIBLO_OK, lo_set_load_order(gh, plugins3, pluginsNum));
}

TEST_F(OblivionOperationsTest, GetLoadOrder) {
    char ** plugins;
    size_t pluginsNum;
    EXPECT_NE(LIBLO_OK, lo_get_load_order(gh, NULL, &pluginsNum));
    EXPECT_NE(LIBLO_OK, lo_get_load_order(gh, &plugins, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_load_order(gh, NULL, NULL));

    EXPECT_EQ(LIBLO_OK, lo_get_load_order(gh, &plugins, &pluginsNum));
}

TEST_F(OblivionOperationsTest, SetPluginPosition) {
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_position(gh, "EnhancedWeather.esp", 1));
}

TEST_F(OblivionOperationsTest, GetPluginPosition) {
    size_t pos;
    EXPECT_EQ(LIBLO_OK, lo_get_plugin_position(gh, "EnhancedWeather.esm", &pos));
    EXPECT_EQ(0, pos);
}

TEST_F(OblivionOperationsTest, GetIndexedPlugin) {
    char * plugin;
    EXPECT_EQ(LIBLO_OK, lo_get_indexed_plugin(gh, 0, &plugin));
    EXPECT_STREQ("EnhancedWeather.esm", plugin);
}

TEST_F(OblivionOperationsTest, GetActivePlugins) {
    char ** plugins;
    size_t numPlugins;

    EXPECT_EQ(LIBLO_OK, lo_get_active_plugins(gh, &plugins, &numPlugins));
}

TEST_F(OblivionOperationsTest, SetActivePlugins) {
    char * plugins[] = {
        "EnhancedWeather.esm",
        "EnhancedWeather.esp",
        "Cava Obscura - Cyrodiil.esp"
    };
    size_t pluginsNum = 3;
    EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gh, plugins, pluginsNum));
}

TEST_F(OblivionOperationsTest, GetPluginActive) {
    bool isActive;
    EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gh, "Cava Obscura - Cyrodiil.esp", &isActive));
}

TEST_F(OblivionOperationsTest, SetPluginActive) {
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Cava Obscura - SI.esp", true));
}

TEST_F(OblivionOperationsTest, FixPluginLists) {
    EXPECT_NE(LIBLO_OK, lo_fix_plugin_lists(NULL));

    EXPECT_EQ(LIBLO_OK, lo_fix_plugin_lists(gh));
}

class SkyrimOperationsTest : public ::testing::Test {
protected:
    virtual void SetUp() {
        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, "./game", "./local"));
    }

    virtual void TearDown() {
        lo_destroy_handle(gh);
    };

    lo_game_handle gh;
};

TEST_F(SkyrimOperationsTest, GetLoadOrderMethod) {
    unsigned int method;
    EXPECT_EQ(LIBLO_OK, lo_get_load_order_method(gh, &method));
    EXPECT_EQ(LIBLO_METHOD_TEXTFILE, method);

    EXPECT_NE(LIBLO_OK, lo_get_load_order_method(NULL, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_load_order_method(gh, NULL));
    EXPECT_NE(LIBLO_OK, lo_get_load_order_method(NULL, &method));
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

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

#ifndef __LIBLO_TEST_API_ACTIVE_PLUGINS__
#define __LIBLO_TEST_API_ACTIVE_PLUGINS__

#include "tests/fixtures.h"

TEST_F(OblivionOperationsTest, GetActivePlugins) {
    char ** plugins;
    size_t numPlugins;

    EXPECT_EQ(LIBLO_OK, lo_get_active_plugins(gh, &plugins, &numPlugins));
}

TEST_F(OblivionOperationsTest, SetActivePlugins) {
    char * plugins[] = {
        "Blank.esm",
        "Blank.esp",
        "Blank - Master Dependent.esp"
    };
    size_t pluginsNum = 3;
    EXPECT_EQ(LIBLO_OK, lo_set_active_plugins(gh, plugins, pluginsNum));
}

TEST_F(OblivionOperationsTest, GetPluginActive) {
    bool isActive;
    EXPECT_EQ(LIBLO_OK, lo_get_plugin_active(gh, "Blank - Master Dependent.esp", &isActive));
}

TEST_F(OblivionOperationsTest, SetPluginActive) {
    EXPECT_EQ(LIBLO_OK, lo_set_plugin_active(gh, "Blank - Different Master Dependent.esp", true));
}

#endif

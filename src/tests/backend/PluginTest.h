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

#include "backend/Plugin.h"

namespace liblo {
    namespace test {
        class PluginTest : public ::testing::Test {};

        TEST_F(PluginTest, activatingAPluginShouldSetItToActive) {
            Plugin plugin;
            ASSERT_FALSE(plugin.isActive());

            plugin.activate();
            EXPECT_TRUE(plugin.isActive());
        }

        TEST_F(PluginTest, activatingAnActivePluginShouldHaveNoEffect) {
            Plugin plugin;
            plugin.activate();
            ASSERT_TRUE(plugin.isActive());

            plugin.activate();
            EXPECT_TRUE(plugin.isActive());
        }

        TEST_F(PluginTest, deactivatingAnActivePluginShouldSetItToNotActive) {
            Plugin plugin;
            plugin.activate();
            ASSERT_TRUE(plugin.isActive());

            plugin.deactivate();
            EXPECT_FALSE(plugin.isActive());
        }

        TEST_F(PluginTest, deactivatingAnInactivePluginShouldHaveNoEffect) {
            Plugin plugin;
            ASSERT_FALSE(plugin.isActive());

            plugin.deactivate();
            EXPECT_FALSE(plugin.isActive());
        }
    }
}

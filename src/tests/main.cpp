/*  libloadorder

    A library for reading and writing the load order of plugin files for
    TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3,
    Fallout: New Vegas and Fallout 4.

    Copyright (C) 2012-2015 Oliver Hamlet

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

#ifdef __GNUC__  // Workaround for GCC linking error.
#pragma message("GCC detected: Defining BOOST_NO_CXX11_SCOPED_ENUMS and BOOST_NO_SCOPED_ENUMS to avoid linking errors for boost::filesystem::copy_file().")
#define BOOST_NO_CXX11_SCOPED_ENUMS
#define BOOST_NO_SCOPED_ENUMS  // For older versions.
#endif

// Including from tests/ folder.
#include "api/_lo_game_handle_intTest.h"
#include "api/activeplugins.h"
#include "api/CHelpersTest.h"
#include "api/libloadorder.h"
#include "api/loadorder.h"
#include "backend/ErrorTest.h"
#include "backend/GameSettingsTest.h"
#include "backend/HelpersTest.h"
#include "backend/LoadOrderTest.h"
#include "backend/PluginTest.h"

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

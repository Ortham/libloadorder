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

#ifndef LIBLO_TEST_GAME_TEST
#define LIBLO_TEST_GAME_TEST

#include "libloadorder/constants.h"

#include <boost/filesystem.hpp>
#include <boost/filesystem/fstream.hpp>
#include <gtest/gtest.h>

namespace liblo {
    namespace test {
        class GameTest : public ::testing::TestWithParam<unsigned int> {
        protected:
            GameTest() :
                loadOrderMethod(getLoadOrderMethod()),
                localPath(getLocalPath()),
                pluginsPath(getPluginsPath()),
                gamePath(pluginsPath.parent_path()),
                activePluginsFilePath(getActivePluginsFilePath()),
                loadOrderFilePath(localPath / "loadorder.txt"),
                masterFile(getMasterFile()),
                blankEsm("Blank.esm"),
                blankDifferentEsm("Blank - Different.esm"),
                automatronDlcEsm("DLCRobot.esm"),
                wastelandWorkshopDlcEsm("DLCworkshop01.esm"),
                farHarborDlcEsm("DLCCoast.esm"),
                contraptionsWorkshopDlcEsm("DLCworkshop02.esm"),
                vaultTecWorkshopDlcEsm("DLCworkshop03.esm"),
                nukaWorldDlcEsm("DLCNukaWorld.esm"),
                invalidPlugin("NotAPlugin.esm") {}

            inline virtual void SetUp() {
                ASSERT_NO_THROW(boost::filesystem::create_directories(localPath));

                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankEsm));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / blankDifferentEsm));

                // Make sure the game master file exists.
                ASSERT_FALSE(boost::filesystem::exists(pluginsPath / masterFile));
                ASSERT_NO_THROW(boost::filesystem::copy_file(pluginsPath / blankEsm, pluginsPath / masterFile));
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / masterFile));

                // Write out an non-empty, non-plugin file.
                boost::filesystem::ofstream out(pluginsPath / invalidPlugin);
                out << "This isn't a valid plugin file.";
                out.close();
                ASSERT_TRUE(boost::filesystem::exists(pluginsPath / invalidPlugin));
            }

            inline virtual void TearDown() {
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / masterFile));
                ASSERT_NO_THROW(boost::filesystem::remove(pluginsPath / invalidPlugin));

                ASSERT_NO_THROW(boost::filesystem::remove(activePluginsFilePath));
                ASSERT_NO_THROW(boost::filesystem::remove(loadOrderFilePath));
            }

            inline std::string getActivePluginsFileLinePrefix() const {
                if (GetParam() == LIBLO_GAME_TES3)
                    return "GameFile0=";
                else if (GetParam() == LIBLO_GAME_FO4 || GetParam() == LIBLO_GAME_TES5SE)
                    return "*";
                else
                    return "";
            }

            const unsigned int loadOrderMethod;

            const boost::filesystem::path localPath;
            const boost::filesystem::path pluginsPath;
            const boost::filesystem::path gamePath;

            const boost::filesystem::path activePluginsFilePath;
            const boost::filesystem::path loadOrderFilePath;

            const std::string masterFile;
            const std::string blankEsm;
            const std::string blankDifferentEsm;
            const std::string automatronDlcEsm;
            const std::string wastelandWorkshopDlcEsm;
            const std::string farHarborDlcEsm;
            const std::string contraptionsWorkshopDlcEsm;
            const std::string vaultTecWorkshopDlcEsm;
            const std::string nukaWorldDlcEsm;
            const std::string invalidPlugin;

        private:
            inline boost::filesystem::path getLocalPath() const {
                if (GetParam() == LIBLO_GAME_TES3)
                    return "./local/Morrowind";
                else if (GetParam() == LIBLO_GAME_TES4)
                    return "./local/Oblivion";
                else
                    return "./local/Skyrim";
            }

            inline boost::filesystem::path getPluginsPath() const {
                if (GetParam() == LIBLO_GAME_TES3)
                    return "./Morrowind/Data Files";
                else if (GetParam() == LIBLO_GAME_TES4)
                    return "./Oblivion/Data";
                else
                    return "./Skyrim/Data";
            }

            inline std::string getMasterFile() const {
                if (GetParam() == LIBLO_GAME_TES3)
                    return "Morrowind.esm";
                else if (GetParam() == LIBLO_GAME_TES4)
                    return "Oblivion.esm";
                else if (GetParam() == LIBLO_GAME_TES5 || GetParam() == LIBLO_GAME_TES5SE)
                    return "Skyrim.esm";
                else if (GetParam() == LIBLO_GAME_FO3)
                    return "Fallout3.esm";
                else if (GetParam() == LIBLO_GAME_FNV)
                    return "FalloutNV.esm";
                else
                    return "Fallout4.esm";
            }

            inline boost::filesystem::path getActivePluginsFilePath() const {
                if (GetParam() == LIBLO_GAME_TES3)
                    return gamePath / "Morrowind.ini";
                else
                    return localPath / "plugins.txt";
            }

            inline unsigned int getLoadOrderMethod() const {
                if (GetParam() == LIBLO_GAME_FO4 || GetParam() == LIBLO_GAME_TES5SE)
                    return LIBLO_METHOD_ASTERISK;
                else if (GetParam() == LIBLO_GAME_TES5)
                    return LIBLO_METHOD_TEXTFILE;
                else
                    return LIBLO_METHOD_TIMESTAMP;
            }
        };
    }
}

#endif

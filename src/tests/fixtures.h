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

#ifdef __GNUC__  // Workaround for GCC linking error.
#pragma message("GCC detected: Defining BOOST_NO_CXX11_SCOPED_ENUMS and BOOST_NO_SCOPED_ENUMS to avoid linking errors for boost::filesystem::copy_file().")
#define BOOST_NO_CXX11_SCOPED_ENUMS
#define BOOST_NO_SCOPED_ENUMS  // For older versions.
#endif

#include "../../include/libloadorder/libloadorder.h"
#include "backend/streams.h"
#include "backend/plugins.h"

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <gtest/gtest.h>
#include <map>

class GameTest : public ::testing::Test {
protected:
    GameTest(const boost::filesystem::path& gameDataPath, const boost::filesystem::path& gameLocalPath)
        : dataPath(gameDataPath), localPath(gameLocalPath), missingPath("./missing"), gh(NULL) {}

    inline virtual void SetUp() {
        ASSERT_NO_THROW(boost::filesystem::create_directories(localPath));

        // Ghost a plugin.
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));
        ASSERT_NO_THROW(boost::filesystem::rename(dataPath / "Blank - Master Dependent.esm", dataPath / "Blank - Master Dependent.esm.ghost"));

        // Write out an empty file.
        liblo::ofstream out(dataPath / "EmptyFile.esm");
        out.close();

        // Write out an non-empty, non-plugin file.
        out.open(dataPath / "NotAPlugin.esm");
        out << "This isn't a valid plugin file.";
        out.close();

        GameTest::AssertInitialState();
    }

    inline virtual void TearDown() {
        // Unghost the ghosted plugin.
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));
        ASSERT_NO_THROW(boost::filesystem::rename(dataPath / "Blank - Master Dependent.esm.ghost", dataPath / "Blank - Master Dependent.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));

        // Delete generated files.
        ASSERT_NO_THROW(boost::filesystem::remove(dataPath / "EmptyFile.esm"));
        ASSERT_NO_THROW(boost::filesystem::remove(dataPath / "NotAPlugin.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "EmptyFile.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "NotAPlugin.esm"));

        ASSERT_NO_THROW(lo_destroy_handle(gh));
    }

    inline virtual bool CheckPluginActive(const std::string& filename) const = 0;

    inline virtual size_t CheckPluginPosition(const std::string& filename) const = 0;

    inline virtual void AssertInitialState() const {
        ASSERT_TRUE(boost::filesystem::exists(localPath));
        ASSERT_FALSE(boost::filesystem::exists(missingPath));

        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different Master Dependent.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different Master Dependent.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Plugin Dependent.esp"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Different Plugin Dependent.esp"));

        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank.missing.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Blank.missing.esp"));

        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Blank - Master Dependent.esm.ghost"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "EmptyFile.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "NotAPlugin.esm"));
    }

    inline static std::string GetFileContents(const boost::filesystem::path& filepath) {
        liblo::ifstream in(filepath.string().c_str(), std::ios::binary);
        if (in.good()) {
            std::string contents;
            in.seekg(0, std::ios::end);
            contents.resize(in.tellg());
            in.seekg(0, std::ios::beg);
            in.read(&contents[0], contents.size());
            in.close();
            return(contents);
        }
        throw std::runtime_error("Could not open file " + filepath.string());
    }

    const boost::filesystem::path dataPath;
    const boost::filesystem::path localPath;
    const boost::filesystem::path missingPath;

    lo_game_handle gh;
};

class NonTes3GameTest : public GameTest {
protected:
    NonTes3GameTest(const boost::filesystem::path& gameDataPath, const boost::filesystem::path& gameLocalPath) :
        GameTest(gameDataPath, gameLocalPath) {}

    inline virtual bool CheckPluginActive(const std::string& filename) const {
        liblo::ifstream activePlugins(localPath / "plugins.txt");

        bool found = false;
        while (activePlugins.good()) {
            std::string line;
            std::getline(activePlugins, line);

            if (boost::iequals(line, filename)) {
                if (found)
                    throw std::runtime_error(filename + " is listed twice in plugins.txt.");
                found = true;
            }
        }
        activePlugins.close();
        return found;
    }
};

class OblivionTest : public NonTes3GameTest {
protected:
    OblivionTest(const boost::filesystem::path& gameDataPath, const boost::filesystem::path& gameLocalPath) :
        NonTes3GameTest(gameDataPath, gameLocalPath) {}

    inline virtual size_t CheckPluginPosition(const std::string& filename) const {
        // Read the modification times of the plugins in the data folder.
        std::map<time_t, std::string> plugins;
        if (boost::filesystem::is_directory(dataPath)) {
            for (boost::filesystem::directory_iterator itr(dataPath); itr != boost::filesystem::directory_iterator(); ++itr) {
                if (boost::filesystem::is_regular_file(itr->status())) {
                    std::string file = itr->path().filename().string();
                    if (liblo::Plugin(file).IsValid(*gh)) {
                        auto result = plugins.insert(std::pair<time_t, std::string>(boost::filesystem::last_write_time(itr->path()), file));
                        if (!result.second) {
                            throw std::runtime_error(filename + " has the same timestamp as " + result.first->second);
                        }
                    }
                }
            }
        }

        size_t i = 0;
        for (auto it = plugins.begin(); it != plugins.end(); ++it) {
            if (boost::iequals(it->second, filename))
                return std::distance(plugins.begin(), it);
            ++i;
        }

        throw std::runtime_error(filename + " has no load order position.");
    }
};

class OblivionHandleCreationTest : public OblivionTest {
protected:
    inline OblivionHandleCreationTest() : OblivionTest("./Oblivion/Data", "./local/Oblivion") {}
};

class OblivionOperationsTest : public OblivionTest {
protected:
    OblivionOperationsTest() : OblivionTest("./Oblivion/Data", "./local/Oblivion") {}

    inline virtual void SetUp() {
        GameTest::SetUp();

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
        // Insert a blank line with a Windows line ending to ensure that the \r
        // doesn't break anything.
        liblo::ofstream activePlugins(localPath / "plugins.txt");
        activePlugins
            << "\r\n"
            << "Blank.esm" << std::endl;
        activePlugins.close();

        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES4, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    }

    inline virtual void TearDown() {
        GameTest::TearDown();

        // Delete existing plugins.txt.
        ASSERT_NO_THROW(boost::filesystem::remove(localPath / "plugins.txt"));
    };

    inline virtual void AssertInitialState() const {
        GameTest::AssertInitialState();

        // Check that active plugins list is in its initial state.
        std::stringstream ss;
        ss << "\r\n"
            << "Blank.esm" << std::endl;
        ASSERT_EQ(ss.str(), GetFileContents(localPath / "plugins.txt"));

        std::map<time_t, std::string> plugins;
        if (boost::filesystem::is_directory(dataPath)) {
            for (boost::filesystem::directory_iterator itr(dataPath); itr != boost::filesystem::directory_iterator(); ++itr) {
                if (boost::filesystem::is_regular_file(itr->status())) {
                    liblo::Plugin plugin(itr->path().filename().string());
                    if (plugin.IsValid(*gh)) {
                        auto result = plugins.insert(std::pair<time_t, std::string>(boost::filesystem::last_write_time(itr->path()), plugin.Name()));
                        if (!result.second) {
                            throw std::runtime_error(plugin.Name() + " has the same timestamp as " + result.first->second);
                        }
                    }
                }
            }
        }

        std::vector<std::string> loadOrder = {
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

        size_t i = 0;
        for (const auto &plugin : plugins) {
            ASSERT_EQ(loadOrder[i], plugin.second);
            ++i;
        }
    }
};

class SkyrimOperationsTest : public NonTes3GameTest {
protected:
    SkyrimOperationsTest() : NonTes3GameTest("./Skyrim/Data", "./local/Skyrim") {}

    inline virtual void SetUp() {
        GameTest::SetUp();

        // Can't change Skyrim's main master file, so mock it.
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Skyrim.esm"));
        ASSERT_NO_THROW(boost::filesystem::copy_file(dataPath / "Blank.esm", dataPath / "Skyrim.esm"));
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Skyrim.esm"));

        // Set Skyrim's load order to a known list before running the test.
        // Insert a blank line with a Windows line ending to ensure that the \r
        // doesn't break anything.
        liblo::ofstream loadOrder(localPath / "loadorder.txt");
        loadOrder
            << "Skyrim.esm" << std::endl
            << "Blank.esm" << std::endl
            << "Blank - Different.esm" << std::endl
            << "\r\n"
            //<< "Blank - Master Dependent.esm" << std::endl  // Ghosted
            << "Blank - Different Master Dependent.esm" << std::endl
            << "Blank.esp" << std::endl
            << "Blank - Different.esp" << std::endl
            << "Blank - Master Dependent.esp" << std::endl
            << "Blank - Different Master Dependent.esp" << std::endl
            << "Blank - Plugin Dependent.esp" << std::endl
            << "Blank - Different Plugin Dependent.esp" << std::endl;
        loadOrder.close();

        // Set Skyrim's active plugins to a known list before running the test.
        // Insert a blank line with a Windows line ending to ensure that the \r
        // doesn't break anything.
        liblo::ofstream activePlugins(localPath / "plugins.txt");
        activePlugins
            << "\r\n"
            << "Blank.esm" << std::endl;
        activePlugins.close();

        ASSERT_EQ(LIBLO_OK, lo_create_handle(&gh, LIBLO_GAME_TES5, dataPath.parent_path().string().c_str(), localPath.string().c_str()));
    }

    inline virtual void TearDown() {
        GameTest::TearDown();

        // Delete the mock Skyrim.esm.
        ASSERT_TRUE(boost::filesystem::exists(dataPath / "Skyrim.esm"));
        ASSERT_NO_THROW(boost::filesystem::remove(dataPath / "Skyrim.esm"));
        ASSERT_FALSE(boost::filesystem::exists(dataPath / "Skyrim.esm"));

        // Delete existing plugins.txt and loadorder.txt.
        ASSERT_NO_THROW(boost::filesystem::remove(localPath / "plugins.txt"));
        ASSERT_NO_THROW(boost::filesystem::remove(localPath / "loadorder.txt"));
    };

    inline virtual size_t CheckPluginPosition(const std::string& filename) const {
        liblo::ifstream activePlugins(localPath / "loadorder.txt");

        size_t i = 0;
        while (activePlugins.good()) {
            std::string line;
            std::getline(activePlugins, line);

            if (boost::iequals(line, filename)) {
                activePlugins.close();
                return i;
            }
            ++i;
        }
        activePlugins.close();

        throw std::runtime_error(filename + " does not have a load order position defined.");
    }

    inline virtual void AssertInitialState() const {
        GameTest::AssertInitialState();

        // Check that active plugins list is in its initial state.
        std::stringstream apss;
        apss << "\r\n"
            << "Blank.esm" << std::endl;
        ASSERT_EQ(apss.str(), GetFileContents(localPath / "plugins.txt"));

        // Check that the load order is in its initial state.
        std::stringstream loss;
        loss << "Skyrim.esm" << std::endl
            << "Blank.esm" << std::endl
            << "Blank - Different.esm" << std::endl
            << "\r\n"
            //<< "Blank - Master Dependent.esm" << std::endl  // Ghosted
            << "Blank - Different Master Dependent.esm" << std::endl
            << "Blank.esp" << std::endl
            << "Blank - Different.esp" << std::endl
            << "Blank - Master Dependent.esp" << std::endl
            << "Blank - Different Master Dependent.esp" << std::endl
            << "Blank - Plugin Dependent.esp" << std::endl
            << "Blank - Different Plugin Dependent.esp" << std::endl;
        ASSERT_EQ(loss.str(), GetFileContents(localPath / "loadorder.txt"));
    }
};

#endif

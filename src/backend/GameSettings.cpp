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

#include "libloadorder/constants.h"
#include "GameSettings.h"
#include "helpers.h"
#include "error.h"

#include <boost/algorithm/string.hpp>

#ifdef _WIN32
#   ifndef UNICODE
#       define UNICODE
#   endif
#   ifndef _UNICODE
#      define _UNICODE
#   endif
#   include "windows.h"
#   include "shlobj.h"
#endif

using namespace std;
using namespace liblo;

namespace fs = boost::filesystem;

namespace liblo {
    GameSettings::GameSettings(unsigned int id, const boost::filesystem::path& gamePath, const boost::filesystem::path& localPath)
        : id(id),
        gamePath(gamePath) {
        if (id == LIBLO_GAME_TES3) {
            loMethod = LIBLO_METHOD_TIMESTAMP;
            masterFile = "Morrowind.esm";

            appdataFolderName = "";
            pluginsFolderName = "Data Files";
            pluginsFileName = "Morrowind.ini";
        }
        else if (id == LIBLO_GAME_TES4) {
            loMethod = LIBLO_METHOD_TIMESTAMP;
            masterFile = "Oblivion.esm";

            appdataFolderName = "Oblivion";
            pluginsFolderName = "Data";
            pluginsFileName = "plugins.txt";
        }
        else if (id == LIBLO_GAME_TES5) {
            loMethod = LIBLO_METHOD_TEXTFILE;
            masterFile = "Skyrim.esm";

            appdataFolderName = "Skyrim";
            pluginsFolderName = "Data";
            pluginsFileName = "plugins.txt";
        }
        else if (id == LIBLO_GAME_FO3) {
            loMethod = LIBLO_METHOD_TIMESTAMP;
            masterFile = "Fallout3.esm";

            appdataFolderName = "Fallout3";
            pluginsFolderName = "Data";
            pluginsFileName = "plugins.txt";
        }
        else if (id == LIBLO_GAME_FNV) {
            loMethod = LIBLO_METHOD_TIMESTAMP;
            masterFile = "FalloutNV.esm";

            appdataFolderName = "FalloutNV";
            pluginsFolderName = "Data";
            pluginsFileName = "plugins.txt";
        }
        else if (id == LIBLO_GAME_FO4) {
            loMethod = LIBLO_METHOD_ASTERISK;
            masterFile = "Fallout4.esm";

            appdataFolderName = "Fallout4";
            pluginsFolderName = "Data";
            pluginsFileName = "plugins.txt";
        }

        if (localPath.empty())
            initPaths(getLocalAppDataPath() / appdataFolderName);
        else
            initPaths(localPath);
    }

    void GameSettings::initPaths(const boost::filesystem::path& localPath) {
        // Set active plugins and load order files.
        if (id == LIBLO_GAME_TES4 && fs::exists(gamePath / "Oblivion.ini")) {
            // Looking up bUseMyGamesDirectory, which only has effect if =0 and
            // exists in Oblivion folder.
            string iniContent = fileToBuffer(gamePath / "Oblivion.ini");
            string iniSetting = "bUseMyGamesDirectory=";

            size_t pos = iniContent.find(iniSetting);
            if (pos != string::npos && pos + iniSetting.length() < iniContent.length() && iniContent[pos + iniSetting.length()] == '0') {
                pluginsPath = gamePath / pluginsFileName;
            }
            else {
                pluginsPath = localPath / pluginsFileName;
            }
        }
        else if (id == LIBLO_GAME_TES3) {
            pluginsPath = gamePath / pluginsFileName;
        }
        else {
            pluginsPath = localPath / pluginsFileName;
            loadorderPath = localPath / "loadorder.txt";
        }
    }

    unsigned int GameSettings::getId() const {
        return id;
    }

    libespm::GameId GameSettings::getLibespmId() const {
        if (id == LIBLO_GAME_TES3)
            return libespm::GameId::MORROWIND;
        else if (id == LIBLO_GAME_TES4)
            return libespm::GameId::OBLIVION;
        else
            return libespm::GameId::SKYRIM;
    }

    string GameSettings::getMasterFile() const {
        return masterFile;
    }

    unsigned int GameSettings::getLoadOrderMethod() const {
        return loMethod;
    }

    std::vector<std::string> GameSettings::getImplicitlyActivePlugins() const {
        if (id == LIBLO_GAME_TES5) {
            return std::vector<std::string>({
                masterFile,
                "Update.esm",
            });
        }
        else if (id == LIBLO_GAME_FO4) {
            return std::vector<std::string>({
                masterFile,
                "DLCRobot.esm",
                "DLCworkshop01.esm",
                "DLCCoast.esm",
            });
        }

        return std::vector<std::string>();
    }

    bool GameSettings::isImplicitlyActive(const std::string & pluginName) const {
        auto implicitlyActivePlugins = getImplicitlyActivePlugins();

        auto it = find_if(begin(implicitlyActivePlugins),
                          end(implicitlyActivePlugins),
                          [&](const string& name) {
            return boost::iequals(pluginName, name);
        });

        return it != end(implicitlyActivePlugins);
    }

    boost::filesystem::path GameSettings::getPluginsFolder() const {
        return gamePath / pluginsFolderName;
    }

    boost::filesystem::path GameSettings::getActivePluginsFile() const {
        if (pluginsPath.empty())
            throw error(LIBLO_ERROR_INVALID_ARGS, "No local app data path set.");
        return pluginsPath;
    }

    boost::filesystem::path GameSettings::getLoadOrderFile() const {
        if (loadorderPath.empty())
            throw error(LIBLO_ERROR_INVALID_ARGS, "No local app data path set.");
        if (loMethod != LIBLO_METHOD_TEXTFILE)
            throw error(LIBLO_ERROR_INVALID_ARGS, "This game has no load order file.");

        return loadorderPath;
    }

    boost::filesystem::path GameSettings::getLocalAppDataPath() const {
#ifdef _WIN32
        HWND owner = 0;
        TCHAR path[MAX_PATH];

        HRESULT res = SHGetFolderPath(owner, CSIDL_LOCAL_APPDATA, NULL, SHGFP_TYPE_CURRENT, path);

        const int utf8Len = WideCharToMultiByte(CP_UTF8, 0, path, -1, NULL, 0, NULL, NULL);
        char * narrowPath = new char[utf8Len];
        WideCharToMultiByte(CP_UTF8, 0, path, -1, narrowPath, utf8Len, NULL, NULL);

        if (res == S_OK)
            return fs::path(narrowPath);
        else
            return fs::path("");
#else
        throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot detect local app data path on non-Windows OS's.");
#endif
    }
}

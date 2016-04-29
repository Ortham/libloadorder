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

#ifndef LIBLO_GAME_SETTINGS_H
#define LIBLO_GAME_SETTINGS_H

#include <string>

#include <boost/filesystem.hpp>

#include <libespm/GameId.h>

namespace liblo {
    class GameSettings {
    public:
        GameSettings(unsigned int id, const boost::filesystem::path& gamePath, const boost::filesystem::path& localPath = "");

        unsigned int getId() const;
        libespm::GameId getLibespmId() const;
        std::string getMasterFile() const;
        unsigned int getLoadOrderMethod() const;
        std::vector<std::string> getImplicitlyActivePlugins() const;
        bool isImplicitlyActive(const std::string& pluginName) const;

        boost::filesystem::path getPluginsFolder() const;
        boost::filesystem::path getActivePluginsFile() const;
        boost::filesystem::path getLoadOrderFile() const;
    private:
        unsigned int id;
        unsigned int loMethod;

        std::string masterFile;

        std::string appdataFolderName;
        std::string pluginsFolderName;
        std::string pluginsFileName;

        boost::filesystem::path gamePath;
        boost::filesystem::path pluginsPath;
        boost::filesystem::path loadorderPath;

        void initPaths(const boost::filesystem::path& localPath);

        boost::filesystem::path getLocalAppDataPath() const;
    };
}

#endif

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

#ifndef __LIBLO_GAME_H__
#define __LIBLO_GAME_H__

#include "plugins.h"
#include <string>
#include <vector>
#include <stdint.h>
#include <boost/filesystem.hpp>
#include <src/playground.h>

struct _lo_game_handle_int {
    public:
        _lo_game_handle_int(unsigned int id, const std::string& path);
        ~_lo_game_handle_int();

        void SetMasterFile(const std::string& file);

        unsigned int Id() const;
        std::string MasterFile() const;
        unsigned int LoadOrderMethod() const;

        boost::filesystem::path PluginsFolder() const;
        boost::filesystem::path ActivePluginsFile() const;
        boost::filesystem::path LoadOrderFile() const;

        liblo::LoadOrder loadOrder;
        liblo::ActivePlugins activePlugins;

        char * extString;
        char ** extStringArray;

        size_t extStringArraySize;

        espm::Settings espm_settings;
    private:
        unsigned int id;
        unsigned int loMethod;

        std::string executable;
        std::string masterFile;

        std::string appdataFolderName;
        std::string pluginsFolderName;
        std::string pluginsFileName;

        boost::filesystem::path gamePath;
        boost::filesystem::path pluginsPath;
        boost::filesystem::path loadorderPath;

        boost::filesystem::path GetLocalAppDataPath() const;
};

#endif

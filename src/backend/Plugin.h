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

#ifndef LIBLO_PLUGIN_H
#define LIBLO_PLUGIN_H

#include <string>
#include <vector>

#include <libespm/Plugin.h>

struct _lo_game_handle_int;

namespace liblo {
    class Plugin {
    public:
        Plugin();
        Plugin(const std::string& filename);  //Automatically trims .ghost extension.

        std::string Name() const;

        bool    IsValid(const _lo_game_handle_int& parentGame) const;  // Attempts to parse the plugin header.
        bool    IsMasterFile(const _lo_game_handle_int& parentGame) const;         // Checks master flag bit.
        bool    IsGhosted(const _lo_game_handle_int& parentGame) const;         //Checks if the file exists in ghosted form.
        bool    Exists(const _lo_game_handle_int& parentGame) const;         //Checks if the file exists in the data folder, ghosted or not.
        time_t  GetModTime(const _lo_game_handle_int& parentGame) const;         //Can throw exception.
        std::vector<Plugin> GetMasters(const _lo_game_handle_int& parentGame) const;

        void    UnGhost(const _lo_game_handle_int& parentGame) const;         //Can throw exception.
        void    SetModTime(const _lo_game_handle_int& parentGame, const time_t modificationTime) const;

        bool isActive() const;

        void activate();
        void deactivate();

        bool operator == (const Plugin& rhs) const;
        bool operator != (const Plugin& rhs) const;
    private:
        std::string name;
        bool active;

        libespm::Plugin ReadHeader(const _lo_game_handle_int& parentGame) const;
    };
}

#endif

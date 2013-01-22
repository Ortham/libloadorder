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

#ifndef __LIBLO_PLUGINS_H__
#define __LIBLO_PLUGINS_H__

#include "exception.h"
#include <string>
#include <vector>
#include <stdint.h>
#include <boost/unordered_set.hpp>
#include <boost/filesystem.hpp>

struct _lo_game_handle_int;

namespace liblo {

    class Plugin {
    public:
        Plugin();
        Plugin(const std::string& filename);  //Automatically trims .ghost extension.

        std::string Name() const;

        bool    IsValid     () const;  //.Checks if plugin is a .esp or .esm file.
        bool    IsMasterFile(const _lo_game_handle_int& parentGame) const;         //This should be implemented using libespm.
        bool    IsFalseFlagged(const _lo_game_handle_int& parentGame) const;           //True if IsMasterFile does not match file extension.
        bool    IsGhosted   (const _lo_game_handle_int& parentGame) const;         //Checks if the file exists in ghosted form.
        bool    Exists      (const _lo_game_handle_int& parentGame) const;         //Checks if the file exists in the data folder, ghosted or not.
        time_t  GetModTime  (const _lo_game_handle_int& parentGame) const;         //Can throw exception.
        std::vector<Plugin> GetMasters(const _lo_game_handle_int& parentGame) const;

        void    UnGhost     (const _lo_game_handle_int& parentGame) const;         //Can throw exception.
        void    SetModTime  (const _lo_game_handle_int& parentGame, const time_t modificationTime) const;

        bool operator == (const Plugin& rhs) const;
        bool operator != (const Plugin& rhs) const;
    private:
        std::string name;
    };

    std::size_t hash_value(const Plugin& p);

    class LoadOrder : public std::vector<Plugin> {
    public:
        void Load(const _lo_game_handle_int& parentGame);
        void Save(_lo_game_handle_int& parentGame);  //Also updates mtime and active plugins list.

        bool IsValid(const _lo_game_handle_int& parentGame) const;  //Game master first, masters before plugins, plugins all exist.

        bool HasChanged(const _lo_game_handle_int& parentGame) const;  //Checks timestamp and also if LoadOrder is empty.

        void Move(size_t newPos, const Plugin& plugin);

        size_t Find(const Plugin& plugin) const;
        size_t LastMasterPos(const _lo_game_handle_int& parentGame) const;

        //Assumes that the content of the file is valid.
        void LoadFromFile(const _lo_game_handle_int& parentGame, const boost::filesystem::path& file);
    private:
        time_t mtime;
    };

    class ActivePlugins : public boost::unordered_set<Plugin> {
    public:
        void Load(const _lo_game_handle_int& parentGame);
        void Save(const _lo_game_handle_int& parentGame);

        bool IsValid(const _lo_game_handle_int& parentGame) const;  //not more than 255 plugins active (254 for Skyrim), plugins all exist.

        bool HasChanged(const _lo_game_handle_int& parentGame) const;
    private:
        time_t mtime;
    };
}

#endif

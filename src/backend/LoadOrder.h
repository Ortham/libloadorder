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

#include "Plugin.h"

#include <string>
#include <vector>
#include <unordered_set>

#include <boost/filesystem.hpp>
#include <boost/locale.hpp>

struct _lo_game_handle_int;

namespace liblo {
    class GameSettings;

    class LoadOrder {
    public:
        static const unsigned int maxActivePlugins = 255;

        LoadOrder(const GameSettings& gameSettings);

        void load();
        void save();  //Also updates mtime and active plugins list.

        std::vector<std::string> getLoadOrder() const;
        size_t getPosition(const std::string& pluginName) const;
        std::string getPluginAtPosition(size_t index) const;

        void setLoadOrder(const std::vector<std::string>& pluginNames);
        void setPosition(const std::string& pluginName, size_t loadOrderIndex);

        std::unordered_set<std::string> getActivePlugins() const;
        bool isActive(const std::string& pluginName) const;

        void setActivePlugins(const std::unordered_set<std::string>& pluginNames);
        void activate(const std::string& pluginName);
        void deactivate(const std::string& pluginName);

        bool hasFilesystemChanged() const;
        static bool isSynchronised(const GameSettings& gameSettings);

        void clear();
    private:
        time_t mtime;
        std::vector<Plugin> loadOrder;
        const GameSettings& gameSettings;

        void loadFromFile(const boost::filesystem::path& file);
        void loadActivePlugins();

        void saveTimestampLoadOrder();
        void saveTextfileLoadOrder();
        void saveActivePlugins();

        size_t getMasterPartitionPoint() const;
        size_t countActivePlugins() const;
        Plugin getPluginObject(const std::string& pluginName) const;

        std::vector<Plugin>::iterator addToLoadOrder(const std::string& pluginName);
        void partitionMasters();
    };
}

namespace std {
    template <>
    struct hash < liblo::Plugin > {
        size_t operator()(const liblo::Plugin& p) const {
            return hash<std::string>()(boost::locale::to_lower(p.Name()));
        }
    };
}

namespace liblo {
    class ActivePlugins : public std::unordered_set < Plugin > {
    public:
        void Load(const GameSettings& parentGame);
        void Save(const _lo_game_handle_int& parentGame);

        void CheckValidity(const GameSettings& parentGame) const;  //not more than 255 plugins active (254 for Skyrim), plugins all exist.

        bool HasChanged(const GameSettings& parentGame) const;
    private:
        time_t mtime;
    };
}

#endif

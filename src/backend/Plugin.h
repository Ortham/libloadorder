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

namespace liblo {
    class GameSettings;

    class Plugin : public libespm::Plugin {
    public:
        Plugin(const std::string& filename, const GameSettings& gameSettings);

        std::string getName() const;
        time_t getModTime() const;
        bool isActive() const;
        bool hasFileChanged(const boost::filesystem::path& pluginsFolder) const;

        void setModTime(const time_t modificationTime, const boost::filesystem::path& pluginsFolder);
        void activate(const boost::filesystem::path& pluginsFolder);
        void deactivate();

        bool operator == (const Plugin& rhs) const;
        bool operator != (const Plugin& rhs) const;
        bool operator == (const std::string& rhs) const;
        bool operator != (const std::string& rhs) const;

        static bool isValid(const std::string& filename, const GameSettings& gameSettings);
    private:
        bool active;
        time_t modTime;

        static std::string trimGhostExtension(const std::string& filename);
    };
}

#endif

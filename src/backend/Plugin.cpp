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

#include "Plugin.h"
#include "libloadorder/constants.h"
#include "error.h"
#include "GameSettings.h"

#include <boost/filesystem.hpp>
#include <boost/algorithm/string.hpp>

using namespace std;
namespace fs = boost::filesystem;

namespace liblo {
    Plugin::Plugin() : active(false) {}

    Plugin::Plugin(const string& filename) : name(filename), active(false) {
        if (!name.empty() && name[name.length() - 1] == '\r')
            name = name.substr(0, name.length() - 1);
        if (boost::iends_with(name, ".ghost"))
            name = fs::path(name).stem().string();
    };

    string Plugin::Name() const {
        return name;
    }

    bool Plugin::IsValid(const GameSettings& parentGame) const {
        // Rather than just checking the extension, try also parsing the file, and see if it fails.
        if (!boost::iends_with(name, ".esm") && !boost::iends_with(name, ".esp"))
            return false;
        try {
            libespm::Plugin plugin = ReadHeader(parentGame);
        }
        catch (std::exception& /*e*/) {
            return false;
        }
        return true;
    }

    bool Plugin::IsMasterFile(const GameSettings& parentGame) const {
        try {
            libespm::Plugin plugin = ReadHeader(parentGame);

            return plugin.isMasterFile();
        }
        catch (std::exception&) {
            return false;
        }
    }

    bool Plugin::IsGhosted(const GameSettings& parentGame) const {
        return (fs::exists(parentGame.getPluginsFolder() / fs::path(name + ".ghost")));
    }

    bool Plugin::Exists(const GameSettings& parentGame) const {
        return (fs::exists(parentGame.getPluginsFolder() / name) || fs::exists(parentGame.getPluginsFolder() / fs::path(name + ".ghost")));
    }

    time_t Plugin::GetModTime(const GameSettings& parentGame) const {
        try {
            if (IsGhosted(parentGame))
                return fs::last_write_time(parentGame.getPluginsFolder() / fs::path(name + ".ghost"));
            else
                return fs::last_write_time(parentGame.getPluginsFolder() / name);
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_READ_FAIL, e.what());
        }
    }

    std::vector<Plugin> Plugin::GetMasters(const GameSettings& parentGame) const {
        libespm::Plugin plugin = ReadHeader(parentGame);

        vector<Plugin> masters;
        for (const auto &master : plugin.getMasters()) {
            masters.push_back(Plugin(master));
        }

        return masters;
    }

    void Plugin::UnGhost(const GameSettings& parentGame) const {
        if (IsGhosted(parentGame)) {
            try {
                fs::rename(parentGame.getPluginsFolder() / fs::path(name + ".ghost"), parentGame.getPluginsFolder() / name);
            }
            catch (fs::filesystem_error& e) {
                throw error(LIBLO_ERROR_FILE_RENAME_FAIL, e.what());
            }
        }
    }

    void Plugin::SetModTime(const GameSettings& parentGame, const time_t modificationTime) const {
        try {
            if (IsGhosted(parentGame))
                fs::last_write_time(parentGame.getPluginsFolder() / fs::path(name + ".ghost"), modificationTime);
            else
                fs::last_write_time(parentGame.getPluginsFolder() / name, modificationTime);
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_WRITE_FAIL, e.what());
        }
    }

    bool Plugin::isActive() const {
        return active;
    }

    void Plugin::activate() {
        active = true;
    }

    void Plugin::deactivate() {
        active = false;
    }

    bool Plugin::operator == (const Plugin& rhs) const {
        return boost::iequals(name, rhs.Name());
    }

    bool Plugin::operator != (const Plugin& rhs) const {
        return !(*this == rhs);
    }

    libespm::Plugin Plugin::ReadHeader(const GameSettings& parentGame) const {
        if (!Exists(parentGame))
            throw error(LIBLO_ERROR_FILE_NOT_FOUND, name.c_str());

        try {
            string filepath = (parentGame.getPluginsFolder() / name).string();
            if (IsGhosted(parentGame))
                filepath += ".ghost";

            libespm::Plugin plugin(parentGame.getLibespmId());
            plugin.load(filepath, true);

            return plugin;
        }
        catch (std::exception& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, name + " : " + e.what());
        }
    }
}

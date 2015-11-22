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

#include "Plugin.h"
#include "libloadorder/constants.h"
#include "error.h"
#include "GameSettings.h"

#include <boost/filesystem.hpp>
#include <boost/algorithm/string.hpp>

using namespace std;
namespace fs = boost::filesystem;

namespace liblo {
    Plugin::Plugin(const string& filename, const GameSettings& gameSettings) :
        libespm::Plugin(gameSettings.getLibespmId()),
        active(false),
        modTime(0) {
        fs::path filePath = gameSettings.getPluginsFolder() / filename;
        if (!fs::exists(filePath) && fs::exists(filePath.string() + ".ghost")) {
            filePath += ".ghost";
        }

        try {
            modTime = fs::last_write_time(filePath);
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_READ_FAIL, e.what());
        }

        try {
            load(filePath, true);
        }
        catch (std::exception& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, getName() + " : " + e.what());
        }
    }

    std::string Plugin::getName() const {
        return trimGhostExtension(libespm::Plugin::getName());
    }

    time_t Plugin::getModTime() const {
        return modTime;
    }

    bool Plugin::isActive() const {
        return active;
    }

    bool Plugin::hasFileChanged(const boost::filesystem::path& pluginsFolder) const {
        return modTime != fs::last_write_time(pluginsFolder / libespm::Plugin::getName());
    }

    void Plugin::setModTime(const time_t modificationTime, const boost::filesystem::path& pluginsFolder) {
        try {
            fs::last_write_time(pluginsFolder / libespm::Plugin::getName(), modificationTime);
            modTime = fs::last_write_time(pluginsFolder / libespm::Plugin::getName());
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_WRITE_FAIL, e.what());
        }
    }

    void Plugin::activate(const boost::filesystem::path& pluginsFolder) {
        if (active)
            return;

        // Also unghost the file if it's ghosted.
        if (boost::iends_with(libespm::Plugin::getName(), ".ghost")) {
            try {
                fs::rename(pluginsFolder / libespm::Plugin::getName(), pluginsFolder / getName());
                load(pluginsFolder / getName(), true);
                modTime = fs::last_write_time(pluginsFolder / getName());
            }
            catch (fs::filesystem_error& e) {
                throw error(LIBLO_ERROR_FILE_RENAME_FAIL, e.what());
            }
        }
        active = true;
    }

    void Plugin::deactivate() {
        active = false;
    }

    bool Plugin::operator == (const Plugin& rhs) const {
        return boost::iequals(getName(), rhs.getName());
    }

    bool Plugin::operator != (const Plugin& rhs) const {
        return !(*this == rhs);
    }

    bool Plugin::operator == (const std::string& rhs) const {
        return boost::iequals(getName(), rhs) || boost::iequals(getName(), trimGhostExtension(rhs));
    }

    bool Plugin::operator != (const std::string& rhs) const {
        return !(*this == rhs);
    }

    bool Plugin::isValid(const std::string& filename, const GameSettings& gameSettings) {
        string name = trimGhostExtension(filename);

        return libespm::Plugin::isValid(gameSettings.getPluginsFolder() / name, gameSettings.getLibespmId())
            || libespm::Plugin::isValid(gameSettings.getPluginsFolder() / (name + ".ghost"), gameSettings.getLibespmId());
    }

    std::string Plugin::trimGhostExtension(const std::string& filename) {
        string name(filename);
        if (boost::iends_with(name, ".ghost"))
            name = name.substr(0, name.length() - 6);
        return name;
    }
}

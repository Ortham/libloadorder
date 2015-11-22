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

#include "LoadOrder.h"
#include "../api/_lo_game_handle_int.h"
#include "libloadorder/constants.h"
#include "error.h"
#include "helpers.h"

#include <regex>
#include <set>
#include <unordered_map>

#include <boost/algorithm/string.hpp>

using namespace std;
namespace fs = boost::filesystem;

namespace liblo {
    LoadOrder::LoadOrder(const GameSettings& gameSettings) : gameSettings(gameSettings) {}

    void LoadOrder::load() {
        loadOrder.clear();
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            if (fs::exists(gameSettings.getLoadOrderFile()))
                loadFromFile(gameSettings.getLoadOrderFile());
            else if (fs::exists(gameSettings.getActivePluginsFile()))
                loadFromFile(gameSettings.getActivePluginsFile());
        }
        if (fs::is_directory(gameSettings.getPluginsFolder())) {
            // Now scan through Data folder. Add any plugins that aren't
            // already in load order.
            auto firstNonMaster = getMasterPartitionPoint();
            for (fs::directory_iterator itr(gameSettings.getPluginsFolder()); itr != fs::directory_iterator(); ++itr) {
                if (fs::is_regular_file(itr->status())) {
                    const std::string filename(itr->path().filename().string());
                    if (Plugin::isValid(filename, gameSettings) && count(begin(loadOrder), end(loadOrder), filename) == 0)
                        loadOrder.push_back(Plugin(filename, gameSettings));
                }
            }
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                stable_sort(begin(loadOrder),
                            end(loadOrder),
                            [&](const Plugin& lhs, const Plugin& rhs) {
                    return difftime(lhs.getModTime(),
                                    rhs.getModTime()) < 0;
                });
            }
            partitionMasters();
            mtime = boost::filesystem::last_write_time(gameSettings.getPluginsFolder());
        }
        if (fs::exists(gameSettings.getActivePluginsFile()))
            loadActivePlugins();
    }

    void LoadOrder::save() {
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
            saveTimestampLoadOrder();
        else
            saveTextfileLoadOrder();
        saveActivePlugins();
    }

    std::vector<std::string> LoadOrder::getLoadOrder() const {
        std::vector<std::string> pluginNames;
        transform(begin(loadOrder),
                  end(loadOrder),
                  back_inserter(pluginNames),
                  [](const Plugin& plugin) {
            return plugin.getName();
        });
        return pluginNames;
    }

    size_t LoadOrder::getPosition(const std::string& pluginName) const {
        return distance(begin(loadOrder),
                        find(begin(loadOrder),
                        end(loadOrder),
                        pluginName));
    }

    std::string LoadOrder::getPluginAtPosition(size_t index) const {
        return loadOrder.at(index).getName();
    }

    void LoadOrder::setLoadOrder(const std::vector<std::string>& pluginNames) {
        // For textfile-based load order games, check that the game's master file loads first.
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE && (pluginNames.empty() || !boost::iequals(pluginNames[0], gameSettings.getMasterFile())))
            throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + gameSettings.getMasterFile() + "\" must load first.");

        // Create vector of Plugin objects, reusing existing objects
        // where possible. Also check for duplicate entries, that new
        // plugins are valid,
        vector<Plugin> plugins;
        unordered_set<string> hashset;
        for_each(begin(pluginNames), end(pluginNames), [&](const std::string& pluginName) {
            if (hashset.find(boost::to_lower_copy(pluginName)) != hashset.end())
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is a duplicate entry.");

            hashset.insert(boost::to_lower_copy(pluginName));
            plugins.push_back(getPluginObject(pluginName));
        });

        // Check that all masters load before non-masters.
        if (!is_partitioned(begin(plugins),
            end(plugins),
            [&](const Plugin& plugin) {
            return plugin.isMasterFile();
        })) {
            throw error(LIBLO_ERROR_INVALID_ARGS, "Master plugins must load before all non-master plugins.");
        }

        // Swap load order for the new one.
        loadOrder.swap(plugins);

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Make sure that game master is active.
            loadOrder.front().activate(gameSettings.getPluginsFolder());
        }
    }

    void LoadOrder::setPosition(const std::string& pluginName, size_t loadOrderIndex) {
        // For textfile-based load order games, check that this doesn't move the game master file from the beginning of the load order.
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            if (loadOrderIndex == 0 && !boost::iequals(pluginName, gameSettings.getMasterFile()))
                throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot set \"" + pluginName + "\" to load first: \"" + gameSettings.getMasterFile() + "\" most load first.");
            else if (loadOrderIndex != 0 && !loadOrder.empty() && boost::iequals(pluginName, gameSettings.getMasterFile()))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" must load first.");
        }

        // If the plugin is already in the load order, use its existing
        // object.
        Plugin plugin = getPluginObject(pluginName);

        // Check that a master isn't being moved before a non-master or the inverse.
        size_t masterPartitionPoint(getMasterPartitionPoint());
        if (!plugin.isMasterFile() && loadOrderIndex < masterPartitionPoint)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot move a non-master plugin before master files.");
        else if (plugin.isMasterFile()
                 && ((loadOrderIndex > masterPartitionPoint && masterPartitionPoint != loadOrder.size())
                 || (getPosition(pluginName) < masterPartitionPoint && loadOrderIndex == masterPartitionPoint)))
                 throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot move a master file after non-master plugins.");

        // Erase any existing entry for the plugin.
        loadOrder.erase(remove(begin(loadOrder), end(loadOrder), pluginName), end(loadOrder));

        // If the index is larger than the load order size, set it equal to the size.
        if (loadOrderIndex > loadOrder.size())
            loadOrderIndex = loadOrder.size();

        loadOrder.insert(next(begin(loadOrder), loadOrderIndex), plugin);
    }

    std::unordered_set<std::string> LoadOrder::getActivePlugins() const {
        unordered_set<string> activePlugins;
        for_each(begin(loadOrder),
                 end(loadOrder),
                 [&](const Plugin& plugin) {
            if (plugin.isActive())
                activePlugins.insert(plugin.getName());
        });
        return activePlugins;
    }

    bool LoadOrder::isActive(const std::string& pluginName) const {
        return find_if(begin(loadOrder), end(loadOrder), [&](const Plugin& plugin) {
            return plugin == pluginName && plugin.isActive();
        }) != end(loadOrder);
    }

    void LoadOrder::setActivePlugins(const std::unordered_set<std::string>& pluginNames) {
        if (pluginNames.size() > maxActivePlugins)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot activate more than " + to_string(maxActivePlugins) + " plugins.");

        // Check all plugins are valid.
        for_each(begin(pluginNames), end(pluginNames), [&](const std::string& pluginName) {
            if (count(begin(loadOrder), end(loadOrder), pluginName) == 0
                && !Plugin::isValid(pluginName, gameSettings))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");
        });

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Check that the game master file is active.
            // Need to first lowercase all plugin names for comparison.
            unordered_set<string> lowercasedPluginNames;
            transform(begin(pluginNames),
                      end(pluginNames),
                      inserter(lowercasedPluginNames, begin(lowercasedPluginNames)),
                      [&](const std::string& pluginName) {
                return boost::to_lower_copy(pluginName);
            });

            // Now do the check.
            if (lowercasedPluginNames.count(boost::to_lower_copy(gameSettings.getMasterFile())) == 0)
                throw error(LIBLO_ERROR_INVALID_ARGS, gameSettings.getMasterFile() + " must be active.");

            // Also check for Skyrim if Update.esm exists.
            if (gameSettings.getId() == LIBLO_GAME_TES5
                && Plugin::isValid("Update.esm", gameSettings)
                && lowercasedPluginNames.count("update.esm") == 0)
                throw error(LIBLO_ERROR_INVALID_ARGS, "Update.esm must be active.");
        }

        // Deactivate all existing plugins.
        for_each(begin(loadOrder), end(loadOrder), [&](Plugin& plugin) {
            plugin.deactivate();
        });

        // Now activate the plugins. If a plugin isn't in the load order,
        // append it.
        for_each(begin(pluginNames), end(pluginNames), [&](const std::string& pluginName) {
            auto it = find(begin(loadOrder), end(loadOrder), pluginName);
            if (it == end(loadOrder))
                it = addToLoadOrder(pluginName);
            it->activate(gameSettings.getPluginsFolder());
        });
    }

    void LoadOrder::activate(const std::string& pluginName) {
        if (countActivePlugins() >= maxActivePlugins)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot activate " + pluginName + " as this would mean more than " + to_string(maxActivePlugins) + " plugins are active.");

        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it == end(loadOrder)) {
            if (!Plugin::isValid(pluginName, gameSettings))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");

            it = addToLoadOrder(pluginName);
        }
        it->activate(gameSettings.getPluginsFolder());
    }

    void LoadOrder::deactivate(const std::string& pluginName) {
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE && boost::iequals(pluginName, gameSettings.getMasterFile()))
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot deactivate " + gameSettings.getMasterFile() + ".");
        else if (gameSettings.getId() == LIBLO_GAME_TES5 && boost::iequals(pluginName, "Update.esm"))
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot deactivate Update.esm.");

        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it != end(loadOrder))
            it->deactivate();
    }

    bool LoadOrder::hasFilesystemChanged() const {
        if (loadOrder.empty())
            return true;

        try {
            // Has the plugins folder been modified since the load order
            // was last read or saved?
            if (fs::last_write_time(gameSettings.getPluginsFolder()) > mtime)
                return true;

            // Has the active plugins file been changed since it was last
            // read or saved?
            if (fs::exists(gameSettings.getActivePluginsFile())
                && fs::last_write_time(gameSettings.getActivePluginsFile()) > mtime)
                return true;

            // Has the full textfile load order been changed since it was
            // last read or saved?
            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE
                && fs::exists(gameSettings.getLoadOrderFile())
                && fs::last_write_time(gameSettings.getLoadOrderFile()) > mtime)
                return true;

            // Plugins folder modification time not changed if a file in it is
            // edited, so check plugin modification dates.
            for (const auto& plugin : loadOrder) {
                if (plugin.hasFileChanged(gameSettings.getPluginsFolder()))
                    return true;
            }

            return false;
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_READ_FAIL, e.what());
        }
    }

    bool LoadOrder::isSynchronised(const GameSettings& gameSettings) {
        if (gameSettings.getLoadOrderMethod() != LIBLO_METHOD_TEXTFILE
            || !boost::filesystem::exists(gameSettings.getActivePluginsFile())
            || !boost::filesystem::exists(gameSettings.getLoadOrderFile()))
            return true;

        //First get load order according to loadorder.txt.
        LoadOrder getLoadOrderFileLO(gameSettings);
        getLoadOrderFileLO.loadFromFile(gameSettings.getLoadOrderFile());

        //Get load order from plugins.txt.
        LoadOrder PluginsFileLO(gameSettings);
        PluginsFileLO.loadFromFile(gameSettings.getActivePluginsFile());

        //Remove any plugins from getLoadOrderFileLO that are not in PluginsFileLO.
        vector<string> loadOrderFileLoadOrder = getLoadOrderFileLO.getLoadOrder();
        loadOrderFileLoadOrder.erase(remove_if(
            begin(loadOrderFileLoadOrder),
            end(loadOrderFileLoadOrder),
            [&](const string& plugin) {
            return PluginsFileLO.getPosition(plugin) == PluginsFileLO.getLoadOrder().size();
        }),
            end(loadOrderFileLoadOrder));

        //Compare the two LoadOrder objects: they should be identical (since mtimes for each have not been touched).
        return PluginsFileLO.getLoadOrder() == loadOrderFileLoadOrder;
    }

    void LoadOrder::clear() {
        loadOrder.clear();
    }

    void LoadOrder::loadFromFile(const boost::filesystem::path& file) {
        try {
            fs::ifstream in(file);
            in.exceptions(std::ios_base::badbit);

            string line;
            bool transcode = file == gameSettings.getActivePluginsFile();
            while (getline(in, line)) {
                if (line.empty() || line[0] == '#')
                    continue;

                if (transcode)
                    line = windows1252toUtf8(line);

                if (Plugin::isValid(line, gameSettings)) {
                    // Erase the entry if it already exists.
                    auto it = find(begin(loadOrder), end(loadOrder), line);
                    if (it != end(loadOrder))
                        loadOrder.erase(it);

                    // Add the entry to the appropriate place in the
                    // load order (eg. masters before plugins).
                    it = addToLoadOrder(line);
                }
            }
        }
        catch (std::ifstream::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
        }

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Add the game master file if it hasn't already been loaded.
            if (count(begin(loadOrder), end(loadOrder), gameSettings.getMasterFile()) == 0)
                addToLoadOrder(gameSettings.getMasterFile());

            // Add Update.esm if it exists and hasn't already been loaded.
            if (gameSettings.getId() == LIBLO_GAME_TES5 && Plugin::isValid("Update.esm", gameSettings)
                && count(begin(loadOrder), end(loadOrder), string("Update.esm")) == 0) {
                addToLoadOrder("Update.esm");
            }
        }
    }

    void LoadOrder::loadActivePlugins() {
        // Deactivate all existing plugins.
        for_each(begin(loadOrder), end(loadOrder), [&](Plugin& plugin) {
            plugin.deactivate();
        });

        try {
            fs::ifstream in(gameSettings.getActivePluginsFile());
            in.exceptions(std::ios_base::badbit);

            string line;
            regex morrowindRegex("GameFile[0-9]{1,3}=(.+\\.es(m|p))", regex::ECMAScript | regex::icase);
            while (getline(in, line)) {
                if (line.empty() || line[0] == '#'
                    || (gameSettings.getId() == LIBLO_GAME_TES3 && !regex_match(line, morrowindRegex)))
                    continue;

                if (gameSettings.getId() == LIBLO_GAME_TES3)
                    line = line.substr(line.find('=') + 1);

                line = windows1252toUtf8(line);

                if (Plugin::isValid(line, gameSettings)) {
                    // Add the entry to the appropriate place in the
                    // load order if it does not already exist.
                    auto it = find(begin(loadOrder), end(loadOrder), line);
                    if (it == end(loadOrder))
                        it = addToLoadOrder(line);

                    // Activate the plugin.
                    it->activate(gameSettings.getPluginsFolder());
                }
            }
        }
        catch (std::exception& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + gameSettings.getActivePluginsFile().string() + "\" could not be read. Details: " + e.what());
        }

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Activate the game master file.
            auto it = find(begin(loadOrder), end(loadOrder), gameSettings.getMasterFile());
            if (it == end(loadOrder))
                it = addToLoadOrder(gameSettings.getMasterFile());
            it->activate(gameSettings.getPluginsFolder());

            // Activate Update.esm if it exists.
            if (gameSettings.getId() == LIBLO_GAME_TES5 && Plugin::isValid("Update.esm", gameSettings)) {
                auto it = find(begin(loadOrder), end(loadOrder), string("Update.esm"));
                if (it == end(loadOrder))
                    it = addToLoadOrder("Update.esm");
                it->activate(gameSettings.getPluginsFolder());
            }
        }

        // Deactivate excess plugins.
        size_t numActivePlugins = countActivePlugins();
        for (auto it = rbegin(loadOrder); numActivePlugins > maxActivePlugins && it != rend(loadOrder); ++it) {
            if (it->isActive()) {
                it->deactivate();
                --numActivePlugins;
            }
        }
    }

    void LoadOrder::saveTimestampLoadOrder() {
        // Want to make a minimum of changes to timestamps, so use the same
        // timestamps as are currently set, but apply them to the plugins
        // in the new order.

        //First we have to read all the timestamps.
        std::set<time_t> timestamps;
        transform(begin(loadOrder),
                  end(loadOrder),
                  inserter(timestamps, begin(timestamps)),
                  [&](const Plugin& plugin) {
            return plugin.getModTime();
        });
        // It may be that two plugins currently share the same timestamp,
        // which will result in fewer timestamps in the set than there are
        // plugins, so pad the set if necessary.
        generate_n(inserter(timestamps, begin(timestamps)),
                   loadOrder.size() - timestamps.size(),
                   [&]() {
            return *rbegin(timestamps) + 60;
        });
        size_t i = 0;
        for (const auto &timestamp : timestamps) {
            loadOrder.at(i).setModTime(timestamp, gameSettings.getPluginsFolder());
            ++i;
        }
        //Now record new plugins folder mtime.
        mtime = fs::last_write_time(gameSettings.getPluginsFolder());
    }

    void LoadOrder::saveTextfileLoadOrder() {
        //Need to write both loadorder.txt and plugins.txt.
        try {
            if (!fs::exists(gameSettings.getLoadOrderFile().parent_path()))
                fs::create_directory(gameSettings.getLoadOrderFile().parent_path());

            fs::ofstream outfile(gameSettings.getLoadOrderFile(), ios_base::trunc);
            outfile.exceptions(std::ios_base::badbit);

            for (const auto &plugin : loadOrder)
                outfile << plugin.getName() << endl;
            outfile.close();

            //Now record new loadorder.txt mtime.
            //Plugins.txt doesn't need its mtime updated as only the order of its contents has changed, and it is stored in memory as an unordered set.
            mtime = fs::last_write_time(gameSettings.getLoadOrderFile());
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_WRITE_FAIL, "\"" + gameSettings.getLoadOrderFile().string() + "\" cannot be written to. Details: " + e.what());
        }
    }

    void LoadOrder::saveActivePlugins() {
        string settings, badFilename;

        if (gameSettings.getId() == LIBLO_GAME_TES3) {
            // If Morrowind, write active plugin list to Morrowind.ini, which
            // also holds a lot of other game settings. libloadorder needs to
            // read everything up to the active plugin list in the current ini
            // and stick that on before the first saved plugin name.
            if (fs::exists(gameSettings.getActivePluginsFile())) {
                std::string contents = fileToBuffer(gameSettings.getActivePluginsFile());
                size_t pos = contents.find("[Game Files]");
                if (pos != string::npos)
                    settings = contents.substr(0, pos + 12); //+12 is for the characters in "[Game Files]".
            }
        }

        try {
            if (!fs::exists(gameSettings.getActivePluginsFile().parent_path()))
                fs::create_directory(gameSettings.getActivePluginsFile().parent_path());

            fs::ofstream outfile(gameSettings.getActivePluginsFile(), ios_base::trunc);
            outfile.exceptions(std::ios_base::badbit);

            if (!settings.empty())
                outfile << settings << endl;  //Get those Morrowind settings back in.

            size_t i = 0;
            for (const auto &plugin : loadOrder) {
                if (!plugin.isActive() || (gameSettings.getId() == LIBLO_GAME_TES5 && plugin == gameSettings.getMasterFile()))
                    continue;

                if (gameSettings.getId() == LIBLO_GAME_TES3) { //Need to write "GameFileN=" before plugin name, where N is an integer from 0 up.
                    outfile << "GameFile" << i << "=";
                    ++i;
                }

                try {
                    outfile << utf8ToWindows1252(plugin.getName()) << endl;
                }
                catch (error& e) {
                    badFilename = e.what();
                }
            }
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_WRITE_FAIL, "\"" + gameSettings.getActivePluginsFile().string() + "\" could not be written. Details: " + e.what());
        }

        mtime = fs::last_write_time(gameSettings.getActivePluginsFile());

        if (!badFilename.empty())
            throw error(LIBLO_WARN_BAD_FILENAME, badFilename);
    }

    size_t LoadOrder::getMasterPartitionPoint() const {
        return distance(begin(loadOrder),
                        partition_point(begin(loadOrder),
                        end(loadOrder),
                        [&](const Plugin& plugin) {
            return plugin.isMasterFile();
        }));
    }

    size_t LoadOrder::countActivePlugins() const {
        return count_if(begin(loadOrder), end(loadOrder), [&](const Plugin& plugin) {
            return plugin.isActive();
        });
    }

    Plugin LoadOrder::getPluginObject(const std::string& pluginName) const {
        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it != end(loadOrder))
            return *it;
        else {
            if (!Plugin::isValid(pluginName, gameSettings))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");
            return Plugin(pluginName, gameSettings);
        }
    }

    std::vector<Plugin>::iterator LoadOrder::addToLoadOrder(const std::string& pluginName) {
        std::vector<Plugin>::iterator it;
        Plugin plugin(pluginName, gameSettings);
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE && boost::iequals(pluginName, gameSettings.getMasterFile()))
            it = loadOrder.insert(begin(loadOrder), plugin);
        else if (plugin.isMasterFile())
            it = loadOrder.insert(next(begin(loadOrder), getMasterPartitionPoint()), plugin);
        else {
            loadOrder.push_back(plugin);
            it = prev(loadOrder.end());
        }
        return it;
    }

    void LoadOrder::partitionMasters() {
        stable_partition(begin(loadOrder),
                         end(loadOrder),
                         [&](const Plugin& plugin) {
            return plugin.isMasterFile();
        });
    }
}

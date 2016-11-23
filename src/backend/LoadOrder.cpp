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

#include "LoadOrder.h"
#include "GameSettings.h"
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
    LoadOrder::LoadOrder(const GameSettings& gameSettings) :
        gameSettings(gameSettings) {}

    void LoadOrder::load() {
        lock_guard<recursive_mutex> guard(mutex);

        // Only reload plugins that have changed.
        auto it = begin(loadOrder);
        while (it != end(loadOrder)) {
            try {
                if (it->hasFileChanged(gameSettings.getPluginsFolder()))
                    *it = Plugin(it->libespm::Plugin::getName(), gameSettings);
                ++it;
            }
            catch (std::exception&) {
                it = loadOrder.erase(it);
            }
        }

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            if (pathCache.isModified(gameSettings.getLoadOrderFile()))
                loadFromFile(gameSettings.getLoadOrderFile());
            else if (pathCache.isModified(gameSettings.getActivePluginsFile())) {
                loadFromFile(gameSettings.getActivePluginsFile());
                loadActivePlugins();
            }
        }
        else if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK) {
            if (pathCache.isModified(gameSettings.getActivePluginsFile())) {
                loadFromFile(gameSettings.getActivePluginsFile());
                loadActivePlugins();
            }
        }

        if (fs::is_directory(gameSettings.getPluginsFolder()) && pathCache.isModified(gameSettings.getPluginsFolder())) {
            addMissingPlugins();

            if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                stable_sort(begin(loadOrder),
                            end(loadOrder),
                            [&](const Plugin& lhs, const Plugin& rhs) {
                    if (lhs.isMasterFile() == rhs.isMasterFile())
                        return difftime(lhs.getModTime(), rhs.getModTime()) < 0;
                    else
                        return lhs.isMasterFile();
                });
            }
        }

        if (pathCache.isModified(gameSettings.getActivePluginsFile()))
            loadActivePlugins();
    }

    void LoadOrder::save() {
        lock_guard<recursive_mutex> guard(mutex);

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TIMESTAMP)
            saveTimestampLoadOrder();
        else if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE)
            saveTextfileLoadOrder();

        saveActivePlugins();
    }

    std::vector<std::string> LoadOrder::getLoadOrder() const {
        lock_guard<recursive_mutex> guard(mutex);

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
        lock_guard<recursive_mutex> guard(mutex);

        return distance(begin(loadOrder),
                        find(begin(loadOrder),
                             end(loadOrder),
                             pluginName));
    }

    std::string LoadOrder::getPluginAtPosition(size_t index) const {
        lock_guard<recursive_mutex> guard(mutex);

        return loadOrder.at(index).getName();
    }

    void LoadOrder::setLoadOrder(const std::vector<std::string>& pluginNames) {
        lock_guard<recursive_mutex> guard(mutex);

        // For textfile- and asterisk-based load order games, check that the game's master file loads first.
        if ((gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE || gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK)
            && (pluginNames.empty() || !boost::iequals(pluginNames[0], gameSettings.getMasterFile())))
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

        // Now append any plugins that are installed but aren't present in the
        // new load order.
        addMissingPlugins();

        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE || gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK) {
            // Make sure that game master is active.
            loadOrder.front().activate(gameSettings.getPluginsFolder());
        }
    }

    void LoadOrder::setPosition(const std::string& pluginName, size_t loadOrderIndex) {
        lock_guard<recursive_mutex> guard(mutex);

        // For textfile- and asterisk-based load order games, check that this doesn't move the game master file from the beginning of the load order.
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE || gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK) {
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

    std::vector<std::string> LoadOrder::getActivePlugins() const {
        lock_guard<recursive_mutex> guard(mutex);

        vector<string> activePlugins;
        for_each(begin(loadOrder),
                 end(loadOrder),
                 [&](const Plugin& plugin) {
            if (plugin.isActive())
                activePlugins.push_back(plugin.getName());
        });

        return activePlugins;
    }

    bool LoadOrder::isActive(const std::string& pluginName) const {
        lock_guard<recursive_mutex> guard(mutex);

        return find_if(begin(loadOrder), end(loadOrder), [&](const Plugin& plugin) {
            return plugin == pluginName && plugin.isActive();
        }) != end(loadOrder);
    }

    void LoadOrder::setActivePlugins(const std::vector<std::string>& pluginNames) {
        lock_guard<recursive_mutex> guard(mutex);

        if (pluginNames.size() > maxActivePlugins)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot activate more than " + to_string(maxActivePlugins) + " plugins.");

        // Check all plugins are valid.
        for_each(begin(pluginNames), end(pluginNames), [&](const std::string& pluginName) {
            if (!this->contains(pluginName)
                && !Plugin::isValid(pluginName, gameSettings))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");
        });

        // Check that all installed implicitly active plugins are explicitly
        // active in the given active plugins list.
        for (const auto& pluginName : gameSettings.getImplicitlyActivePlugins()) {
            if (!Plugin::isValid(pluginName, gameSettings))
                continue;

            auto it = find_if(begin(pluginNames), end(pluginNames), [&](const string& name) {
                return boost::iequals(pluginName, name);
            });

            if (it == end(pluginNames))
                throw error(LIBLO_ERROR_INVALID_ARGS, pluginName + " must be active.");
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
        lock_guard<recursive_mutex> guard(mutex);

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
        lock_guard<recursive_mutex> guard(mutex);

        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it != end(loadOrder)) {
            // Check that the plugin is not implicitly active.
            if (gameSettings.isImplicitlyActive(pluginName))
                throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot deactivate " + pluginName + ".");

            it->deactivate();
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

        return PluginsFileLO.getLoadOrder() == loadOrderFileLoadOrder;
    }

    void LoadOrder::clear() {
        lock_guard<recursive_mutex> guard(mutex);

        loadOrder.clear();
        pathCache.clear();
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

                if (transcode) {
                    line = windows1252toUtf8(line);

                    if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK && line[0] == '*')
                        line = line.substr(1);
                }

                if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK
                    && gameSettings.isImplicitlyActive(line)) {
                  continue;
                }

                // If the entry already exists, move it to the last
                // valid position for it. Otherwise, add it at that
                // position.
                auto it = find(begin(loadOrder), end(loadOrder), line);
                if (it != end(loadOrder)) {
                    // Check if new pos will be different from old pos.
                    size_t newPos = getAppendPosition(*it);

                    size_t currentPos = distance(begin(loadOrder), it);
                    if (newPos != currentPos) {
                        if (newPos > currentPos)
                            --newPos;

                        Plugin plugin = *it;
                        loadOrder.erase(it);
                        loadOrder.insert(next(begin(loadOrder), newPos), plugin);
                    }
                }
                else if (Plugin::isValid(line, gameSettings)) {
                 // Add the entry to the appropriate place in the
                 // load order (eg. masters before plugins).
                    addToLoadOrder(line);
                }
            }

            pathCache.updateCachedState(file);
        }
        catch (std::ifstream::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
        }

        addImplicitlyActivePlugins();
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
                    || (gameSettings.getId() == LIBLO_GAME_TES3 && !regex_match(line, morrowindRegex))
                    || (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK && line[0] != '*'))
                    continue;

                if (gameSettings.getId() == LIBLO_GAME_TES3)
                    line = line.substr(line.find('=') + 1);
                else if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK)
                    line = line.substr(1);

                line = windows1252toUtf8(line);

                // Check if the line exists as a plugin. If it doesn't,
                // check if it is for a valid plugin, and add it if so.
                auto it = find(begin(loadOrder), end(loadOrder), line);
                if (it == end(loadOrder) && Plugin::isValid(line, gameSettings))
                    it = addToLoadOrder(line);

                // Activate the plugin.
                if (it != end(loadOrder))
                    it->activate(gameSettings.getPluginsFolder());
            }

            pathCache.updateCachedState(gameSettings.getActivePluginsFile());
        }
        catch (std::exception& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + gameSettings.getActivePluginsFile().string() + "\" could not be read. Details: " + e.what());
        }

        addImplicitlyActivePlugins();
        deactivateExcessPlugins();
    }

    void LoadOrder::addMissingPlugins() {
        const vector<string> implicitlyActivePlugins = gameSettings.getImplicitlyActivePlugins();

        // Add any missing plugins, apart from implicitly active plugins, which
        // if missing must be loaded last (of the master files).
        for (fs::directory_iterator itr(gameSettings.getPluginsFolder()); itr != fs::directory_iterator(); ++itr) {
            if (fs::is_regular_file(itr->status())) {
                const std::string filename(itr->path().filename().string());

                auto iequals = [&](const string& pluginName) {
                    return boost::iequals(pluginName, filename);
                };

                if (!this->contains(filename)
                    && find_if(begin(implicitlyActivePlugins),
                               end(implicitlyActivePlugins),
                               iequals) == end(implicitlyActivePlugins)
                    && Plugin::isValid(filename, gameSettings))
                    addToLoadOrder(filename);
            }
        }

        pathCache.updateCachedState(gameSettings.getPluginsFolder());

        addImplicitlyActivePlugins();
    }

    void LoadOrder::addImplicitlyActivePlugins() {
      for (const auto& pluginName : gameSettings.getImplicitlyActivePlugins()) {
        if (isActive(pluginName) || !Plugin::isValid(pluginName, gameSettings))
          continue;

        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it == end(loadOrder))
          it = addToLoadOrder(pluginName);

        it->activate(gameSettings.getPluginsFolder());
      }
    }

    void LoadOrder::deactivateExcessPlugins() {
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

            pathCache.updateCachedState(gameSettings.getLoadOrderFile());
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
                if ((gameSettings.getLoadOrderMethod() != LIBLO_METHOD_ASTERISK && !plugin.isActive())
                    || (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE
                        && plugin == gameSettings.getMasterFile())
                    || (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK
                        && gameSettings.isImplicitlyActive(plugin.getName())))
                    continue;

                if (gameSettings.getId() == LIBLO_GAME_TES3) { //Need to write "GameFileN=" before plugin name, where N is an integer from 0 up.
                    outfile << "GameFile" << i << "=";
                    ++i;
                }

                try {
                    if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK && plugin.isActive())
                        outfile << '*';

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

        pathCache.updateCachedState(gameSettings.getActivePluginsFile());

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

    size_t LoadOrder::getAppendPosition(const Plugin& plugin) const {
        if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_TEXTFILE
            && boost::iequals(plugin.getName(), gameSettings.getMasterFile())) {
          return 0;
        }
        else if (gameSettings.getLoadOrderMethod() == LIBLO_METHOD_ASTERISK) {
          size_t installedPluginCount = 0;
          for (const auto& implicitlyActivePlugin : gameSettings.getImplicitlyActivePlugins()) {
            if (boost::iequals(plugin.getName(), implicitlyActivePlugin))
              return installedPluginCount;
            
            if (contains(implicitlyActivePlugin) || Plugin::isValid(implicitlyActivePlugin, gameSettings))
              ++installedPluginCount;
          }
        }
        
        if (plugin.isMasterFile())
            return getMasterPartitionPoint();
        else
            return loadOrder.size();
    }

    bool LoadOrder::contains(const std::string& pluginName) const {
        return find(begin(loadOrder), end(loadOrder), pluginName) != end(loadOrder);
    }

    std::vector<Plugin>::iterator LoadOrder::addToLoadOrder(const std::string& pluginName) {
        std::vector<Plugin>::iterator it;
        Plugin plugin(pluginName, gameSettings);
        size_t pos = getAppendPosition(plugin);
        if (pos >= loadOrder.size()) {
            return loadOrder.insert(end(loadOrder), plugin);
        }
        else
            return loadOrder.insert(next(begin(loadOrder), pos), plugin);
    }
}

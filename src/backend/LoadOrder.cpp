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
#include "libloadorder/constants.h"
#include "error.h"
#include "game.h"
#include "helpers.h"

#include <regex>
#include <set>
#include <unordered_map>

#include <boost/algorithm/string.hpp>

using namespace std;
namespace fs = boost::filesystem;

namespace liblo {
    /////////////////////////
    // LoadOrder Members
    /////////////////////////

    struct PluginSortInfo {
        PluginSortInfo() : isMasterFile(false), modTime(0) {}
        bool isMasterFile;
        time_t modTime;
    };

    struct pluginComparator {
        const _lo_game_handle_int& parentGame;
        std::unordered_map <std::string, PluginSortInfo> pluginCache;

        pluginComparator(const _lo_game_handle_int& game) : parentGame(game) {}

        bool    operator () (const Plugin& plugin1, const Plugin& plugin2) {
            //Return true if plugin1 goes before plugin2, false otherwise.
            //Master files should go before other files.
            //Earlier stamped plugins should go before later stamped plugins.

            auto p1It = pluginCache.find(plugin1.Name());
            auto p2It = pluginCache.find(plugin2.Name());

            // If either of the plugins haven't been cached, cache them now,
            // but defer reading timestamps, since it's not always necessary.
            if (p1It == pluginCache.end()) {
                PluginSortInfo psi;
                psi.isMasterFile = plugin1.IsMasterFile(parentGame);
                p1It = pluginCache.insert(std::pair<std::string, PluginSortInfo>(plugin1.Name(), psi)).first;
            }

            if (p2It == pluginCache.end()) {
                PluginSortInfo psi;
                psi.isMasterFile = plugin2.IsMasterFile(parentGame);
                p2It = pluginCache.insert(std::pair<std::string, PluginSortInfo>(plugin2.Name(), psi)).first;
            }

            if (p1It->second.isMasterFile && !p2It->second.isMasterFile)
                return true;
            else if (!p1It->second.isMasterFile && p2It->second.isMasterFile)
                return false;
            else {
                // Need to compare timestamps to decide. If either cached
                // timestamp is zero, read and cache the actual timestamp.
                if (p1It->second.modTime == 0) {
                    p1It->second.modTime = plugin1.GetModTime(parentGame);
                }
                if (p2It->second.modTime == 0) {
                    p2It->second.modTime = plugin2.GetModTime(parentGame);
                }

                return (difftime(p1It->second.modTime, p2It->second.modTime) < 0);
            }
        }
    };

    void LoadOrder::load(const _lo_game_handle_int& gameHandle) {
        loadOrder.clear();
        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            if (fs::exists(gameHandle.LoadOrderFile()))
                loadFromFile(gameHandle.LoadOrderFile(), gameHandle);
            else if (fs::exists(gameHandle.ActivePluginsFile()))
                loadFromFile(gameHandle.ActivePluginsFile(), gameHandle);
        }
        if (fs::is_directory(gameHandle.PluginsFolder())) {
            // Now scan through Data folder. Add any plugins that aren't
            // already in load order.
            auto firstNonMaster = getMasterPartitionPoint(gameHandle);
            for (fs::directory_iterator itr(gameHandle.PluginsFolder()); itr != fs::directory_iterator(); ++itr) {
                if (fs::is_regular_file(itr->status())) {
                    const std::string filename(itr->path().filename().string());
                    if (Plugin(filename).IsValid(gameHandle) && count(begin(loadOrder), end(loadOrder), filename) == 0)
                        loadOrder.push_back(filename);
                }
            }
            if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                stable_sort(begin(loadOrder),
                            end(loadOrder),
                            [&](const Plugin& lhs, const Plugin& rhs) {
                    return difftime(lhs.GetModTime(gameHandle),
                                    rhs.GetModTime(gameHandle)) < 0;
                });
            }
            partitionMasters(gameHandle);
        }
        if (fs::exists(gameHandle.ActivePluginsFile()))
            loadActivePlugins(gameHandle);
    }

    void LoadOrder::Save(_lo_game_handle_int& parentGame) {
        if (parentGame.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
            //Update timestamps.
            //Want to make a minimum of changes to timestamps, so use the same timestamps as are currently set, but apply them to the plugins in the new order.
            //First we have to read all the timestamps.
            std::set<time_t> timestamps;
            for (const auto &plugin : loadOrder) {
                timestamps.insert(plugin.GetModTime(parentGame));
            }
            // It may be that two plugins currently share the same timestamp,
            // which will result in fewer timestamps in the set than there are
            // plugins, so pad the set if necessary.
            while (timestamps.size() < loadOrder.size()) {
                timestamps.insert(*timestamps.crbegin() + 60);
            }
            size_t i = 0;
            for (const auto &timestamp : timestamps) {
                loadOrder.at(i).SetModTime(parentGame, timestamp);
                ++i;
            }
            //Now record new plugins folder mtime.
            mtime = fs::last_write_time(parentGame.PluginsFolder());
        }
        else {
            //Need to write both loadorder.txt and plugins.txt.
            try {
                if (!fs::exists(parentGame.LoadOrderFile().parent_path()))
                    fs::create_directory(parentGame.LoadOrderFile().parent_path());
                fs::ofstream outfile(parentGame.LoadOrderFile(), ios_base::trunc);
                outfile.exceptions(std::ios_base::badbit);

                for (const auto &plugin : loadOrder)
                    outfile << plugin.Name() << endl;
                outfile.close();

                //Now record new loadorder.txt mtime.
                //Plugins.txt doesn't need its mtime updated as only the order of its contents has changed, and it is stored in memory as an unordered set.
                mtime = fs::last_write_time(parentGame.LoadOrderFile());
            }
            catch (std::ios_base::failure& e) {
                throw error(LIBLO_ERROR_FILE_WRITE_FAIL, "\"" + parentGame.LoadOrderFile().string() + "\" cannot be written to. Details: " + e.what());
            }

            //Now write plugins.txt. Update cache if necessary.
            if (parentGame.activePlugins.HasChanged(parentGame))
                parentGame.activePlugins.Load(parentGame);
            parentGame.activePlugins.Save(parentGame);
        }
    }

    std::vector<std::string> LoadOrder::getLoadOrder() const {
        std::vector<std::string> pluginNames;
        transform(begin(loadOrder),
                  end(loadOrder),
                  back_inserter(pluginNames),
                  [](const Plugin& plugin) {
            return plugin.Name();
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
        return loadOrder.at(index).Name();
    }

    void LoadOrder::setLoadOrder(const std::vector<std::string>& pluginNames, const _lo_game_handle_int& gameHandle) {
        // For textfile-based load order games, check that the game's master file loads first.
        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE && (pluginNames.empty() || !boost::iequals(pluginNames[0], gameHandle.MasterFile())))
            throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + gameHandle.MasterFile() + "\" must load first.");

        // Create vector of Plugin objects, reusing existing objects
        // where possible. Also check for duplicate entries, that new
        // plugins are valid,
        vector<Plugin> plugins;
        unordered_set<string> hashset;
        for_each(begin(pluginNames), end(pluginNames), [&](const std::string& pluginName) {
            if (hashset.find(boost::to_lower_copy(pluginName)) != hashset.end())
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is a duplicate entry.");

            hashset.insert(boost::to_lower_copy(pluginName));
            plugins.push_back(getPluginObject(pluginName, gameHandle));
        });

        // Check that all masters load before non-masters.
        if (!is_partitioned(begin(plugins),
            end(plugins),
            [&](const Plugin& plugin) {
            return plugin.IsMasterFile(gameHandle);
        })) {
            throw error(LIBLO_ERROR_INVALID_ARGS, "Master plugins must load before all non-master plugins.");
        }

        // Swap load order for the new one.
        loadOrder.swap(plugins);

        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Make sure that game master is active.
            loadOrder.front().activate();
        }
    }

    void LoadOrder::setPosition(const std::string& pluginName, size_t loadOrderIndex, const _lo_game_handle_int& gameHandle) {
        // For textfile-based load order games, check that this doesn't move the game master file from the beginning of the load order.
        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            if (loadOrderIndex == 0 && !boost::iequals(pluginName, gameHandle.MasterFile()))
                throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot set \"" + pluginName + "\" to load first: \"" + gameHandle.MasterFile() + "\" most load first.");
            else if (loadOrderIndex != 0 && !loadOrder.empty() && boost::iequals(pluginName, gameHandle.MasterFile()))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" must load first.");
        }

        // If the plugin is already in the load order, use its existing
        // object.
        Plugin plugin = getPluginObject(pluginName, gameHandle);

        // Check that a master isn't being moved before a non-master or the inverse.
        size_t masterPartitionPoint(getMasterPartitionPoint(gameHandle));
        if (!plugin.IsMasterFile(gameHandle) && loadOrderIndex < masterPartitionPoint)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot move a non-master plugin before master files.");
        else if (plugin.IsMasterFile(gameHandle)
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
                activePlugins.insert(plugin.Name());
        });
        return activePlugins;
    }

    bool LoadOrder::isActive(const std::string& pluginName) const {
        return find_if(begin(loadOrder), end(loadOrder), [&](const Plugin& plugin) {
            return plugin == pluginName && plugin.isActive();
        }) != end(loadOrder);
    }

    void LoadOrder::setActivePlugins(const std::unordered_set<std::string>& pluginNames, const _lo_game_handle_int& gameHandle) {
        if (pluginNames.size() > maxActivePlugins)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot activate more than " + to_string(maxActivePlugins) + " plugins.");

        // Check all plugins are valid.
        for_each(begin(pluginNames), end(pluginNames), [&](const std::string& pluginName) {
            if (count(begin(loadOrder), end(loadOrder), pluginName) == 0
                && !Plugin(pluginName).IsValid(gameHandle))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");
        });

        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
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
            if (lowercasedPluginNames.count(boost::to_lower_copy(gameHandle.MasterFile())) == 0)
                throw error(LIBLO_ERROR_INVALID_ARGS, gameHandle.MasterFile() + " must be active.");

            // Also check for Skyrim if Update.esm exists.
            if (gameHandle.Id() == LIBLO_GAME_TES5
                && Plugin("Update.esm").IsValid(gameHandle)
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
                it = addToLoadOrder(pluginName, gameHandle);
            it->activate();
        });
    }

    void LoadOrder::activate(const std::string& pluginName, const _lo_game_handle_int& gameHandle) {
        if (countActivePlugins() >= maxActivePlugins)
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot activate " + pluginName + " as this would mean more than " + to_string(maxActivePlugins) + " plugins are active.");

        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it == end(loadOrder)) {
            Plugin plugin(pluginName);
            if (!plugin.IsValid(gameHandle))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");

            it = addToLoadOrder(pluginName, gameHandle);
        }
        it->activate();
    }

    void LoadOrder::deactivate(const std::string& pluginName, const _lo_game_handle_int& gameHandle) {
        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE && boost::iequals(pluginName, gameHandle.MasterFile()))
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot deactivate " + gameHandle.MasterFile() + ".");
        else if (gameHandle.Id() == LIBLO_GAME_TES5 && boost::iequals(pluginName, "Update.esm"))
            throw error(LIBLO_ERROR_INVALID_ARGS, "Cannot deactivate Update.esm.");

        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it != end(loadOrder))
            it->deactivate();
    }

    bool LoadOrder::HasChanged(const _lo_game_handle_int& parentGame) const {
        if (loadOrder.empty())
            return true;

        try {
            if (parentGame.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE && fs::exists(parentGame.LoadOrderFile())) {
                //Load order is stored in parentGame.LoadOrderFile(), but load order must also be reloaded if parentGame.PluginsFolder() has been altered.
                time_t t1 = fs::last_write_time(parentGame.LoadOrderFile());
                time_t t2 = fs::last_write_time(parentGame.PluginsFolder());
                if (t1 > t2) //Return later time.
                    return (t1 > mtime);
                else
                    return (t2 > mtime);
            }
            else
                //Checking parent folder modification time doesn't work consistently, and to check if the load order has changed would probably take as long as just assuming it's changed.
                return true;
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_READ_FAIL, e.what());
        }
    }

    bool LoadOrder::isSynchronised(const _lo_game_handle_int& gameHandle) {
        if (gameHandle.LoadOrderMethod() != LIBLO_METHOD_TEXTFILE
            || !boost::filesystem::exists(gameHandle.ActivePluginsFile())
            || !boost::filesystem::exists(gameHandle.LoadOrderFile()))
            return true;

        //First get load order according to loadorder.txt.
        LoadOrder LoadOrderFileLO;
        LoadOrderFileLO.loadFromFile(gameHandle.LoadOrderFile(), gameHandle);

        //Get load order from plugins.txt.
        LoadOrder PluginsFileLO;
        PluginsFileLO.loadFromFile(gameHandle.ActivePluginsFile(), gameHandle);

        //Remove any plugins from LoadOrderFileLO that are not in PluginsFileLO.
        vector<string> loadOrderFileLoadOrder = LoadOrderFileLO.getLoadOrder();
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

    void LoadOrder::loadFromFile(const boost::filesystem::path& file, const _lo_game_handle_int& gameHandle) {
        try {
            fs::ifstream in(file);
            in.exceptions(std::ios_base::badbit);

            string line;
            bool transcode = file == gameHandle.ActivePluginsFile();
            while (getline(in, line)) {
                if (line.empty() || line[0] == '#')
                    continue;

                if (transcode)
                    line = ToUTF8(line);

                Plugin plugin(line);
                if (plugin.IsValid(gameHandle)) {
                    // Erase the entry if it already exists.
                    auto it = find(begin(loadOrder), end(loadOrder), line);
                    if (it != end(loadOrder))
                        loadOrder.erase(it);

                    // Add the entry to the appropriate place in the
                    // load order (eg. masters before plugins).
                    it = addToLoadOrder(line, gameHandle);
                }
            }
        }
        catch (std::ifstream::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
        }

        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Add the game master file if it hasn't already been loaded.
            if (count(begin(loadOrder), end(loadOrder), gameHandle.MasterFile()) == 0)
                addToLoadOrder(gameHandle.MasterFile(), gameHandle);

            // Add Update.esm if it exists and hasn't already been loaded.
            if (gameHandle.Id() == LIBLO_GAME_TES5 && Plugin("Update.esm").IsValid(gameHandle)
                && count(begin(loadOrder), end(loadOrder), string("Update.esm")) == 0) {
                addToLoadOrder("Update.esm", gameHandle);
            }
        }
    }

    void LoadOrder::loadActivePlugins(const _lo_game_handle_int& gameHandle) {
        // Deactivate all existing plugins.
        for_each(begin(loadOrder), end(loadOrder), [&](Plugin& plugin) {
            plugin.deactivate();
        });

        try {
            fs::ifstream in(gameHandle.ActivePluginsFile());
            in.exceptions(std::ios_base::badbit);

            string line;
            regex morrowindRegex("GameFile[0-9]{1,3}=(.+\\.es(m|p))", regex::ECMAScript | regex::icase);
            while (getline(in, line)) {
                if (line.empty() || line[0] == '#'
                    || (gameHandle.Id() == LIBLO_GAME_TES3 && !regex_match(line, morrowindRegex)))
                    continue;

                if (gameHandle.Id() == LIBLO_GAME_TES3)
                    line = line.substr(line.find('=') + 1);

                line = ToUTF8(line);

                if (Plugin(line).IsValid(gameHandle)) {
                    // Add the entry to the appropriate place in the
                    // load order if it does not already exist.
                    auto it = find(begin(loadOrder), end(loadOrder), line);
                    if (it == end(loadOrder))
                        it = addToLoadOrder(line, gameHandle);

                    // Activate the plugin.
                    it->activate();
                }
            }
        }
        catch (std::exception& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + gameHandle.ActivePluginsFile().string() + "\" could not be read. Details: " + e.what());
        }

        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            // Activate the game master file.
            auto it = find(begin(loadOrder), end(loadOrder), gameHandle.MasterFile());
            if (it == end(loadOrder))
                it = addToLoadOrder(gameHandle.MasterFile(), gameHandle);
            it->activate();

            // Activate Update.esm if it exists.
            if (gameHandle.Id() == LIBLO_GAME_TES5 && Plugin("Update.esm").IsValid(gameHandle)) {
                auto it = find(begin(loadOrder), end(loadOrder), string("Update.esm"));
                if (it == end(loadOrder))
                    it = addToLoadOrder("Update.esm", gameHandle);
                it->activate();
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

    size_t LoadOrder::getMasterPartitionPoint(const _lo_game_handle_int& gameHandle) const {
        return distance(begin(loadOrder),
                        partition_point(begin(loadOrder),
                        end(loadOrder),
                        [&](const Plugin& plugin) {
            return plugin.IsMasterFile(gameHandle);
        }));
    }

    size_t LoadOrder::countActivePlugins() const {
        return count_if(begin(loadOrder), end(loadOrder), [&](const Plugin& plugin) {
            return plugin.isActive();
        });
    }

    Plugin LoadOrder::getPluginObject(const std::string& pluginName, const _lo_game_handle_int& gameHandle) const {
        auto it = find(begin(loadOrder), end(loadOrder), pluginName);
        if (it != end(loadOrder))
            return *it;
        else {
            Plugin plugin(pluginName);
            if (!plugin.IsValid(gameHandle))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");
            return plugin;
        }
    }

    std::vector<Plugin>::iterator LoadOrder::addToLoadOrder(const std::string& pluginName, const _lo_game_handle_int& gameHandle) {
        std::vector<Plugin>::iterator it;
        Plugin plugin(pluginName);
        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE && boost::iequals(pluginName, gameHandle.MasterFile()))
            it = loadOrder.insert(begin(loadOrder), plugin);
        else if (plugin.IsMasterFile(gameHandle))
            it = loadOrder.insert(next(begin(loadOrder), getMasterPartitionPoint(gameHandle)), plugin);
        else {
            loadOrder.push_back(plugin);
            it = prev(loadOrder.end());
        }
        return it;
    }

    void LoadOrder::partitionMasters(const _lo_game_handle_int& gameHandle) {
        stable_partition(begin(loadOrder),
                         end(loadOrder),
                         [&](const Plugin& plugin) {
            return plugin.IsMasterFile(gameHandle);
        });
    }

    ///////////////////////////
    // ActivePlugins Members
    ///////////////////////////

    void ActivePlugins::Load(const _lo_game_handle_int& parentGame) {
        clear();

        if (fs::exists(parentGame.ActivePluginsFile())) {
            string line;
            try {
                fs::ifstream in(parentGame.ActivePluginsFile());
                in.exceptions(std::ios_base::badbit);

                if (parentGame.Id() == LIBLO_GAME_TES3) {  //Morrowind's active file list is stored in Morrowind.ini, and that has a different format from plugins.txt.
                    regex reg = regex("GameFile[0-9]{1,3}=.+\\.es(m|p)", regex::ECMAScript | regex::icase);
                    while (getline(in, line)) {
                        if (line.empty() || !regex_match(line, reg))
                            continue;

                        //Now cut off everything up to and including the = sign.
                        Plugin plugin(ToUTF8(line.substr(line.find('=') + 1)));
                        if (plugin.IsValid(parentGame))
                            insert(plugin);
                    }
                }
                else {
                    while (getline(in, line)) {
                        // Check if it's a valid plugin line. The stream doesn't filter out '\r' line endings, hence the check.
                        if (line.empty() || line[0] == '#' || line[0] == '\r')
                            continue;

                        Plugin plugin(ToUTF8(line));
                        if (plugin.IsValid(parentGame))
                            insert(plugin);
                    }
                }
                in.close();
            }
            catch (std::ios_base::failure& e) {
                throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + parentGame.ActivePluginsFile().string() + "\" could not be read. Details: " + e.what());
            }
        }

        //Add skyrim.esm, update.esm if missing.
        if (parentGame.Id() == LIBLO_GAME_TES5) {
            if (find(Plugin(parentGame.MasterFile())) == end())
                insert(Plugin(parentGame.MasterFile()));
            if (Plugin("Update.esm").IsValid(parentGame) && find(Plugin("Update.esm")) == end())
                insert(Plugin("Update.esm"));
        }
    }

    void ActivePlugins::Save(const _lo_game_handle_int& parentGame) {
        string settings, badFilename;

        if (parentGame.Id() == LIBLO_GAME_TES3) {  //Must be the plugins file, since loadorder.txt isn't used for MW.
            string contents;
            //If Morrowind, write active plugin list to Morrowind.ini, which also holds a lot of other game settings.
            //libloadorder needs to read everything up to the active plugin list in the current ini and stick that on before the first saved plugin name.
            if (fs::exists(parentGame.ActivePluginsFile())) {
                fileToBuffer(parentGame.ActivePluginsFile(), contents);
                size_t pos = contents.find("[Game Files]");
                if (pos != string::npos)
                    settings = contents.substr(0, pos + 12); //+12 is for the characters in "[Game Files]".
            }
        }

        try {
            if (!fs::exists(parentGame.ActivePluginsFile().parent_path()))
                fs::create_directory(parentGame.ActivePluginsFile().parent_path());
            fs::ofstream outfile(parentGame.ActivePluginsFile(), ios_base::trunc);
            outfile.exceptions(std::ios_base::badbit);

            if (!settings.empty())
                outfile << settings << endl;  //Get those Morrowind settings back in.

            if (parentGame.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
                //Can write the active plugins in any order.
                size_t i = 0;
                for (const auto &plugin : *this) {
                    if (parentGame.Id() == LIBLO_GAME_TES3) //Need to write "GameFileN=" before plugin name, where N is an integer from 0 up.
                        outfile << "GameFile" << i << "=";

                    try {
                        outfile << FromUTF8(plugin.Name()) << endl;
                    }
                    catch (error& e) {
                        badFilename = e.what();
                    }
                    i++;
                }
            }
            else {
                //Need to write the active plugins in load order.
                for (const auto &plugin : parentGame.loadOrder.getLoadOrder()) {
                    if (find(plugin) == end() || (parentGame.Id() == LIBLO_GAME_TES5 && plugin == parentGame.MasterFile()))
                        continue;

                    try {
                        outfile << FromUTF8(plugin) << endl;
                    }
                    catch (error& e) {
                        badFilename = e.what();
                    }
                }
            }
            outfile.close();
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_WRITE_FAIL, "\"" + parentGame.ActivePluginsFile().string() + "\" could not be written. Details: " + e.what());
        }

        if (!badFilename.empty())
            throw error(LIBLO_WARN_BAD_FILENAME, badFilename);
    }

    void ActivePlugins::CheckValidity(const _lo_game_handle_int& parentGame) const {
        for (const auto& plugin : *this) {
            if (!plugin.Exists(parentGame))
                throw error(LIBLO_WARN_INVALID_LIST, "\"" + plugin.Name() + "\" is not installed.");
            /*vector<Plugin> masters = plugin.GetMasters(parentGame);
            //Disabled because it causes false positives for Filter patches. This means libloadorder doesn't check to ensure all a plugin's masters are active, but I don't think it should get mixed up with Bash Tag detection.
            for (const auto& master: masters) {
            if (this->find(master) == this->end())
            throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + plugin.Name() + "\" has a master (\"" + master.Name() + "\") which isn't active.");
            }*/
        }

        if (size() > 255)
            throw error(LIBLO_WARN_INVALID_LIST, "More than 255 plugins are active.");
        else if (parentGame.Id() == LIBLO_GAME_TES5) {
            if (find(Plugin(parentGame.MasterFile())) == end())
                throw error(LIBLO_WARN_INVALID_LIST, parentGame.MasterFile() + " isn't active.");
            else if (Plugin("Update.esm").IsValid(parentGame) && find(Plugin("Update.esm")) == end())
                throw error(LIBLO_WARN_INVALID_LIST, "Update.esm is installed but isn't active.");
        }
    }

    bool ActivePlugins::HasChanged(const _lo_game_handle_int& parentGame) const {
        if (empty())
            return true;

        try {
            if (fs::exists(parentGame.ActivePluginsFile()))
                return (fs::last_write_time(parentGame.ActivePluginsFile()) > mtime);
            else
                return false;
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_READ_FAIL, e.what());
        }
    }
}

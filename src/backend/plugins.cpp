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

#include "libloadorder/constants.h"
#include "error.h"
#include "plugins.h"
#include "game.h"
#include "helpers.h"
#include <boost/filesystem.hpp>
#include <boost/locale.hpp>
#include <boost/algorithm/string.hpp>
#include <regex>
#include <set>
#include <unordered_map>

using namespace std;
namespace fs = boost::filesystem;

namespace liblo {
    //////////////////////
    // Plugin Members
    //////////////////////

    Plugin::Plugin() : name("") {}

    Plugin::Plugin(const string& filename) : name(filename) {
        if (!name.empty() && name[name.length() - 1] == '\r')
            name = name.substr(0, name.length() - 1);
        if (boost::iends_with(name, ".ghost"))
            name = fs::path(name).stem().string();
    };

    string Plugin::Name() const {
        return name;
    }

    bool Plugin::IsValid(const _lo_game_handle_int& parentGame) const {
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

    bool Plugin::IsMasterFile(const _lo_game_handle_int& parentGame) const {
        try {
            libespm::Plugin plugin = ReadHeader(parentGame);

            return plugin.isMasterFile();
        }
        catch (std::exception&) {
            return false;
        }
    }

    bool Plugin::IsGhosted(const _lo_game_handle_int& parentGame) const {
        return (fs::exists(parentGame.PluginsFolder() / fs::path(name + ".ghost")));
    }

    bool Plugin::Exists(const _lo_game_handle_int& parentGame) const {
        return (fs::exists(parentGame.PluginsFolder() / name) || fs::exists(parentGame.PluginsFolder() / fs::path(name + ".ghost")));
    }

    time_t Plugin::GetModTime(const _lo_game_handle_int& parentGame) const {
        try {
            if (IsGhosted(parentGame))
                return fs::last_write_time(parentGame.PluginsFolder() / fs::path(name + ".ghost"));
            else
                return fs::last_write_time(parentGame.PluginsFolder() / name);
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_READ_FAIL, e.what());
        }
    }

    std::vector<Plugin> Plugin::GetMasters(const _lo_game_handle_int& parentGame) const {
        libespm::Plugin plugin = ReadHeader(parentGame);

        vector<Plugin> masters;
        for (const auto &master : plugin.getMasters()) {
            masters.push_back(Plugin(master));
        }

        return masters;
    }

    void Plugin::UnGhost(const _lo_game_handle_int& parentGame) const {
        if (IsGhosted(parentGame)) {
            try {
                fs::rename(parentGame.PluginsFolder() / fs::path(name + ".ghost"), parentGame.PluginsFolder() / name);
            }
            catch (fs::filesystem_error& e) {
                throw error(LIBLO_ERROR_FILE_RENAME_FAIL, e.what());
            }
        }
    }

    void Plugin::SetModTime(const _lo_game_handle_int& parentGame, const time_t modificationTime) const {
        try {
            if (IsGhosted(parentGame))
                fs::last_write_time(parentGame.PluginsFolder() / fs::path(name + ".ghost"), modificationTime);
            else
                fs::last_write_time(parentGame.PluginsFolder() / name, modificationTime);
        }
        catch (fs::filesystem_error& e) {
            throw error(LIBLO_ERROR_TIMESTAMP_WRITE_FAIL, e.what());
        }
    }

    bool Plugin::operator == (const Plugin& rhs) const {
        return boost::iequals(name, rhs.Name());
    }

    bool Plugin::operator != (const Plugin& rhs) const {
        return !(*this == rhs);
    }

    libespm::Plugin Plugin::ReadHeader(const _lo_game_handle_int& parentGame) const {
        if (!Exists(parentGame))
            throw error(LIBLO_ERROR_FILE_NOT_FOUND, name.c_str());

        try {
            string filepath = (parentGame.PluginsFolder() / name).string();
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

    void LoadOrder::Load(const _lo_game_handle_int& parentGame) {
        clear();
        if (parentGame.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE) {
            /*Game uses the new load order system.

            Check if loadorder.txt exists, and read that if it does.
            If it doesn't exist, then read plugins.txt and scan the given directory for mods,
            adding those that weren't in the plugins.txt to the end of the load order, in the order they are read.

            There is no sure-fire way of managing such a situation. If no loadorder.txt, then
            no utilties compatible with that load order method have been installed, so it won't
            break anything apart from the load order not matching the load order in the Bashed
            Patch's Masters list if it exists. That isn't something that can be easily accounted
            for though.
            */
            if (fs::exists(parentGame.LoadOrderFile()))  //If the loadorder.txt exists, get the load order from that.
                LoadFromFile(parentGame, parentGame.LoadOrderFile());
            else if (fs::exists(parentGame.ActivePluginsFile()))  //If the plugins.txt exists, get the active load order from that.
                LoadFromFile(parentGame, parentGame.ActivePluginsFile());
            else if (parentGame.Id() == LIBLO_GAME_TES5) {
                //Make sure that Skyrim.esm is first.
                Move(Plugin(parentGame.MasterFile()), this->begin());
                //Add Update.esm if not already present.
                if (Plugin("Update.esm").IsValid(parentGame) && Find(Plugin("Update.esm")) == this->cend())
                    Move(Plugin("Update.esm"), FindFirstNonMaster(parentGame));
            }
        }
        if (fs::is_directory(parentGame.PluginsFolder())) {
            //Now scan through Data folder. Add any plugins that aren't already in loadorder to loadorder, at the end.
            auto firstNonMaster = FindFirstNonMaster(parentGame);
            for (fs::directory_iterator itr(parentGame.PluginsFolder()); itr != fs::directory_iterator(); ++itr) {
                if (fs::is_regular_file(itr->status())) {
                    const Plugin plugin(itr->path().filename().string());
                    if (plugin.IsValid(parentGame) && Find(plugin) == this->cend()) {
                        //If it is a master, add it after the last master, otherwise add it at the end.
                        if (plugin.IsMasterFile(parentGame)) {
                            firstNonMaster = ++insert(firstNonMaster, plugin);
                        }
                        else {
                            // push_back may invalidate all current iterators, so reassign firstNonMaster in case.
                            size_t firstNonMasterPos = distance(this->begin(), firstNonMaster);
                            this->push_back(plugin);
                            firstNonMaster = this->begin() + firstNonMasterPos + 1;
                        }
                    }
                }
            }
        }
        //Arrange into timestamp order if required.
        if (parentGame.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
            pluginComparator pc(parentGame);
            sort(begin(), end(), pc);
        }
    }

    void LoadOrder::Save(_lo_game_handle_int& parentGame) {
        if (parentGame.LoadOrderMethod() == LIBLO_METHOD_TIMESTAMP) {
            //Update timestamps.
            //Want to make a minimum of changes to timestamps, so use the same timestamps as are currently set, but apply them to the plugins in the new order.
            //First we have to read all the timestamps.
            std::set<time_t> timestamps;
            for (const auto &plugin : *this) {
                timestamps.insert(plugin.GetModTime(parentGame));
            }
            // It may be that two plugins currently share the same timestamp,
            // which will result in fewer timestamps in the set than there are
            // plugins, so pad the set if necessary.
            while (timestamps.size() < size()) {
                timestamps.insert(*timestamps.crbegin() + 60);
            }
            size_t i = 0;
            for (const auto &timestamp : timestamps) {
                at(i).SetModTime(parentGame, timestamp);
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

                for (const auto &plugin : *this)
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
        transform(std::begin(*this),
                  std::end(*this),
                  back_inserter(pluginNames),
                  [](const Plugin& plugin) {
            return plugin.Name();
        });
        return pluginNames;
    }

    void LoadOrder::setLoadOrder(const std::vector<std::string>& pluginNames, const _lo_game_handle_int& gameHandle) {
        // For textfile-based load order games, check that the game's master file loads first.
        if (gameHandle.LoadOrderMethod() == LIBLO_METHOD_TEXTFILE && (pluginNames.empty() || !boost::iequals(pluginNames[0], gameHandle.MasterFile())))
            throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + gameHandle.MasterFile() + "\" must load first.");

        // Check that all masters load before non-masters.
        if (!is_partitioned(std::begin(pluginNames),
            std::end(pluginNames),
            [&](const std::string& pluginName) {
            return Plugin(pluginName).IsMasterFile(gameHandle);
        })) {
            throw error(LIBLO_ERROR_INVALID_ARGS, "Master plugins must load before all non-master plugins.");
        }

        // Check all plugins are valid and unique.
        unordered_set<string> hashset;
        for_each(std::begin(pluginNames), std::end(pluginNames), [&](const std::string& pluginName) {
            if (hashset.find(boost::to_lower_copy(pluginName)) != hashset.end())
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is a duplicate entry.");

            if (!Plugin(pluginName).IsValid(gameHandle))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + pluginName + "\" is not a valid plugin file.");

            hashset.insert(boost::to_lower_copy(pluginName));
        });

        this->clear();
        transform(std::begin(pluginNames),
                  std::end(pluginNames),
                  back_inserter(*this),
                  [](const std::string& pluginName) {
            return Plugin(pluginName);
        });
    }

    void LoadOrder::CheckValidity(const _lo_game_handle_int& parentGame) {
        if (empty())
            return;

        if (at(0) != Plugin(parentGame.MasterFile()))
            throw error(LIBLO_WARN_INVALID_LIST, "\"" + parentGame.MasterFile() + "\" is not the first plugin in the load order. " + at(0).Name() + " is first.");

        bool wasMaster = true;
        unordered_set<Plugin> hashset;
        for (const auto plugin : *this) {
            if (!plugin.Exists(parentGame))
                throw error(LIBLO_WARN_INVALID_LIST, "\"" + plugin.Name() + "\" is not installed.");
            else if (!plugin.IsValid(parentGame))
                throw error(LIBLO_WARN_INVALID_LIST, "\"" + plugin.Name() + "\" is not a valid plugin file.");
            bool isMaster = plugin.IsMasterFile(parentGame);
            if (isMaster && !wasMaster)
                throw error(LIBLO_WARN_INVALID_LIST, "Master plugin \"" + plugin.Name() + "\" loaded after a non-master plugin.");
            if (hashset.find(plugin) != hashset.end())
                throw error(LIBLO_WARN_INVALID_LIST, "\"" + plugin.Name() + "\" is in the load order twice.");
            for (const auto &master : plugin.GetMasters(parentGame)) {
                if (hashset.find(master) == hashset.end() && this->Find(master) != this->end())  //Only complain about  masters loading after the plugin if the master is installed (so that Filter patches do not cause false positives). This means libloadorder doesn't check to ensure all a plugin's masters are present, but I don't think it should get mixed up with Bash Tag detection.
                    throw error(LIBLO_WARN_INVALID_LIST, "\"" + plugin.Name() + "\" is loaded before one of its masters (\"" + master.Name() + "\").");
            }
            hashset.insert(plugin);
            wasMaster = isMaster;
        }
    }

    bool LoadOrder::HasChanged(const _lo_game_handle_int& parentGame) const {
        if (empty())
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

    std::vector<Plugin>::iterator LoadOrder::Move(const Plugin& plugin, std::vector<Plugin>::iterator newPos) {
        if (distance(this->begin(), newPos) > this->size())
            throw error(LIBLO_ERROR_INVALID_ARGS, "New plugin position is beyond end of container.");

        if (newPos != this->end() && *newPos == plugin)
            return newPos;  // No movement necessary.

        // Inserting and erasing iterators invalidates later iterators, so first insert into
        // the vector.
        bool moveToEnd = (newPos == this->end());

        newPos = this->insert(newPos, plugin);

        auto it = this->begin();
        while (it != this->end()) {
            if (it != newPos
                && (!moveToEnd || it != --this->end())
                && *it == plugin)
                it = this->erase(it);
            else
                ++it;
        }

        return newPos;
    }

    std::vector<Plugin>::iterator LoadOrder::Find(const Plugin& plugin) {
        return find(this->begin(), this->end(), plugin);
    }

    std::vector<Plugin>::iterator LoadOrder::FindFirstNonMaster(const _lo_game_handle_int& parentGame) {
        return find_if(this->begin(), this->end(), [&parentGame](const Plugin& plugin) {
            return !plugin.IsMasterFile(parentGame);
        });
    }

    void LoadOrder::LoadFromFile(const _lo_game_handle_int& parentGame, const fs::path& file) {
        if (!fs::exists(file))
            throw error(LIBLO_ERROR_FILE_NOT_FOUND, file.string() + " cannot be found.");

        //loadorder.txt is simple enough that we can avoid needing a formal parser.
        //It's just a text file with a plugin filename on each line. Skip lines which are blank or start with '#'.
        try {
            fs::ifstream in(file);
            in.exceptions(std::ios_base::badbit);

            string line;
            regex reg("GameFile[0-9]{1,3}=.+\\.es(m|p)", regex::ECMAScript | regex::icase);
            bool transcode = (file == parentGame.ActivePluginsFile());
            while (getline(in, line)) {
                // Check if it's a valid plugin line. The stream doesn't filter out '\r' line endings, hence the check.
                if (line.empty() || line[0] == '#' || line[0] == '\r')
                    continue;

                if (parentGame.Id() == LIBLO_GAME_TES3) {
                    //Morrowind's active file list is stored in Morrowind.ini, and that has a different format from plugins.txt.
                    if (regex_match(line, reg))
                        line = line.substr(line.find('=') + 1);
                    else
                        continue;
                }

                if (transcode)
                    line = ToUTF8(line);
                else {
                    //Test that the string is UTF-8 encoded by trying to convert it to UTF-16. It should throw if an invalid byte is found.
                    try {
                        boost::locale::conv::utf_to_utf<wchar_t>(line, boost::locale::conv::stop);
                    }
                    catch (...) {
                        throw error(LIBLO_ERROR_FILE_NOT_UTF8, "\"" + file.string() + "\" is not encoded in valid UTF-8.");
                    }
                }

                Plugin plugin(line);
                if (plugin.IsValid(parentGame))
                    this->push_back(plugin);
            }
            in.close();
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
        }

        if (parentGame.Id() == LIBLO_GAME_TES5 && file == parentGame.ActivePluginsFile()) {
            //Make sure that Skyrim.esm is first.
            Move(Plugin(parentGame.MasterFile()), this->begin());
            //Add Update.esm if not already present.
            if (Plugin("Update.esm").IsValid(parentGame) && Find(Plugin("Update.esm")) == this->cend())
                Move(Plugin("Update.esm"), FindFirstNonMaster(parentGame));
        }
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
                for (const auto &plugin : parentGame.loadOrder) {
                    if (find(plugin) == end() || (parentGame.Id() == LIBLO_GAME_TES5 && plugin.Name() == parentGame.MasterFile()))
                        continue;

                    try {
                        outfile << FromUTF8(plugin.Name()) << endl;
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

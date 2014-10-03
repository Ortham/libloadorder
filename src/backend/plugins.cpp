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

#include "../api/constants.h"
#include "error.h"
#include "plugins.h"
#include "game.h"
#include "helpers.h"
#include "streams.h"
#include <boost/filesystem.hpp>
#include <boost/locale.hpp>
#include <regex>
#include <set>

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
        // Rather than just checking the extension, try parsing the file, and see if it fails.
        try {
            espm::File * file = ReadHeader(parentGame);
            delete file;
        }
        catch (std::exception& /*e*/) {
            return false;
        }
        return true;
    }

    bool Plugin::IsMasterFile(const _lo_game_handle_int& parentGame) const {
        espm::File * file = ReadHeader(parentGame);

        bool ret = file->isMaster(parentGame.espm_settings);

        delete file;
        return ret;
    }

    bool Plugin::IsFalseFlagged(const _lo_game_handle_int& parentGame) const {
        string ext;
        if (IsGhosted(parentGame))
            ext = fs::path(name).stem().extension().string();
        else
            ext = fs::path(name).extension().string();
        return ((IsMasterFile(parentGame) && !boost::iequals(ext, ".esm")) || (!IsMasterFile(parentGame) && boost::iequals(ext, ".esm")));
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
        espm::File * file = ReadHeader(parentGame);

        vector<Plugin> masters;
        for (const auto &master : file->getMasters()) {
            masters.push_back(Plugin(master));
        }

        delete file;
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

    espm::File * Plugin::ReadHeader(const _lo_game_handle_int& parentGame) const {
        try {
            string filepath = (parentGame.PluginsFolder() / name).string();
            if (IsGhosted(parentGame))
                filepath += ".ghost";

            espm::File * file = nullptr;
            if (parentGame.Id() == LIBLO_GAME_TES3)
                file = new espm::tes3::File(filepath, parentGame.espm_settings, false, true);
            else if (parentGame.Id() == LIBLO_GAME_TES4)
                file = new espm::tes4::File(filepath, parentGame.espm_settings, false, true);
            else if (parentGame.Id() == LIBLO_GAME_TES5)
                file = new espm::tes5::File(filepath, parentGame.espm_settings, false, true);
            else if (parentGame.Id() == LIBLO_GAME_FO3)
                file = new espm::fo3::File(filepath, parentGame.espm_settings, false, true);
            else
                file = new espm::fonv::File(filepath, parentGame.espm_settings, false, true);

            return file;
        }
        catch (std::exception& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, name + " : " + e.what());
        }
    }

    /////////////////////////
    // LoadOrder Members
    /////////////////////////

    struct pluginComparator {
        const _lo_game_handle_int& parentGame;
        pluginComparator(const _lo_game_handle_int& game) : parentGame(game) {}

        bool    operator () (const Plugin& plugin1, const Plugin& plugin2) {
            //Return true if plugin1 goes before plugin2, false otherwise.
            //Master files should go before other files.
            //Earlier stamped plugins should go before later stamped plugins.

            bool isPlugin1MasterFile = plugin1.IsMasterFile(parentGame);
            bool isPlugin2MasterFile = plugin2.IsMasterFile(parentGame);

            if (isPlugin1MasterFile && !isPlugin2MasterFile)
                return true;
            else if (!isPlugin1MasterFile && isPlugin2MasterFile)
                return false;
            else
                return (difftime(plugin1.GetModTime(parentGame), plugin2.GetModTime(parentGame)) < 0);
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
            else {
                if (fs::exists(parentGame.ActivePluginsFile()))  //If the plugins.txt exists, get the active load order from that.
                    LoadFromFile(parentGame, parentGame.ActivePluginsFile());
                if (parentGame.Id() == LIBLO_GAME_TES5) {
                    //Make sure that Skyrim.esm is first.
                    Move(Plugin("Skyrim.esm"), this->begin());
                    //Add Update.esm if not already present.
                    if (Plugin("Update.esm").Exists(parentGame) && Find(Plugin("Update.esm")) == this->cend())
                        Move(Plugin("Update.esm"), FindFirstNonMaster(parentGame));
                }
            }
        }
        if (fs::exists(parentGame.PluginsFolder()) && fs::is_directory(parentGame.PluginsFolder())) {
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
            for (size_t i = 1, max = size(); i < max; i++) {
                timestamps.insert(at(i).GetModTime(parentGame));
            }
            size_t i = 1;
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
                liblo::ofstream outfile(parentGame.LoadOrderFile(), ios_base::trunc);
                outfile.exceptions(std::ios_base::badbit);

                for (const auto &plugin : *this)
                    outfile << plugin.Name() << endl;
                outfile.close();

                //Now record new loadorder.txt mtime.
                //Plugins.txt doesn't need its mtime updated as only the order of its contents has changed, and it is stored in memory as an unordered set.
                mtime = fs::last_write_time(parentGame.LoadOrderFile());
            }
            catch (std::ios_base::failure& e) {
                throw error(LIBLO_ERROR_FILE_WRITE_FAIL, "\"" + parentGame.LoadOrderFile().string() + "\" cannot be written to. Details" + e.what());
            }

            //Now write plugins.txt. Update cache if necessary.
            if (parentGame.activePlugins.HasChanged(parentGame))
                parentGame.activePlugins.Load(parentGame);
            parentGame.activePlugins.Save(parentGame);
        }
    }

    void LoadOrder::CheckValidity(const _lo_game_handle_int& parentGame) {
        if (empty())
            return;

        if (at(0) != Plugin(parentGame.MasterFile()))
            throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + parentGame.MasterFile() + "\" is not the first plugin in load order.");

        bool wasMaster = true;
        unordered_set<Plugin> hashset;
        for (const auto plugin : *this) {
            if (!plugin.Exists(parentGame))
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + plugin.Name() + "\" is not installed.");
            bool isMaster = plugin.IsMasterFile(parentGame);
            if (isMaster && !wasMaster)
                throw error(LIBLO_ERROR_INVALID_ARGS, "Master plugin \"" + plugin.Name() + "\" loaded after a non-master plugin.");
            if (hashset.find(plugin) != hashset.end())
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + plugin.Name() + "\" is in the load order twice.");
            vector<Plugin> masters(plugin.GetMasters(parentGame));
            for (const auto &master : masters) {
                if (hashset.find(master) == hashset.end() && this->Find(master) != this->end())  //Only complain about  masters loading after the plugin if the master is installed (so that Filter patches do not cause false positives). This means libloadorder doesn't check to ensure all a plugin's masters are present, but I don't think it should get mixed up with Bash Tag detection.
                    throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + plugin.Name() + "\" is loaded before one of its masters (\"" + master.Name() + "\").");
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
        // Inserting and erasing iterators invalidates later iterators, so first insert into
        // the vector.
        newPos = this->insert(newPos, plugin);

        // Now erase any other instances of this plugin.
        auto oldPos = find(this->cbegin(), this->cend(), plugin);
        while (oldPos != this->cend()) {
            if (oldPos != newPos)
                oldPos = this->erase(oldPos);
            else
                ++oldPos;
            oldPos = find(oldPos, this->cend(), plugin);
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
        bool transcode = file == parentGame.ActivePluginsFile();

        //loadorder.txt is simple enough that we can avoid needing a formal parser.
        //It's just a text file with a plugin filename on each line. Skip lines which are blank or start with '#'.
        try {
            liblo::ifstream in(file);
            in.exceptions(std::ios_base::badbit);

            string line;
            bool isTES3 = parentGame.Id() == LIBLO_GAME_TES3;  //Morrowind's active file list is stored in Morrowind.ini, and that has a different format from plugins.txt.
            regex reg = regex("GameFile[0-9]{1,3}=.+\\.es(m|p)", regex::ECMAScript | regex::icase);

            while (getline(in, line)) {
                if (line.empty() || (isTES3 && !regex_match(line, reg)) || (!isTES3 && line[0] == '#'))
                    continue;

                if (isTES3)  //Now cut off everything up to and including the = sign.
                    line = line.substr(line.find('=') + 1);
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

                //We need to remove plugins that are no longer installed from the load order, otherwise it'll cause problems later.
                Plugin p(line);
                if (p.Exists(parentGame))
                    push_back(p);
            }
            in.close();
        }
        catch (std::ios_base::failure& e) {
            throw error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + file.string() + "\" could not be read. Details: " + e.what());
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
                liblo::ifstream in(parentGame.ActivePluginsFile());
                in.exceptions(std::ios_base::badbit);

                if (parentGame.Id() == LIBLO_GAME_TES3) {  //Morrowind's active file list is stored in Morrowind.ini, and that has a different format from plugins.txt.
                    regex reg = regex("GameFile[0-9]{1,3}=.+\\.es(m|p)", regex::ECMAScript | regex::icase);
                    while (getline(in, line)) {
                        if (line.empty() || !regex_match(line, reg))
                            continue;

                        //Now cut off everything up to and including the = sign.
                        insert(Plugin(ToUTF8(line.substr(line.find('=') + 1))));
                    }
                }
                else {
                    while (getline(in, line)) {
                        if (line.empty() || line[0] == '#')  //Character comparison is OK because it's ASCII.
                            continue;

                        insert(Plugin(ToUTF8(line)));
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
            if (find(Plugin("Skyrim.esm")) == end())
                insert(Plugin("Skyrim.esm"));
            if (Plugin("Update.esm").Exists(parentGame) && find(Plugin("Update.esm")) == end())
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
            liblo::ofstream outfile(parentGame.ActivePluginsFile(), ios_base::trunc);
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
                    if (find(plugin) == end() || parentGame.Id() == LIBLO_GAME_TES5 && plugin.Name() == parentGame.MasterFile())
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
                throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + plugin.Name() + "\" is not installed.");
            /*vector<Plugin> masters = plugin.GetMasters(parentGame);
            //Disabled because it causes false positives for Filter patches. This means libloadorder doesn't check to ensure all a plugin's masters are active, but I don't think it should get mixed up with Bash Tag detection.
            for (const auto& master: masters) {
            if (this->find(master) == this->end())
            throw error(LIBLO_ERROR_INVALID_ARGS, "\"" + plugin.Name() + "\" has a master (\"" + master.Name() + "\") which isn't active.");
            }*/
        }

        if (size() > 255)
            throw error(LIBLO_ERROR_INVALID_ARGS, "More than 255 plugins are active.");
        else if (parentGame.Id() == LIBLO_GAME_TES5) {
            if (find(Plugin("Skyrim.esm")) == end())
                throw error(LIBLO_ERROR_INVALID_ARGS, "Skyrim.esm isn't active.");
            else if (Plugin("Update.esm").Exists(parentGame) && find(Plugin("Update.esm")) == end())
                throw error(LIBLO_ERROR_INVALID_ARGS, "Update.esm is installed but isn't active.");
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

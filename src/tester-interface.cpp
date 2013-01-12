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

#include "tester-interface.h"

namespace tester {

    namespace liblo {

        /*------------------------------
           Version Functions
        ------------------------------*/

        bool IsCompatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch) {
            return ::lo_is_compatible(versionMajor, versionMinor, versionPatch);
        }

        void GetVersionNums(unsigned int& versionMajor, unsigned int& versionMinor, unsigned int& versionPatch) {
            ::lo_get_version(&versionMajor, &versionMinor, &versionPatch);
        }


        /*------------------------------
           Error Handling Functions
        ------------------------------*/

        exception::exception() : errCode(0) {}

        exception::exception(unsigned int code, std::string message) : errCode(0), errMessage(message) {}

        unsigned int exception::code() {
            return errCode;
        }

        std::string exception::what() {
            return errMessage;
        }


        /*----------------------------------
           Game Handle Based Functions
        ----------------------------------*/

        GameHandle::GameHandle(const unsigned int gameId, const std::string gamePath) {
            char * p = ToNewCString(gamePath);
            Handler(::lo_create_handle(&gh, gameId, p), p);
        }

        GameHandle::~GameHandle() {
            ::lo_destroy_handle(gh);
            gh = NULL;
        }

        void GameHandle::SetGameMaster(const std::string filename) {
            char * p = ToNewCString(filename);
            Handler(::lo_set_game_master(gh, p), p);
        }

        unsigned int GameHandle::LoadOrderMethod() {
            unsigned int method;
            Handler(::lo_get_load_order_method(gh, &method));
            return method;
        }

        std::vector<std::string> GameHandle::LoadOrder() {
            char ** pluginArray;
            size_t arrSize;
            std::vector<std::string> pluginVector;
            Handler(::lo_get_load_order(gh, &pluginArray, &arrSize));
            for (size_t i=0; i<arrSize; i++)
                pluginVector.push_back(ToStdString(pluginArray[i]));
            return pluginVector;
        }

        void GameHandle::LoadOrder(const std::vector<std::string>& newLoadOrder) {
            size_t arrSize = newLoadOrder.size();
            char ** pluginArray = new char*[arrSize];
            for (size_t i=0; i < arrSize; i++)
                pluginArray[i] = ToNewCString(newLoadOrder[i]);
            Handler(::lo_set_load_order(gh, pluginArray, arrSize), pluginArray, arrSize);
        }

        size_t GameHandle::PluginLoadOrder(const std::string plugin) {
            size_t index;
            char * p = ToNewCString(plugin);
            Handler(::lo_get_plugin_position(gh, p, &index), p);
            return index;
        }

        void GameHandle::PluginLoadOrder(const std::string plugin, const size_t index) {
            char * p = ToNewCString(plugin);
            Handler(::lo_set_plugin_position(gh, p, index), p);
        }

        std::string GameHandle::PluginAtIndex(const size_t index) {
            char * plugin;
            Handler(::lo_get_indexed_plugin(gh, index, &plugin));
            return ToStdString(plugin);
        }

        std::set<std::string> GameHandle::ActivePlugins() {
            char ** pluginArray;
            size_t arrSize;
            std::set<std::string> pluginSet;
            Handler(::lo_get_active_plugins(gh, &pluginArray, &arrSize));
            for (size_t i=0; i<arrSize; i++)
                pluginSet.insert(ToStdString(pluginArray[i]));
            return pluginSet;
        }

        void GameHandle::ActivePlugins(const std::set<std::string>& newActivePlugins) {
            size_t arrSize = newActivePlugins.size();
            char ** pluginArray = new char*[arrSize];
            size_t i = 0;
            for (std::set<std::string>::iterator it = newActivePlugins.begin(), endIt = newActivePlugins.end(); it != endIt; ++it) {
                pluginArray[i] = ToNewCString(*it);
                i++;
            }
            Handler(::lo_set_active_plugins(gh, pluginArray, arrSize), pluginArray, arrSize);
        }

        bool GameHandle::IsPluginActive(const std::string plugin) {
            char * p = ToNewCString(plugin);
            bool result;
            Handler(::lo_get_plugin_active(gh, p, &result), p);
            return result;
        }

        void GameHandle::SetPluginActiveStatus(const std::string plugin, const bool active) {
            char * p = ToNewCString(plugin);
            Handler(::lo_set_plugin_active(gh, p, active), p);
        }

        //Return code handler - throws exception on receiving an error code.
        void GameHandle::Handler(unsigned int retCode) {
            if (retCode != LIBLO_OK) {
                char * message;
                std::string msgStr;
                if (::lo_get_error_message(&message) != LIBLO_OK)
                    msgStr = "The error message could not be retrieved as a second error was encountered by the retrieval function.";
                else
                    msgStr = ToStdString(message);
                ::lo_cleanup();
                throw exception(retCode, msgStr);
            }
        }

        void GameHandle::Handler(unsigned int retCode, char * pointer) {
            delete [] pointer;
            Handler(retCode);
        }

        void GameHandle::Handler(unsigned int retCode, char ** arrPointer, size_t arrSize) {
            for (size_t i=0; i < arrSize; i++)
                delete [] arrPointer[i];
            delete [] arrPointer;
            Handler(retCode);
        }

        //Explicit memory management, need to call delete on the output when finished with it.
        char * GameHandle::ToNewCString(std::string str) {
            size_t length = str.length() + 1;
            char * p = new char[length];

            for (size_t j=0; j < str.length(); j++) {  //UTF-8, so this is code-point by code-point rather than char by char, but same result here.
                p[j] = str[j];
            }
            p[length - 1] = '\0';
            return p;
        }

        //No explicit memory management. Returns new string object, so probably (not sure) lasts until calling function terminates.
        std::string GameHandle::ToStdString(char * str) {
            return std::string(reinterpret_cast<const char *>(str));
        }
    }
}

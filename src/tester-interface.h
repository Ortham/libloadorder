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

#ifndef LIBLO_TESTER_INTERFACE_H
#define LIBLO_TESTER_INTERFACE_H

#include "libloadorder.h"
#include <stdint.h>
#include <string>
#include <vector>
#include <set>

//The following functions wrap libloadorder's C API to a C++ interface that easier to use.

namespace tester {

    namespace liblo {

        /*------------------------------
           Version Functions
        ------------------------------*/

        bool IsCompatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch);

        void GetVersionNums(unsigned int& versionMajor, unsigned int& versionMinor, unsigned int& versionPatch);


        /*------------------------------
           Error Handling Functions
        ------------------------------*/

        class exception {
        public:
            exception();
            exception(unsigned int code, std::string message);

            unsigned int code();
            std::string what();
        private:
            unsigned int errCode;
            std::string errMessage;
        };


        /*----------------------------------
           Game Handle Based Functions
        ----------------------------------*/

        class GameHandle {
        public:
            GameHandle(const unsigned int gameId, const std::string gamePath);
            ~GameHandle();

            void SetGameMaster(const std::string filename);

            unsigned int LoadOrderMethod();

            std::vector<std::string> LoadOrder();
            void LoadOrder(const std::vector<std::string>& newLoadOrder);

            size_t PluginLoadOrder(const std::string plugin);
            void PluginLoadOrder(const std::string plugin, const size_t index);

            std::string PluginAtIndex(const size_t index);

            std::set<std::string> ActivePlugins();
            void ActivePlugins(const std::set<std::string>& newActivePlugins);

            bool IsPluginActive(const std::string plugin);
            void SetPluginActiveStatus(const std::string plugin, const bool active);
        private:
            lo_game_handle gh;

            //Return code handler - throws exception on receiving an error code.
            void Handler(unsigned int retCode);
            void Handler(unsigned int retCode, char * pointer);
            void Handler(unsigned int retCode, char ** arrPointer, size_t arrSize);

            //Explicit memory management, need to call delete on the output when finished with it.
            char * ToNewCString(std::string str);

            //No explicit memory management. Returns new string object, so probably (not sure) lasts until calling function terminates.
            std::string ToStdString(char * str);
        };

    }
}

#endif

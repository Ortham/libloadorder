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

#ifndef LIBLO_GAME_H
#define LIBLO_GAME_H

#include "../backend/LoadOrder.h"
#include "../backend/GameSettings.h"
#include "c_helpers.h"

#include <mutex>
#include <unordered_map>

#include <boost/filesystem.hpp>

struct _lo_game_handle_int : public liblo::GameSettings {
public:
    _lo_game_handle_int(unsigned int id, const boost::filesystem::path& gamePath, const boost::filesystem::path& localPath = "");
    ~_lo_game_handle_int();

    liblo::LoadOrder loadOrder;

    const char * getExternalString() const;
    const std::vector<const char *>& getExternalStringArray() const;

    void setExternalString(const std::string& str);

    template<class T>
    void setExternalStringArray(T container) {
        std::lock_guard<std::mutex> guard(mutex);

        auto it = dataStore.find(this);
        if (it == end(dataStore))
            it = dataStore.emplace(this, GameHandleData()).first;

        it->second.internalStringArray.resize(container.size());
        std::move(begin(container), end(container), begin(it->second.internalStringArray));

        it->second.externalStringArray.resize(it->second.internalStringArray.size());
        std::transform(begin(it->second.internalStringArray),
                       end(it->second.internalStringArray),
                       begin(it->second.externalStringArray),
                       [](const std::string& element) {
            return element.c_str();
        });
    }
private:
    struct GameHandleData {
        std::string externalString;
        std::vector<const char *> externalStringArray;
        std::vector<std::string> internalStringArray;
    };

    thread_local static std::unordered_map<const _lo_game_handle_int *, GameHandleData> dataStore;
    static std::mutex mutex;
};

#endif

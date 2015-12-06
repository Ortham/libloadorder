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

#include "_lo_game_handle_int.h"

using namespace liblo;

_lo_game_handle_int::_lo_game_handle_int(unsigned int id, const boost::filesystem::path& gamePath, const boost::filesystem::path& localPath)
    : GameSettings(id, gamePath, localPath),
    loadOrder(*this) {
    std::lock_guard<std::mutex> guard(mutex);

    dataStore.emplace(this, GameHandleData());
}

_lo_game_handle_int::~_lo_game_handle_int() {
    std::lock_guard<std::mutex> guard(mutex);

    dataStore.erase(this);
}

const char * _lo_game_handle_int::getExternalString() const {
    std::lock_guard<std::mutex> guard(mutex);

    auto it = dataStore.find(this);
    if (it == end(dataStore))
        it = dataStore.emplace(this, GameHandleData()).first;

    return it->second.externalString.c_str();
}

const std::vector<const char *>& _lo_game_handle_int::getExternalStringArray() const {
    std::lock_guard<std::mutex> guard(mutex);

    auto it = dataStore.find(this);
    if (it == end(dataStore))
        it = dataStore.emplace(this, GameHandleData()).first;

    return it->second.externalStringArray;
}

void _lo_game_handle_int::setExternalString(const std::string& str) {
    std::lock_guard<std::mutex> guard(mutex);

    auto it = dataStore.find(this);
    if (it == end(dataStore))
        it = dataStore.emplace(this, GameHandleData()).first;

    it->second.externalString = str;
}

thread_local std::unordered_map<const _lo_game_handle_int *, _lo_game_handle_int::GameHandleData> _lo_game_handle_int::dataStore;

std::mutex _lo_game_handle_int::mutex;

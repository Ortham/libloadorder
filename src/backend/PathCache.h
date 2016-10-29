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

#ifndef LIBLO_PATH_CACHE_H
#define LIBLO_PATH_CACHE_H

#include <map>

#include <boost/filesystem.hpp>

namespace liblo {
    class PathCache {
    public:
        bool isModified(const boost::filesystem::path& file) const;

        void updateCachedState(const boost::filesystem::path& file);
        void clear();
    private:
      std::map<boost::filesystem::path, time_t> modificationTimes;
    };
}

#endif

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

#ifndef LIBLO_C_HELPERS_H
#define LIBLO_C_HELPERS_H

#include <string>

#include <boost/filesystem.hpp>

namespace liblo {
    class error;

    // std::string to null-terminated char string converter.
    char * copyString(const std::string& str);

    template<class T>
    T copyToContainer(const char * const * const plugins, size_t numPlugins) {
        T container;
        for (size_t i = 0; i < numPlugins; i++)
            container.insert(end(container), plugins[i]);

        return container;
    }

    extern char * extErrorString;

    unsigned int c_error(const error& e);

    unsigned int c_error(const unsigned int code, const std::string& what);
}

#endif

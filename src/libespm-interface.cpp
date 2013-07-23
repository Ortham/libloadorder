/*  libloadorder

    A library for reading and writing the load order of plugin files for
    TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, Fallout 3 and
    Fallout: New Vegas.

    Copyright (C) 2012    WrinklyNinja, Leandro Conde

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

#include "libespm-interface.h"
#include "streams.h"
#include "error.h"
#include "libloadorder.h"

#include <cstring>

namespace libespm {

    using namespace std;

    typedef unsigned long   ulong;
    const ulong TES4    =   '4SET';
    const ulong MAXLENGTH = 4096UL;  // An arbitrary large number. Controls the size of some buffers used to read data from files.


    //
    // T Peek<T>(pointer&):
    //  - Peeks into the received buffer and returns the value pointed
    //  converting it to the type T.
    //
    template <typename T>
    T Peek(char* buffer) {
        return *reinterpret_cast<T*>(buffer);
    }

    //
    // T Read<T>(pointer&):
    //  - Tries to extract a value of the specified type T from the
    //  received buffer, incrementing the pointer to point past the readen
    //  value.
    //
    template <typename T>
    inline T Read(char*& buffer) {
        T value = Peek<T>(buffer);
        buffer += sizeof(T);
        return value;
    }

    //-
    // ModHeader ReadHeader(string):
    //  - Parses the mod file contents and extract the header information
    //  returning the most important data using a ModHeader struct.
    //  --> see:
    //          http://www.uesp.net/wiki/Tes4Mod:Mod_File_Format,
    //
    //  and in particular this link:
    //          http://www.uesp.net/wiki/Tes4Mod:Mod_File_Format/TES4
    //

    bool IsPluginMaster(const _lo_game_handle_int& parentGame, const std::string& filename) {
        char        buffer[MAXLENGTH];
        char*       bufptr = buffer;

        if (filename.empty())
            return false;

        try {
            boost::filesystem::path filepath = parentGame.PluginsFolder() / filename;
            liblo::ifstream    file(filepath, ios_base::binary);
            file.exceptions(std::ios_base::badbit);

            // Reads the first MAXLENGTH bytes into the buffer
            file.read(&buffer[0], sizeof(buffer));

            // Check for the 'magic' marker at start
            if (Read<unsigned int>(bufptr) != TES4)
                return false;

            // Next field is the total header size
            /*unsigned int headerSize =*/ Read<unsigned int>(bufptr);

            // Next comes the header record Flags
            unsigned int flags = Read<unsigned int>(bufptr);

            // LSb of this record's flags is used to indicate if the
            //  mod is a master or a plugin
            return ((flags & 0x1) != 0);
        } catch (std::ios_base::failure& e) {
            throw liblo::error(LIBLO_ERROR_FILE_READ_FAIL, "\"" + (parentGame.PluginsFolder() / filename).string() + "\" could not be read. Details: " + e.what());
        }
    }

    std::vector<std::string> GetPluginMasters(const _lo_game_handle_int& parentGame, const std::string& filename) {
        vector<string> vec;
        return vec;
    }
}

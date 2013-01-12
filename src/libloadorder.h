/*      libloadorder

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

#ifndef LIBLO_H
#define LIBLO_H

#include <stdint.h>
#include <stddef.h>

#if defined(_MSC_VER)
//MSVC doesn't support C99, so do the stdbool.h definitions ourselves.
//START OF stdbool.h DEFINITIONS.
#       ifndef __cplusplus
#               define bool     _Bool
#               define true     1
#               define false   0
#       endif
#       define __bool_true_false_are_defined   1
//END OF stdbool.h DEFINITIONS.
#else
#       include <stdbool.h>
#endif

// set up dll import/export decorators
// when compiling the dll on windows, ensure LIBLO_EXPORT is defined.  clients
// that use this header do not need to define anything to import the symbols
// properly.
#if defined(_WIN32) || defined(_WIN64)
#       ifdef LIBLO_STATIC
#               define LIBLO
#   elif defined LIBLO_EXPORT
#       define LIBLO __declspec(dllexport)
#   else
#       define LIBLO __declspec(dllimport)
#   endif
#else
#   define LIBLO
#endif

#ifdef __cplusplus
extern "C"
{
#endif

/*------------------------------
   Types
------------------------------*/

/* All library strings are char* strings encoded in UTF-8. Strings returned
   by the library should not have their memory freed by the client: the API will
   clean up after itself.
   All library numbers and error codes are unsigned int integers. */

/* Abstracts the definition of libloadorder's internal state while still providing
   type safety across the library. */
typedef struct Game * lo_game_handle;

/* The following are the possible codes that the library can return. */
LIBLO extern const unsigned int LIBLO_OK;
LIBLO extern const unsigned int LIBLO_WARN_BAD_FILENAME;
LIBLO extern const unsigned int LIBLO_WARN_LO_MISMATCH;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_READ_FAIL;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_WRITE_FAIL;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_RENAME_FAIL;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_PARSE_FAIL;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_NOT_UTF8;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_NOT_FOUND;
LIBLO extern const unsigned int LIBLO_ERROR_TIMESTAMP_READ_FAIL;
LIBLO extern const unsigned int LIBLO_ERROR_TIMESTAMP_WRITE_FAIL;
LIBLO extern const unsigned int LIBLO_ERROR_NO_MEM;
LIBLO extern const unsigned int LIBLO_ERROR_INVALID_ARGS;
LIBLO extern const unsigned int LIBLO_RETURN_MAX;

/* The following are for signifying what load order method is being used. */
LIBLO extern const unsigned int LIBLO_METHOD_TIMESTAMP;
LIBLO extern const unsigned int LIBLO_METHOD_TEXTFILE;

/* The following are the games identifiers used by the library. */
LIBLO extern const unsigned int LIBLO_GAME_TES3;
LIBLO extern const unsigned int LIBLO_GAME_TES4;
LIBLO extern const unsigned int LIBLO_GAME_TES5;
LIBLO extern const unsigned int LIBLO_GAME_FO3;
LIBLO extern const unsigned int LIBLO_GAME_FNV;


/*------------------------------
   Version Functions
------------------------------*/

/* Returns whether this version of libloadorder is compatible with the given
   version of libloadorder. */
LIBLO bool lo_is_compatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch);

/* Gets the version numbers for the libary. */
LIBLO void lo_get_version(unsigned int * versionMajor, unsigned int * versionMinor, unsigned int * versionPatch);


/*------------------------------
   Error Handling Functions
------------------------------*/

/* Outputs a string giving the a message containing the details of the
   last error or warning encountered by a function. */
LIBLO unsigned int lo_get_error_message(char ** details);

/* Frees the memory allocated to the last error details string. */
LIBLO void lo_cleanup();


/*----------------------------------
   Lifecycle Management Functions
----------------------------------*/

/* Creates a handle for the game given by gameId, which is found at gamePath. This handle allows
   clients to free memory when they want to. gamePath is case-sensitive if the underlying filesystem
   is case-sensitive. */
LIBLO unsigned int lo_create_handle(lo_game_handle * gh, const unsigned int gameId, const char * gamePath);

/* Destroys the given game handle, freeing up memory allocated during its use. */
LIBLO void lo_destroy_handle(lo_game_handle gh);

/* Sets the game's master file to a given filename, eg. for use with total conversions where
   the original main master file is replaced. */
LIBLO unsigned int lo_set_game_master(lo_game_handle gh, const char * masterFile);


/*------------------------------
   Load Order Functions
------------------------------*/

/* Returns which method the game uses for the load order. */
LIBLO unsigned int lo_get_load_order_method(lo_game_handle gh, unsigned int * method);

/* Outputs a list of the plugins installed in the data path specified when the DB was
   created in load order, with the number of plugins given by numPlugins. */
LIBLO unsigned int lo_get_load_order(lo_game_handle gh, char *** plugins, size_t * numPlugins);

/* Sets the load order to the given plugins list of length numPlugins.
   Then scans the Data directory and appends any other plugins not included in the
   array passed to the function. */
LIBLO unsigned int lo_set_load_order(lo_game_handle gh, char ** plugins, const size_t numPlugins);

/* Gets the load order of the specified plugin, giving it as index. The first position
   in the load order is 0. */
LIBLO unsigned int lo_get_plugin_position(lo_game_handle gh, const char * plugin, size_t * index);

/* Sets the load order of the specified plugin, removing it from its current position
   if it has one. The first position in the load order is 0. If the index specified is
   greater than the number of plugins in the load order, the plugin will be inserted at
   the end of the load order. */
LIBLO unsigned int lo_set_plugin_position(lo_game_handle gh, const char * plugin, size_t index);

/* Gets the plugin filename is at the specified load order position. The first position
   in the load order is 0. */
LIBLO unsigned int lo_get_indexed_plugin(lo_game_handle gh, const size_t index, char ** plugin);


/*----------------------------------
   Plugin Active Status Functions
----------------------------------*/

/* Returns the list of active plugins. */
LIBLO unsigned int lo_get_active_plugins(lo_game_handle gh, char *** plugins, size_t * numPlugins);

/* Replaces the current list of active plugins with the given list. */
LIBLO unsigned int lo_set_active_plugins(lo_game_handle gh, char ** plugins, const size_t numPlugins);

/* Activates or deactivates the given plugin depending on the value of the active argument. */
LIBLO unsigned int lo_set_plugin_active(lo_game_handle gh, const char * plugin, const bool active);

/* Checks to see if the given plugin is active. */
LIBLO unsigned int lo_get_plugin_active(lo_game_handle gh, const char * plugin, bool * result);

#ifdef __cplusplus
}
#endif

#endif

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
    along with libloadorder. If not, see <http://www.gnu.org/licenses/>.
*/

/**
    @file libloadorder.h
    @brief This file contains the API frontend.

    @note libloadorder is *not* thread safe. Thread safety is a goal, but one that has not yet been achieved. Bear this in mind if using it in a multi-threaded client.

    @section var_sec Variable Types

    libloadorder uses character strings and integers for information input/output.
      - All strings are null-terminated byte character strings encoded in UTF-8.
      - All return, game and load order method codes are unsigned integers at least 16 bits in size.
      - All array sizes are unsigned integers at least 16 bits in size.
      - File paths are case-sensitive if and only if the underlying file system is case-sensitive.

    @section memory_sec Memory Management

    libloadorder manages the memory of strings and arrays it returns internally, so such strings and arrays should not be deallocated by the client.

    Data returned by a function lasts until a function is called which returns data of the same type (eg. a string is stored until the client calls another function which returns a string, an integer array lasts until another integer array is returned, etc.).

    All allocated memory is freed when lo_destroy_handle() is called, except the string allocated by lo_get_error_message(), which must be freed by calling lo_cleanup().

    @section valid_lo_sec Valid Load Orders

    Any load order that is set using libloadorder must be valid, ie. it must meet all the following conditions:
      - Contains only installed plugins.
      - Contains no duplicate entries.
      - The first plugin in the load order must be the game's main master file.
      - Loads all master files before all plugin files. Master bit flag value, rather than file extension, is checked.


    @section valid_apl_sec Valid Active Plugin Lists

    Any active plugin list that is set using libloadorder must be valid, ie. it must meet all the following conditions:
      - Contains only installed plugins.
      - Contains no duplicate entries.
      - Contains no more than 255 plugins.
      - If a Skyrim load order, contains `Skyrim.esm`.
      - If a Skyrim load order and `Update.esm` is installed, contains `Update.esm`.
*/

#ifndef __LIBLO_H__
#define __LIBLO_H__

#include <stddef.h>

#if defined(_MSC_VER)
//MSVC doesn't support C99, so do the stdbool.h definitions ourselves.
//START OF stdbool.h DEFINITIONS.
#   ifndef __cplusplus
#       define bool  _Bool
#       define true  1
#       define false 0
#   endif
#   define __bool_true_false_are_defined 1
//END OF stdbool.h DEFINITIONS.
#else
#   include <stdbool.h>
#endif

// set up dll import/export decorators
// when compiling the dll on windows, ensure LIBLO_EXPORT is defined.  clients
// that use this header do not need to define anything to import the symbols
// properly.
#if defined(_WIN32) || defined(_WIN64)
#   ifdef LIBLO_STATIC
#       define LIBLO
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

/**
    @brief A structure that holds all game-specific data used by libloadorder.
    @details Used to keep each game's data independent. Abstracts the definition of libloadorder's internal state while still providing type safety across the library. Multiple handles can also be made for each game, though it should be kept in mind that libloadorder is not thread-safe.
*/
typedef struct _lo_game_handle_int * lo_game_handle;


/*********************//**
    @name Return Codes
    @brief Error codes signify an issue that caused a function to exit prematurely, while warning codes signify a problem that did not stop the function from completing. If a function exits prematurely, a reversal of any changes made during its execution is attempted before it exits.
*************************/
///@{

LIBLO extern const unsigned int LIBLO_OK;  ///< The function completed successfully.
LIBLO extern const unsigned int LIBLO_WARN_BAD_FILENAME;  ///< A plugin filename contains characters that do not have Windows-1252 code points. The plugin cannot be activated.
/**
    @brief There is a mismatch between the files used to keep track of load order.
    @details This error can only occur when using libloadorder with a game that uses the textfile-based load order system. The load order in the active plugins list file (`plugins.txt`) does not match the load order in the full load order file (`loadorder.txt`). Synchronisation between the two is automatic when load order is managed through libloadorder. It is left to the client to decide how best to restore synchronisation.
*/
LIBLO extern const unsigned int LIBLO_WARN_LO_MISMATCH;
LIBLO extern const unsigned int LIBLO_ERROR_FILE_READ_FAIL;  ///< A file could not be read.
LIBLO extern const unsigned int LIBLO_ERROR_FILE_WRITE_FAIL;  ///< A file could not be written to.
LIBLO extern const unsigned int LIBLO_ERROR_FILE_RENAME_FAIL;  ///< A file could not be renamed.
LIBLO extern const unsigned int LIBLO_ERROR_FILE_PARSE_FAIL;  ///< There was an error parsing the file.
LIBLO extern const unsigned int LIBLO_ERROR_FILE_NOT_UTF8;  ///< The specified file is not encoded in UTF-8.
LIBLO extern const unsigned int LIBLO_ERROR_FILE_NOT_FOUND;  ///< The specified file could not be found.
LIBLO extern const unsigned int LIBLO_ERROR_TIMESTAMP_READ_FAIL;  ///< The modification date of a file could not be read.
LIBLO extern const unsigned int LIBLO_ERROR_TIMESTAMP_WRITE_FAIL;  ///< The modification date of a file could not be set.
LIBLO extern const unsigned int LIBLO_ERROR_NO_MEM;  ///< The library was unable to allocate the required memory.
LIBLO extern const unsigned int LIBLO_ERROR_INVALID_ARGS;  ///< Invalid arguments were given for the function.

/**
    @brief Matches the value of the highest-numbered return code.
    @details Provided in case clients wish to incorporate additional return codes in their implementation and desire some method of avoiding value conflicts.
*/
LIBLO extern const unsigned int LIBLO_RETURN_MAX;

///@}

/********************************//**
    @name Load Order Method Codes
************************************/
///@{

LIBLO extern const unsigned int LIBLO_METHOD_TIMESTAMP;  ///< The game handle is using the timestamp-based load order system. Morrowind, Oblivion, Fallout 3 and Fallout: New Vegas all use this system, as does pre-v1.4.26 Skyrim.
LIBLO extern const unsigned int LIBLO_METHOD_TEXTFILE;   ///< The game handle is using the textfile-based load order system. Skyrim v1.4.26+ uses this system.

///@}

/*******************//**
    @name Game Codes
***********************/
///@{

LIBLO extern const unsigned int LIBLO_GAME_TES3;  ///< Game code for The Elder Scrolls III: Morrowind.
LIBLO extern const unsigned int LIBLO_GAME_TES4;  ///< Game code for The Elder Scrolls IV: Oblivion.
LIBLO extern const unsigned int LIBLO_GAME_TES5;  ///< Game code for The Elder Scrolls V: Skyrim.
LIBLO extern const unsigned int LIBLO_GAME_FO3;  ///< Game code for Fallout 3.
LIBLO extern const unsigned int LIBLO_GAME_FNV;  ///< Game code for Fallout: New Vegas.

///@}


/**************************//**
    @name Version Functions
******************************/
///@{

/**
    @brief Checks for library compatibility.
    @details Checks whether the loaded libloadorder is compatible with the given version of libloadorder, abstracting library stability policy away from clients. The version numbering used is major.minor.patch.
    @param versionMajor The major version number to check.
    @param versionMinor The minor version number to check.
    @param versionPatch The patch version number to check.
    @returns True if the library versions are compatible, false otherwise.
*/
LIBLO bool lo_is_compatible(const unsigned int versionMajor, const unsigned int versionMinor, const unsigned int versionPatch);

/**
    @brief Gets the library version.
    @details Outputs the major, minor and patch version numbers for the loaded libloadorder. The version numbering used is major.minor.patch.
    @param versionMajor A pointer to the major version number.
    @param versionMinor A pointer to the minor version number.
    @param versionPatch A pointer to the patch version number.
*/
LIBLO void lo_get_version(unsigned int * const versionMajor, unsigned int * const versionMinor, unsigned int * const versionPatch);

///@}

/*********************************//**
    @name Error Handling Functions
*************************************/
///@{

/**
   @brief Returns the message for the last error or warning encountered. @details Outputs a string giving the a message containing the details of the last error or warning encountered by a function. Each time this function is called, the memory for the previous message is freed, so only one error message is available at any one time.
   @param details A pointer to the error details string outputted by the function.
   @returns A return code.
*/
LIBLO unsigned int lo_get_error_message(const char ** const details);

/**
   @brief Frees the memory allocated to the last error details string.
*/
LIBLO void lo_cleanup();

///@}


/***************************************//**
    @name Lifecycle Management Functions
*******************************************/
///@{

/**
    @brief Initialise a new game handle.
    @details Creates a handle for a game, which is then used by all load order and active plugin functions. If the game uses the textfile-based load order system, this function also checks if the two load order files are in sync, provided they both exist.
    @param gh A pointer to the handle that is created by the function.
    @param gameId A game code specifying which game to create the handle for.
    @param gamePath The relative or absolute path to the game folder.
    @returns A return code.
*/
LIBLO unsigned int lo_create_handle(lo_game_handle * const gh, const unsigned int gameId, const char * const gamePath);

/**
    @brief Destroy an existing game handle.
    @details Destroys the given game handle, freeing up memory allocated during its use, excluding any memory allocated to error messages.
    @param gh The game handle to destroy.
*/
LIBLO void lo_destroy_handle(lo_game_handle gh);

/**
    @brief Changes a game handle's associated master file.
    @details Sets the master file for the given game handle to the given filename, for use with total conversions that replace the vanilla game master file.
    @param gh The game handle to be operated on.
    @param masterFile The filename of the replacement game master file.
    @returns A return code.
*/
LIBLO unsigned int lo_set_game_master(lo_game_handle gh, const char * const masterFile);

///@}


/*****************************//**
    @name Load Order Functions
*********************************/
///@{

/**
    @brief Get which method is used for the load order.
    @param gh The game handle the function operates on.
    @param method A pointer to the outputted code for the load order method being used.
    @returns A return code.
*/
LIBLO unsigned int lo_get_load_order_method(lo_game_handle gh, unsigned int * const method);

/**
    @brief Get the current load order.
    @details Gets the current load order for the given game. This load order may be invalid if an invalid load order was previously set or a valid load order invalidated outside of libloadorder.
    @param gh The game handle the function operates on.
    @param plugins A pointer to the outputted array of plugins in the load order. "NULL" if no plugins are in the current load order.
    @param numPlugins A pointer to the size of the outputted array of plugins. "0" if no plugins are in the current load order.
    @returns A return code.
*/
LIBLO unsigned int lo_get_load_order(lo_game_handle gh, char *** const plugins, size_t * const numPlugins);

/**
    @brief Set the load order.
    @details Sets the load order to the passed plugin array, then scans the plugins directory and inserts any plugins not included in the passed array. Plugin files are inserted at the end of the load order, and master files are inserted after the last master file in the load order. The order of plugin insertion is undefined besides the distinction made between master files and plugin files.
    @param gh The game handle the function operates on.
    @param The inputted array of plugins in their new load order. This load order must be valid.
    @param numPlugins The size of the inputted array.
    @returns A return code.
*/
LIBLO unsigned int lo_set_load_order(lo_game_handle gh, char ** const plugins, const size_t numPlugins);

/**
    @brief Get the load order position of a plugin.
    @details Load order positions are zero-based, so the first plugin in the load order has a position of "0", the next has a position of "1", and so on.
    @param gh The game handle the function operates on.
    @param plugin The filename of the plugin to get the load order position of.
    @param index A pointer to the outputted load order index of the given plugin.
    @returns A return code.
*/
LIBLO unsigned int lo_get_plugin_position(lo_game_handle gh, const char * const plugin, size_t * const index);

/**
    @brief Set the load order position of a plugin.
    @details Sets the load order position of a plugin, removing it from its current position if it has one. If the supplied position is greater than the last position in the load order, the plugin will be positioned at the end of the load order. Load order positions are zero-based, so the first plugin in the load order has a position of "0", the next has a position of "1", and so on.
    @param gh The game handle the function operates on.
    @param plugin The filename of the plugin to set the load order position of.
    @param index The load order position to be set for the the given plugin.
    @returns A return code.
*/
LIBLO unsigned int lo_set_plugin_position(lo_game_handle gh, const char * const plugin, size_t index);

/**
    @brief Get filename of the plugin at a specific load order position.
    @details Load order positions are zero-based, so the first plugin in the load order has a position of "0", the next has a position of "1", and so on.
    @param gh The game handle the function operates on.
    @param index The load order position to check.
    @param plugin The filename of the plugin at the given load order position.
    @returns A return code.
*/
LIBLO unsigned int lo_get_indexed_plugin(lo_game_handle gh, const size_t index, char ** const plugin);

///@}

/***************************************//**
    @name Plugin Active Status Functions
*******************************************/
///@{

/**
    @brief Gets the list of currently active plugins.
    @details Outputs an unordered list of the plugins that are currently active.  This list may be invalid if an invalid active plugins list was previously set or a valid active plugins list invalidated outside of libloadorder.
    @param gh The game handle the function operates on.
    @param plugins A pointer to the outputted array of active plugins. "NULL" if no plugins are active.
    @param numPlugins A pointer to the size of the outputted array. "0" if no plugins are active.
    @returns A return code.
*/
LIBLO unsigned int lo_get_active_plugins(lo_game_handle gh, char *** const plugins, size_t * const numPlugins);

/**
    @brief Sets the list of currently active plugins.
    @details Replaces the current active plugins list with the plugins in the given array. The replacement list must be valid.
    @param gh The game handle the function operates on.
    @param plugins The inputted array of plugins to be made active.
    @param numPlugins The size of the inputted array.
    @returns A return code.
*/
LIBLO unsigned int lo_set_active_plugins(lo_game_handle gh, char ** const plugins, const size_t numPlugins);

/**
    @brief Activates or deactivates a given plugin.
    @details When activating a plugin that is ghosted, the ".ghost" extension is removed. If a plugin is already in its target state, ie. a plugin to be activated is already activate, or a plugin to be deactivated is already inactive, no changes are made.
    @param gh The game handle the function operates on.
    @param plugin The plugin to be activated or deactivated.
    @param active If \active is true, the given plugin is activated. If \active is false, the given plugin is deactivated.
    @returns A return code.
*/
LIBLO unsigned int lo_set_plugin_active(lo_game_handle gh, const char * const plugin, const bool active);

/**
    @brief Checks if a given plugin is active.
    @param gh The game handle the function operates on.
    @param plugin The plugin to check the active status of.
    @param result The outputted plugin status, "true" is the plugin is active, "false" otherwise.
    @returns A return code.
*/
LIBLO unsigned int lo_get_plugin_active(lo_game_handle gh, const char * const plugin, bool * const result);

///@}

/***********************//**
    @name Misc Functions
***************************/
///@{

/**
    @brief Fix up the text file(s) used by the load order and active plugins systems.
    @details Removes any plugins that are not present in the filesystem from plugins.txt (and loadorder.txt if used). This can be useful for when plugins are uninstalled manually or by a utility that does not also update the load order / active plugins systems.
    @param gh The game handle the function operates on.
    @returns A return code.
*/
LIBLO unsigned int lo_fix_plugin_lists(lo_game_handle gh);

///@}

#ifdef __cplusplus
}
#endif

#endif

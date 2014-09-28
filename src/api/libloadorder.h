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
 *  @file libloadorder.h
 *  @brief This file contains misc. API frontend functions.
 */

#ifndef __LIBLO_H__
#define __LIBLO_H__

#include "loadorder.h"
#include "activeplugins.h"
#include "constants.h"

#ifdef __cplusplus
extern "C"
{
#endif

    /**************************//**
     *  @name Version Functions
     *****************************/
    /**@{*/

    /**
     *  @brief Checks for library compatibility.
     *  @details Checks whether the loaded libloadorder is compatible with the
     *           given version of libloadorder, abstracting library stability
     *           policy away from clients. The version numbering used is
     *           major.minor.patch.
     *  @param versionMajor
     *      The major version number to check.
     *  @param versionMinor
     *      The minor version number to check.
     *  @param versionPatch
     *      The patch version number to check.
     *  @returns True if the library versions are compatible, false otherwise.
     */
    LIBLO bool lo_is_compatible(const unsigned int versionMajor,
                                const unsigned int versionMinor,
                                const unsigned int versionPatch);

    /**
     *  @brief Gets the library version.
     *  @details Outputs the major, minor and patch version numbers for the
     *           loaded libloadorder. The version numbering used is
     *           major.minor.patch.
     *  @param versionMajor
     *      A pointer to the major version number.
     *  @param versionMinor
     *      A pointer to the minor version number.
     *  @param versionPatch
     *      A pointer to the patch version number.
     */
    LIBLO void lo_get_version(unsigned int * const versionMajor,
                              unsigned int * const versionMinor,
                              unsigned int * const versionPatch);

    /**@}*/
    /*********************************//**
     *  @name Error Handling Functions
     ************************************/
    /**@{*/

    /**
     *  @brief Returns the message for the last error or warning encountered.
     *  @details Outputs a string giving the a message containing the details
     *           of the last error or warning encountered by a function. Each
     *           time this function is called, the memory for the previous
     *           message is freed, so only one error message is available at
     *           any one time.
     *  @param details
     *      A pointer to the error details string outputted by the function.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_get_error_message(const char ** const details);

    /**
     *  @brief Frees the memory allocated to the last error details string.
     */
    LIBLO void lo_cleanup();

    /**@}*/
    /***************************************//**
     *  @name Lifecycle Management Functions
     ******************************************/
    /**@{*/

    /**
     *  @brief Initialise a new game handle.
     *  @details Creates a handle for a game, which is then used by all load
     *           order and active plugin functions. If the game uses the
     *           textfile-based load order system, this function also checks
     *           if the two load order files are in sync, provided they both
     *           exist.
     *
     *           The game's local application data folder is the one that
     *           contains its `plugins.txt`, found within `%LOCALAPPDATA%` on
     *           Windows. If running libloadorder on Windows, it is not
     *           necessary to run this function, as libloadorder looks up the
     *           location itself. However, on other operating systems, lookup
     *           is not possible, and this function must be used to provide
     *           the necessary path.
     *  @param gh
     *      A pointer to the handle that is created by the function.
     *  @param gameId
     *      A game code specifying which game to create the handle for.
     *  @param gamePath
     *      The relative or absolute path to the game folder.
     *  @param localPath
     *      The path to the game's local application data folder, or `NULL`.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_create_handle(lo_game_handle * const gh,
                                        const unsigned int gameId,
                                        const char * const gamePath,
                                        const char * const localPath);

    /**
     *  @brief Destroy an existing game handle.
     *  @details Destroys the given game handle, freeing up memory allocated
     *           during its use, excluding any memory allocated to error
     *           messages.
     *  @param gh The game handle to destroy.
     */
    LIBLO void lo_destroy_handle(lo_game_handle gh);

    /**
     *  @brief Changes a game handle's associated master file.
     *  @details Sets the master file for the given game handle to the given
     *           filename, for use with total conversions that replace the
     *           vanilla game master file.
     *  @param gh
     *      The game handle to be operated on.
     *  @param masterFile
     *      The filename of the replacement game master file.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_set_game_master(lo_game_handle gh,
                                          const char * const masterFile);

    /**@}*/
    /***********************//**
     *  @name Misc Functions
     **************************/
    /**@{*/

    /**
     *  @brief Fix up the text file(s) used by the load order and active
     *         plugins systems.
     *  @details Removes any plugins that are not present in the filesystem
     *           from plugins.txt (and loadorder.txt if used). This can be
     *           useful for when plugins are uninstalled manually or by a
     *           utility that does not also update the load order / active
     *           plugins systems.
     *  @param gh
     *      The game handle the function operates on.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_fix_plugin_lists(lo_game_handle gh);

    /**@}*/

#ifdef __cplusplus
}
#endif

#endif

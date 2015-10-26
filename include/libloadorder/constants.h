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

/**
 *  @file constants.h
 *  @brief This file contains the API constants and type definitions.
 */

#ifndef __LIBLO_CONSTANTS__
#define __LIBLO_CONSTANTS__

#include <stddef.h>
#if defined(_MSC_VER) && _MSC_VER < 1800
/* MSVC doesn't support C99, so do the stdbool.h definitions ourselves. */
/* START OF stdbool.h DEFINITIONS. */
#   ifndef __cplusplus
#       define bool  _Bool
#       define true  1
#       define false 0
#   endif
#   define __bool_true_false_are_defined 1
/* END OF stdbool.h DEFINITIONS. */
#else
#   include <stdbool.h>
#endif

/* set up dll import/export decorators
   when compiling the dll on windows, ensure LIBLO_EXPORT is defined.  clients
   that use this header do not need to define anything to import the symbols
   properly. */
#ifdef _WIN32
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
     *  @brief A structure that holds all game-specific data used by
     *         libloadorder.
     *  @details Used to keep each game's data independent. Abstracts the
     *           definition of libloadorder's internal state while still
     *           providing type safety across the library. Multiple handles
     *           can also be made for each game, though it should be kept in
     *           mind that libloadorder is not thread-safe.
     */
    typedef struct _lo_game_handle_int * lo_game_handle;

    /*********************//**
     *  @name Return Codes
     *  @brief Error codes signify an issue that caused a function to exit
     *         prematurely, while warning codes signify a problem that did not
     *         stop the function from completing. If a function exits
     *         prematurely, a reversal of any changes made during its execution
     *         is attempted before it exits.
     ************************/
    /**@{*/

    LIBLO extern const unsigned int LIBLO_OK;  /**< The function completed successfully. */
    LIBLO extern const unsigned int LIBLO_WARN_BAD_FILENAME;  /**< A plugin filename contains characters that do not have Windows-1252 code points. The plugin cannot be activated. */
    /**
     *  @brief There is a mismatch between the files used to keep track of
     *         load order.
     *  @details This warning can only occur when using libloadorder with a
     *           game that uses the textfile-based load order system. The
     *           load order in the active plugins list file (`plugins.txt`)
     *           does not match the load order in the full load order file
     *           (`loadorder.txt`). Synchronisation between the two is
     *           automatic when load order is managed through libloadorder.
     *           It is left to the client to decide how best to restore
     *           synchronisation.
     */
    LIBLO extern const unsigned int LIBLO_WARN_LO_MISMATCH;
    /**
     * @brief The current load order and/or active plugins list is invalid.
     * @details This warning occurs when libloadorder reads a load order
     *          or active plugins list that does not conform to its validity
     *          requirements (see \ref valid_lo_sec and \ref valid_apl_sec
     *          respectively).
     */
    LIBLO extern const unsigned int LIBLO_WARN_INVALID_LIST;
    LIBLO extern const unsigned int LIBLO_ERROR_FILE_READ_FAIL;  /**< A file could not be read. */
    LIBLO extern const unsigned int LIBLO_ERROR_FILE_WRITE_FAIL;  /**< A file could not be written to. */
    LIBLO extern const unsigned int LIBLO_ERROR_FILE_NOT_UTF8;  /**< The specified file is not encoded in UTF-8. */
    LIBLO extern const unsigned int LIBLO_ERROR_FILE_NOT_FOUND;  /**< The specified file could not be found. */
    LIBLO extern const unsigned int LIBLO_ERROR_FILE_RENAME_FAIL;  /**< A file could not be renamed. */
    LIBLO extern const unsigned int LIBLO_ERROR_TIMESTAMP_READ_FAIL;  /**< The modification date of a file could not be read. */
    LIBLO extern const unsigned int LIBLO_ERROR_TIMESTAMP_WRITE_FAIL;  /**< The modification date of a file could not be set. */
    LIBLO extern const unsigned int LIBLO_ERROR_FILE_PARSE_FAIL;  /**< There was an error parsing the file. */
    LIBLO extern const unsigned int LIBLO_ERROR_NO_MEM;  /**< The library was unable to allocate the required memory. */
    LIBLO extern const unsigned int LIBLO_ERROR_INVALID_ARGS;  /**< Invalid arguments were given for the function. */

    /**
     *  @brief Matches the value of the highest-numbered return code.
     *  @details Provided in case clients wish to incorporate additional
     *           return codes in their implementation and desire some
     *           method of avoiding value conflicts.
     */
    LIBLO extern const unsigned int LIBLO_RETURN_MAX;

    /**@}*/
    /********************************//**
     *  @name Load Order Method Codes
     ***********************************/
    /**@{*/

    LIBLO extern const unsigned int LIBLO_METHOD_TIMESTAMP;  /**< The game handle is using the timestamp-based load order system. Morrowind, Oblivion, Fallout 3 and Fallout: New Vegas all use this system, as does pre-v1.4.26 Skyrim. */
    LIBLO extern const unsigned int LIBLO_METHOD_TEXTFILE;   /**< The game handle is using the textfile-based load order system. Skyrim v1.4.26+ uses this system. */

    /**@}*/
    /*******************//**
     *  @name Game Codes
     **********************/
    /**@{*/

    LIBLO extern const unsigned int LIBLO_GAME_TES3;  /**< Game code for The Elder Scrolls III: Morrowind. */
    LIBLO extern const unsigned int LIBLO_GAME_TES4;  /**< Game code for The Elder Scrolls IV: Oblivion. */
    LIBLO extern const unsigned int LIBLO_GAME_TES5;  /**< Game code for The Elder Scrolls V: Skyrim. */
    LIBLO extern const unsigned int LIBLO_GAME_FO3;  /**< Game code for Fallout 3. */
    LIBLO extern const unsigned int LIBLO_GAME_FNV;  /**< Game code for Fallout: New Vegas. */

    /**@}*/

#ifdef __cplusplus
}
#endif

#endif

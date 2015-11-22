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
 *  @file activeplugins.h
 *  @brief This file contains the API frontend for active plugin management.
 *
 *  @section valid_apl_sec Valid Active Plugin Lists
 *
 *  Any active plugin list that is set using libloadorder must be valid,
 *  ie. it must meet all the following conditions:
 *  - Contains only installed plugins.
 *  - Contains no duplicate entries.
 *  - Contains no more than 255 plugins.
 *  - If a Skyrim or Fallout 4 load order, contains `Skyrim.esm` or
 *    `Fallout4.esm` respectively.
 *  - If a Skyrim load order and `Update.esm` is installed, contains
 *    `Update.esm`.
 *
 *  Libloadorder is less strict when loading active plugin lists. If loading
 *  a Skyrim or Fallout 4 list and the relevant main master file is missing, it
 *  will be inferred to load first.
 *  Similarly, if Update.esm is installed but not in the active list, it will
 *  be inferred to load after all other master files.
 */

#ifndef __LIBLO_ACTIVE_PLUGINS__
#define __LIBLO_ACTIVE_PLUGINS__

#include "constants.h"

#ifdef __cplusplus
extern "C"
{
#endif

    /***************************************//**
     *  @name Plugin Active Status Functions
     ******************************************/
    /**@{*/

    /**
     *  @brief Gets the list of currently active plugins.
     *  @details Outputs an unordered list of the plugins that are currently
     *           active.
     *  @param gh
     *      The game handle the function operates on.
     *  @param plugins
     *      A pointer to the outputted array of active plugins. `NULL` if no
     *      plugins are active.
     *  @param numPlugins
     *      A pointer to the size of the outputted array. "0" if no plugins are
     *      active.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_get_active_plugins(lo_game_handle gh,
                                             char *** const plugins,
                                             size_t * const numPlugins);

    /**
     *  @brief Sets the list of currently active plugins.
     *  @details Replaces the current active plugins list with the plugins in
     *           the given array. The replacement list must be valid. If, for
     *           Skyrim or Fallout 4, a plugin to be activated does not have a
     *           defined load order position, this function will append it to
     *           the load order. If multiple such plugins exist, the order in
     *           which they are appended is undefined.
     *  @param gh
     *      The game handle the function operates on.
     *  @param plugins
     *      The inputted array of plugins to be made active.
     *  @param numPlugins
     *      The size of the inputted array.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_set_active_plugins(lo_game_handle gh,
                                             const char * const * const plugins,
                                             const size_t numPlugins);

    /**
     *  @brief Activates or deactivates a given plugin.
     *  @details When activating a plugin that is ghosted, the ".ghost"
     *           extension is removed. If a plugin is already in its target
     *           state, ie. a plugin to be activated is already activate, or
     *           a plugin to be deactivated is already inactive, no changes
     *           are made.
     *  @param gh
     *      The game handle the function operates on.
     *  @param plugin
     *      The plugin to be activated or deactivated.
     *  @param active
     *      If true, the given plugin is activated. If false, the given plugin
     *      is deactivated.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_set_plugin_active(lo_game_handle gh,
                                            const char * const plugin,
                                            const bool active);

    /**
     *  @brief Checks if a given plugin is active.
     *  @param gh
     *      The game handle the function operates on.
     *  @param plugin
     *      The plugin to check the active status of.
     *  @param result
     *      The outputted plugin status, `true` is the plugin is active,
     *      `false` otherwise.
     *  @returns A return code.
     */
    LIBLO unsigned int lo_get_plugin_active(lo_game_handle gh,
                                            const char * const plugin,
                                            bool * const result);

    /**@}*/

#ifdef __cplusplus
}
#endif

#endif

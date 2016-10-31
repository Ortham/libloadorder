************
Introduction
************

libloadorder is a free software library that provides functions for manipulating plugin load order and active status. Its features are:

- C API.
- Can be built as x86 and x64 static and dynamic libraries.
- Supports TES III: Morrowind, TES IV: Oblivion, TES V: Skyrim, TES V: Skyrim Special Edition, Fallout 3, Fallout: New Vegas and Fallout 4.
- Provides a uniform interface to the supported games' differing load order systems.
- Supports the community standard textfile-based load order system for TES V: Skyrim.
- Supports ghosted plugins.
- Get/Set the active plugin list.
- Get/Set the full load order.
- Get/Set the load order position of an individual plugin.
- Get/Set the active status of an individual plugin.
- Uses load order and active plugin list caching to avoid unnecessary disk reads, increasing performance.
- Free and open source software licensed under the GNU General Public License v3.0.

libloadorder is designed to free modding utility developers from the task of implementing and maintaining their own code for the functionality it provides.

This documentation assumes a familiarity with the basics of load ordering. An introduction to the concepts involved may be found in :doc:`load_order_introduction`.

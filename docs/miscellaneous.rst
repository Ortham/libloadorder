*********************
Miscellaneous Details
*********************

Variable Types
==============

libloadorder uses character strings and integers for information input/output.

- All strings are null-terminated byte character strings encoded in UTF-8.
- All return, game and load order method codes are unsigned integers at least 16 bits in size.
- All array sizes are unsigned integers at least 16 bits in size.
- File paths are case-sensitive if and only if the underlying file system is case-sensitive.

Memory Management
=================

libloadorder manages the memory of strings and arrays it returns internally, so such strings and arrays should not be deallocated by the client.

Data returned by a function lasts until a function is called which returns data of the same type (eg. a string is stored until the client calls another function which returns a string, an integer array lasts until another integer array is returned, etc.).

All allocated memory is freed when :cpp:func:`lo_destroy_handle()` is called, except the string allocated by :cpp:func:`lo_get_error_message()`, which must be freed by calling :cpp:func:`lo_cleanup()`.

Thread Safety
=============

libloadorder is thread-safe and all data output is thread-local.

Reading and writing data for a single game handle is protected by mutual exclusion. Game handles operate independently, so using more than one game handle for a single game across multiple threads is not advised, as filesystem changes made when writing data are not atomic and data races may occur under such usage.

Plugin Validity
===============

Where libloadorder functions take one or more plugin filenames, it checks that these filenames correspond to valid plugins. libloadorder defines a valid plugin as one that:

- Ends with ``.esp``, ``.esm``, ``.esp.ghost`` or ``.esm.ghost``.
- Contains a header record with:

    - The correct type (``TES3`` for Morrowind, ``TES4`` otherwise).
    - A size that is not larger than the total file size.
    - Subrecords with sizes that do not together sum to larger than the expected total subrecords size.

This definition is substantially more permissive than games or other utilities may be for performance reasons, and because libloadorder uses no plugin data beyond the header record, so later corruption or invalid data would not affect its behaviour.

This permissivity does allow more strictly invalid plugins to be positioned in the load order and activated, which may cause game issues, but protecting against such circumstances is beyond the scope of libloadorder.

Data Caching
============

libloadorder caches plugin, load order and active status data to improve performance. Each game handle has its own unique cache, and change detection is performed whenever an API function that takes a game handle is called. If changes are detected, the necessary data are reloaded before the function operates on the data.

Change detection is carried out by timestamp comparison, checking the current timestamps against the timestamps for the cached data. The files and folders which libloadorder checks the timestamps of are:

- The folder into which plugins are installed.
- The installed plugins.
- The file that holds the active plugins data.
- The file that holds the load order data, if the game is one handled by the textfile-based load order system.

Edits made to a file will only be detected if they call that file's timestamp to change. If edits are made and the timestamp is unchanged, the changes can only be detected by destroying the existing game handle and creating a new game handle to use.

Valid Active Plugin Lists
=========================

Any active plugin list that is set using libloadorder must be valid,
ie. it must meet all the following conditions:

- Contains only installed plugins.
- Contains no duplicate entries.
- Contains no more than 255 plugins.
- If a Skyrim or Fallout 4 load order, contains ``Skyrim.esm`` or
  ``Fallout4.esm`` respectively.
- If a Skyrim load order and ``Update.esm`` is installed, contains
  ``Update.esm``.

Libloadorder is less strict when loading active plugin lists. If loading
a Skyrim or Fallout 4 list and the relevant main master file is missing, it
will be inferred to load first.

Similarly, if Update.esm is installed but not in the active list, it will
be inferred to load after all other master files.

Valid Load Orders
=================

Any load order that is set using libloadorder must be valid, ie. it must
meet all the following conditions:

- Contains only installed plugins.
- Contains no duplicate entries.
- Loads all master files before all plugin files. Master bit flag value,
  rather than file extension, is checked.
- For Skyrim or Fallout 4, the first plugin in the load order must be
  Skyrim.esm or Fallout4.esm respectively.

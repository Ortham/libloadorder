*************
API Reference
*************

.. contents::

Version Functions
=================

.. doxygenfunction:: lo_is_compatible

.. doxygenfunction:: lo_get_version

Error Handling
==============

.. doxygenfunction:: lo_get_error_message

.. doxygenfunction:: lo_cleanup

Lifecycle Management
====================

.. doxygentypedef:: lo_game_handle

.. doxygenfunction:: lo_create_handle

.. doxygenfunction:: lo_destroy_handle

.. doxygenfunction:: lo_set_game_master

Miscellaneous
=============

.. doxygenfunction:: lo_fix_plugin_lists

Load Order
==========

.. doxygenfunction:: lo_get_load_order_method

.. doxygenfunction:: lo_get_load_order

.. doxygenfunction:: lo_set_load_order

.. doxygenfunction:: lo_get_plugin_position

.. doxygenfunction:: lo_set_plugin_position

.. doxygenfunction:: lo_get_indexed_plugin

Plugin Active Status
====================

.. doxygenfunction:: lo_get_active_plugins

.. doxygenfunction:: lo_set_active_plugins

.. doxygenfunction:: lo_set_plugin_active

.. doxygenfunction:: lo_get_plugin_active

Return Codes
============

.. doxygenvariable:: LIBLO_OK
.. doxygenvariable:: LIBLO_WARN_BAD_FILENAME
.. doxygenvariable:: LIBLO_WARN_LO_MISMATCH
.. doxygenvariable:: LIBLO_WARN_INVALID_LIST
.. doxygenvariable:: LIBLO_ERROR_FILE_READ_FAIL
.. doxygenvariable:: LIBLO_ERROR_FILE_WRITE_FAIL
.. doxygenvariable:: LIBLO_ERROR_FILE_NOT_UTF8
.. doxygenvariable:: LIBLO_ERROR_FILE_NOT_FOUND
.. doxygenvariable:: LIBLO_ERROR_FILE_RENAME_FAIL
.. doxygenvariable:: LIBLO_ERROR_TIMESTAMP_READ_FAIL
.. doxygenvariable:: LIBLO_ERROR_TIMESTAMP_WRITE_FAIL
.. doxygenvariable:: LIBLO_ERROR_FILE_PARSE_FAIL
.. doxygenvariable:: LIBLO_ERROR_NO_MEM
.. doxygenvariable:: LIBLO_ERROR_INVALID_ARGS
.. doxygenvariable:: LIBLO_RETURN_MAX

Load Order Method Codes
=======================

.. doxygenvariable:: LIBLO_METHOD_TIMESTAMP
.. doxygenvariable:: LIBLO_METHOD_TEXTFILE
.. doxygenvariable:: LIBLO_METHOD_ASTERISK

Game Codes
==========

.. doxygenvariable:: LIBLO_GAME_TES3
.. doxygenvariable:: LIBLO_GAME_TES4
.. doxygenvariable:: LIBLO_GAME_TES5
.. doxygenvariable:: LIBLO_GAME_FO3
.. doxygenvariable:: LIBLO_GAME_FNV
.. doxygenvariable:: LIBLO_GAME_FO4
.. doxygenvariable:: LIBLO_GAME_TES5SE

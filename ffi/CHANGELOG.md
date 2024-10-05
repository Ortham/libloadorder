# Changelog

Version numbers are shared between libloadorder and libloadorder-ffi. This
changelog only contains libloadorder-ffi changes.

## [18.1.1] - 2024-10-05

### Changed

- Updated to libloadorder v18.1.1.

## [18.1.0] - 2024-10-05

### Changed

- Updated to libloadorder v18.1.0.
- Updated to libc v0.2.159.

## [18.0.0] - 2024-08-23

### Changed

- Updated to libloadorder v18.0.0.

## [17.1.0] - 2024-08-23

### Changed

- Updated to libloadorder v17.1.0.

### Removed

- `LIBLO_ERROR_FILE_NOT_UTF8` and `LIBLO_ERROR_TIMESTAMP_WRITE_FAIL` as they are
  unused.

## [17.0.1] - 2024-06-29

### Changed

- Updated to libloadorder v17.0.1.

## [17.0.0] - 2024-06-28

### Changed

- Updated to libloadorder v17.0.0.

## [16.0.0] - 2024-05-02

### Changed

- Updated to libloadorder v16.0.0.

### Removed

- The `ffi-headers` build feature: if you want to generate C or C++ headers,
  install and run cbindgen separately.

## [15.0.2] - 2023-11-25

### Changed

- Updated to libloadorder v15.0.2.

## [15.0.1] - 2023-10-06

### Changed

- Updated to libloadorder v15.0.1.

## [15.0.0] - 2023-09-28

### Added

- `lo_get_early_loading_plugins()`, which can be used to get the plugins
  previously gotten using `lo_get_implicitly_active_plugins()`, without any of
  the newly-supported implicitly active plugins from `.nam` files or ini file
  properties.
- `LIBLO_ERROR_SYSTEM_ERROR` to represent unknown OS errors.
- `LIBLO_GAME_STARFIELD` as the game code for Starfield.


### Changed

- Updated to libloadorder v15.0.0.
- Updated to cbindgen v0.26.

## [14.2.2] - 2023-09-16

### Changed

- Updated to libloadorder v14.2.2.

## [14.2.1] - 2023-08-22

### Changed

- Updated to libloadorder v14.2.1.

## [14.2.0] - 2023-08-20

### Changed

- `lo_create_handle()` will no longer fail if passed a local data path that
  does not exist.
- `lo_create_handle()` will no longer fail on Linux if passed a null local data
  path and `LIBLO_GAME_TES3` as the game ID. Passing any other game ID with a
  null local data path will still fail.
- Updated to libloadorder v14.2.0.

## [14.1.0] - 2023-04-26

### Added

- `lo_set_additional_plugins_directories()` for providing the paths to any
  directories other than the game's plugins directory that contain plugins that
  should be considered part of the load order. This is intended to support the
  Microsoft Store's Fallout 4 DLCs, which are installed outside of the base game's
  install path.

### Changed

- Updated to libloadorder v14.1.0.

## [14.0.0] - 2023-03-18

### Changed

- Updated to libloadorder v14.0.0.

## [13.3.0] - 2022-10-11

### Changed

- Updated to libloadorder v13.3.0.

## [13.2.0] - 2022-10-01

### Added

- `lo_get_active_plugins_file_path()` for getting the active plugins file path
  that libloadorder uses for a given game handle.

### Changed

- Updated to Rust's 2018 edition.
- Updated to libloadorder v13.2.0.

### Fixed

- `lo_get_implicitly_active_plugins()` did not check if its `plugins` or
  `num_plugins` argument values were null.

## [13.1.1] - 2022-09-15

### Changed

- Updated to cbindgen v0.24.

## [13.1.0] - 2022-02-23

### Added

- `lo_is_ambiguous()` for checking if all currently-loaded plugins have a well
  defined load order position and that all data sources are consistent.

## [13.0.0] - 2021-04-17

### Changed

- cbindgen now generates a single `libloadorder.h` header file that can be used
  by C and C++ callers.
- `GameHandle`'s type has changed as a wrapper type is no longer required.
  This should have no effect on C/C++ callers as the type remains opaque to
  them.
- Some error messages have changed.
- Updated to libloadorder v13.0.0.
- Updated to cbindgen v0.19.

### Fixed

- Bare trait object deprecation warnings.

### Removed

- cbindgen no longer generates a `libloadorder.hpp` header, C++ callers should
  include `libloadorder.h` instead.

## [12.0.0] - 2018-10-29

### Changed

- Updated to libloadorder v12.0.0.

## [11.4.1] - 2018-09-10

### Changed

- Updated to libloadorder v11.4.1.

## [11.4.0] - 2018-06-24

### Changed

- If loading or saving the load order during `lo_set_active_plugins()`,
  `lo_set_plugin_active()`, `lo_load_current_state()`, `lo_fix_plugin_lists()`,
  `lo_set_load_order()` or `lo_set_plugin_position()` fails, the current state
  will no longer be cleared. This is now consistent with other failure state
  handling and simplifies retries.
- Updated to libloadorder v11.4.0.

## [11.3.0] - 2018-06-02

### Added

- `lo_get_implicitly_active_plugins()` for getting the current game handle's
  implicitly active plugins in their hardcoded load order.

### Changed

- Updated to libloadorder v11.2.3.

## [11.2.2] - 2018-05-26

### Changed

- Updated to libloadorder v11.2.2.

## [11.2.1] - 2018-04-27

### Changed

- Updated to libloadorder v11.2.1.
- Updated to cbindgen v0.6.

## [11.2.0] - 2018-04-08

### Added

- Support for Skyrim VR using `LIBLO_GAME_SKYRIMVR`.

### Changed

- Updated to libloadorder v11.2.0.

## [11.1.0] - 2018-04-02

### Changed

- Updated to libloadorder v11.1.0.

## [11.0.2] - 2018-03-29

### Changed

- Updated to libloadorder v11.0.2.

## [11.0.1] - 2018-02-17

### Changed

- Updated to libloadorder v11.0.1.

## [11.0.0] - 2018-02-16

### Changed

- Updated to libloadorder v11.0.0.
- Updated documentation, fixing several inaccuracies.

## [10.1.1] - 2018-02-14

### Changed

- Updated to libloadorder v10.1.1.

## [10.1.0] - 2018-02-04

### Added

- The `LIBLO_GAME_FO4VR` game code for Fallout 4 VR support.

### Changed

- Updated to libloadorder v10.1.0.
- Updated to cbindgen v0.4.3.

## [10.0.4] - 2017-11-21

### Added

- The `LIBLO_ERROR_PANICKED` return code for indicating that a panic was caught.

### Changed

- Updated to libloadorder v10.0.4.
- Unwinding panics are now caught at the FFI boundary.

## [10.0.3] - 2017-10-31

### Changed

- Updated to libloadorder v10.0.3.

## [10.0.2] - 2017-10-27

### Changed

- Updated to libloadorder v10.0.2.

## [10.0.1] - 2017-10-27

### Changed

- Updated to libloadorder v10.0.1.

## [10.0.0] - 2017-10-14

Initial release of libloadorder-ffi. The changes listed below are relative to
the previous libloadorder C API.

### Added

- `lo_load_current_state()` must be used to load load order state before
  operating on it for the first time, and reload it whenever there are external
  changes made to the load order.
- `lo_free_string()` must be used to free the memory allocated by any API
  function that outputs a C string, excluding `lo_get_error_message()`.
- `lo_free_string_array()` must be used to free the memory allocated by any API
  function that outputs an array of C strings.
- New C API error codes:

  - `LIBLO_ERROR_POISONED_THREAD_LOCK`
  - `LIBLO_ERROR_IO_ERROR`
  - `LIBLO_ERROR_IO_PERMISSION_DENIED`
  - `LIBLO_ERROR_TEXT_ENCODE_FAIL`
  - `LIBLO_ERROR_TEXT_DECODE_FAIL`
  - `LIBLO_ERROR_INTERNAL_LOGIC_ERROR`

### Changed

- libloadorder has been rewritten in Rust. The library has been split into two
  crates: `libloadorder`, which contains the Rust implementation, and
  `libloadorder-ffi`, which contains the FFI wrapper.
- Memory allocation failure now causes a panic instead of returning an error
  code.
- C strings and arrays of C strings output by API game handle functions now have
  indefinite lifetime and must be explicitly freed using the provided API
  functions. Previous behaviour was that such output would be stored in
  thread-local storage until the next time the same data type was outputted, or
  until the associated game handle was destroyed.
- The API functions no longer manage the library's load order cache, instead
  exposing control to the client via `lo_load_current_state()`.
- The C string output of `lo_get_error_message()` no longer needs to be
  explicitly freed.
- The C/C++ headers have been replaced with one C header (`libloadorder.h`) and
  one C++ header (`libloadorder.hpp`).
- Many API function parameters have lost `const` qualifiers due to the C/C++
  headers being autogenerated and changes to string ownership.
- The library binary name has changed. Omitting
  platform-specific prefixes and suffixes, it is now `loadorder_ffi`.
- The documentation has been converted to Markdown and split up: the API
  reference documentation is stored with the code and generated by rustdoc, and
  the general load order documentation is now stored in `/doc` and generated
  by [mdBook](https://azerupi.github.io/mdBook/).

### Removed

- `lo_set_game_master()` was removed as it had no effect.
- `lo_is_compatible()` was removed as it was unnecessary as the library uses
  semantic versioning.
- `lo_cleanup()` was removed as it was made obsolete.
- Some error codes were removed or replaced:

  - `LIBLO_WARN_BAD_FILENAME` is now indicated by `LIBLO_ERROR_TEXT_ENCODE_FAIL`
  - `LIBLO_ERROR_FILE_READ_FAIL` is now indicated by `LIBLO_ERROR_IO_ERROR`
  - `LIBLO_ERROR_FILE_WRITE_FAIL` is now indicated by `LIBLO_ERROR_IO_ERROR`,
    `LIBLO_ERROR_IO_PERMISSION_DENIED` and `LIBLO_ERROR_TEXT_DECODE_FAIL`
  - `LIBLO_ERROR_TIMESTAMP_READ_FAIL` is now indicated by `LIBLO_ERROR_IO_ERROR`
  - `LIBLO_ERROR_NO_MEM` is obsolete as the Rust implementation panics on
    memory allocation failure
  - `LIBLO_WARN_INVALID_LIST` was unused for a few major versions

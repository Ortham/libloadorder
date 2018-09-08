# Changelog

Version numbers are shared between libloadorder and libloadorder-ffi. This
changelog does not include libloadorder-ffi changes.

## [?] - 2018-??-??

### Added

- Fallout 4's Ultra High Resolution DLC plugin is now recognised as always
  being active if installed.

## [11.4.0] - 2018-06-24

### Changed

- `WritableLoadOrder::set_load_order()` no longer errors if given a load order
  that doesn't include all installed plugins, as libloadorder might wrongly
  detect invalid plugins as valid when expecting them to be present. Instead,
  the load order is set as given, so missing plugins' load order positions are
  left undefined.
- Updated esplugin dependency to 2.0.0.

### Removed

- `WritableLoadOrder` no longer has the private trait `MutableLoadOrder` as a
  supertrait. Instead, it has `ReadableLoadOrder` as a supertrait (which was
  previously a supertrait of `MutableLoadOrder`).
- `ReadableLoadOrder::plugins()`, as it returned the private `Plugin` type and
  should not have been exposed.

## [11.2.3] - 2018-06-02

### Changed

- Updated message for `Error::GameMasterMustLoadFirst` to more accurately
  reflect that it is now used for incorrectly positioned hardcoded plugins in
  general.
- Updated esplugin dependency to 1.0.10.

## [11.2.2] - 2018-05-26

### Changed

- Updated regex dependency.

### Fixed

- Check that hardcoded plugins load in their correct positions in the load order
  passed to `WritableLoadOrder::set_load_order()`.

## [11.2.1] - 2018-04-27

### Changed

- Switched from tempfile to tempdir for creating temporary directories for
  tests.

### Fixed

- `WritableLoadOrder::load()` would incorrectly move light-master-flagged
  plugins with the `.esp` file extension to load before non-masters. It now
  allows such plugins to load in amongst non-masters.

## [11.2.0] - 2018-04-08

### Added

- Support for Skyrim VR using `GameId::SkyrimVR`.

## [11.1.0] - 2018-04-02

### Changed

- `WritableLoadOrder::load()` now falls back to reading `loadorder.txt` as
  encoded in Windows-1252 if it is not encoded in valid UTF-8, for compatibility
  with utilities that incorrectly encode it in Windows-1252.

## [11.0.2] - 2018-03-29

### Fixed

- Setting a load order for Morrowind, Oblivion, Fallout 3 and Fallout: New Vegas
  with plugin timestamps earlier than the Unix epoch would fail.

## [11.0.1] - 2018-02-17

### Fixed

- `WritableLoadOrder::set_load_order()` would error if given an unghosted plugin
  name for an installed plugin that was ghosted. It now treats unghosted and
  ghosted plugin names as equivalent, so they can be used interchangeably.

## [11.0.0] - 2018-02-16

### Changed

- `ReadableLoadOrder` methods now return strings as string slices, making the
  API more symmetrical and improving performance.
- `WritableLoadOrder::set_load_order()` now errors if passed a load order that
  does not include all installed plugins. Previously any missing plugins would
  be appended to the passed load order, which could cause unexpected results.
- Extended benchmarks to cover all `ReadableLoadOrder` and `WritableLoadOrder`
  methods, and significantly reduced their running time.
- Updated Rayon dependency to 1.0.0.

## [10.1.1] - 2018-02-14

### Added

- Benchmarks for some `ReadableLoadOrder` and `WritableLoadOrder` methods, built
  using [Criterion.rs](https://github.com/japaric/criterion.rs), and which can
  be run using `cargo bench`.

### Changed

- Various optimisations that have improved performance in general, with
  improvements of between 2x to 19x observed for the benchmarked functions.

### Fixed

- The plugins directory was being searched for plugins recursively, which was
  totally unnecessary and had a potentially huge performance impact (~ 500x for
  the user who reported the issue). Fixing this also removed the WalkDir
  dependency.
- `WritableLoadOrder::set_active_plugins()` was counting normal plugins and
  light masters according to their file extension, so it wouldn't count
  false-flagged plugins correctly when validating against the active plugin
  limits.
- Saving a timestamp-based load order would preserve the plugins' existing
  access times, for correctness the access times are now set to the current time
  when setting the modification time.

## [10.1.0] - 2018-02-04

### Added

- Support for Fallout 4 VR using `GameId::Fallout4VR`.

### Changed

- Updated esplugin, walkdir and rayon dependencies.

## [10.0.4] - 2017-11-21

### Changed

- Identify Creation Club plugins using `.ccc` files instead of hardcoding them.
- Updated to esplugin v1.0.7.

## [10.0.3] - 2017-10-31

### Fixed

- Panic that could occur when loading state and implicitly-active plugins were
  not loaded in the order they appear in the load order.

## [10.0.2] - 2017-10-27

### Fixed

- Ghosted plugins being written to `plugins.txt` with their `.ghost` file
  extension, when saving a Fallout 4 or Skyrim Special Edition load order. This
  was broken in v10.0.1.

## [10.0.1] - 2017-10-27

### Changed

- Improved performance of setting and saving load order.

## [10.0.0] - 2017-10-14

### Added

- Support for light master plugins, i.e. plugins with a `.esl` file extension,
  in Fallout 4 and Skyrim Special Edition.
- All released Creation Club plugins as of Skyrim SE v1.5.3.0.8 and Fallout 4
  v1.10.26.0.0 are recognised as always being active if installed.

### Changed

- libloadorder has been rewritten in Rust. The library has been split into two
  crates: `libloadorder`, which contains the Rust implementation, and
  `libloadorder-ffi`, which contains the FFI wrapper.
- Attempting to write a plugin filename that cannot be encoded in Windows-1252
  is now a hard error, instead of that filename getting skipped and a warning
  code being returned.
- The documentation has been converted to Markdown and split up: the API
  reference documentation is stored with the code and generated by rustdoc, and
  the general load order documentation is now stored in `/doc` and generated
  by [mdBook](https://azerupi.github.io/mdBook/).

### Removed

- Caching of plugin folder, active plugins file and load order file content, as
  profiling showed it was no longer effective.

### Fixed

- Attempting to deactivate an implicitly active plugin that is not installed now
  causes an error.
- Attempting to set the active plugins giving an array including an implicitly
  active plugin that is not installed now causes an error.

## [9.5.5] - 2017-07-15

### Changed

- Rewrite the documentation using Sphinx, Breathe and Doxygen to produce
  better-looking documentation.
- The documentation is no longer generated as a post-build step, to fix builds
  when the documentation dependencies are not installed.

## [9.5.4] - 2017-11-23

### Fixed

- A crash caused by missing implicitly-active plugins.
- Implicitly active plugins being positioned incorrectly if one or more were
  missing.

## [9.5.3] - 2016-11-12

### Fixed

- The positions of implicitly-active plugins in `plugins.txt` is now ignored.

## [9.5.2] - 2016-11-11

### Changed

- Implicitly-active plugins are no longer written to `plugins.txt` for
  Fallout 4 and Skyrim Special Edition.
- The order assumed for Skyrim SE's implicitly active plugins, to match that
  assumed by Nexus Mod Manager and other utilities.

## [9.5.1] - 2016-11-08

### Fixed

- Implicitly-active plugins not loading first for Fallout 4 and Skyrim Special
  Edition.

## [9.5.0] - 2016-10-29

### Added

- Support for Skyrim Special Edition.

### Fixed

- Implicitly-active plugins were not treated as such if the active plugins file
  was missing.

## [9.4.1] - 2016-08-13

### Added

- Support for building with Clang.

### Changed

- Updated libespm to 2.5.4.
- Decouple Boost linking and runtime linking, so that one can be linked
  statically while the other is linked dynamically.

## [9.4.0] - 2016-06-21

### Added

- Support for upcoming Fallout 4 DLC plugins to be recognised as always being
  active if installed.

### Changed

- Don't throw if deactivating an implicitly active plugin that is not installed.

## [9.3.0] - 2016-05-21

### Added

- Fallout 4's Far Harbor DLC plugin is now recognised as always being active if
  installed.

## [9.2.0] - 2016-04-29

### Added

- Fallout 4's Automatron and Wasteland Workshop DLC plugins are now recognised
  as always being active if installed.

### Changed

- Documentation is now automatically generated and packaged.

## [9.1.0] - 2016-04-14

### Changed

- Getting active plugins now returns them in load order.
- Setting active plugins now appends any plugins that weren't already present to
  the load order in the order they were given.
- PDF documentation is now generated as a post-build step.

## [9.0.0] - 2016-04-11

### Added

- The `LIBLO_METHOD_ASTERISK` constant to indicate the new load order method
  now used by Fallout 4.

### Changed

- Updated Fallout 4 support to use new load order method introduced in
  Fallout 4 v1.5.

## [8.0.1] - 2016-04-10

### Fixed

- Mismatched CMake variable names causing build failures.

## [8.0.0] - 2016-03-22

### Changed

- Removed the 32/64 suffix from library binary filename.

## [7.0.1] - 2016-03-21

### Changed

- Use CMake's `ExternalProject_Add` to resolve dependencies.

## [7.0.0] - 2015-12-07

### Added

- Support for Fallout 4.
- The API is now thread-safe. String output is allocated in thread-local storage
  and freed when the game handle is destroyed. Error messages are allocated in
  thread-local storage that exists for the lifetime of the library.

### Changed

- Moved headers to `include` subdirectory.
- Improved generation of 64-bit build project for MSVC.
- Updated to libespm v2.5.0.
- libloadorder now checks that a plugin exists before trying to read its header,
  improving error handling.
- Internal refactoring.
- `lo_fix_plugin_lists()` no longer removes invalid plugins.
- `lo_set_load_order()` no longer appends installed plugins that were not
  specified.
- Cached data is now reloaded only when changes are detected in the relevant
  paths, rather than reloading all data whenever any change is detected.

### Removed

- Boost.Iostreams and Boost.Regex dependencies.
- Support for MSVC earlier than 2015.

### Fixed

- Moving plugins to the end of the load order.
- Running `lo_fix_plugin_lists()` for Skyrim.
- Don't unghost plugin the plugin passed to `lo_set_plugin_active()` if it's not
  being activated.
- Unghost the plugins passed to `lo_set_active_plugins()`.
- Passing `lo_set_active_plugins()` one or more plugins that have no existing
  load order position.
- `lo_get_load_order()` now returns an error code if passed a zero-length plugin
  array.
- `lo_set_active_plugins()` and `lo_set_plugin_active()` now error if passed an
  invalid plugin.
- `lo_set_active_plugins()` had no effect if it detected the load order had
  changed and reloaded it.
- libloadorder now checks that plugin files are valid instead of just that they
  exist when performing load order operations.
- Only valid plugins are now read from the active plugins and load order files.
- `lo_set_load_order()` now retains existing plugin active states.
- Setting the load order for a Timestamp-based game now appends plugins that are
  installed but not given in the input array to the end of the load order.
- Change detection now detects when timestamps have been updated with older
  values.
- Change detection now stores individual timestamps for each path monitored, so
  a change in one path is no longer obscured if the others don't change.
- Don't load plugins that don't end in `.esp`, `.esm` or `.ghost`.
- A memory leak when reallocating a string array.
- Error message typo.
- Building the tests against a static library.
- Building without Google Test present.

## [6.0.3] - 2014-12-21

### Changed

- Removed unnecessary validity checks when overwriting existing state.
- Plugin data is cached during reading of the load order to improve performance.

### Fixed

- Reading plugins from the load order file and active plugins file when Windows
  line endings are present.
- The timestamp for the first plugin in the load order was never set for
  Timestamp-based games.
- Non-unique timestamps causing not all plugins to be redated when setting the
  load order for Timestamp-based games.

## [6.0.2] - 2014-12-14

### Fixed

- Crash due to libespm trying to parse plugins that are not installed.

## [6.0.1] - 2014-10-10

### Fixed

- Validity checks were checking only active plugins when they should have been
  checking the whole load order.

## [6.0.0] - 2014-10-06

### Changed

- `lo_get_version()` now returns an unsigned integer code as it can fail.
- `lo_fix_plugin_lists()` is now more thorough, also checking that:
  - The game main master file loads first
  - All active plugins are installed
  - Masters load before non-masters
  - No more than 255 plugins are active
  - `Update.esm` is active if installed (for Skyrim).
- `lo_create_handle()` now checks that the given paths are valid filesystem
  paths.
- `lo_set_game_master()` now checks that the given plugin is valid.
- Skyrim's game master can no longer be set, as it must be `Skyrim.esm`.
- If an invalid load order is loaded, the API functions return
  `LIBLO_WARN_INVALID_LIST` instead of `LIBLO_OK`.

### Added

- Google Test-based tests.
- Travis CI builds.

### Fixed

- `lo_create_handle()` nulls the file handle output parameter if it fails.
- Crash when a zero-length load order is set.
- Crashes due to uncaught exceptions.
- Plugin validity checks now try to parse the file header, rather than just
  checking the file extension, to catch files that have the correct extension
  but aren't actually plugin files.
- Too many active plugins being deactivated when more than the maximum number
  of plugins are active.

## [5.0.0] - 2014-09-28

### Changed

- `lo_create_handle()` now takes a new parameter to allow clients to set the
  game's local data path. If `NULL`, the Registry is used to look the path up.
- Made API function parameters more `const`.

### Fixed

- Getting a plugin's masters for validity checks would always return an empty
  vector.

## [4.0.1] - 2014-07-07

### Fixed

- API constants not being exposed.

## [4.0.0] - 2014-06-27

### Added

- Support for MSVC 2012.

### Changed

- Libespm parsing errors are now propagated through the API.

### Removed

- Support for Skyrim versions earlier than v1.4.16, as Steam's auto-update means
  the vast majority of people will be using a newer version, and the support
  was interfering with a fairly common SKSE usage pattern.
- UTF8-CPP dependency. Boost.Locale is used instead.
- zlib dependency, as libespm no longer requires it when not reading compressed
  records.

### Fixed

- XP support when building with MSVC 2013.
- DLL linking errors.

## [3.0.3] - 2014-03-23

### Changed

- When setting plugin timestamps, libloadorder now recycles existing values,
  adding new timestamps only when the existing timestamps are non-unique.

### Fixed

- Crash due to error message use-after-free.
- Crash due to libespm parsing failure.
- Compiler error when building with MSVC 2013.
- Unreliable load order change detection based on plugins folder timestamp,
  which is no longer used.
- Potential crash due to allocation failure when storing an error string.

## [3.0.2] - 2013-09-26

### Changed

- Libespm is now used to parse plugin headers.
- More details are now supplied in some error messages.

### Fixed

- Crash when the load order file, active plugins file or their parent paths
  are missing.
- Filter patches causing missing master errors.
- `lo_get_load_order()` no longer returns missing plugins.

## [3.0.1] - 2013-07-23

- First tagged release.

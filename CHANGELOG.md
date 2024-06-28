# Changelog

Version numbers are shared between libloadorder and libloadorder-ffi. This
changelog does not include libloadorder-ffi changes.

## [17.0.0] - 2024-06-28

### Added

- Starfield's `Starfield.ccc` file will now be read from the
  `My Games\Starfield` directory as well as its install directory, with the
  former taking precedence over the latter, to match the game's behaviour.
- Starfield plugins will now be loaded from the `My Games\Starfield\Data`
  directory as well as the game install path's `Data` directory, but only for
  plugins that are present in both directories. This matches the game's
  behaviour.
- Support for medium plugin (introduced by Starfield), which are now counted
  separately to full plugins when checking active plugin limits, as they have
  their own limit of 256 active medium plugins.

### Changed

- Master files are now hoisted to load before other master files that depend on
  them, to match the behaviour of all supported games.
- Starfield's `Starfield.esm`, `Constellation.esm` and `OldMars.esm` are no
  longer treated as hardcoded: instead, they are now treated as implicitly
  active, along with `BlueprintShips-Starfield.esm`, `SFBGS003.esm`,
  `SFBGS006.esm`, `SFBGS007.esm` and `SFBGS008.esm`.
- Plugins that have the update flag (introduced by Starfield) set are no longer
  given special treatment when checking active plugin limits, to match
  Starfield's current behaviour. Previously such plugins would not count towards
  the maximum number of plugins you could have active at the same time.
- `Error::TooManyActivePlugins` has gained a `medium_count` field, and its
  `normal_count` field has been renamed to `full_count` to match the terminology
  introduced by Starfield.
- Updated esplugin to 6.0.0.
- Updated libc to 0.2.155.
- Updated regex to 1.10.5.
- Updated windows to 0.57.0.

### Fixed

- If a non-master plugin was a master of two master plugins, it would be hoisted
  to load before the master that loaded second instead of the master that loaded
  first.

## [16.0.0] - 2024-05-02

### Added

- Support for Fallout 4 from the Epic Games Store.
- `Cargo.lock` is no longer ignored by Git.

### Changed

- `Error::IoError`,  `Error::NoFilename`, `Error::PluginParsingError`,
  `Error::TooManyActivePlugins`, `Error::DuplicatePlugin`,
  `Error::NonMasterBeforeMaster`, `Error::GameMasterMustLoadFirst`,
  `Error::IniParsingError` and `Error::VdfParsingError` now hold contextual
  data.
- `Error::DecodeError` now holds a `Vec<u8>`.
- `Error::EncodeError` now holds a `String`.

- Updated to Rust's 2021 edition.
- Updated esplugin to 5.0.0.
- Updated rust-ini to 0.21.0.
- Updated windows to 0.56.0.

### Removed

- `Error::InvalidPlugin` as it doesn't provide any value over more granular
  errors now that they hold contextual data.
- `Error::InvalidRegex` as it was only used for a hardcoded regex.
- The filetime dependency.

### Fixed

- Removing a master file that is responsible for hoisting another plugin was not
  validated correctly. The validation logic now correctly prevents such masters
  from being removed until the plugin(s) they hoist are removed first, unless
  the next master in the load order also hoists the same plugin(s) or the master
  being removed is the last master in the load order.

## [15.0.2] - 2023-11-25

### Changed

- Case insensitivity is now consistently implemented using case folding instead
  of a mix of case folding and lowercasing.
- Updated rust-ini to 0.20.0.
- Updated keyvalues-parser to 0.2.0.
- Updated windows to 0.52.0.

### Fixed

- When parsing ini files, single and double quote characters are no longer
  treated as special characters, and backslashes are no longer treated as
  potentially the start of an escape sequence.

## [15.0.1] - 2023-10-06

### Fixed

- If two plugins in a timestamp-based load order have the same timestamp, they
  are now sorted in descending filename order instead of ascending filename
  order, matching the behaviour of all relevant games.
- Plugins that are installed but not listed in the load order file (if relevant)
  or the active plugins file (if the load order file is not relevant or does not
  exist) are now sorted by ascending file modification timestamp instead of
  ascending filename.

  If two plugins share the same timestamp, then they are sorted by filename.
  For Starfield, the filename sort order is ascending, while for all other games
  it is descending.

  This matches the games' behaviour for unlisted implicitly active plugins, and
  matches the behaviour of xEdit and Wrye Bash for all unlisted plugins.
- `WritableLoadOrder::save()` for asterisk-based load orders now sets the load
  order by setting plugin timestamps when `plugins.txt` is being ignored, in
  addition to writing `plugins.txt`. This ensures that the load order that is
  saved is seen when it is next loaded, even if `plugins.txt` is still ignored.
- `WritableLoadOrder::is_ambiguous()` now always returns false for
  timestamp-based load orders, because there is never any real ambiguity.
- `WritableLoadOrder::is_ambiguous()` for asterisk-based load orders now ignores
  implicitly active plugins when checking that plugins are listed in
  `plugins.txt`. Previously it only ignored early-loading plugins, which was
  incorrect because implicitly-active plugin load order positions are always
  well-defined.

## [15.0.0] - 2023-09-28

### Added

- Support for Starfield, using `GameId::Starfield`.
- Support for detecting Fallout: New Vegas `.nam` files. Any plugin with the
  same basename as a `.nam` file in the Data folder is now treated as implicitly
  active.
- Support for detecting the correct local app data folders for Microsoft Store
  installs of Skyrim Special Edition and Fallout 4, and Epic Games Store
  installs of Fallout: New Vegas.
- Support for plugins activated using the `sTestFile1` through `sTestFile10` ini
  file properties for all games other than Morrowind, which does not use those
  properties. Plugins activated using those ini file properties are treated as
  implicitly active.
- `GameSettings::early_loading_plugins()` and `GameSettings::loads_early()`.
- `Error::InvalidEarlyLoadingPluginPosition`, which is now used instead of
  `Error::GameMasterMustLoadFirst` when an early-loading plugin has the wrong
  position in a load order that is being set.
- `Error::NoDocumentsPath` is used when the user's Documents path cannot be
  detected.
- `Error::VdfParsingError` is used to represent errors encountered while parsing
  VDF files. This is currently only done for Starfield's Steam app manifest
  file.
- `Error::SystemError` is used to represent unknown operating system errors.
  This is currently only relevant for Microsoft Store installs of Starfield.

### Changed

- libloadorder now distinguishes between implicitly active plugins and those
  that load in specific positions earlier than plugins listed in `plugins.txt`,
  which are now referred to as early-loading plugins. The set of implicitly
  active plugins is a superset of early loading plugins plus any plugins
  activated by `.nam` files or ini file properties.
- `Fallout4.ccc` and `plugins.txt` are now ignored when Fallout 4 or Fallout 4
  VR have any plugins activated using ini file properties, to match the
  behaviour of Fallout 4. Fallout 4 VR is assumed to have the same behaviour.
- Replaced the `app_dirs2` dependency with `dirs`.
- `Error::UnrepresentedHoist`'s two `String` members are now named to
  disambiguate them.

### Fixed

- libloadorder now looks for `Oblivion.ini`'s `bUseMyGamesDirectory` property in
  the ini file's `General` section, instead of anywhere in the file.

## [14.2.2] - 2023-09-16

## Changed

- `GameSettings::active_plugins_file()` now returns a path ending in
  `Plugins.txt` instead of `plugins.txt` for all games other than Morrowind, to
  case-sensitively match the filenames used by the games.

## [14.2.1] - 2023-08-22

## Changed

- `GameSettings::active_plugins_file()` now returns a path ending in
  `Plugins.txt` instead of `plugins.txt` for Oblivion and Nehrim, to
  case-sensitively match the filenames created by those games.

## [14.2.0] - 2023-08-20

### Changed

- `GameSettings::new()` is now available on Linux. Calling it for any game other
  than Morrowind will always fail, as all other games use a local data path,
  which must be provided on Linux.

### Fixed

- A typo in the `Error::GameMasterMustLoadFirst` error message.

## [14.1.0] - 2023-04-26

### Added

- Support for providing the paths to any plugins directories other than the
  game's plugins directory that contain plugins which should be considered part
  of the load order. This is intended to support the Microsoft Store's Fallout 4 DLCs, which are installed outside of the base game's install path.

  - libloadorder will detect if a given Fallout 4 path is for a Microsoft Store
    install by looking for `appxmanifest.xml` in the game directory when
    creating a `GameSettings` struct. If found, libloadorder will initialise the
    settings with the paths to the external DLC data directories in case the
    DLCs are installed.
  - `GameSettings::set_additional_plugins_directories()` can be used to
    customise the paths that libloadorder will take into account.
  - `WritableLoadOrder::game_settings_mut()` can be used to get a mutable
    `GameSettings` reference from a `WritableLoadOrder` impl.

## [14.0.0] - 2023-03-18

### Added

- Support for Enderal: Forgotten Stories and Enderal: Forgotten Stories (Special
  Edition), which are total conversion mods for Skyrim and Skyrim Special
  Edition respectively. They operate in the same way as their base games, but
  store their load orders in different directories. If given the `Skyrim` or
  `SkyrimSE` game IDs and a game path but no local path, libloadorder will now
  check for `Enderal Launcher.exe` in the game path and use the appropriate
  local path.

### Changed

- Excess active plugins are no longer deactivated on load. This means that
  changing the load order when there are more than the game's supported number
  of plugins active will no longer risk deactivating any plugins.
- Performance improvements to loading plugins, setting active plugins and
  counting the number of active plugins.

## [13.3.0] - 2022-10-11

### Added

- `GameSettings::new()` now sets the local app data folder name for Skyrim SE to
  `Skyrim Special Edition EPIC` if the game install path does not contain
  `Galaxy64.dll` and does contain `EOSSDK-Win64-Shipping.dll`. The
  `EOSSDK-Win64-Shipping.dll` file is present when the Epic Games Store's
  distribution of Skyrim SE is installed.

## [13.2.0] - 2022-10-01

### Added

- `GameSettings::new()` now sets the local app data folder name for Skyrim SE to
  `Skyrim Special Edition GOG` if the game install path contains `Galaxy64.dll`,
  and otherwise uses `Skyrim Special Edition` as before. The `Galaxy64.dll` file
  is present when GOG's distribution of Skyrim SE is installed.

### Changed

- Updated to Rust's 2018 edition.

### Fixed

- If Oblivion's `Oblivion.ini` could not be found or read, or if it did not
  contain the `bUseMyGamesDirectory` setting, the game's install path would be
  used as the parent directory for `plugins.txt`. It now correctly defaults to
  using the game's local app data directory, and only uses the install path if
  `bUseMyGamesDirectory=0` is found.

## [13.1.1] - 2022-09-15

### Changed

- Updated esplugin to v4.0.0.
- The encoding dependency has been replaced by a dependency on encoding_rs.

## [13.1.0] - 2022-02-23

### Added

- `WritableLoadOrder::is_ambiguous()` for checking if all currently-loaded
  plugins have a well defined load order position and that all data sources are
  consistent.

## [13.0.0] - 2021-04-17

### Changed

- `GameId::supports_light_masters()` has been renamed to
  `GameId::supports_light_plugins()` to reflect that not all light-flagged
  plugins are masters.
- Updated to esplugin v3.4.0.
- Updated to criterion v0.3.0.
- The app_dirs dependency has been replaced by a dependency on app_dirs2.

### Fixed

- Bare trait object deprecation warnings.

## [12.0.1] = 2019-02-26

### Fixed

- Fixed the casing of `Hearthfires.esm` to `HearthFires.esm` in the hardcoded
  plugin lists for Skyrim and Skyrim Special Edition, to fix recognising that
  plugin on case-sensitive filesystems.

## [12.0.0] - 2018-10-29

### Added

- `WritableLoadOrder::add()` inserts the given plugin into the load order at the
  latest valid position, and returns the plugin's new index on success.
- `WritableLoadOrder::remove()` removes the given plugin from the load order.

### Changed

- `WritableLoadOrder::activate()` and `WritableLoadOrder::set_active_plugins()`
  will now error if attempting to activate a plugin that has no existing load
  order position.
- `WritableLoadOrder::set_plugin_index()` now returns the index set for the
  given plugin on success, which can be useful if passing a index larger than
  the length of the load order.

### Fixed

- `WritableLoadOrder`'s `load()`, `set_load_order()`, `set_plugin_index()`,
  `set_active_plugins()` and `activate()` now all respect the game behaviour of
  'hoisting' non-master plugins that are masters of master plugins to load
  immediately before the earliest master plugin that depends on them. It is now
  an error to attempt to set a load order that contains a plugin in an unhoisted
  position that the game will hoist.
- `WritableLoadOrder::is_self_consistent()` now falls back to reading
  `loadorder.txt` as encoded in Windows-1252 if it is not encoded in valid
  UTF-8, for compatibility with utilities that incorrectly encode it in
  Windows-1252 and consistency with `WritableLoadOrder::load()`.

## [11.4.1] - 2018-09-10

### Fixed

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

# Libloadorder

Libloadorder is a free software library for manipulating the load order and
active status of plugins for TES III: Morrowind, TES IV: Oblivion,
TES V: Skyrim, Fallout 3 and Fallout: New Vegas.


## Build Instructions

Libloadorder uses [CMake](http://cmake.org) v2.8.9 or later, because it's possible to cross-compile it, though Linux instructions are no longer provided as they have become outdated.

### Requirements

* [Boost](http://www.boost.org) (tested with v1.55.0)
* [Libespm](http://github.com/WrinklyNinja/libespm)
* [zlib](http://zlib.net/) (tested with v1.2.8)

### Windows

#### Boost

```
bootstrap.bat
b2 toolset=msvc threadapi=win32 link=static runtime-link=static variant=release address-model=32 --with-log --with-date_time --with-thread --with-filesystem --with-locale --with-regex --with-system --with-iostreams
```

`link`, `runtime-link` and `address-model` can all be modified if shared linking or 64 bit builds are desired. Libloadorder uses statically-linked Boost libraries by default: to change this, edit [CMakeLists.txt](CMakeLists.txt).

#### Libloadorder

Libloadorder uses the following CMake variables to set build parameters:

Parameter | Values | Description
--------------------------------
`BUILD_SHARED_LIBS` | `ON`, `OFF` | Whether or not to build a shared libloadorder. Defaults to `OFF`.
`PROJECT_STATIC_RUNTIME` | `ON`, `OFF` | Whether to link the C++ runtime statically or not. This also affects the Boost libraries used. Defaults to `ON`.
`PROJECT_ARCH` | `32`, `64` | Whether to build 32 or 64 bit libloadorder binaries. Defaults to `32`.
`LIBESPM_ROOT` | path | Path to the root of the libespm repository folder. Defaults to `../libespm`, ie. the libespm folder is next to the libloadorder folder.

You may also need to define `BOOST_ROOT` and `ZLIB_ROOT` if CMake can't find them.

1. Set CMake up so that it builds the binaries in the `build` subdirectory of the libloadorder folder.
2. Define any necessary parameters.
3. Configure CMake, then generate a build system for Visual Studio 12.
4. Open the generated solution file, and build it.
# Libloadorder

[![Build Status](https://travis-ci.org/WrinklyNinja/libloadorder.svg?branch=master)](https://travis-ci.org/WrinklyNinja/libloadorder)

Libloadorder is a free software library for manipulating the load order and active status of plugins for the following games:

* TES III: Morrowind
* TES IV: Oblivion
* TES V: Skyrim
* Fallout 3
* Fallout: New Vegas
* Fallout 4

## Build Instructions

Libloadorder uses [CMake](http://cmake.org) to generate build files. Instructions for Windows are given below.

Instructions for other platforms are not provided, but the process for building on Ubuntu (12.04) is laid out fairly clearly in the [Travis config file](.travis.yml). The same CMake variables documented below apply to Windows and Linux.

### Requirements

* [Boost](http://www.boost.org) Filesystem and Locale libraries: tested with v1.55.0 and v1.59.0.
* [Google Test](https://github.com/google/googletest): Required to build libloadorder's tests, but not the library itself. Tested with v1.7.0.
* [Libespm](http://github.com/WrinklyNinja/libespm) v2.1.0

### Windows

#### Google Test

Just generate an MSVC solution using Google Test's CMake config, and build the `gtest-main` project.

#### Boost

```
bootstrap.bat
b2 toolset=msvc threadapi=win32 link=static runtime-link=static variant=release address-model=32 --with-filesystem --with-locale --with-system
```

`link`, `runtime-link` and `address-model` can all be modified if shared linking or 64 bit builds are desired. Libloadorder uses statically-linked Boost libraries by default: to change this, edit [CMakeLists.txt](CMakeLists.txt).

#### Libloadorder

Libloadorder uses the following CMake variables to set build parameters:

Parameter | Values | Description
----------|--------|------------
`BUILD_SHARED_LIBS` | `ON`, `OFF` | Whether or not to build a shared libloadorder. Defaults to `OFF`.
`PROJECT_STATIC_RUNTIME` | `ON`, `OFF` | Whether to link the C++ runtime statically or not. This also affects the Boost libraries used. Defaults to `ON`.
`PROJECT_ARCH` | `32`, `64` | Whether to build 32 or 64 bit libloadorder binaries. Defaults to `32`.
`LIBESPM_ROOT` | path | Path to the root of the libespm repository folder. Defaults to `../libespm`, ie. the libespm folder is next to the libloadorder folder.

You may also need to define `BOOST_ROOT` if CMake can't find Boost, and `GTEST_ROOT` if CMake can't find Google Test.

1. Set CMake up so that it builds the binaries in the `build` subdirectory of the libloadorder folder.
2. Define any necessary parameters.
3. Configure CMake, then generate a build system for Visual Studio 12.
4. Open the generated solution file, and build it.

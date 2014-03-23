# Libloadorder

Libloadorder is a free software library for manipulating the load order and
active status of plugins for TES III: Morrowind, TES IV: Oblivion,
TES V: Skyrim, Fallout 3 and Fallout: New Vegas.


## Build Instructions

Libloadorder uses [CMake](http://cmake.org) v2.8.9 or later for cross-platform building support, as though it is a Windows library (due to Registry lookup), development has taken place on Windows and Linux.

Libloadorder expects all libraries' folders to be present alongside the libloadorder repository folder that contains this readme, or otherwise installed such that the compiler and linker used can find them without suppling additional paths. All paths below are relative to the folder(s) containing the libraries and libloadorder.

### Requirements

* [CMake](http://cmake.org/) v2.8.9.
* [Boost](http://www.boost.org) v1.51.0.
* [Libespm](http://github.com/WrinklyNinja/libespm)
* [UTF8-CPP](http://sourceforge.net/projects/utfcpp/) v2.3.2.

### Windows

#### Boost

```
bootstrap.bat
b2 toolset=msvc-12.0 threadapi=win32 link=static variant=release address-model=32 --with-log --with-date_time --with-thread --with-filesystem --with-locale --with-regex --with-system --stagedir=stage-32
```

Pass `address-model=64` and `--stagedir=stage-64` instead if building 64-bit libloadorder.

#### Libloadorder

1. Set CMake up so that it builds the binaries in the `build` subdirectory of the libloadorder folder.
2. Define `PROJECT_ARCH=32` or `PROJECT_ARCH=64` to build 32 or 64 bit executables respectively.
3. Define `PROJECT_LINK=STATIC` to build a static API, or `PROJECT_LINK=SHARED` to build a DLL API.
4. Define `PROJECT_LIBS_DIR` to point to the folder holding all the required libraries' folders.
5. Configure CMake, then generate a build system for Visual Studio 12.
6. Open the generated solution file, and build it.

### Linux

#### Boost

```
./bootstrap.sh
echo "using gcc : 4.6.3 : i686-w64-mingw32-g++ : <rc>i686-w64-mingw32-windres <archiver>i686-w64-mingw32-ar <ranlib>i686-w64-mingw32-ranlib ;" > tools/build/v2/user-config.jam
./b2 toolset=gcc-4.6.3 target-os=windows link=static runtime-link=static variant=release address-model=32 cxxflags=-fPIC --with-filesystem --with-locale --with-regex --with-system --with-iostreams --stagedir=stage-32
```
Change `i686` to `x86_64` in the `echo` string, and pass `address-model=64` and `--stagedir=stage-64` when running `b2` if building 64-bit libloadorder.

#### Libloadorder

```
mkdir build
cd build
cmake .. -DPROJECT_LIBS_DIR=.. -DPROJECT_ARCH=32 -DPROJECT_LINK=STATIC -DCMAKE_TOOLCHAIN_FILE=mingw-toolchain.cmake
make
```

Pass `-DPROJECT_ARCH=64` if building 64-bit libloadorder, and `-DPROJECT_LINK=SHARED` if building a shared library.
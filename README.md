# Libloadorder

Libloadorder is a free software library for manipulating the load order and
active status of plugins for TES III: Morrowind, TES IV: Oblivion,
TES V: Skyrim, Fallout 3 and Fallout: New Vegas.


## Build Instructions

Libloadorder uses [CMake](http://cmake.org) v2.8.9 or later for cross-platform building support, though development takes place on Linux, and the instructions below reflect this. Building on Windows should be straightforward using analogous commands though.

Libloadorder expects all libraries' folders to be present alongside the libloadorder repository folder that contains this readme, or otherwise installed such that the compiler and linker used can find them without suppling additional paths. All paths below are relative to the folder(s) containing the libraries and libloadorder.

### Requirements

  * [CMake](http://cmake.org/) v2.8.9.
  * [Boost](http://www.boost.org) v1.51.0.
  * [UTF8-CPP](http://sourceforge.net/projects/utfcpp/) v2.3.2.


### Boost

```
./bootstrap.sh
echo "using gcc : 4.6.3 : i686-w64-mingw32-g++ : <rc>i686-w64-mingw32-windres <archiver>i686-w64-mingw32-ar <ranlib>i686-w64-mingw32-ranlib ;" > tools/build/v2/user-config.jam
./b2 toolset=gcc-4.6.3 target-os=windows link=static variant=release address-model=32 cxxflags=-fPIC --with-filesystem --with-locale --with-regex --with-system --stagedir=stage-32
```

### Libloadorder

```
mkdir build
cd build
cmake .. -DPROJECT_LIBS_DIR=.. -DPROJECT_ARCH=32 -DPROJECT_LINK=STATIC -DCMAKE_TOOLCHAIN_FILE=mingw-toolchain.cmake
make
```

If natively compiling, all the ```-DCMAKE_TOOLCHAIN_FILE``` arguments can be omitted, as can the ```echo``` line when building Boost.

To build a shared library, swap ```-DPROJECT_LINK=STATIC``` with ```-DPROJECT_LINK=SHARED```.

To build a 64 bit library, swap all instances of ```i686``` with ```x86_64``` and ```32``` with ```64```.

# Libloadorder

[![AppVeyor Build Status](https://ci.appveyor.com/api/projects/status/github/Ortham/libloadorder?branch=master&svg=true)](https://ci.appveyor.com/project/Ortham/libloadorder)
[![Travis Build Status](https://travis-ci.org/Ortham/libloadorder.svg?branch=master)](https://travis-ci.org/Ortham/libloadorder)
[![dependency status](https://deps.rs/repo/github/Ortham/libloadorder/status.svg)](https://deps.rs/repo/github/Ortham/libloadorder)
[![docs](https://docs.rs/libloadorder-ffi/badge.svg)](https://docs.rs/crate/libloadorder-ffi)

Libloadorder is a cross-platform library for manipulating the load order and
active status of plugins for the following games:

- TES III: Morrowind
- TES IV: Oblivion
- TES V: Skyrim
- TES V: Skyrim Special Edition
- TES V: Skyrim VR
- Fallout 3
- Fallout: New Vegas
- Fallout 4
- Fallout 4 VR

This repository hosts two Rust crates: `libloadorder` is the Rust library, and
`libloadorder-ffi` is the C FFI that wraps it. The `doc` directory also hosts an
[mdbook](https://github.com/rust-lang-nursery/mdBook) that provides a general
introduction to load orders.

To build libloadorder and its C FFI and generate C/C++ headers for it, install
[Rust](https://www.rust-lang.org/) and run
`cargo build --release --all --all-features`.

## Tests

The tests require
[testing-plugins](https://github.com/Ortham/testing-plugins), put them in
`testing-plugins` in the repo root.

Run `cargo test` and `cargo bench` to run the Rust tests and benchmarks
respectively.

To run the FFI tests, make sure you have CMake and C and C++ toolchains
installed (e.g. MSVC on Windows, GCC on Linux), then create a directory at
`ffi/build`, then from that directory run:

```
cmake ..
cmake --build .
ctest
```

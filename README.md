# Libloadorder

![CI](https://github.com/Ortham/libloadorder/actions/workflows/ci.yml/badge.svg?branch=master&event=push)
[![Coverage Status](https://coveralls.io/repos/github/Ortham/libloadorder/badge.svg?branch=master)](https://coveralls.io/github/Ortham/libloadorder?branch=master)
[![docs](https://docs.rs/libloadorder/badge.svg)](https://docs.rs/crate/libloadorder)

Libloadorder is a cross-platform library for manipulating the load order and
active status of plugins for the following games:

- TES III: Morrowind
- OpenMW
- TES IV: Oblivion
- TES IV: Oblivion Remastered
- TES V: Skyrim
- TES V: Skyrim Special Edition
- TES V: Skyrim VR
- Fallout 3
- Fallout: New Vegas
- Fallout 4
- Fallout 4 VR
- Starfield

This repository hosts two Rust crates: `libloadorder` is the Rust library, and
`libloadorder-ffi` is the C FFI that wraps it. The `doc` directory also hosts an
[mdbook](https://github.com/rust-lang-nursery/mdBook) that provides a general
introduction to load orders.

To build libloadorder and its C FFI and generate C/C++ headers for it, install
[Rust](https://www.rust-lang.org/) and run
`cargo build --release --workspace --all-features`.

## Tests

The tests require
[testing-plugins](https://github.com/Ortham/testing-plugins), put them in
`testing-plugins` in the repo root.

Run `cargo test` and `cargo bench` to run the Rust tests and benchmarks
respectively.

To run the FFI tests, make sure you have CMake and C and C++ toolchains
installed (e.g. MSVC on Windows, GCC on Linux), then run:

```
mkdir ffi/build
cmake -B ffi/build ffi
cmake --build ffi/build
ctest --test-dir ffi/build --output-on-failure
```

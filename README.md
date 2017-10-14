# Libloadorder

[![Build Status](https://travis-ci.org/WrinklyNinja/libloadorder.svg?branch=master)](https://travis-ci.org/WrinklyNinja/libloadorder)
[![Coverage Status](https://coveralls.io/repos/github/WrinklyNinja/libloadorder/badge.svg?branch=master)](https://coveralls.io/github/WrinklyNinja/libloadorder?branch=master)

Libloadorder is a free software library for manipulating the load order and active status of plugins for the following games:

* TES III: Morrowind
* TES IV: Oblivion
* TES V: Skyrim
* TES V: Skyrim Special Edition
* Fallout 3
* Fallout: New Vegas
* Fallout 4

This repository hosts two Rust crates: `libloadorder` is the Rust implementation, and `libloadorder-ffi` is the C FFI that wraps it. The `doc` directory also hosts an mdbook that provides a general introduction to load orders.

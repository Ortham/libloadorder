cmake_minimum_required(VERSION 3.10)
project(libloadorder-ffi-tests CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(LINUX)
    set(SYSTEM_LIBS pthread dl m)
elseif(WIN32)
    set(SYSTEM_LIBS ntdll ws2_32)
endif()

if(CMAKE_CROSSCOMPILING AND NOT DEFINED RUST_TARGET)
    if(WIN32 AND MINGW AND CMAKE_SIZEOF_VOID_P EQUAL 8)
        set(RUST_TARGET "x86_64-pc-windows-gnu")
    elseif(WIN32 AND MINGW AND CMAKE_SIZEOF_VOID_P EQUAL 4)
        set(RUST_TARGET "i686-pc-windows-gnu")
    else()
        message(FATAL_ERROR "Cross-compiling without RUST_TARGET set, and an appropriate value could not be set automatically!")
    endif()
endif()

if(DEFINED RUST_TARGET)
    message(STATUS "Building for Rust target ${RUST_TARGET}")
    set(TARGET_PATH "${PROJECT_SOURCE_DIR}/../target/${RUST_TARGET}")
else()
    set(TARGET_PATH "${PROJECT_SOURCE_DIR}/../target")
endif()

set(LIBLOADORDER_FFI_LIB_NAME "${CMAKE_STATIC_LIBRARY_PREFIX}loadorder_ffi${CMAKE_STATIC_LIBRARY_SUFFIX}")

add_library(libloadorder_ffi INTERFACE)
target_include_directories(libloadorder_ffi INTERFACE "${CMAKE_SOURCE_DIR}/include")
target_link_libraries(libloadorder_ffi INTERFACE
        optimized "${TARGET_PATH}/release/${LIBLOADORDER_FFI_LIB_NAME}"
        debug "${TARGET_PATH}/debug/${LIBLOADORDER_FFI_LIB_NAME}"
        ${SYSTEM_LIBS})

add_executable(ffi_cpp_tests "${CMAKE_SOURCE_DIR}/tests/ffi.cpp")
target_link_libraries(ffi_cpp_tests PRIVATE libloadorder_ffi)

enable_testing()
add_test(NAME ffi_cpp_tests COMMAND ffi_cpp_tests)

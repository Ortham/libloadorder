cmake_minimum_required(VERSION 3.10)
project(libloadorder-ffi-tests CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(LINUX)
    set(SYSTEM_LIBS pthread dl m)
elseif(WIN32)
    set(SYSTEM_LIBS ntdll ws2_32)
endif()

set(TARGET_PATH "${CMAKE_SOURCE_DIR}/../target")
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

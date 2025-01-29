cmake_minimum_required(VERSION 2.8)
project(libloadorder-ffi-tests CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories("${CMAKE_SOURCE_DIR}/include")

if (CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set (SYSTEM_LIBS pthread dl m)
endif ()

if (MSVC)
    set (SYSTEM_LIBS ntdll windowsapp Userenv Propsys)
endif ()

set (LIBLOADORDER_FFI_LIBRARY "${CMAKE_SOURCE_DIR}/../target/debug/${CMAKE_STATIC_LIBRARY_PREFIX}loadorder_ffi${CMAKE_STATIC_LIBRARY_SUFFIX}")

add_executable(ffi_cpp_tests "${CMAKE_SOURCE_DIR}/tests/ffi.cpp")
target_link_libraries(ffi_cpp_tests ${LIBLOADORDER_FFI_LIBRARY} ${SYSTEM_LIBS})

enable_testing()
add_test(ffi_cpp_tests ffi_cpp_tests)

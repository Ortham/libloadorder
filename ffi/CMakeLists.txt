cmake_minimum_required(VERSION 2.8)
project(libloadorder-ffi-tests CXX C)

include_directories("${CMAKE_SOURCE_DIR}/include")

if (CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set (CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")
    set (SYSTEM_LIBS pthread dl m)
endif ()

if (MSVC)
    set (SYSTEM_LIBS ws2_32 Userenv bcrypt)
endif ()

set (LIBLOADORDER_FFI_LIBRARY "${CMAKE_SOURCE_DIR}/../target/debug/${CMAKE_STATIC_LIBRARY_PREFIX}loadorder_ffi${CMAKE_STATIC_LIBRARY_SUFFIX}")

add_executable(ffi_cpp_tests "${CMAKE_SOURCE_DIR}/tests/ffi.cpp")
target_link_libraries(ffi_cpp_tests ${LIBLOADORDER_FFI_LIBRARY} ${SYSTEM_LIBS})

add_executable(ffi_c_tests "${CMAKE_SOURCE_DIR}/tests/ffi.c")
target_link_libraries(ffi_c_tests ${LIBLOADORDER_FFI_LIBRARY} ${SYSTEM_LIBS})

enable_testing()
add_test(ffi_cpp_tests ffi_cpp_tests)
add_test(ffi_c_tests ffi_c_tests)

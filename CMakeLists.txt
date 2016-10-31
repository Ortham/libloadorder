##############################
# General Settings
##############################

cmake_minimum_required (VERSION 2.8)
project (libloadorder)
include(ExternalProject)

option(BUILD_SHARED_LIBS "Build a shared library" OFF)
option(MSVC_STATIC_RUNTIME "Build with static runtime libs (/MT)" ON)

IF (${MSVC_STATIC_RUNTIME})
    set (MSVC_SHARED_RUNTIME OFF)
ELSE()
    set (MSVC_SHARED_RUNTIME ON)
ENDIF()

set (Boost_USE_STATIC_LIBS ON)
set (Boost_USE_MULTITHREADED ON)
IF (MSVC)
    set (Boost_USE_STATIC_RUNTIME ${MSVC_STATIC_RUNTIME})
ELSE()
    set (Boost_USE_STATIC_RUNTIME OFF)
ENDIF()

find_package(Boost REQUIRED COMPONENTS locale filesystem system iostreams)

ExternalProject_Add(libespm
                    PREFIX "external"
                    URL "https://github.com/WrinklyNinja/libespm/archive/2.5.4.tar.gz"
                    CONFIGURE_COMMAND ""
                    BUILD_COMMAND ""
                    INSTALL_COMMAND "")
ExternalProject_Get_Property(libespm SOURCE_DIR)
set (LIBESPM_INCLUDE_DIRS "${SOURCE_DIR}/include")

ExternalProject_Add(GTest
                    PREFIX "external"
                    URL "https://github.com/google/googletest/archive/release-1.7.0.tar.gz"
                    CMAKE_ARGS -Dgtest_force_shared_crt=${MSVC_SHARED_RUNTIME}
                    INSTALL_COMMAND "")
ExternalProject_Get_Property(GTest SOURCE_DIR BINARY_DIR)
set (GTEST_INCLUDE_DIRS "${SOURCE_DIR}/include")
set (GTEST_LIBRARIES "${BINARY_DIR}/${CMAKE_CFG_INTDIR}/${CMAKE_STATIC_LIBRARY_PREFIX}gtest${CMAKE_STATIC_LIBRARY_SUFFIX}")

ExternalProject_Add(testing-plugins
                    PREFIX "external"
                    URL "https://github.com/WrinklyNinja/testing-plugins/archive/1.0.0.tar.gz"
                    CONFIGURE_COMMAND ""
                    BUILD_COMMAND ""
                    INSTALL_COMMAND "")

set (PROJECT_SRC    "${CMAKE_SOURCE_DIR}/src/backend/error.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/helpers.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/GameSettings.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/LoadOrder.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/PathCache.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/Plugin.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/_lo_game_handle_int.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/constants.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/c_helpers.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/libloadorder.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/activeplugins.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/loadorder.cpp")

set (PROJECT_HEADERS "${CMAKE_SOURCE_DIR}/src/backend/error.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/helpers.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/GameSettings.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/LoadOrder.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/PathCache.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/Plugin.h"
                    "${CMAKE_SOURCE_DIR}/src/api/_lo_game_handle_int.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/constants.h"
                    "${CMAKE_SOURCE_DIR}/src/api/c_helpers.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/libloadorder.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/activeplugins.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/loadorder.h")

set (TESTER_SRC "${CMAKE_SOURCE_DIR}/src/tests/main.cpp"
                 "${CMAKE_SOURCE_DIR}/src/backend/error.cpp"
                 "${CMAKE_SOURCE_DIR}/src/backend/helpers.cpp"
                 "${CMAKE_SOURCE_DIR}/src/backend/GameSettings.cpp"
                 "${CMAKE_SOURCE_DIR}/src/backend/LoadOrder.cpp"
                 "${CMAKE_SOURCE_DIR}/src/backend/Plugin.cpp"
                 "${CMAKE_SOURCE_DIR}/src/api/_lo_game_handle_int.cpp"
                 "${CMAKE_SOURCE_DIR}/src/api/c_helpers.cpp")

set (TESTER_HEADERS "${CMAKE_SOURCE_DIR}/src/tests/GameTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/_lo_game_handle_intTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/activeplugins.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/CApiGameOperationTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/CHelpersTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/libloadorder.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/lo_create_handle_test.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/api/loadorder.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/backend/ErrorTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/backend/GameSettingsTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/backend/HelpersTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/backend/LoadOrderTest.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/backend/PluginTest.h")

source_group("Header Files" FILES ${PROJECT_HEADERS} ${TESTER_HEADERS})

# Include source and library directories.
include_directories ("${CMAKE_SOURCE_DIR}/src"
                     "${CMAKE_SOURCE_DIR}/include"
                     ${LIBESPM_INCLUDE_DIRS}
                     ${Boost_INCLUDE_DIRS}
                     ${GTEST_INCLUDE_DIRS})


##############################
# Platform-Specific Settings
##############################

# Settings when compiling for Windows.
IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
    add_definitions (-DUNICODE -D_UNICODE)
ENDIF ()

# GCC and MinGW settings.
IF (CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -O3")
    set (CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3 -std=c++14")

    set(PROJECT_LIBRARIES "pthread")
ENDIF ()

IF (MSVC)
    # Update compiler flags.
    IF (MSVC_STATIC_RUNTIME)
        FOREACH(flag
            CMAKE_C_FLAGS_RELEASE CMAKE_C_FLAGS_RELWITHDEBINFO
            CMAKE_C_FLAGS_DEBUG CMAKE_C_FLAGS_DEBUG_INIT
            CMAKE_CXX_FLAGS_RELEASE  CMAKE_CXX_FLAGS_RELWITHDEBINFO
            CMAKE_CXX_FLAGS_DEBUG  CMAKE_CXX_FLAGS_DEBUG_INIT)
            STRING(REPLACE "/MD"  "/MT" "${flag}" "${${flag}}")
            SET("${flag}" "${${flag}} /EHsc")
        ENDFOREACH()
    ENDIF ()
    set (PROJECT_LIBRARIES version)
ENDIF ()


##############################
# Define Targets
##############################

# Build libloadorder library.
add_library           (loadorder ${PROJECT_SRC} ${PROJECT_HEADERS})
add_dependencies      (loadorder libespm)
target_link_libraries (loadorder ${Boost_LIBRARIES} ${PROJECT_LIBRARIES})

    # Build libloadorder tester.
add_executable        (tests ${TESTER_SRC} ${TESTER_HEADERS})
add_dependencies      (tests GTest testing-plugins)
target_link_libraries (tests loadorder ${Boost_LIBRARIES} ${GTEST_LIBRARIES} ${PROJECT_LIBRARIES})


##############################
# Set Target-Specific Flags
##############################

IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
    IF (BUILD_SHARED_LIBS)
        set_target_properties (loadorder PROPERTIES COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS} LIBLO_EXPORT")
    ELSE ()
        set_target_properties (loadorder PROPERTIES COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS} LIBLO_STATIC")
        set_target_properties (tests PROPERTIES COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS} LIBLO_STATIC")
    ENDIF ()
ENDIF ()

##############################
# Post-Build Steps
##############################

# Copy testing plugins
ExternalProject_Get_Property(testing-plugins SOURCE_DIR)
add_custom_command(TARGET tests POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${SOURCE_DIR}
        $<TARGET_FILE_DIR:tests>)

##############################
# Install
##############################

install(TARGETS loadorder
        DESTINATION bin
        CONFIGURATIONS Release)

install(DIRECTORY "${CMAKE_SOURCE_DIR}/include/libloadorder"
        DESTINATION include
        CONFIGURATIONS Release)

install(DIRECTORY "${CMAKE_SOURCE_DIR}/build/docs/html"
        DESTINATION "."
        CONFIGURATIONS Release
        RENAME "docs")

##############################
# CPack
##############################

# Get version info using Git if available
find_package(Git)

IF (GIT_FOUND)
    execute_process(COMMAND ${GIT_EXECUTABLE} describe --tags --long
                  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
                  OUTPUT_VARIABLE GIT_DESCRIBE_STRING
                  OUTPUT_STRIP_TRAILING_WHITESPACE)
ELSE()
    SET (GIT_DESCRIBE_STRING "unknown-version")
ENDIF ()

set(CPACK_GENERATOR "7Z")
set(CPACK_PACKAGE_VERSION ${GIT_DESCRIBE_STRING})
set(CPACK_PACKAGE_DIRECTORY "${CMAKE_BINARY_DIR}/package")

include(CPack)

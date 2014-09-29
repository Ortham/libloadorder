# Settings passed on the command line:
#
# PROJECT_ARCH = the build architecture
# LIBESPM_ROOT = the path to the root of the libespm source.

##############################
# General Settings
##############################

cmake_minimum_required (VERSION 2.8.9)
project (libloadorder)

option(BUILD_SHARED_LIBS "Build a shared library" OFF)
option(PROJECT_STATIC_RUNTIME "Build with static runtime libs (/MT)" ON)

IF (NOT DEFINED PROJECT_ARCH)
    set (PROJECT_ARCH "32")
ENDIF ()

IF (NOT DEFINED LIBESPM_ROOT)
    set (LIBESPM_ROOT "../libespm")
ENDIF ()

set (Boost_USE_STATIC_LIBS ON)
set (Boost_USE_MULTITHREADED ON)
set (Boost_USE_STATIC_RUNTIME PROJECT_STATIC_RUNTIME)

find_package(Boost REQUIRED COMPONENTS locale filesystem system iostreams)
find_package(GTest REQUIRED)

set (PROJECT_SRC    "${CMAKE_SOURCE_DIR}/src/backend/error.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/helpers.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/game.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/plugins.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/constants.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/libloadorder.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/activeplugins.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/loadorder.cpp")

set (PROJECT_HEADERS "${CMAKE_SOURCE_DIR}/src/backend/error.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/helpers.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/game.h"
                    "${CMAKE_SOURCE_DIR}/src/backend/plugins.h"
                    "${CMAKE_SOURCE_DIR}/src/api/constants.h"
                    "${CMAKE_SOURCE_DIR}/src/api/libloadorder.h"
                    "${CMAKE_SOURCE_DIR}/src/api/activeplugins.h"
                    "${CMAKE_SOURCE_DIR}/src/api/loadorder.h")

source_group("Header Files" FILES ${PROJECT_HEADERS})

set (TESTER_SRC "${CMAKE_SOURCE_DIR}/src/tester.cpp")

# Include source and library directories.
include_directories ("${CMAKE_SOURCE_DIR}/src"
                     ${LIBESPM_ROOT}
                     ${Boost_INCLUDE_DIRS}
                     ${GTEST_INCLUDE_DIRS})


##############################
# Platform-Specific Settings
##############################

# Settings when compiling for Windows.
IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
    add_definitions (-DUNICODE -D_UNICODE)
    IF (BUILD_SHARED_LIBS)
        add_definitions (-DLIBLO_EXPORT)
    ELSE ()
        add_definitions (-DLIBLO_STATIC)
    ENDIF ()
ENDIF ()


IF (MINGW)
    set (CMAKE_C_FLAGS "-m${PROJECT_ARCH} -O3 -std=c++0x")
    set (CMAKE_CXX_FLAGS "-m${PROJECT_ARCH} -O3 -std=c++0x")
    IF (PROJECT_STATIC_RUNTIME)
        set (CMAKE_EXE_LINKER_FLAGS "-static-libstdc++ -static-libgcc -Wl,--subsystem,windows")
        set (CMAKE_SHARED_LINKER_FLAGS "-static-libstdc++ -static-libgcc -Wl,--subsystem,windows")
        set (CMAKE_MODULE_LINKER_FLAGS "-static-libstdc++ -static-libgcc -Wl,--subsystem,windows")
    ENDIF ()
    set (PROJECT_LIBS "")
ELSEIF (CMAKE_COMPILER_IS_GNUCXX)
    set (CMAKE_C_FLAGS "-m${PROJECT_ARCH} -O3 -std=c++0x")
    set (CMAKE_CXX_FLAGS "-m${PROJECT_ARCH} -O3 -std=c++0x")
    IF (PROJECT_STATIC_RUNTIME)
        set (CMAKE_EXE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
        set (CMAKE_SHARED_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
        set (CMAKE_MODULE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
    ENDIF ()
    set (PROJECT_LIBS "")
ELSEIF (MSVC)
    # Update compiler flags.
    IF (PROJECT_STATIC_RUNTIME)
        FOREACH(flag
            CMAKE_C_FLAGS_RELEASE CMAKE_C_FLAGS_RELWITHDEBINFO
            CMAKE_C_FLAGS_DEBUG CMAKE_C_FLAGS_DEBUG_INIT
            CMAKE_CXX_FLAGS_RELEASE  CMAKE_CXX_FLAGS_RELWITHDEBINFO
            CMAKE_CXX_FLAGS_DEBUG  CMAKE_CXX_FLAGS_DEBUG_INIT)
            STRING(REPLACE "/MD"  "/MT" "${flag}" "${${flag}}")
            SET("${flag}" "${${flag}} /EHsc")
        ENDFOREACH()
    ENDIF ()
    set (PROJECT_LIBS version)
ENDIF ()


##############################
# Actual Building
##############################

# Build libloadorder library.
add_library           (loadorder${PROJECT_ARCH} ${PROJECT_SRC} ${PROJECT_HEADERS})
target_link_libraries (loadorder${PROJECT_ARCH} ${Boost_LIBRARIES} ${PROJECT_LIBS})

# Build libloadorder tester.
add_executable        (libloadorder-tester ${TESTER_SRC})
target_link_libraries (libloadorder-tester loadorder${PROJECT_ARCH} ${Boost_LIBRARIES} ${PROJECT_LIBS} ${GTEST_BOTH_LIBRARIES})

# Settings passed on the command line:
#
# PROJECT_ARCH = the build architecture
# LIBESPM_ROOT = the path to the root of the libespm source.

##############################
# General Settings
##############################

cmake_minimum_required (VERSION 2.8)
project (libloadorder)

option(BUILD_SHARED_LIBS "Build a shared library" OFF)
option(PROJECT_STATIC_RUNTIME "Build with static runtime libs (/MT)" ON)

# With MSVC, the 32/64 bit compilers have separate generators, so PROJECT_ARCH
# should always match whichever is being used.
IF (MSVC)
    IF (CMAKE_CL_64)
        set (PROJECT_ARCH "64")
    ELSE ()
        set (PROJECT_ARCH "32")
    ENDIF ()
ELSE ()
    IF (NOT DEFINED PROJECT_ARCH)
        set (PROJECT_ARCH "32")
    ENDIF ()
ENDIF ()

IF (NOT DEFINED LIBESPM_ROOT)
    set (LIBESPM_ROOT "../libespm")
ENDIF ()

set (Boost_USE_STATIC_LIBS ${PROJECT_STATIC_RUNTIME})
set (Boost_USE_MULTITHREADED ON)
set (Boost_USE_STATIC_RUNTIME ${PROJECT_STATIC_RUNTIME})

find_package(Boost REQUIRED COMPONENTS locale filesystem system)
find_package(GTest)

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
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/constants.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/libloadorder.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/activeplugins.h"
                    "${CMAKE_SOURCE_DIR}/include/libloadorder/loadorder.h")

set (TESTER_SRC "${CMAKE_SOURCE_DIR}/src/tests/main.cpp")

set (TESTER_HEADERS "${CMAKE_SOURCE_DIR}/src/tests/fixtures.h"
					"${CMAKE_SOURCE_DIR}/src/tests/api/libloadorder.h"
					"${CMAKE_SOURCE_DIR}/src/tests/api/activeplugins.h"
					"${CMAKE_SOURCE_DIR}/src/tests/api/loadorder.h"
                    "${CMAKE_SOURCE_DIR}/src/tests/backend/GameHandleTest.h")

source_group("Header Files" FILES ${PROJECT_HEADERS} ${TESTER_HEADERS})

# Include source and library directories.
include_directories ("${CMAKE_SOURCE_DIR}/src"
                     "${CMAKE_SOURCE_DIR}/include"
                     "${LIBESPM_ROOT}/include"
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
IF (CMAKE_COMPILER_IS_GNUCXX)
    set (CMAKE_C_FLAGS "-m${PROJECT_ARCH} -O3 -std=c++11")
    set (CMAKE_CXX_FLAGS "-m${PROJECT_ARCH} -O3 -std=c++11")
    IF (PROJECT_STATIC_RUNTIME)
        set (CMAKE_EXE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
        set (CMAKE_SHARED_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
        set (CMAKE_MODULE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
    ENDIF ()
ENDIF ()


IF (MINGW)
    set (CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -Wl,--subsystem,windows")
    set (CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -Wl,--subsystem,windows")
    set (CMAKE_MODULE_LINKER_FLAGS "${CMAKE_MODULE_LINKER_FLAGS} -Wl,--subsystem,windows")
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
# Define Targets
##############################

# Build libloadorder library.
add_library           (loadorder${PROJECT_ARCH} ${PROJECT_SRC} ${PROJECT_HEADERS})
target_link_libraries (loadorder${PROJECT_ARCH} ${Boost_LIBRARIES} ${PROJECT_LIBS})

IF (${GTEST_FOUND})
    # Build libloadorder tester.
    add_executable        (tests ${TESTER_SRC} ${TESTER_HEADERS})
    target_link_libraries (tests loadorder${PROJECT_ARCH} ${Boost_LIBRARIES} ${GTEST_BOTH_LIBRARIES})
ENDIF ()


##############################
# Set Target-Specific Flags
##############################

IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
    IF (BUILD_SHARED_LIBS)
        set_target_properties (loadorder${PROJECT_ARCH} PROPERTIES COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS} LIBLO_EXPORT")
    ELSE ()
        set_target_properties (loadorder${PROJECT_ARCH} PROPERTIES COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS} LIBLO_STATIC")
        IF (${GTEST_FOUND})
            set_target_properties (tests PROPERTIES COMPILE_DEFINITIONS "${COMPILE_DEFINITIONS} LIBLO_STATIC")
        ENDIF ()
    ENDIF ()
ENDIF ()

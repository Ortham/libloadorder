# Settings passed on the command line:
#
# LIBLO_LIBS_DIR = the directory which all external libraries may be referenced from.
# LIBLO_ARCH = the build architecture
# LIBLO_LINK = whether to build a static or dynamic library.

##############################
# General Settings
##############################

cmake_minimum_required (VERSION 2.8.9)
project (libloadorder)

set (LIBLO_SRC "${CMAKE_SOURCE_DIR}/src/error.cpp" "${CMAKE_SOURCE_DIR}/src/activeplugins.cpp" "${CMAKE_SOURCE_DIR}/src/helpers.cpp" "${CMAKE_SOURCE_DIR}/src/game.cpp" "${CMAKE_SOURCE_DIR}/src/libespm-interface.cpp" "${CMAKE_SOURCE_DIR}/src/libloadorder.cpp" "${CMAKE_SOURCE_DIR}/src/loadorder.cpp" "${CMAKE_SOURCE_DIR}/src/plugins.cpp")

# Include source and library directories.
include_directories ("${LIBLO_LIBS_DIR}/boost" "${LIBLO_LIBS_DIR}/utf8" "${CMAKE_SOURCE_DIR}/src")

##############################
# Platform-Specific Settings
##############################

# Settings when compiling for Windows.
IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
    IF (${LIBLO_LINK} MATCHES "STATIC")
        add_definitions (-DLIBLO_STATIC)
    ELSE ()
        add_definitions (-DLIBLO_EXPORT)
    ENDIF ()
ENDIF ()

# Settings when compiling on Windows.
IF (CMAKE_HOST_SYSTEM_NAME MATCHES "Windows")
    set (LIBLO_LIBS libboost_filesystem-vc110-mt-1_52 libboost_system-vc110-mt-1_52 libboost_regex-vc110-mt-1_52 version)
    set (CMAKE_CXX_FLAGS "/EHsc")
ENDIF ()

# Settings when not cross-compiling.
IF (CMAKE_SYSTEM_NAME MATCHES CMAKE_HOST_SYSTEM_NAME)
    link_directories ("${LIBLO_LIBS_DIR}/boost/stage-${LIBLO_ARCH}/lib")
ENDIF ()

# Settings when compiling and cross-compiling on Linux.
IF (CMAKE_HOST_SYSTEM_NAME MATCHES "Linux")
    set (CMAKE_C_FLAGS  "-m${LIBLO_ARCH}")
    set (CMAKE_CXX_FLAGS "-m${LIBLO_ARCH}")
    set (CMAKE_EXE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
    set (CMAKE_SHARED_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
    set (CMAKE_MODULE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")

    IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
        link_directories ("${LIBLO_LIBS_DIR}/boost/stage-mingw-${LIBLO_ARCH}/lib")
        set (LIBLO_LIBS boost_filesystem boost_system boost_regex version)
    ENDIF ()
ENDIF ()

##############################
# Actual Building
##############################

# Build libloadorder library.
add_library           (loadorder ${LIBLO_LINK} ${LIBLO_SRC})
target_link_libraries (loadorder ${LIBLO_LIBS})

# Build libloadorder tester.
add_executable        (libloadorder-tester "${CMAKE_SOURCE_DIR}/src/tester.cpp")
target_link_libraries (libloadorder-tester loadorder ${LIBLO_LIBS})

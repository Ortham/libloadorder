# Settings passed on the command line:
#
# PROJECT_LIBS_DIR = the directory which all external libraries may be referenced from.
# PROJECT_ARCH = the build architecture
# PROJECT_LINK = whether to build a static or dynamic library.

##############################
# General Settings
##############################

cmake_minimum_required (VERSION 2.8.9)
project (libloadorder)

IF (NOT DEFINED PROJECT_LINK)
    set (PROJECT_LINK "STATIC")
ENDIF ()

IF (NOT DEFINED PROJECT_ARCH)
    set (PROJECT_ARCH "32")
ENDIF ()

IF (NOT DEFINED PROJECT_LIBS_DIR)
    set (PROJECT_LIBS_DIR "..")
ENDIF ()

set (PROJECT_SRC    "${CMAKE_SOURCE_DIR}/src/backend/error.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/helpers.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/game.cpp"
                    "${CMAKE_SOURCE_DIR}/src/backend/plugins.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/libloadorder.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/activeplugins.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/loadorder.cpp"
                    "${CMAKE_SOURCE_DIR}/src/api/constants.cpp")

set (TESTER_SRC "${CMAKE_SOURCE_DIR}/src/tester.cpp")

# Include source and library directories.
include_directories ("${PROJECT_LIBS_DIR}/boost"
                     "${CMAKE_SOURCE_DIR}/src"
                     "${PROJECT_LIBS_DIR}/libespm"
                     "${PROJECT_LIBS_DIR}/zlib")

link_directories ("${PROJECT_LIBS_DIR}/boost/stage-${PROJECT_ARCH}/lib")

##############################
# Platform-Specific Settings
##############################

# Settings when compiling for Windows.
IF (CMAKE_SYSTEM_NAME MATCHES "Windows")
    add_definitions (-DUNICODE -D_UNICODE)
    IF (${PROJECT_LINK} MATCHES "STATIC")
        add_definitions (-DLIBLO_STATIC)
    ELSE ()
        add_definitions (-DLIBLO_EXPORT)
    ENDIF ()
ENDIF ()

# MinGW settings.
IF (MINGW)
    link_directories ("${PROJECT_LIBS_DIR}/zlib/build")
    set (CMAKE_C_FLAGS "-m${PROJECT_ARCH} -O3")
    set (CMAKE_CXX_FLAGS "-m${PROJECT_ARCH} -O3")
    set (CMAKE_EXE_LINKER_FLAGS "-static-libstdc++ -static-libgcc -Wl,--subsystem,windows")
    set (CMAKE_SHARED_LINKER_FLAGS "-static-libstdc++ -static-libgcc -Wl,--subsystem,windows")
    set (CMAKE_MODULE_LINKER_FLAGS "-static-libstdc++ -static-libgcc -Wl,--subsystem,windows")
    set (PROJECT_LIBS zlibstatic
                      boost_locale
                      boost_filesystem
                      boost_system
                      boost_regex
                      boost_iostreams
                      version)
ELSEIF (MSVC)
    IF (MSVC_VERSION EQUAL 1800)
        set (CMAKE_GENERATOR_TOOLSET "v120_xp" CACHE STRING "Platform Toolset" FORCE)
    ENDIF ()
    link_directories ("${PROJECT_LIBS_DIR}/zlib/build/Release")
	set (CMAKE_CXX_FLAGS "/EHsc /MD")
    set (BOOST_SUFFIX "-vc120-mt-1_55")
    set (PROJECT_LIBS zlibstatic
                      libboost_locale${BOOST_SUFFIX}
                      libboost_filesystem${BOOST_SUFFIX}
                      libboost_system${BOOST_SUFFIX}
                      libboost_regex${BOOST_SUFFIX}
                      libboost_iostreams${BOOST_SUFFIX}
                      version)
ENDIF ()



##############################
# Actual Building
##############################

# Build libloadorder library.
add_library           (loadorder${PROJECT_ARCH} ${PROJECT_LINK} ${PROJECT_SRC})
target_link_libraries (loadorder${PROJECT_ARCH} ${PROJECT_LIBS})

# Build libloadorder tester.
add_executable        (libloadorder-tester ${TESTER_SRC})
target_link_libraries (libloadorder-tester loadorder${PROJECT_ARCH} ${PROJECT_LIBS})

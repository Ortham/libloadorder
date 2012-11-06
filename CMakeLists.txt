cmake_minimum_required (VERSION 2.8)
project (libloadorder)

# Set some vars.
set (libloadorder_VERSION_MAJOR 1)
set (libloadorder_VERSION_MINOR 0)
set (libloadorder_VERSION_PATCH 0)

set (LIBLO_SRC "${CMAKE_SOURCE_DIR}/src")
set (LIBLO_LIBS boost_filesystem boost_system boost_regex)
IF (DEFINED LIBLO_CC_LINUX_WINDOWS)
    set (LIBLO_LIBS ${LIBLO_LIBS} version)
ENDIF (DEFINED LIBLO_CC_LINUX_WINDOWS)
set (LIBLO_LIB_SRC "${LIBLO_SRC}/exception.cpp" "${LIBLO_SRC}/activeplugins.cpp" "${LIBLO_SRC}/helpers.cpp" "${LIBLO_SRC}/game.cpp" "${LIBLO_SRC}/libespm-interface.cpp" "${LIBLO_SRC}/libloadorder.cpp" "${LIBLO_SRC}/loadorder.cpp" "${LIBLO_SRC}/plugins.cpp")

# Set compiler flags.
set (CMAKE_C_FLAGS  "-m${LIBLO_ARCH} -static-libgcc")
set (CMAKE_CXX_FLAGS "-m${LIBLO_ARCH} -static-libstdc++ -static-libgcc")
set (CMAKE_EXE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
set (CMAKE_SHARED_LINKER_FLAGS "-static-libstdc++ -static-libgcc")
set (CMAKE_MODULE_LINKER_FLAGS "-static-libstdc++ -static-libgcc")

# Include source and library directories.
include_directories     ("${CMAKE_SOURCE_DIR}/externals/boost" "${CMAKE_SOURCE_DIR}/externals/utf8" ${LIBLO_SRC})
# Depending on platform and architecture, the libs are stored in different places.
IF (DEFINED LIBLO_CC_LINUX_WINDOWS)
    link_directories    ("${CMAKE_SOURCE_DIR}/externals/boost/stage-mingw-${LIBLO_ARCH}/lib")
ELSE (DEFINED LIBLO_CC_LINUX_WINDOWS)
    link_directories    ("${CMAKE_SOURCE_DIR}/externals/boost/stage-${LIBLO_ARCH}/lib")
ENDIF (DEFINED LIBLO_CC_LINUX_WINDOWS)

# Build libstrings library.
add_library         (loadorder ${LIBLO_LINK} ${LIBLO_LIB_SRC})
target_link_libraries   (loadorder ${LIBLO_LIBS})

# Build libstrings tester.
add_executable          (libloadorder-tester ${LIBLO_SRC}/tester.cpp ${LIBLO_SRC}/tester-interface.cpp)
target_link_libraries   (libloadorder-tester loadorder)

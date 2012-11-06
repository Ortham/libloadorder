# Cross-compiling from Linux to Windows.
set (CMAKE_SYSTEM_NAME Windows)

IF (LIBLO_ARCH MATCHES "32")
    set (MINGW i586-mingw32msvc)
ELSE (LIBLO_ARCH MATCHES "32")
    set (MINGW x86_64-w64-mingw32)
ENDIF (LIBLO_ARCH MATCHES "32")

set (CMAKE_C_COMPILER   ${MINGW}-gcc)
set (CMAKE_CXX_COMPILER ${MINGW}-g++)
set (CMAKE_RC_COMPILER  ${MINGW}-windres)
set (CMAKE_RANLIB       ${MINGW}-ranlib)
#set (CMAKE_FORCE_AR           ${MINGW}-ar)

IF (${LIBLO_LINK} MATCHES "STATIC")
    add_definitions (-DLIBLO_STATIC)
ELSE (${LIBLO_LINK} MATCHES "STATIC")
    add_definitions (-DLIBLO_EXPORT)
ENDIF (${LIBLO_LINK} MATCHES "STATIC")

set (LIBLO_CC_LINUX_WINDOWS true)

Rusty Battleships
=================

Client/Server implementation of Battleship (Rust, group 6)


Build requirements
------------------

 * Rust 1.6 (on Windows: 64-bit MSVC ABI)
 * CMake
 * GCC + make (Linux) / Visual Studio 2015, including C++ support (Windows)
 * Qt 5.5 (Linux) / 5.6 beta, 64-bit, VS 2015 (Windows)

Environment setup on Windows
----------------------------

 * Set the QTDIR environment variable to `{$QT_INSTALL_DIR}\5.6\msvc2015_64`
 * Add `{$QT_INSTALL_DIR}\5.6\msvc2015_64\bin` to the path (contains .dlls)

The above steps should become unnecessary once Qt 5.6 stable is released,
since the installer for stable versions should set up the environment
automatically. However, Qt 5.5 does not support Visual Studio 2015, so we have
to use the beta to build on Windows at the moment.

(On Linux/Mac, any stable 5.x version should be fine.)

Environment setup on OS X
----------------------------

CMAKE_PREFIX_PATH=$QTDIR
PKG_CONFIG_PATH=$QTDIR/lib/pkgconfig
DYLD_FRAMEWORK_PATH=$QTDIR/lib

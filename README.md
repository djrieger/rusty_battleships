Rusty Battleships
=================

Client/Server implementation of Battleship (Rust, group 6)


Build requirements
------------------

 * Rust 1.6 (on Windows: 64-bit MSVC ABI)
 * CMake
 * C++ compiler and build tools:
   * Linux: g++ (4.8+) and make
   * OS X: XCode command line tools (include clang)
   * Windows: Visual Studio 2015, including C++ support
 * Qt 5.5 (Linux, OS X) / 5.6 beta, 64-bit, VS 2015 (Windows)
   * When using the online installer only the following Qt modules have to be selected: 
     - Quick Controls
     - Quick
     - Script

Environment setup on Windows
----------------------------

 * Set the QTDIR environment variable to `{$QT_INSTALL_DIR}\5.6\msvc2015_64`
 * Add `{$QT_INSTALL_DIR}\5.6\msvc2015_64\bin` to the path (contains .dlls)

The above steps should become unnecessary once Qt 5.6 stable is released,
since the installer for stable versions should set up the environment
automatically. However, Qt 5.5 does not support Visual Studio 2015, so we have
to use the beta to build on Windows at the moment.

Environment setup on OS X
----------------------------

Qt (QtQuick and base libraries) must be installed via the official installer,
the Homebrew version does not work. The following environment variables need to
be set:

```bash
CMAKE_PREFIX_PATH=$QTDIR
PKG_CONFIG_PATH=$QTDIR/lib/pkgconfig
DYLD_FRAMEWORK_PATH=$QTDIR/lib
```

Ubuntu packages
---------------

The following Ubuntu packages must be installed to compile the project (Only Ubuntu 15.04 and newer):

* qml-module-qtquick-controls
* qml-module-qtquick-dialogs
* qtbase5-dev
* qtdeclarative5-dev

For older versions, such as 14.04 LTS, use the online installer from http://www.qt.io/download-open-source/ to install the Qt modules for your platform (you don't need the source and android packages!). Then export the following, after replacing <QT-Path> and <QT-Version> with the path to your Qt installation and your Qt version (e.g. /Qt/5.5/...):

$ export CMAKE_PREFIX_PATH=<QT-Path>/<QT-Version>/gcc_64/
$ export QTDIR=<QT-Path>/<QT-Version>/gcc_64/
$ export LD_LIBRARY_PATH=<QT-Path>/<QT-Version>/gcc_64/lib
$ export PKG_CONFIG_PATH=<QT-Path>/<QT-Version>/gcc_64/lib/pkgconfig/

Please note, that exports only apply to the terminal in which the exports were executed. You can also add these four lines to your ~/.bashrc.

You also need the OpenGL dev libraries:
$ sudo apt-get install build-essential libgl1-mesa-dev






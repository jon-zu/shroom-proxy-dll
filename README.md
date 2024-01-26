# Shroom dinput8 proxy dll

## Building Instructions

Download `rustup` and install the latest rust `nightly` compiler and install either the `i686-pc-windows-gnu`(linux/mac) or `i686-pc-windows-msvc` toolchain. If you are on windows adjust `.cargo/config.toml` and `build.sh` to the msvc toolchain. Either run `build.sh` or execute the same command in your shell and copy the `dinput8.dll` from `target/i686-pc-windows-gnu` into your maple folder.

For the overlay If you build with gnu you either need to download those DLL(https://code.google.com/archive/p/wtfu/downloads) and place them in your game folder or install a full mingw toolchain.

## Features

* Basic imgui overlay(dx9 only)
* Offset `GetTickCount` and `timeGetTime` to the client launch time
* Tamper `FindFirstFileA` for debug checks, `CreateMutexA` for Debug checks
* Basic stack traces with symbols via a PDB file
* Basic logo skipper(v95 only)
* Some basic z* types
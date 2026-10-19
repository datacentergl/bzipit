# bzipit — Native BZIP2 compressor for Windows

**bzipit** is a lightweight Windows 10/11 (x64) application for compressing files to BZIP2 (`.bz2`). It provides a native drag-and-drop interface and also accepts files or folders dropped directly onto `bzipit.exe`.

## Features

- Compresses each file individually, creating a `.bz2` file next to its original.
- Accepts multiple files and folders and processes folders recursively.
- Preserves the original files.
- Skips `.bz2` input files, existing `.bz2` output files, and the executable itself.
- Supports files without extensions.
- Processes files sequentially on a background thread, keeping the GUI responsive.
- Includes an embedded icon and translations for Spanish (`es`), English (`en`), Galician (`gl`), Catalan (`ca`), Valencian (`val`), Basque (`eu`), French (`fr`), Portuguese (`pt`), German (`de`), and Italian (`it`).

## Two launch modes

**Without file or folder arguments:** bzipit opens its native Windows GUI, featuring a drag-and-drop area, an activity log, and compression status. Launching it with only a language option (such as `--lang=es`) also opens this mode.

**With one or more file or folder arguments:** bzipit compresses the supplied paths recursively without opening the main window. Once processing finishes, a single summary dialog reports the numbers of compressed, skipped, and failed files. This is also the behavior when files or folders are dragged directly onto `bzipit.exe`.

The argument-based mode does not display intermediate progress; its summary dialog appears after the entire batch completes.

## Language selection

bzipit detects the Windows display language automatically. To override it, pass `--lang=<code>`, for example:

```text
bzipit.exe --lang=en
bzipit.exe --lang=val
bzipit.exe --lang=ca-ES-valencia
```

## Cross-compile for Windows from Linux

On Debian or Ubuntu Linux, install the MinGW-w64 cross-compilation tools and the Rust Windows target:

```bash
sudo apt install -y mingw-w64 binutils-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
```

Build the release executable from the project directory:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

Output:

```text
target/x86_64-pc-windows-gnu/release/bzipit.exe
```

The icon is embedded using `windres` and `build.rs`. The `bzip2-sys` dependency compiles libbz2 statically. The goal is a single portable `.exe` without additional application DLLs or a separate runtime installation.

## Verify the Windows executable

Inspect its DLL imports:

```bash
x86_64-w64-mingw32-objdump -p target/x86_64-pc-windows-gnu/release/bzipit.exe | grep 'DLL Name'
```

Inspect its embedded resources (requires `wrestool`):

```bash
wrestool -l target/x86_64-pc-windows-gnu/release/bzipit.exe
```

## Technical notes

- The GUI uses the native Win32 API and `WM_DROPFILES` for drag-and-drop support.
- Drag-and-drop between processes running at different Windows integrity levels may be restricted. Shell virtual files are not supported.
- `musl` is a Linux C library and does not apply to Windows executables. The Linux fallback is command-line based.
- This project has not been compiled or tested on Windows in the current environment.

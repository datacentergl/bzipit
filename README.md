# bzipit

Standalone Windows BZIP2 compressor with embedded icon and multilingual console messages.

## Languages

Windows display language is selected automatically using `GetUserPreferredUILanguages`, checking the user's preferred UI languages in order. Supported: Spanish (`es`), English (`en`), Galician (`gl`), Catalan (`ca`), Valencian (`ca-ES-valencia`, manual override `val`), Basque (`eu`), French (`fr`), Portuguese (`pt`), German (`de`), Italian (`it`). English is the fallback. No external translation files.

Optional override: `bzipit.exe --lang=gl "C:\\folder"` (also `--lang=es`, `--lang=en`, `--lang=val`, etc.). Dragging folders onto the EXE requires no options.

## Behavior

Drag one or more folders onto `bzipit.exe`. It walks their subfolders, compressing each regular file separately to `<original name>.bz2` in the same directory, including files without extensions. It skips existing `.bz2` files, itself, and pre-existing destinations. Originals are preserved. If no folder is supplied, an error is displayed.

## Build from WSL

```bash
sudo apt install -y mingw-w64 binutils-mingw-w64-x86-64 build-essential
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

Result: `target/x86_64-pc-windows-gnu/release/bzipit.exe`.

## Verify

```bash
x86_64-w64-mingw32-objdump -p target/x86_64-pc-windows-gnu/release/bzipit.exe | grep 'DLL Name'
wrestool -l target/x86_64-pc-windows-gnu/release/bzipit.exe
```

Expect icon resource types 3 and 14. No third-party DLLs should be required. A real Windows build and runtime check are still recommended.

Valencian: Windows locale `ca-ES-valencia` is detected before the generic Catalan `ca` mapping. To force it, use `--lang=val` or `--lang=ca-ES-valencia`.


## Archivos y carpetas (arrastrar y soltar)

Se admiten archivos individuales, varias selecciones de archivos, carpetas y combinaciones de ambos. Cada archivo genera un `.bz2` separado en su misma ubicación; las carpetas se recorren recursivamente. Se omiten `.bz2`, el ejecutable y destinos ya existentes.

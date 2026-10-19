use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/bzipit.ico");
    let target = env::var("TARGET").expect("TARGET no definido");
    if !target.contains("windows") { return; }

    if target.ends_with("-gnu") {
        // En MinGW, enlazar directamente el objeto COFF generado por windres.
        // No depender de la deteccion de herramientas ni del enlace de winres.
        let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR no definido"));
        let ico = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("assets/bzipit.ico").canonicalize().expect("No se encuentra el icono");
        let rc = out.join("bzipit.rc");
        let obj = out.join("bzipit-res.o");
        let ico_path = ico.to_string_lossy().replace('\\', "/");
        fs::write(&rc, format!("1 ICON \"{}\"\n", ico_path)).expect("No se pudo escribir RC");
        let tool = if target.starts_with("x86_64") {
            "x86_64-w64-mingw32-windres"
        } else if target.starts_with("i686") {
            "i686-w64-mingw32-windres"
        } else {
            panic!("Destino MinGW no soportado: {target}");
        };
        let status = Command::new(tool)
            .args(["-i", rc.to_str().unwrap(), "-o", obj.to_str().unwrap(), "-O", "coff"])
            .status().unwrap_or_else(|e| panic!("No se pudo ejecutar {tool}: {e}"));
        assert!(status.success(), "windres ha fallado: {status}");
        // Cargo pasa este objeto al enlazador del ejecutable final.
        println!("cargo:rustc-link-arg={}", obj.display());
    } else if target.ends_with("-msvc") {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("assets/bzipit.ico");
        resource.set("FileDescription", "bzipit - Compresor individual BZIP2");
        resource.set("ProductName", "bzipit");
        resource.compile().expect("No se pudo incorporar el icono");
    }
}

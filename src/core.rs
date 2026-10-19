use bzip2::write::BzEncoder;
use bzip2::Compression;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Default)]
pub struct Stats { pub compressed: usize, pub skipped: usize, pub failed: usize }

// All messages are embedded in the executable; no external translation files.
pub struct Messages {
    pub summary: &'static str,
    pub compressed: &'static str, pub skipped: &'static str, pub errors: &'static str,
}

pub fn messages(lang: &str) -> Messages {
    let v = match lang {
        "es" => ["Pulsa INTRO para cerrar...", "ERROR: Arrastra uno o varios ARCHIVOS o CARPETAS sobre este ejecutable.", "No es un archivo ni una carpeta válida", "Procesando carpeta", "YA EXISTE", "ERROR DE RECORRIDO", "RESUMEN", "ERROR", "Idioma no disponible", "comprimidos", "omitidos", "errores"],
        "gl" => ["Preme INTRO para pechar...", "ERRO: Arrastra un ou varios FICHEIROS ou CARTAFOLES sobre este executable.", "Non é un ficheiro nin un cartafol válido", "Procesando cartafol", "XA EXISTE", "ERRO DE PERCORRIDO", "RESUMO", "ERRO", "Idioma non dispoñible", "comprimidos", "omitidos", "erros"],
        "val" => ["Prem RETORN per a tancar...", "ERROR: Arrossega un o més FITXERS o CARPETES sobre este executable.", "No és un fitxer ni una carpeta vàlids", "Processant carpeta", "JA EXISTIX", "ERROR DE RECORREGUT", "RESUM", "ERROR", "Idioma no disponible", "comprimits", "omesos", "errors"],
        "ca" => ["Prem RETORN per tancar...", "ERROR: Arrossega un o més FITXERS o CARPETES sobre aquest executable.", "No és un fitxer ni una carpeta vàlids", "Processant carpeta", "JA EXISTEIX", "ERROR DE RECORREGUT", "RESUM", "ERROR", "Idioma no disponible", "comprimits", "omesos", "errors"],
        "eu" => ["Sakatu ENTER ixteko...", "ERROREA: Arrastatu FITXATEGI edo KARPETA bat edo gehiago exekutagarri honetara.", "Ez da baliozko fitxategi edo karpeta", "Karpeta prozesatzen", "BADAGO LEHENDIK", "IBILBIDE-ERROREA", "LABURPENA", "ERROREA", "Hizkuntza ez dago erabilgarri", "konprimitu", "saltatu", "errore"],
        "fr" => ["Appuyez sur ENTRÉE pour fermer...", "ERREUR : Glissez un ou plusieurs FICHIERS ou DOSSIERS sur cet exécutable.", "Ce n’est ni un fichier ni un dossier valide", "Traitement du dossier", "EXISTE DÉJÀ", "ERREUR DE PARCOURS", "RÉSUMÉ", "ERREUR", "Langue non disponible", "compressés", "ignorés", "erreurs"],
        "pt" => ["Prima ENTER para fechar...", "ERRO: Arraste um ou mais FICHEIROS ou PASTAS para este executável.", "Não é um ficheiro nem uma pasta válida", "A processar pasta", "JÁ EXISTE", "ERRO DE PERCURSO", "RESUMO", "ERRO", "Idioma indisponível", "comprimidos", "ignorados", "erros"],
        "de" => ["Zum Schließen EINGABETASTE drücken...", "FEHLER: Ziehen Sie eine oder mehrere DATEIEN oder ORDNER auf diese Anwendung.", "Keine gültige Datei oder Ordner", "Ordner wird verarbeitet", "BEREITS VORHANDEN", "DURCHSUCHUNGSFEHLER", "ZUSAMMENFASSUNG", "FEHLER", "Sprache nicht verfügbar", "komprimiert", "übersprungen", "Fehler"],
        "it" => ["Premi INVIO per chiudere...", "ERRORE: Trascina uno o più FILE o CARTELLE su questo eseguibile.", "Non è un file o una cartella valida", "Elaborazione cartella", "GIÀ ESISTENTE", "ERRORE DI SCANSIONE", "RIEPILOGO", "ERRORE", "Lingua non disponibile", "compressi", "saltati", "errori"],
        _ => ["Press ENTER to close...", "ERROR: Drag one or more FILES or FOLDERS onto this executable.", "Not a valid file or folder", "Processing folder", "ALREADY EXISTS", "TRAVERSAL ERROR", "SUMMARY", "ERROR", "Unsupported language", "compressed", "skipped", "errors"],
    };
    Messages { summary:v[6], compressed:v[9], skipped:v[10], errors:v[11] }
}

pub fn normalized_lang(tag: &str) -> Option<&'static str> {
    let tag = tag.trim().to_ascii_lowercase().replace('_', "-");
    // Valencian uses the Catalan language code with the Valencian variant.
    // Windows commonly identifies it as ca-ES-valencia.
    if tag == "val" || tag == "va" || tag.starts_with("ca-es-valencia")
        || tag.starts_with("ca-valencia") || tag.starts_with("ca-es-valencian") {
        return Some("val");
    }
    let base = tag.split('-').next().unwrap_or("");
    match base {
        "es" => Some("es"), "en" => Some("en"), "gl" => Some("gl"),
        "ca" => Some("ca"), "eu" => Some("eu"), "fr" => Some("fr"),
        "pt" => Some("pt"), "de" => Some("de"), "it" => Some("it"),
        _ => None,
    }
}

#[cfg(windows)]
pub fn system_language() -> &'static str {
    // MUI_LANGUAGE_NAME = 0x8; Windows returns a double-NUL-terminated
    // list in preference order, e.g. "gl-ES\0es-ES\0en-US\0\0".
    #[link(name = "kernel32")]
    extern "system" {
        fn GetUserPreferredUILanguages(flags: u32, count: *mut u32, buffer: *mut u16, size: *mut u32) -> i32;
    }
    let (mut count, mut size) = (0u32, 0u32);
    let first = unsafe { GetUserPreferredUILanguages(0x8, &mut count, std::ptr::null_mut(), &mut size) };
    if first != 0 && size > 0 {
        let mut buffer = vec![0u16; size as usize];
        let ok = unsafe { GetUserPreferredUILanguages(0x8, &mut count, buffer.as_mut_ptr(), &mut size) };
        if ok != 0 {
            for name in buffer.split(|&unit| unit == 0).filter(|s| !s.is_empty()) {
                if let Ok(tag) = String::from_utf16(name) {
                    if let Some(lang) = normalized_lang(&tag) { return lang; }
                }
            }
        }
    }
    "en"
}

#[cfg(not(windows))]
pub fn system_language() -> &'static str {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(var) {
            if let Some(lang) = normalized_lang(value.split('.').next().unwrap_or("")) { return lang; }
        }
    }
    "en"
}

fn is_bz2(path: &Path) -> bool {
    path.extension().map(|ext| ext.eq_ignore_ascii_case("bz2")).unwrap_or(false)
}

fn output_path(source: &Path) -> PathBuf {
    let mut name: OsString = source.as_os_str().to_owned();
    name.push(".bz2");
    PathBuf::from(name)
}

fn compress_one(source: &Path, destination: &Path) -> io::Result<bool> {
    if destination.exists() { return Ok(false); }
    let mut temp = destination.as_os_str().to_owned();
    temp.push(format!(".{}.part", std::process::id()));
    let temp = PathBuf::from(temp);
    let result = (|| -> io::Result<()> {
        let mut input = File::open(source)?;
        let temp_file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
        let mut encoder = BzEncoder::new(temp_file, Compression::best());
        io::copy(&mut input, &mut encoder)?;
        let mut completed = encoder.finish()?;
        completed.flush()?;
        drop(completed);
        if destination.exists() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Destination already exists"));
        }
        fs::rename(&temp, destination)?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temp); }
    result.map(|()| true)
}



pub fn process_paths<F: FnMut(String)>(paths: Vec<PathBuf>, mut log: F) -> Stats {
    let executable = env::current_exe().and_then(fs::canonicalize).unwrap_or_default();
    let mut stats = Stats::default();
    for path in paths {
        if path.is_file() { process_entry(&path, &executable, &mut stats, &mut log); }
        else if path.is_dir() {
            for entry in WalkDir::new(&path).follow_links(false) {
                match entry {
                    Ok(e) if e.file_type().is_file() => process_entry(e.path(), &executable, &mut stats, &mut log),
                    Ok(_) => (),
                    Err(e) => { stats.failed += 1; log(format!("[ERROR] {e}")); }
                }
            }
        } else { stats.failed += 1; log(format!("[ERROR] {}", path.display())); }
    }
    stats
}
fn process_entry<F: FnMut(String)>(path: &Path, executable: &Path, stats: &mut Stats, log: &mut F) {
    if is_bz2(path) || fs::canonicalize(path).map(|p| p == executable).unwrap_or(false) {
        stats.skipped += 1; return;
    }
    let output = output_path(path);
    match compress_one(path, &output) {
        Ok(true) => { stats.compressed += 1; log(format!("[OK] {}", output.display())); }
        Ok(false) => { stats.skipped += 1; log(format!("[SKIP] {}", output.display())); }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists && output.exists() => {
            stats.skipped += 1; log(format!("[SKIP] {}", output.display()));
        }
        Err(e) => { stats.failed += 1; log(format!("[ERROR] {}: {e}", path.display())); }
    }
}

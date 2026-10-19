#![cfg_attr(windows, windows_subsystem = "windows")]
mod core;

#[cfg(windows)]
mod gui;

#[cfg(windows)]
fn main() { gui::run(); }

#[cfg(not(windows))]
fn main() {
    use std::{env, path::PathBuf};
    let paths: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() { eprintln!("Usage: bzipit <files or directories>"); return; }
    let lang = core::system_language();
    let stats = core::process_paths(paths, |line| println!("{line}"));
    let words = core::messages(lang);
    println!("{}: {}, {}: {}, {}: {}", words.compressed, stats.compressed, words.skipped, stats.skipped, words.errors, stats.failed);
}

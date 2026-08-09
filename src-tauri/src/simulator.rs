use std::path::{Path, PathBuf};
use sysinfo::System;

/// Process name of OP TCG Simulator (Unity build).
const SIMULATOR_PROCESS: &str = "OPTCGSim.exe";

/// Looks up the running simulator process once, returning its executable path
/// (`None` when it isn't running, or `Some(None)` shape flattened away when the
/// path can't be read — callers that only need liveness use `is_running`).
fn find_process() -> Option<(bool, Option<PathBuf>)> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes()
        .values()
        .find(|p| {
            p.name()
                .to_string_lossy()
                .eq_ignore_ascii_case(SIMULATOR_PROCESS)
        })
        .map(|p| (true, p.exe().map(|e| e.to_path_buf())))
}

/// True when the simulator is currently running.
pub fn is_running() -> bool {
    find_process().is_some()
}

fn decks_dir_of(install_dir: &Path) -> Option<String> {
    let decks = install_dir.join("Decks");
    decks
        .is_dir()
        .then(|| decks.to_string_lossy().to_string())
}

/// Best-effort discovery of the simulator's `Decks` folder, so the user
/// normally never has to browse for it by hand:
/// 1. next to the running process' executable,
/// 2. otherwise scanning the usual install locations for a `Builds_Windows`
///    folder (the simulator ships as `<version>_Windows/Builds_Windows/`).
pub fn autodetect_decks_folder() -> Option<String> {
    if let Some((_, Some(exe))) = find_process() {
        if let Some(dir) = exe.parent().and_then(decks_dir_of) {
            return Some(dir);
        }
    }

    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        for sub in ["Desktop", "Downloads", "Documents", "Games"] {
            roots.push(home.join(sub));
        }
        roots.push(home);
    }
    for drive in ["C:\\", "D:\\"] {
        roots.push(PathBuf::from(drive).join("Games"));
        roots.push(PathBuf::from(drive).join("OPTCGSim"));
    }

    for root in roots {
        if let Some(found) = scan_for_builds(&root, 2) {
            return Some(found);
        }
    }
    None
}

/// Resolves a folder the user picked by hand into the actual `Decks` folder,
/// so a `.deck` file only ever lands next to the ones the simulator already
/// reads. Handles the mistake that prompted this: picking the simulator's
/// install root (or its `Builds_Windows` folder) instead of `Decks` itself.
///
/// - `path` already IS a folder named `Decks` -> used as-is.
/// - `path` CONTAINS a `Decks` subfolder (install root, or `Builds_Windows`)
///   -> that subfolder is used instead.
/// - `path` sits one level above `Builds_Windows/Decks` (e.g. the versioned
///   `1.29a_Windows` folder) -> found by one extra level of search.
/// - nothing matches -> `None`, and the caller must ask the user to pick
///   again rather than silently writing files to the wrong place.
pub fn resolve_decks_folder(path: &str) -> Option<String> {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return None;
    }
    if dir
        .file_name()
        .map(|n| n.to_string_lossy().eq_ignore_ascii_case("Decks"))
        .unwrap_or(false)
    {
        return Some(dir.to_string_lossy().to_string());
    }
    if let Some(found) = decks_dir_of(dir) {
        return Some(found);
    }
    scan_for_builds(dir, 2)
}

/// Looks for `Builds_Windows/Decks` (or a direct `Decks` sibling of the exe)
/// under `dir`, descending at most `depth` levels to keep this cheap.
fn scan_for_builds(dir: &Path, depth: usize) -> Option<String> {
    if !dir.is_dir() {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name.eq_ignore_ascii_case("Builds_Windows") {
            if let Some(found) = decks_dir_of(&path) {
                return Some(found);
            }
        }
        if path.join(SIMULATOR_PROCESS).is_file() {
            if let Some(found) = decks_dir_of(&path) {
                return Some(found);
            }
        }
        subdirs.push(path);
    }

    if depth == 0 {
        return None;
    }
    subdirs
        .into_iter()
        .find_map(|sub| scan_for_builds(&sub, depth - 1))
}

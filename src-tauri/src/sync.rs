use crate::backend::{self, DeckSummary};
use crate::config::{AppConfig, SyncedDeckRecord};
use crate::deckfile;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone, Serialize)]
pub struct SyncReport {
    pub written: Vec<String>,
    pub deleted: Vec<String>,
    pub skipped_unchanged: usize,
    pub errors: Vec<String>,
    /// How many decks the account has, period — lets the UI tell "you have
    /// zero saved decks" apart from "everything was already in sync", which
    /// otherwise look identical (both leave written/deleted/errors empty).
    pub total_decks: usize,
}

/// Pushes every deck from the account into `config.simulator_decks_folder`,
/// one-way (Nakama Companion -> simulator). Only ever touches files this tool
/// created itself, tracked via `config.synced_decks` (deckId -> filename).
pub async fn run(config: &mut AppConfig, device_token: &str) -> Result<SyncReport, String> {
    let folder = config
        .simulator_decks_folder
        .clone()
        .ok_or_else(|| "no_folder".to_string())?;
    let folder = Path::new(&folder);
    if !folder.is_dir() {
        return Err(format!("folder_missing:{}", folder.display()));
    }

    let decks = backend::fetch_decks(&config.backend_api_url, device_token).await?;
    let mut report = SyncReport {
        total_decks: decks.len(),
        ..SyncReport::default()
    };

    let mut taken_filenames: HashSet<String> = config
        .synced_decks
        .values()
        .map(|r| r.filename.clone())
        .collect();

    let mut still_present: HashMap<String, SyncedDeckRecord> = HashMap::new();

    for deck in &decks {
        match sync_one_deck(folder, deck, config, &mut taken_filenames) {
            Ok((Some(record), warnings)) => {
                report.written.push(record.filename.clone());
                still_present.insert(deck.id.clone(), record);
                report.errors.extend(warnings);
            }
            Ok((None, _)) => {
                // Unchanged: keep the existing record as-is.
                if let Some(existing) = config.synced_decks.get(&deck.id) {
                    still_present.insert(deck.id.clone(), existing.clone());
                }
                report.skipped_unchanged += 1;
            }
            Err(e) => report.errors.push(e),
        }
    }

    // Decks that used to be synced but no longer exist on the account: remove
    // the local file (only ones we created, never anything else in the folder).
    let current_ids: HashSet<&String> = decks.iter().map(|d| &d.id).collect();
    for (deck_id, record) in config.synced_decks.iter() {
        if !current_ids.contains(deck_id) {
            let path = folder.join(&record.filename);
            if path.is_file() {
                if fs::remove_file(&path).is_err() {
                    report
                        .errors
                        .push(format!("delete_failed:{}", record.filename));
                    continue;
                }
            }
            report.deleted.push(record.filename.clone());
        }
    }

    config.synced_decks = still_present;
    Ok(report)
}

fn sync_one_deck(
    folder: &Path,
    deck: &DeckSummary,
    config: &AppConfig,
    taken_filenames: &mut HashSet<String>,
) -> Result<(Option<SyncedDeckRecord>, Vec<String>), String> {
    let existing = config.synced_decks.get(&deck.id);

    // Matching updated_at alone isn't enough: if the target folder changed
    // (the bug that prompted this check — a user had originally pointed the
    // app at the wrong folder), the recorded filename may not exist there
    // anymore, and skipping the write would leave the deck missing with no
    // indication anything was wrong.
    let unchanged = existing
        .map(|r| r.updated_at == deck.updated_at && folder.join(&r.filename).is_file())
        .unwrap_or(false);

    if unchanged {
        return Ok((None, Vec::new()));
    }

    let cards = backend::parse_cards(deck)?;
    let rendered = deckfile::render(deck, &cards)?;

    let mut warnings = Vec::new();
    if !rendered.skipped.is_empty() {
        warnings.push(format!(
            "unresolved_cards:{}|{}",
            deck.name,
            rendered.skipped.len()
        ));
    }

    // Renamed deck (or first sync): pick a fresh filename, freeing the old one.
    let filename = if let Some(prev) = existing {
        taken_filenames.remove(&prev.filename);
        let candidate = deckfile::sanitize_filename(&deck.name, taken_filenames);
        if candidate != prev.filename {
            let old_path = folder.join(&prev.filename);
            if old_path.is_file() {
                let _ = fs::remove_file(&old_path);
            }
        }
        candidate
    } else {
        deckfile::sanitize_filename(&deck.name, taken_filenames)
    };
    taken_filenames.insert(filename.clone());

    let path = folder.join(&filename);
    fs::write(&path, rendered.content).map_err(|_| format!("write_failed:{}", path.display()))?;

    Ok((
        Some(SyncedDeckRecord {
            filename,
            updated_at: deck.updated_at,
        }),
        warnings,
    ))
}

use crate::backend::DeckSummary;
use std::collections::HashMap;

/// Nakama Companion card ids aren't always simulator-compatible codes as-is:
/// leak/spoiler cards (unreleased at save time) are keyed `leak:<...>` — see
/// src/app/builder/editor/page.tsx exportAsText, the site's own "copy as
/// text" feature, which resolves the exact same ambiguity for the same
/// reason. Two shapes appear under that prefix:
///   - `leak:OP17-058[_variant]` — a real code assigned since, recoverable.
///   - `leak:Bt7lJ6qilNISJMpsiUxz` / `leak:external:OP17-999_1` — an opaque
///     Firestore doc id or an external-source id with no official code at
///     all, because the card had none yet when the deck was saved.
///
/// The site's clipboard export falls back to the card's display name for the
/// second case ("4x Some Card (no code yet)") because a human is reading it.
/// A `.deck` file has no such option — the simulator only understands real
/// codes — so unresolvable cards are dropped and reported instead of writing
/// a line the simulator can't parse.
fn base_code(card_id: &str) -> String {
    let stripped = if card_id.len() >= 5 && card_id[..5].eq_ignore_ascii_case("leak:") {
        &card_id[5..]
    } else {
        card_id
    };
    // Variant/print suffix, e.g. `OP17-058_1` for an alternate art.
    let before_variant = stripped.split('_').next().unwrap_or(stripped);
    before_variant.trim().to_ascii_uppercase()
}

/// Matches the site's `hasCode` check (`/^[A-Z0-9]+-\d+$/i`): letters/digits,
/// one dash, digits — e.g. `OP17-058`, `ST34-004`. Anything else (opaque
/// Firestore ids, `EXTERNAL:...`) isn't a real card code.
fn is_card_code(s: &str) -> bool {
    match s.split_once('-') {
        Some((left, right)) => {
            !left.is_empty()
                && !right.is_empty()
                && left.chars().all(|c| c.is_ascii_alphanumeric())
                && right.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

pub struct RenderedDeck {
    pub content: String,
    /// Original card ids that had no resolvable code and were left out.
    pub skipped: Vec<String>,
}

/// Renders a deck as the simulator's plain-text `.deck` format:
/// leader first as `1x<leaderId>`, then every other card as `<qty>x<cardId>`,
/// one per line, cards sorted by id for a deterministic/diffable output.
///
/// Errs if the LEADER has no resolvable code: a deck can't be built in the
/// simulator without one, so there is nothing useful to write.
pub fn render(deck: &DeckSummary, cards: &HashMap<String, i64>) -> Result<RenderedDeck, String> {
    let leader_code = base_code(&deck.leader_id);
    if !is_card_code(&leader_code) {
        return Err(format!("unreleased_leader:{}", deck.name));
    }

    let mut lines = Vec::with_capacity(cards.len() + 1);
    lines.push(format!("1x{leader_code}"));
    let mut skipped = Vec::new();

    let mut entries: Vec<(&String, &i64)> = cards.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    for (card_id, qty) in entries {
        if card_id == &deck.leader_id {
            continue;
        }
        let code = base_code(card_id);
        if !is_card_code(&code) {
            skipped.push(card_id.clone());
            continue;
        }
        lines.push(format!("{qty}x{code}"));
    }
    lines.push(String::new());

    Ok(RenderedDeck {
        content: lines.join("\n"),
        skipped,
    })
}

const RESERVED_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// Marks a .deck file as having come from Nakama Companion, so it's obvious in
/// the simulator's deck list which files are sync-managed (and, in a shared
/// screenshot/folder listing, that the source is legitimate — not an
/// impersonation).
const SOURCE_SUFFIX: &str = " - Nakama";

/// Sanitizes a deck name into a filesystem-safe `.deck` filename, resolving
/// collisions against `taken` (other filenames already used in this sync pass).
pub fn sanitize_filename(deck_name: &str, taken: &std::collections::HashSet<String>) -> String {
    let mut base: String = deck_name
        .chars()
        .map(|c| if RESERVED_CHARS.contains(&c) { '_' } else { c })
        .collect();
    base = base.trim().trim_end_matches('.').to_string();
    if base.is_empty() {
        base = "deck".to_string();
    }
    const MAX_BASE_LEN: usize = 250 - SOURCE_SUFFIX.len(); // leaves room for the suffix, ".deck" and a numeric collision marker
    if base.len() > MAX_BASE_LEN {
        base.truncate(MAX_BASE_LEN);
    }
    base.push_str(SOURCE_SUFFIX);

    let mut candidate = format!("{base}.deck");
    let mut n = 2;
    while taken.contains(&candidate) {
        candidate = format!("{base} ({n}).deck");
        n += 1;
    }
    candidate
}

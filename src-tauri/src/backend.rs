use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeckSummary {
    pub id: String,
    pub name: String,
    #[serde(rename = "leaderId")]
    pub leader_id: String,
    #[serde(rename = "cardsJson")]
    pub cards_json: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

/// `cardsJson` is a JSON-encoded string (cardId -> quantity), matching the
/// shape saved by the deck builder (src/app/api/decks/route.ts DeckSummary).
pub fn parse_cards(deck: &DeckSummary) -> Result<HashMap<String, i64>, String> {
    serde_json::from_str(&deck.cards_json)
        .map_err(|_| format!("invalid_cards:{}", deck.name))
}

pub async fn fetch_decks(backend_api_url: &str, device_token: &str) -> Result<Vec<DeckSummary>, String> {
    let url = format!("{}/desktop/decks", backend_api_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(device_token)
        .send()
        .await
        .map_err(|e| format!("network_error:{e}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("device_revoked".to_string());
    }
    if !status.is_success() {
        return Err(format!("backend_error:{status}"));
    }

    resp.json::<Vec<DeckSummary>>()
        .await
        .map_err(|e| format!("invalid_response:{e}"))
}

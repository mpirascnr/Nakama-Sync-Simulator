//! Secure storage for the device token issued by the pairing flow.
//!
//! The token is a bearer credential for this user's decks, so it lives in the
//! Windows Credential Manager rather than next to the JSON config on disk.

const KEYRING_SERVICE: &str = "com.nakamacompanion.decksync";
const KEYRING_ACCOUNT: &str = "device-token";

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| format!("keyring_unavailable:{e}"))
}

pub fn store(token: &str) -> Result<(), String> {
    entry()?
        .set_password(token)
        .map_err(|e| format!("keyring_write_failed:{e}"))
}

pub fn load() -> Option<String> {
    entry().ok()?.get_password().ok()
}

pub fn clear() {
    if let Ok(entry) = entry() {
        let _ = entry.delete_credential();
    }
}

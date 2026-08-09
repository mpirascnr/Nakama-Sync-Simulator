//! Account linking for the desktop app.
//!
//! The app deliberately never talks to Firebase. App Check is enforced on this
//! project's Authentication API and Firebase publishes no App Check provider
//! for native desktop apps — only web, iOS and Android — so any Identity
//! Toolkit call from here is rejected with an invalid-App-Check error, whether
//! the user signs in with a password or with Google.
//!
//! Instead the user authorizes this machine from their browser on
//! nakamacompanion.com, where App Check is satisfied normally and both sign-in
//! methods already work. The browser sends back a short-lived claim code over
//! loopback, which this module trades for a long-lived device token issued by
//! our own backend. See backend/app/desktop/desktop.py.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::Duration;

/// How long we wait for the user to finish authorizing in their browser.
const AUTHORIZE_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Deserialize)]
struct ClaimResponse {
    token: String,
}

/// Wording for the browser page shown after the redirect, supplied by the UI
/// so it follows the user's chosen language.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BrowserPageCopy {
    pub done_title: String,
    pub done_body: String,
    pub failed_title: String,
    pub failed_body: String,
}

fn random_state() -> String {
    let bytes: [u8; 24] = rand::thread_rng().gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn closing_page(heading: &str, body: &str) -> String {
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\
         <title>Nakama Companion</title><style>\
         body{{font-family:system-ui,sans-serif;background:#111;color:#f0f0f0;\
         display:grid;place-items:center;height:100vh;margin:0;text-align:center}}\
         h1{{font-size:20px;margin:0 0 8px}}p{{color:#a0a0a0;font-size:14px;margin:0}}\
         </style></head><body><div><h1>{heading}</h1><p>{body}</p></div></body></html>"
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    )
}

fn urldecode(input: &str) -> String {
    let spaced = input.replace('+', " ");
    let bytes = spaced.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&spaced[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn urlencode(input: &str) -> String {
    input
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Blocks until the browser hits the loopback redirect, returning the claim
/// code once `state` matches what we sent.
fn wait_for_code(
    listener: TcpListener,
    expected_state: &str,
    copy: &BrowserPageCopy,
) -> Result<String, String> {
    listener
        .set_nonblocking(false)
        .map_err(|e| format!("loopback_failed:{e}"))?;

    let deadline = std::time::Instant::now() + AUTHORIZE_TIMEOUT;

    for incoming in listener.incoming() {
        if std::time::Instant::now() > deadline {
            return Err("pairing_timeout".to_string());
        }
        let mut stream = match incoming {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut request_line = String::new();
        if BufReader::new(&stream).read_line(&mut request_line).is_err() {
            continue;
        }

        let path = request_line.split_whitespace().nth(1).unwrap_or("");
        // Browsers ask for /favicon.ico alongside the real redirect; answering
        // that as a failure would abort a perfectly good pairing.
        if path.starts_with("/favicon.ico") {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            continue;
        }

        let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
        let mut code = None;
        let mut state = None;
        for pair in query.split('&') {
            match pair.split_once('=') {
                Some(("code", v)) => code = Some(urldecode(v)),
                Some(("state", v)) => state = Some(urldecode(v)),
                _ => {}
            }
        }

        let outcome = if state.as_deref() != Some(expected_state) {
            // Someone else's redirect reached our port: never redeem this.
            Err("state_mismatch".to_string())
        } else if let Some(code) = code {
            Ok(code)
        } else {
            Err("no_claim_code".to_string())
        };

        let page = match &outcome {
            Ok(_) => closing_page(&copy.done_title, &copy.done_body),
            Err(_) => closing_page(&copy.failed_title, &copy.failed_body),
        };
        let _ = stream.write_all(page.as_bytes());
        let _ = stream.flush();
        return outcome;
    }

    Err("pairing_timeout".to_string())
}

/// Runs the whole linking flow and returns the device token to persist.
pub async fn link_account<F>(
    site_base_url: &str,
    api_base_url: &str,
    copy: BrowserPageCopy,
    open_browser: F,
) -> Result<String, String>
where
    F: FnOnce(String) -> Result<(), String>,
{
    // Port 0 = let the OS pick a free one.
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| format!("loopback_failed:{e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("loopback_failed:{e}"))?
        .port();
    let state = random_state();

    let authorize_url = format!(
        "{}/desktop-link?port={port}&state={}",
        site_base_url.trim_end_matches('/'),
        urlencode(&state),
    );
    open_browser(authorize_url)?;

    // The listener blocks, so keep it off the async runtime's threads.
    let state_for_wait = state.clone();
    let claim_code = tauri::async_runtime::spawn_blocking(move || {
        wait_for_code(listener, &state_for_wait, &copy)
    })
    .await
    .map_err(|e| format!("loopback_failed:{e}"))??;

    redeem(api_base_url, &claim_code).await
}

async fn redeem(api_base_url: &str, claim_code: &str) -> Result<String, String> {
    let url = format!("{}/desktop/claim", api_base_url.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({ "claimCode": claim_code }))
        .send()
        .await
        .map_err(|e| format!("network_error:{e}"))?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        #[derive(Deserialize)]
        struct ApiError {
            error: Option<String>,
        }
        let detail = serde_json::from_str::<ApiError>(&body)
            .ok()
            .and_then(|e| e.error)
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(format!("claim_failed:{detail}"));
    }

    serde_json::from_str::<ClaimResponse>(&body)
        .map(|r| r.token)
        .map_err(|e| format!("invalid_response:{e}"))
}

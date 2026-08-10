# Nakama Companion: Deck Sync

Windows desktop app (Tauri) that syncs decklists saved on
[Nakama Companion](https://nakamacompanion.com) into **OP TCG Simulator**
(Batsu's PC simulator), writing `.deck` files into its `Decks` folder.

**One-way** sync: the site is the source of truth, the simulator the
destination. Decks are created and edited in the site's deck builder.

## How it works

1. The user links their computer to their account by authorizing it from
   their browser (see [Authentication](#authentication-browser-pairing)).
2. The app locates the simulator's `Decks` folder (automatically, or by
   picking it manually).
3. From there it syncs: on app startup, on the simulator launching, and on
   demand via the "Sync now" button.

The app stays in the system tray: closing the window hides it, it doesn't
quit the process.

## `.deck` format

Plain text, one card per line, leader first:

```
1xOP06-022
4xOP01-033
4xOP05-101
```

The filename is `<DeckName> - Nakama.deck`. The suffix makes it obvious at a
glance, in the simulator's deck list, which files are sync-managed, and it
identifies the source in a shared screenshot or folder. Cards are sorted by
id, so the output is deterministic.

## User file safety

The app only ever touches files it created itself. The `deckId -> filename`
mapping lives in `config.json` (`synced_decks`): a deck renamed on the site
renames the file, a deck deleted on the site deletes the file, but a
`.deck` file the app never wrote is never touched.

## Authentication: browser pairing

The app **never talks to Firebase directly**. App Check is enforced on this
project's Authentication API, and Firebase publishes no App Check provider
for native desktop apps (only web, iOS and Android exist), so any call to
Identity Toolkit from here is rejected with
`Firebase App Check token is invalid`, whether with a password or with
Google.

So the user authorizes the computer **from their own browser**, where App
Check is satisfied normally and the site's logins already work:

```
browser (authenticated session, App Check ok)
   POST /api/desktop/pair    -> single-use claim code, valid 2 minutes
desktop app (the loopback redirect carries only the claim code)
   POST /api/desktop/claim   -> long-lived device token
   GET  /api/desktop/decks   -> Authorization: Bearer <device token>
```

The device token is issued and verified by the **Nakama Companion backend**,
not Firebase (server-side implementation, not published in this repo). Only
its SHA-256 is stored, so a database read isn't reproducible as a credential.
The claim code exists so that it, not the token, is what crosses the
browser redirect. That way the long-lived credential never ends up in a URL
or in browser history, the same reason OAuth flows carry a code rather than a
token.

On the app side the token lives in the **Windows Credential Manager**
(`credentials.rs`), never on disk in clear text. It's revocable per device
(`DELETE /desktop/devices/{id}`).

All networking lives in Rust, not in the webview: no token ever reaches the
JS context, and CORS doesn't apply.

## How it reaches the data

Nakama Companion's application backend has no direct public URL (not
reachable from a user's PC). The app therefore calls the public routes
exposed at `https://nakamacompanion.com/api/desktop/*`, which forward
internally to the backend, an implementation not published in this repo.

## Languages

English and Italian ([`src/i18n.ts`](src/i18n.ts)). The language is detected
from the system on first launch and can be changed from Settings.

Rust **never returns user-facing text**: it returns codes
(`device_revoked`, `write_failed:<detail>`) that `translateError()`
translates. Adding a language only needs a new dictionary in `i18n.ts`;
missing keys fall back to English.

## Development

```bash
npm install
npm run tauri dev
```

Windows NSIS installer:

```bash
npm run tauri build
```

Output lands in `src-tauri/target/release/bundle/nsis/`.

### Backend URL

Configurable from **Settings → Advanced**. The value is saved in
`config.json` and **must be a public URL**: the app runs on the user's PC, so
an internal hostname of the hosting provider (reachable only from inside its
own private network) wouldn't work from outside.

### Icons

Regenerated from any square logo:

```bash
npm run tauri icon path/to/square-logo.png
```

(the logo must be squared first: `tauri icon` rejects non-square images).

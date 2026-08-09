# Nakama Companion — Deck Sync

App desktop Windows (Tauri) che sincronizza le decklist salvate su
[Nakama Companion](https://nakamacompanion.com) dentro **OP TCG Simulator**
(il simulatore PC di Batsu), scrivendo i file `.deck` nella sua cartella
`Decks`.

Sincronizzazione **one-way**: il sito è la fonte di verità, il simulatore la
destinazione. I mazzi si creano e si modificano nel deck builder del sito.

## Come funziona

1. L'utente collega il computer al proprio account autorizzandolo dal browser
   (vedi [Autenticazione](#autenticazione-pairing-dal-browser)).
2. L'app individua la cartella `Decks` del simulatore (in automatico, o
   scegliendola a mano).
3. Da lì in poi sincronizza: a intervalli regolari, **all'avvio del
   simulatore**, e su richiesta col pulsante "Sincronizza ora".

L'app resta nella barra delle applicazioni (tray): chiudere la finestra la
nasconde, non chiude il processo.

## Formato `.deck`

Testo semplice, una riga per carta, leader per primo:

```
1xOP06-022
4xOP01-033
4xOP05-101
```

### Carte leak/spoiler senza codice ufficiale

Gli id delle carte non sono sempre codici compatibili col simulatore così
come sono: le carte leak (non ancora ufficiali quando il mazzo è stato
salvato) sono identificate come `leak:<...>` — stessa ambiguità già risolta
lato sito nel "copia come testo" del deck builder. Due casi:

- `leak:OP17-058[_variante]` — codice reale assegnato nel frattempo,
  recuperabile: si toglie il prefisso `leak:` e il suffisso variante.
- `leak:Bt7lJ6qilNISJMpsiUxz` / `leak:external:OP17-999_1` — un id opaco di
  Firestore o una fonte esterna, senza alcun codice ufficiale perché la carta
  non ne aveva ancora uno quando il mazzo è stato salvato.

Il "copia testo" del sito, nel secondo caso, ripiega sul nome della carta
("4x Nome Carta (no code yet)") perché lo legge una persona. Un file
`.deck` non ha questa possibilità — il simulatore capisce solo codici reali —
quindi le carte non risolvibili vengono **scartate e segnalate**
nel report di sync, invece di scrivere una riga che il simulatore non può
interpretare. Logica in
[`src-tauri/src/deckfile.rs`](src-tauri/src/deckfile.rs) (`base_code`/
`is_card_code`) — stessa identica logica implementata anche lato sito, per il
percorso di sync via browser.

Il nome del file è `<NomeMazzo> - Nakama.deck` — il suffisso rende
riconoscibile a colpo d'occhio, nella lista mazzi del simulatore, quali file
sono gestiti dalla sync (e in uno screenshot/cartella condivisa, che la fonte
è legittima). Le carte sono ordinate per id, così l'output è deterministico.

## Sicurezza dei file dell'utente

L'app tocca **solo** i file che ha creato lei. Il mapping `deckId -> filename`
vive in `config.json` (`synced_decks`): un mazzo rinominato sul sito fa
rinominare il file, un mazzo cancellato fa cancellare il file — ma un `.deck`
che l'app non ha mai scritto non viene mai toccato.

## Autenticazione: pairing dal browser

L'app **non parla mai con Firebase**. App Check è enforced sull'API
Authentication di questo progetto, e Firebase non pubblica alcun provider App
Check per app desktop native (esistono solo web, iOS e Android): qualunque
chiamata a Identity Toolkit da qui viene rifiutata con
`Firebase App Check token is invalid`, sia con password che con Google.

Quindi l'utente autorizza il computer **dal proprio browser**, dove App Check
è soddisfatto normalmente e i login del sito funzionano già:

```
browser (sessione autenticata, App Check ok)
   POST /api/desktop/pair    -> claim code monouso, valido 2 minuti
app desktop (il redirect su loopback porta solo il claim code)
   POST /api/desktop/claim   -> device token a vita lunga
   GET  /api/desktop/decks   -> Authorization: Bearer <device token>
```

Il device token è emesso e verificato dal **backend di Nakama Companion**, non
da Firebase (implementazione lato server, non pubblicata in questo repo). Di
esso si salva solo lo SHA-256, quindi una lettura del database non è
riproducibile come credenziale. Il claim code esiste
perché sia lui — e non il token — a passare per il redirect del browser: così
la credenziale a vita lunga non finisce mai in un URL o nella cronologia, lo
stesso motivo per cui OAuth fa viaggiare un code e non un token.

Lato app il token sta nel **Credential Manager di Windows**
(`credentials.rs`), mai su disco in chiaro. È revocabile per singolo
dispositivo (`DELETE /desktop/devices/{id}`).

Tutto il networking vive in Rust, non nel webview: nessun token raggiunge il
contesto JS, e CORS non si applica.

## Come raggiunge i dati

Il backend applicativo di Nakama Companion non ha un URL pubblico diretto
(non raggiungibile dal PC di un utente). L'app chiama quindi le route
pubbliche esposte su `https://nakamacompanion.com/api/desktop/*`, che
inoltrano internamente al backend — implementazione non pubblicata in questo
repo.

## Lingue

Inglese e italiano ([`src/i18n.ts`](src/i18n.ts)). La lingua viene rilevata dal
sistema al primo avvio ed è cambiabile da Impostazioni.

Rust **non restituisce mai testo destinato all'utente**: restituisce codici
(`device_revoked`, `write_failed:<dettaglio>`) che `translateError()`
traduce. Per aggiungere una lingua basta un nuovo dizionario in `i18n.ts`;
le chiavi mancanti ricadono sull'inglese.

## Sviluppo

```bash
npm install
npm run tauri dev
```

Installer NSIS per Windows:

```bash
npm run tauri build
```

L'output finisce in `src-tauri/target/release/bundle/nsis/`.

### URL del backend

Configurabile da **Impostazioni → Avanzate**. Il valore va salvato in
`config.json` e **deve essere un URL pubblico**: l'app gira sul PC
dell'utente, quindi un hostname interno del provider di hosting (raggiungibile
solo dentro la sua rete privata) non funzionerebbe da fuori.

### Icone

Rigenerate da un logo quadrato a piacere:

```bash
npm run tauri icon path/to/logo-quadrato.png
```

(il logo va reso quadrato prima — `tauri icon` rifiuta immagini non quadrate).

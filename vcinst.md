# Analog-Vibe — Regole Architetturali & Metodo Operativo

Questo documento definisce le **regole architetturali inderogabili** e il **metodo operativo obbligatorio** per lo sviluppo dell'intera suite Analog-Vibe. È volutamente snello per preservare il contesto di lavoro ed evitare ridondanze.

---

## 1. Metodo Operativo Obbligatorio

Ogni agente o sviluppatore che lavora su questo repository DEVE seguire rigorosamente questo ciclo operativo:

```
┌────────────────────────────────────────────────────────────────────────┐
│                      FLUSSO OPERATIVO OBBLIGATORIO                     │
├────────────────────────────────────────────────────────────────────────┤
│ 1. Leggi TODO.md          → Identifica il task attivo a massima priorità│
│ 2. Esegui il task attivo  → Sviluppa codice, test unitari e doc         │
│ 3. Spunta [x] in TODO.md  → Marca il task completato                   │
│ 4. Aggiorna STATE.md      → Riassumi lo stato operativo aggiornato  
│ 5.Aggiorna i file di documentazione e il file readme.txt inerente       |
└────────────────────────────────────────────────────────────────────────┘
```

### Riferimenti Documentali
- **Coda dei Task & Priorità:** [`TODO.md`](file:///home/greg/Documenti/analog%20vibes/TODO.md)
- **Catalogo Plugin & Roadmap Post-Analog-Vibe:** [`docs/PLUGIN_CATALOG.md`](file:///home/greg/Documenti/analog%20vibes/docs/PLUGIN_CATALOG.md)
- **Stato Operativo Corrente:** [`STATE.md`](file:///home/greg/Documenti/analog%20vibes/STATE.md)
- **Architettura Tecnica Dettagliata:** [`ARCHITECTURE.md`](file:///home/greg/Documenti/analog%20vibes/ARCHITECTURE.md)
- **Dossier Ricerca Hardware:** `docs/specs/outboard_hardware_research.md`
- **Fisica dei Componenti Elettronici:** `docs/specs/Guida_Componenti_Audio_Fisica_Acustica.pdf`
- **Schede Singoli Plugin/Moduli:** `docs/plugins/<id>.txt` e `docs/modules/<id>.txt`

---

## 2. Regole Architetturali Inderogabili

### 2.1 Sicurezza del Codice & Memory Safety
- **`#![forbid(unsafe_code)]` tassativo:** Vietato qualsiasi blocco `unsafe` nell'intero codebase (`core-dsp`, `plugin-support`, `greg_gui_core`, tutti i plugin e i tool di supporto).
- **Nessun panic nel thread audio:** È severamente vietato l'uso di `.unwrap()`, `.expect()`, `panic!()` o indicizzazioni di slice non protette all'interno del percorso audio `process()`.
- **Sanitizzazione numerica continua:** Protezione rigorosa contro `NaN` e `Inf`, sanitizzati a `0.0`. Ogni plugin deve includere il modulo `PanicProtection` per gestire degradazioni anomale senza crash della DAW.

### 2.2 Vincoli Real-Time nel Percorso Audio (`process()`)
- **Zero allocazioni heap:** Nessun `Vec::new()`, `Box::new()`, `clone()` di strutture non-Copy, stringhe o ridimensionamenti di buffer in `process()`. Tutta la memoria (buffer interni, linee di ritardo, tabelle FFT, stati dei filtri) deve essere pre-allocata in `initialize()` o `prepare()`.
- **Zero lock o mutex bloccanti:** Nessun `std::sync::Mutex` bloccante o operazione di sincronizzazione nel thread audio. I dati telemetrici e i parametri verso la GUI devono essere scambiati tramite primitive lock-free o atomici (`AtomicU64`, `AtomicU32`, `AtomicBool`, `Ordering::Relaxed`).
- **Zero chiamate di sistema o I/O:** Nessun accesso al filesystem, log su file, allocazioni o chiamate di rete nel thread audio.
- **Gestione blocchi arbitrari:** Elaborazione corretta per qualsiasi dimensione di blocco fornita dall'host (`1..max_block_size`), inclusi buffer irregolari o piccoli.

### 2.3 Precisione di Calcolo & Silenzio a Riposo
- **Elaborazione interna a 64-bit (`f64`):** Tutti i calcoli DSP, coefficienti di filtri, stadi non lineari, integratori e modelli fisici devono operare internamente in precisione doppia (`f64`). La conversione da/verso `f32` (formato standard host CLAP/VST3) avviene esclusivamente ai confini di `process()`.
- **Silenzio digitale assoluto a riposo:** Con segnale in ingresso nullo (`0.0`), l'uscita deve essere rigorosamente `0.0`. Nessun ronzio, rumore o oscillazione autonoma consentiti a riposo. I generatori di rumore analogico/ripple di alimentazione devono essere tassativamente **disattivati (`OFF`) di default**.

### 2.4 Routing Dinamico Multicanale (`Mono → 7.1`)
- **Supporto universale 1..8 canali:** Ogni plugin deve supportare dinamicamente da 1 a 8 canali indipendenti:
  - Mono (1 ch), Stereo (2 ch), LCR (3 ch), Quad (4 ch), 5.0 (5 ch), 5.1 (6 ch), 7.0 (7 ch), 7.1 (8 ch).
- **Isolamento e linking:** Gestione dello stato DSP per-canale isolata tramite `RoutingMatrix`. Lo stereo-link (dove previsto) deve calcolare il massimo key dinamico su tutti i canali attivi `0..channel_count` e applicarlo uniformemente a tutti i canali linkati.

### 2.5 Catena Analogica Modellata & Trasformatori
- **Modellazione fisica dei componenti:** Ogni plugin analogico DEVE integrare all'ingresso e all'uscita (e negli stadi intermedi rilevanti):
  - `TransformerHysteresis` (modello isteretico Jiles-Atherton con saturazione magnetica).
  - `AcCoupling` (disaccoppiamento DC e bias shift asimmetrico).
  - `AnalogVariance` (deriva termica asincrona aggiornata a *block-rate*, zero `sin()` per campione, con rumore/ripple `OFF` di default).
  - Stadi non lineari dedicati (`TubeTriode`, `TubePentode`, `SolidStateClassA`, `SolidStateFet`, `DiscreteOpAmp`, `PsuSag`, `MillerRollOff`).
- **Controlli utente (+ ADV):** Esporre nel pannello avanzato i parametri `Transformer / Iron` (0%..100%+) e `Analog Drift`.
- **Eccezione utility digitali:** I plugin digitali puri (`greg_phase`, `greg_ms_encoder`, `greg_ms_decoder`, `greg_surgical_eq`, `greg_leveler`, `greg_limiter`) rimangono trasparenti di default.

### 2.6 Allineamento di Latenza & Bypass Trasparente
- **Dichiarazione esatta della latenza:** Ogni plugin che introduce ritardo (oversampling FIR, lookahead, convoluzione partizionata) DEVE dichiarare all'host la latenza esatta in campioni tramite `latency_samples(&self) -> u32`.
- **Bypass click-free:** Il bypass interno del plugin deve eseguire una dissolvenza incrociata morbida (crossfade di 5 ms) e mantenere allineata la compensazione di ritardo tra percorso wet e dry.

### 2.7 Doppia Licenza & Export Multi-Formato
- **Dual Licensing:** Il codice è licenziato sotto **GNU GPLv3** per uso e derivati open-source, con disponibilità di licenza commerciale separata per prodotti proprietari/chiusi.
- **Export obbligatorio:** Ogni plugin deve utilizzare la macro condivisa `export_greg_plugin!(PluginStruct, "nome-url")` da `plugin_export.rs`, esponendo nativamente sia **CLAP** che **VST3**.
- **Compatibilità LV2 e AUv2:** La generazione dei bundle LV2 e Audio Unit avviene tramite `scripts/package_all_formats.sh` basato su `clap-wrapper`.

### 2.8 Fallback GUI 2D Funzionale
- Ogni plugin del workspace include obbligatoriamente un editor grafico 2D leggero e reattivo basato su `greg_gui_core::create_greg_editor` (Main Panel con controlli principali, pannello a scomparsa `+ ADV` per parametri avanzati e modellazione analogica, meter di livello Input/Output e Gain Reduction).

---

## 3. Standard Obbligatorio di Commento del Codice

Ogni file sorgente `.rs` deve essere commentato secondo questo standard (con commenti scritti in **inglese** per leggibilità internazionale):

1. **Intestazione modulo (`//!`):** All'inizio di ogni file, descrivere lo scopo del modulo, la topologia circuitale/algoritmica emulata e i moduli `core-dsp` impiegati.
2. **Doc-comment pubblici (`///`):** Ogni `struct`, `enum`, campo di parametri (`Params`) e funzione pubblica deve riportare:
   - Descrizione dello scopo.
   - Unità di misura esatta (es. `dB`, `ms`, `Hz`, `samples`, `ratio`).
   - Garanzie real-time (es. zero heap allocations, block-rate execution).
3. **Commenti matematici DSP (`//`):** Ogni calcolo non banale in `initialize()` o `process()` (coefficienti filtri, costanti d'attacco/rilascio, curve non lineari, isteresi) deve spiegare la motivazione fisica, circuitale o acustica.
4. **Divieto di commenti ovvi:** Vietato commentare operazioni banali (es. `x += 1; // increment x`). Commentare solo il **perché fisico o matematico**.

# TODO — Coda Operativa di Sviluppo Analog-Vibe

> **Metodo Operativo Obbligatorio per l'Agente:**
> `1. Leggi TODO.md -> 2. Consulta Guida_Componenti_Audio_Fisica_Acustica.pdf e outboard_hardware_research.md -> 3. Esegui il task attivo -> 4. Spunta [x] in TODO.md -> 5. Aggiorna STATE.md -> 6. Aggiorna documentazione e README.txt`

---

## Task Immediati (Ordinati per Priorità)

### [x] P0: Modulo `dsp_graph.rs` & PDC Compensato (`core-dsp`)
- [x] Creare `core-dsp/src/core_dsp_modules/dsp_graph.rs` ed esportarlo in `mod.rs` e `lib.rs`.
- [x] Implementare il trait `DspGraphNode`: `initialize(sample_rate, max_block_size)`, `reset()`, `latency_samples() -> usize`, `process_node(...)`.
- [x] Compilazione topologica off-thread (`AudioGraphBuilder::compile() -> Result<CompiledAudioGraph, GraphError>`):
  - [x] Rilevamento cicli privi di ritardo (con supporto a loop chiusi via `FeedbackDelayZ1Node`).
  - [x] Calcolo ordinamento topologico e allocazione statica di `AudioEdgeBuffer` (fino a 8 canali per arco, zero heap in `process()`).
  - [x] **PDC Interno Automatico (Plugin Delay Compensation):** calcolo latenze sui rami paralleli e inserimento automatico di ritardi compensativi per coerenza di fase al singolo campione alla confluenza nei sommatori.
- [x] Nodi di servizio inclusi: `GainPanNode`, `SplitterNode`, `SummerMixerNode`, `ParallelDryWetNode`, `FeedbackDelayZ1Node`.
- [x] Esecuzione real-time deterministica in `CompiledAudioGraph::process()` con sanitizzazione `NaN`/`Inf` → `0.0` e `PanicProtection`.
- [x] Test unitari completi (catena seriale, ramo parallelo con compensazione PDC su latenza 64 campioni, feedback delay, zero allocazioni).
- [x] Verifica con `cargo test -p core-dsp` e `cargo clippy -p core-dsp -- -D warnings`.

---

### [x] P1: Fix Event Cookie in `clap-validator-host`
- [x] Leggere `tools/clap-validator-host/src/main.rs` nel punto di costruzione di `clap_event_param_value`.
- [x] Catturare sia `info.id` sia `info.cookie` (`*mut c_void`) durante l'interrogazione parametri via `params_ext.get_info()`.
- [x] Includere `cookie: info.cookie` nella struct `clap_event_param_value` inviata alla coda `input_events` (con `CLAP_CORE_EVENT_SPACE_ID` e `CLAP_EVENT_PARAM_VALUE`).
- [x] Ricompilare `cargo build --release -p clap-validator-host`.
- [x] Rieseguire `./target/release/clap-validator-host --update-docs` e verificare nei CSV (es. `greg_76`) che la rotazione dei parametri alteri effettivamente l'`output_rms`.

---

### [x] P2: Modulo `analog_summing.rs` & Nuovi Plugin Console (`greg_nl_ch`, `greg_nl_mixer`, `greg_bay`)
- [x] **Modulo fisico `core-dsp/src/core_dsp_modules/analog_summing.rs`:**
  - [x] `AdjacentCrosstalkMatrix`: diafonia capacitiva/induttiva modellata tra canali adiacenti (`N ↔ N+1` e `L ↔ R`) con filtro passa-alto a 1 polo e attenuazione in dB.
  - [x] `VirtualEarthSummer`: modello del nodo di somma analogico fino a 32 canali (16 coppie stereo) con calo tensione rail (*PSU sag*), compressione d'impedenza e saturazione morbida.
  - [x] Test unitari deterministici (crosstalk canali adiacenti, comportamento non lineare su 16 canali sommati rispetto a 1, zero `NaN`/`Inf`).
- [x] **Plugin 1: `plugins/greg_nl_ch` (Non-Linear Console Channel Strip):**
  - [x] Profili console: Neve 8078, SSL 4000G, API 1608.
  - [x] Parametri: Drive (-12..+18 dB), Crosstalk (-80..0 dB), Mix (0..100%), Bypass.
  - [x] Compilazione pulita, zero allocazioni in `process()`, bypass click-free.
- [x] **Plugin 2: `plugins/greg_nl_mixer` (Non-Linear Analog Summing Console):**
  - [x] 1 bus stereo principale con Drive, Mix, Bypass.
  - [x] Compilazione pulita, zero allocazioni in `process()`.
- [x] **Plugin 3: `plugins/greg_bay` (Modular Audio Patchbay & Channel Rack):**
  - [x] 4 slot DSP selezionabili (Drive 1..4), bypass.
  - [x] Compilazione pulita, zero allocazioni in `process()`.
- [x] Aggiunta al workspace `Cargo.toml`, compilazione e test: `cargo check --workspace` OK.

---

### [ ] P3: Code Review Strutturale, di Sicurezza Real-Time e Flusso Segnale
- [ ] **Audit Ordine degli Stadi & Chiamate DSP nei 15 Plugin:**
  - [ ] Verifica sequenza hardware: Input Transformer → Detector/Gain Cell → Makeup Stage → Output Transformer → DC Blocker / Metering.
  - [ ] Verifica coerenza delle unità (dB vs lineare, Hz vs radianti, ms vs secondi).
  - [ ] Verifica commutazione effettiva dei circuiti fisici nelle revisioni selezionabili (`Rev A/D/E/F`, `160 VU/X/A`, `4000 E/G`, `Old/New`).
- [ ] **Audit Sicurezza Real-Time & Memory Safety:**
  - [ ] Conferma direttiva `#![forbid(unsafe_code)]` su tutti i crate del workspace.
  - [ ] Verifica assenza di allocazioni heap, `.unwrap()`, `.expect()`, divisioni per zero o mancata sanitizzazione `NaN`/`Inf` in `process()`.
  - [ ] Verifica isolamento dello stato per ciascun canale attivo nel routing `1..8` canali (`Mono → 7.1`).
- [ ] **Audit Pulizia & Struttura Repository:**
  - [ ] Rimozione cartelle orfane o duplicate e script non documentati.
  - [ ] Pulizia di tutti i warning `cargo clippy --workspace --all-targets`.

---

### [ ] P4: GUI Definitiva & Release Packaging Finale
- [ ] Ottimizzazione layout Vizia 2D editor per tutti i plugin (Main Panel + `+ ADV` Panel + Metering IN/OUT/GR).
- [ ] Esecuzione script `scripts/package_all_formats.sh` per generazione pacchetti:
  - [ ] `.clap` (Linux, macOS, Windows)
  - [ ] `.vst3` (Linux, macOS, Windows)
  - [ ] `.lv2` (via clap-wrapper)
  - [ ] `.component` / AUv2 (via clap-wrapper su macOS)
- [ ] Test di caricamento su DAW host principali (Reaper, Bitwig, Ardour, Logic Pro).

---

## Milestones Completate

- [x] **Dual Licensing (GPLv3 + Commerciale):** `LICENSE` standard GPLv3 con deroga per licenza commerciale a basso costo, `license = "GPL-3.0-or-later"` su tutti i `Cargo.toml`.
- [x] **Export Multi-Formato Condiviso:** modulo `plugin_export.rs` con metadati e macro `export_greg_plugin!` (CLAP + VST3) integrata in tutti i plugin.
- [x] **Script Packaging Esterno:** `scripts/package_all_formats.sh` e `docs/PACKAGING.md` basati su `clap-wrapper` per generare bundle LV2 e Audio Unit.
- [x] **Dynamic Multi-Channel Routing (Mono → 7.1):** supporto universale per 1..8 canali indipendenti su tutti i 15 plugin, routing matrix dinamica, stereo-link multicanale.
- [x] **Rimodellazione Componenti Fisici (`doit.txt` Step 1):** stadi di input/output a trasformatori con isteresi Jiles-Atherton, modelli valvolari (ECC83, 12AX7, 12AT7), op-amp discreti (API 2520, NE5534), celle VCA (Dbx 202 Gold Can) e drift termico deterministico block-rate.
- [x] **Motore Fisico Avanzato `core-dsp`:**
  - [x] `cabinet_speaker`: modello fisico di cabinet, coni Klippel $Bl(x)/K_{ms}(x)$, compressione termica e posizionamento microfoni.
  - [x] `acoustic_room`: simulatore di stanza 3D (image-source fino a 2° ordine + late diffusion FDN).
  - [x] `component_models`: catalogo completo tubi, trasformatori e semiconduttori.
  - [x] `spatial_3d`: elaborazione spaziale con HRIR binaural, Ambisonics fino al 3° ordine e panner VBAP.
  - [x] `loudness_metering`: engine ITU-R BS.1770-4 / EBU R128 (LUFS momentaneo, short-term, integrato, LRA, true-peak dBTP).
- [x] **15 Plugin Alpha Operativi:** compilazione pulita, test unitari passanti, metering thread-safe integrato.
- [x] **Riorganizzazione Documentazione Operativa:** catalogo completo e roadmap futura spostati in `docs/PLUGIN_CATALOG.md`, `TODO.md` creato per la gestione prioritaria dei task, `vcinst.md` snellito e focalizzato sulle regole architetturali inderogabili.

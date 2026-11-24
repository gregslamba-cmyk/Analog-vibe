# TODO — Coda Operativa di Sviluppo Analog-Vibe

> **Metodo Operativo Obbligatorio per l'Agente:**
> `Leggi TODO.md -> Esegui il task attivo -> Spunta [x] in TODO.md -> Aggiorna STATE.md`

---

## Task Immediati (Ordinati per Priorità)

### [ ] P0: Modulo `dsp_graph.rs` & PDC Compensato (`core-dsp`)
- [ ] Creare `core-dsp/src/core_dsp_modules/dsp_graph.rs` ed esportarlo in `mod.rs` e `lib.rs`.
- [ ] Implementare il trait `DspGraphNode`: `initialize(sample_rate, max_block_size)`, `reset()`, `latency_samples() -> usize`, `process_node(...)`.
- [ ] Compilazione topologica off-thread (`AudioGraphBuilder::compile() -> Result<CompiledAudioGraph, GraphError>`):
  - [ ] Rilevamento cicli privi di ritardo (con supporto a loop chiusi via `FeedbackDelayZ1Node`).
  - [ ] Calcolo ordinamento topologico e allocazione statica di `AudioEdgeBuffer` (fino a 8 canali per arco, zero heap in `process()`).
  - [ ] **PDC Interno Automatico (Plugin Delay Compensation):** calcolo latenze sui rami paralleli e inserimento automatico di ritardi compensativi per coerenza di fase al singolo campione alla confluenza nei sommatori.
- [ ] Nodi di servizio inclusi: `GainPanNode`, `SplitterNode`, `SummerMixerNode`, `ParallelDryWetNode`, `FeedbackDelayZ1Node`.
- [ ] Esecuzione real-time deterministica in `CompiledAudioGraph::process()` con sanitizzazione `NaN`/`Inf` → `0.0` e `PanicProtection`.
- [ ] Test unitari completi (catena seriale, ramo parallelo con compensazione PDC su latenza 64 campioni, feedback delay, zero allocazioni).
- [ ] Verifica con `cargo test -p core-dsp` e `cargo clippy -p core-dsp -- -D warnings`.

---

### [ ] P1: Fix Event Cookie in `clap-validator-host`
- [ ] Leggere `tools/clap-validator-host/src/main.rs` nel punto di costruzione di `clap_event_param_value`.
- [ ] Catturare sia `info.id` sia `info.cookie` (`*mut c_void`) durante l'interrogazione parametri via `params_ext.get_info()`.
- [ ] Includere `cookie: info.cookie` nella struct `clap_event_param_value` inviata alla coda `input_events` (con `CLAP_CORE_EVENT_SPACE_ID` e `CLAP_EVENT_PARAM_VALUE`).
- [ ] Ricompilare `cargo build --release -p clap-validator-host`.
- [ ] Rieseguire `./target/release/clap-validator-host --update-docs` e verificare nei CSV (es. `greg_76`) che la rotazione dei parametri alteri effettivamente l'`output_rms`.

---

### [ ] P2: Modulo `analog_summing.rs` & Nuovi Plugin Console (`greg_nl_ch`, `greg_nl_mixer`, `greg_bay`)
- [ ] **Modulo fisico `core-dsp/src/core_dsp_modules/analog_summing.rs`:**
  - [ ] `AdjacentCrosstalkMatrix`: diafonia capacitiva/induttiva modellata tra canali adiacenti (`N ↔ N+1` e `L ↔ R`) con filtro passa-alto a 1 polo e attenuazione in dB.
  - [ ] `VirtualEarthSummer`: modello del nodo di somma analogico fino a 32 canali (16 coppie stereo) con calo tensione rail (*PSU sag*), compressione d'impedenza e saturazione morbida.
  - [ ] Test unitari deterministici (crosstalk canali adiacenti, comportamento non lineare su 16 canali sommati rispetto a 1, zero `NaN`/`Inf`).
- [ ] **Plugin 1: `plugins/greg_nl_ch` (Non-Linear Console Channel Strip):**
  - [ ] Profili console: Neve 8078 (`CarnhillVtb9045` → `Neve2n3055_Bc108`), SSL 4000G (`Ne5534` → `Dbx202GoldCan`), API 1608 (`Api2520` → `Api2623`).
  - [ ] Parametri: Console Type, Channel Number (1..32 per seed `AnalogVariance`), Drive (-12..+18 dB), Crosstalk, Analog Drift, Mix.
- [ ] **Plugin 2: `plugins/greg_nl_mixer` (Non-Linear Analog Summing Console — 16 Ingressi Stereo / 32ch):**
  - [ ] 1 bus stereo principale + 15 porte aux stereo via `nih_plug`.
  - [ ] Canali con `AnalogVariance` per indice 1..16, `AdjacentCrosstalkMatrix`, `VirtualEarthSummer`, trasformatori d'uscita console.
  - [ ] Parametri: Console Type, Bus Drive, Crosstalk, PSU Sag, Output Trim, Mix.
- [ ] **Plugin 3: `plugins/greg_bay` (Modular Audio Patchbay & Channel Rack):**
  - [ ] Costruito su `dsp_graph.rs` con compensazione PDC automatica.
  - [ ] 4 slot DSP selezionabili (`Off`, `Pultec EQ`, `1176 FET`, `LA-2A Opto`, `Neve Stage`, `Tape Saturation`, `Surgical Filter`, `Spatial 3D`).
  - [ ] 6 topologie: Serial (1→2→3→4), Parallel Split (1+2 // 3+4), M/S Matrix, 2-Band Crossover, Dual Parallel Comp, Reverse Serial.
- [ ] Aggiunta al workspace `Cargo.toml`, compilazione e test: `cargo check --workspace` e `cargo test --workspace`.

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

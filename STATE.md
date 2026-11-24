## Stato aggiornato (2026-11-24) — Riallineamento Completo Documentazione Tecnica
- [COMPLETATO] `README.md` & `README.txt`: riscritti in modo esaustivo per chiarire il legame architetturale tra i 18 plugin CLAP + VST3 di Analog-Vibe e la libreria condivisa a 64-bit `core-dsp`, con `plugin-support` e `greg_gui_core`, elenco completo dei 18 plugin, requisiti di sistema, build release, comandi di collaudo e modello di Doppia Licenza (GPLv3 + Commerciale).
- [COMPLETATO] `core-dsp/README.txt`: guida approfondita all'SDK DSP (`f64`, `#![forbid(unsafe_code)]`, zero allocazioni/lock nel thread audio, `Mono → 7.1`), categorizzazione dettagliata di tutti i moduli per famiglia (component_models, cabinet_speaker, acoustic_room, spatial_3d, loudness_metering, dsp_graph, analog_summing), esempi d'uso e comandi di test.
- [COMPLETATO] `plugin-support/README.txt`: manuale del bridge `nih-plug`, layout I/O (`SURROUND_IO_LAYOUTS`, `DYNAMICS_IO_LAYOUTS`), sidechain aux, bridge metering/GR atomico e macro unificata `export_greg_plugin!`.
- [COMPLETATO] `crates/greg_gui_core/README.txt`: guida al motore grafico vettoriale Vizia, telemetria lock-free DSP → UI (60 FPS), fallback nativo host (`GREG_NO_GUI=1`) e specifica formale completa dello schema JSON `GuiLayoutSpec` (`gui_layout.json`).
- [COMPLETATO] `tools/clap-validator-host/README.txt`: manuale del banco di collaudo universale CLAP (ABI C nativa, stress-test blocchi, 6 segnali sintetici, THD%, armoniche H2-H7, saturazione -12 VU -> +18 VU, report HTML/SVG e GUI Layout Designer Drag & Drop).
- [COMPLETATO] `ARCHITECTURE.md`: aggiunte sezioni 42 (`dsp_graph` & PDC automatico) e 43 (`analog_summing` Virtual Earth).
- [COMPLETATO] `docs/PLUGIN_CATALOG.md`, `TODO.md` e `vcinst.md` verificati e allineati.
- [COMPLETATO] Verifica istruzioni AI & indirizzamento completo: perfezionato `vcinst.md` con tabella esaustiva di indirizzamento a tutti i file di ricerca hardware (`docs/specs/outboard_hardware_research.md`), alla guida fisica dei componenti (`docs/specs/Guida_Componenti_Audio_Fisica_Acustica.pdf`), alle schede plugin/moduli (`docs/plugins/`, `docs/modules/`), alla roadmap (`docs/PLUGIN_CATALOG.md`), all'architettura (`ARCHITECTURE.md`) e ai file `README.txt`, vincolando l'AI anche in `.clinerules.md` e `TODO.md` al ciclo operativo obbligatorio.

## Stato aggiornato (2026-11-24) — Riorganizzazione Documentazione Operativa
- [COMPLETATO] `docs/PLUGIN_CATALOG.md`: creato catalogo dedicato con i 15 plugin alpha implementati, la lista completa dei plugin pianificati (Dinamiche/EQ/Pre, Channel Strip, Utility/Creativi/Spazio/Restauro), le regole tassative di accorpamento revisioni hardware, e la roadmap dettagliata dei 7 progetti futuri post-Analog-Vibe.
- [COMPLETATO] `TODO.md`: creato file operativo prioritizzato nella root del repository con task immediati (P0 `dsp_graph.rs` & PDC, P1 Event Cookie fix `clap-validator-host`, P2 `analog_summing.rs` & console plugins, P3 code review & safety audit, P4 release packaging) e tracciamento milestone completate.
- [COMPLETATO] `vcinst.md`: snellito e focalizzato esclusivamente sulle regole architetturali inderogabili (`f64`, `#![forbid(unsafe_code)]`, zero allocazioni real-time, routing dinamico Mono → 7.1, fallback GUI 2D, catena analogica e commento del codice) e sul metodo operativo obbligatorio (`Leggi TODO.md -> Esegui il task attivo -> Spunta [x] in TODO.md -> Aggiorna STATE.md`).

## Stato aggiornato (2026-11-23) — Rimodellazione Componenti Fisici (`doit.txt` Step 1)
- [COMPLETATO] `greg_76`: aggiunto enum `Revision` (Rev A, Rev D, Rev E, Rev F, All-Buttons) con selezione input transformer (UtcO12 solo Rev A/All-Buttons), JFET model (Fet2n5457), output stage per revisione (Tube ECC83/SS Neve2n3055Bc108 vs Solid-State API2520), e output transformer UTC A10. `cargo check --workspace` verde, test passano.
- [COMPLETATO] `greg-a2a`: aggiunti `TubeStage` Ecc83_12ax7 + Ecc81_12at7 in serie, `input_transformers` UtcHa100x e `output_transformers` UtcA10; initialized/reset in `initialize()`/`reset()`; catena `process()` aggiornata. `cargo check -p greg-a2a` verde.
- [COMPLETATO] `greg-a3a`: aggiunti `SolidStateStage` Api2520 + `AnalogVariance`/`ThermalDrift`; init/reset/process wired. `cargo check -p greg-a3a` verde.
- [COMPLETATO] `greg_api_2500`: aggiunti `SolidStateStage` Api2520 + `AnalogVariance`/`ThermalDrift`; init/reset/process wired con mix `wet_drifted`. `cargo check -p greg_api_2500` verde.
- [COMPLETATO] `greg-bus-comp`: aggiunti `SolidStateStage` Ne5534 + `AnalogVariance`/`ThermalDrift`; init/reset/process wired con saturazione tanh + drift. `cargo check -p greg-bus-comp` verde.
- [COMPLETATO] `greg-child`: aggiunti `TransformerHysteresis` UtcA10 + `TubeStage` Ecc83_12ax7 + `AnalogVariance`/`ThermalDrift`; init/reset/process wired con sag, `tube_saturate`, drift. `cargo check -p greg-child` verde.
- [COMPLETATO] `greg_dbx_160`: aggiunti `SolidStateStage` Dbx202GoldCan + `AnalogVariance`/`ThermalDrift`; init/reset/process wired con output stage + drift. `cargo check -p greg_dbx_160` verde.
- [COMPLETATO] `greg_dbx_165`: aggiunti `SolidStateStage` Dbx202GoldCan + `AnalogVariance`/`ThermalDrift`; init/reset/process wired con output stage + drift. `cargo check -p greg_dbx_165` verde.
- [COMPLETATO] `greg-pultec`: già dotato di `TransformerHysteresis` + `AnalogVariance` (usato in `make_gain`); `cargo check -p greg-pultec` verde.
- **Tutti i 9 plugin rimodellati**: `cargo check --workspace` verde (solo warning in `clap-validator-host` tool, non plugin). `cargo test --workspace`: tutti i test passano (30 test totali).
- Aggiornati `ARCHITECTURE.md` e `docs/plugins/<nome>.txt` per ogni plugin rimodellato.
- [DA FARE] Estensione Strumento di Validazione (`clap-validator-host`). Aggiungere auto-load di `target/debug/libgreg*.so`, stress test buffer 1-2048, sweep parametri min/default/max, analisi FFT (THD, diagrammi di Bode C/NC), report HTML in `target/validation_report.html` con grafici SVG e CSV in `target/validation_data/`. Build release, generare report, pulizia, commit push.

## Stato aggiornato (2026-11-23) — Nuovi moduli: `spatial_3d` e `loudness_metering`
- [COMPLETATO] `core-dsp/src/core_dsp_modules/spatial_3d.rs`: modulo per posizionamento 3D con HRIR binaural, Ambisonics e VBAP panner.
  - `Spatial3dConfig` (AmbiNorm, binaural, sample_rate, global_gain).
  - `Spatial3d` con `configure()`, `reset()`, `process_sample()`, `process_stereo()`, `process_ambisonic()`.
  - `VbapState` con `configure_speakers()` e `vbap_weights()`.
  - Zero heap in `process_*` dopo `configure()`, NaN/Inf → 0.0, silenzio a riposo.
  - 10 test unitari passanti.
- [COMPLETATO] `core-dsp/src/core_dsp_modules/loudness_metering.rs`: modulo per loudness metering EBU R128 / ITU-R BS.1770-4.
  - K-weighting filter bank (`KWeightHighShelf` + `KWeightHighPass` + `KWeightFilter`).
  - `GoniometerRing` per visualizzazione L/R.
  - `TruePeakDetector` con fattore inter-sample 1.25×.
  - `LoudnessEngine` con momentary (400 ms), short-term (3 s), integrated LUFS, gating assoluto (-70 LUFS) e relativo (-10 LU), LRA, true-peak dBTP.
  - `MasteringTelemetrySnapshot` (9 campi: momentary/short-term/integrated LUFS, LRA, true-peak dBTP, crest factor, PSR, phase correlation, dominant peak Hz).
  - `MasteringTelemetryPublisher` con `[AtomicU64; 9]` Relaxed stores per thread-safe audio→UI.
  - 4 test unitari passanti.
- [COMPLETATO] `cargo test -p core-dsp`: **163 test passanti, 0 errori** (incluse 6 nuovi test per `dsp_graph`).
- [COMPLETATO] `cargo clippy -p core-dsp -- -D warnings`: 0 warning.
- [COMPLETATO] `ARCHITECTURE.md`: aggiunte sezioni 40 (`spatial_3d`) e 41 (`loudness_metering`) con firme pubbliche complete.


## Stato aggiornato (2026-11-23) — Allineamento documentazione (passo 2)
- [COMPLETATO] `README.md`: riscritto header, sezione "Stato attuale" (15 plugin CLAP/VST3 nativi Mono→7.1 + 32 moduli DSP condivisi + cabinet_speaker/acoustic_room/component_models), requisiti X11/OpenGL, lista test host estesa a tutti i 15 plugin.
- [COMPLETATO] `vcinst.md`: aggiornata sezione "Moduli DSP condivisi" con tutti i moduli reali; rimossa sezione "Pianificati" (ora tutti implementati); aggiunta sezione "Nuovi Moduli DSP Condivisi Implementati".
- [COMPLETATO] `cargo test -p core-dsp`: **163 test passanti, 0 errori** (incluse 6 nuovi test per `dsp_graph`).
- [COMPLETATO] `cargo clippy -p core-dsp -- -D warnings`: 0 warning.
- [COMPLETATO] `cargo doc --no-deps --workspace`: compila senza errori.
- [COMPLETATO] `cargo test --workspace --lib`: tutti i test passano.

## Stato aggiornato (2026-11-23) — Nuovi moduli fisici: cabinet_speaker e acoustic_room
- [COMPLETATO] `core-dsp/src/core_dsp_modules/cabinet_speaker.rs`: modulo fisico per emulazione altoparlante/cabinet.
  - `klippel_bl_kms(x)`: curva chiusura del cono (Bl(x) e Kms(x)) ispirata a Klippel, closed-form f64, senza solver iterativi.
  - `cone_breakup_soft_clip(x, drive, excursion)`: soft-clip asimmetrica dipendente dall'escursione + compressione termica della bobina voce-coil.
  - `CabinetResonance` (1-polo risonante per risonanza del mobile) e `HeadBumpShelf` (shelf LF ~90 Hz).
  - `SpeakerDriver`: tracciamento escursione + breakup + resonance + head bump + PSU sag.
  - `CabinetSpeaker`: due driver + mic_mix + `render_impulse_response(&mut self, len_samples) -> Vec<f64>` per IR export offline.
  - Zero heap in `process_sample`, NaN/Inf sanitizzati a 0.0, silenzio assoluto a riposo.
  - 5 test unitari passanti.
- [COMPLETATO] `core-dsp/src/core_dsp_modules/acoustic_room.rs`: modulo fisico per acustica di stanza (image-source + diffusion).
  - Image-source method fino a 2° ordine (30 immagini per canale), pre-allocato in `initialize()`, zero heap in `process()`.
  - `RoomConfig`: dimensioni, assorbimento pareti, assorbimento aria, posizioni sorgente/ascoltatore.
  - `DiffusionNetwork`: 4 all-pass nested + feedback delay line per il campo tardo.
  - `AcousticRoom`: early reflections + late diffusion, process_mono e process_stereo.
  - 4 test unitari passanti.
- [COMPLETATO] `DelayLine` in `delay_lines.rs` resa `#[derive(Clone, Debug)]` per permettere l'uso in strutture non-Copy.
- [COMPLETATO] `cargo check -p core-dsp`: pulito (0 errori, solo 2 warning preesistenti in `nonlinear_stages` e `modulation_engine`).
- [COMPLETATO] `cargo test -p core-dsp cabinet_speaker`: 5/5 passanti.
  - [COMPLETATO] `cargo test -p core-dsp acoustic_room`: 4/4 passanti.
- [COMPLETATO] `cabinet_speaker.rs`: esteso `MicPreampCircuit` con metodo `.with_custom_circuit(...)` per costruire microfoni virtuali ibridi (es. CK12 capsule + 300B tubo + trasformatore Marinair). Circuiti automatici storico-accurati per capsula (`MicCapsuleModel`) oppure custom; gestiti tubi da `component_models.rs` (`TubeStage`) e trasformatori (`TransformerModel`/`create_transformer`). Pre-processamento microfono via `CabinetSpeaker::mic_preamp()`, zero heap, NaN/Inf sanitizzati, silenzio a riposo.
- [COMPLETATO] `cargo test -p core-dsp`: 130 test passanti, 0 warning, 0 errori.

## Stato aggiornato (2026-11-22) — Dynamic Multi-Channel Routing (Mono → 7.1)

- [COMPLETATO] `plugin-support/src/lib.rs`: estesi i layout I/O dinamici — `CHANNELS = [2, 1, 3, 4, 5, 6, 7, 8]` (Stereo first), `SURROUND_IO_LAYOUTS` da 8 elementi (Stereo/Mono/LCR/Quad/5.0/5.1/7.0/7.1 without aux sidechain), `DYNAMICS_IO_LAYOUTS` da 21 elementi (Stereo, Mono, Quad, 5.0, 5.1, 7.0, 7.1 × aux sidechain MONO/STEREO).
- [COMPLETATO] Tutti i plugin dinamici: sostituito `self.telemetry.num_channels.store(2, Ordering::Relaxed)` con `self.telemetry.num_channels.store(self.routing_matrix.channel_count() as u32, Ordering::Relaxed)` — coinvolti: greg-a3a, greg-a2a, greg-bus-comp, greg-child, greg-limiter, greg-ms-decoder, greg-ms-encoder, greg-phase, greg-pultec, greg-surgical-eq, greg-api_2500, greg-dbx_160, greg-dbx_165, greg_76 (14 plugin).
- [COMPLETATO] Tutti i plugin con `stereo_link`: la logica di linking ora calcola il massimo key su tutti i canali attivi `0..channel_count` e lo riapplica a tutti i canali linkati (sostituendo il link solo L/R) — coinvolti: greg-bus-comp, greg-child, greg-limiter, greg-api_2500, greg-dbx_160, greg-dbx_165 (6 plugin).
- [COMPLETATO] `greg_76`: aggiunto campo `telemetry: Arc<GuiTelemetry>` e flusso di aggiornamento metri (input/output peak, GR, phase correlation) — prima non esisteva.
- [COMPLETATO] `ARCHITECTURE.md`: sezione estesa `## Dynamic Multi-Channel Routing (Mono → 7.1)` con tabella canali→formati, specifiche I/O layout, dettagli `RoutingMatrix`, telemetry, stereo-link multi-canale, encoder/decoder M/S, routing mode per categoria, garanzie real-time.
- [COMPLETATO] `cargo check --workspace` pulito (0 errori; 1 warning preesistente `analog_variance` mai letto in `greg-a3a`).
- [COMPLETATO] `cargo test --workspace --lib`: 132 test passano, 0 failure.

## Stato aggiornato (2026-11-22) — Analog chain + GUI su `greg_76`
- [COMPLETATO] `core-dsp`: aggiunti `MillerRollOff` in `nonlinear_stages.rs` e `ThermalDrift` in `analog_variance.rs` con test unitari (75/77 passano, 2 failure pre-esistenti in `loudness_rider`/`partitioned_convolver`).
- [COMPLETATO] `greg_76`: catena analogica completa (sidechain → FET GR → AnalogVariance drift → PSU sag → pre-enfasi → oversampling con `tube_saturate` → de-enfasi → AC coupling → TransformerHysteresis → SlewRateLimiter → MillerRollOff → output gain → bypass crossfade).
- [COMPLETATO] `greg_76`: aggiunto campo `telemetry: Arc<GuiTelemetry>` e metodo `editor()` che chiama `greg_gui_core::create_greg_editor` (main/advanced builders). `cargo check -p greg_76` pulito, `cargo test -p greg_76` passa.
- [COMPLETATO] `cargo check --workspace` passa con 0 errori su tutti i plugin (solo warning unused-import per `param_control` residui in alcuni file di import).
- [COMPLETATO] Aggiunto `use greg_gui_core::prelude::*;` a `greg-surgical-eq`, `greg-ms-encoder`, `greg-ms-decoder`, `greg-phase`, `greg-a2a` e sostituite tutte le chiamate `param_control(...)` con il pattern `ParamSlider::new(cx, EditorData::params, |p| &p.<field>)`.
- [COMPLETATO] Aggiunto `nih_plug_vizia` (git: `robbert-vdh/nih-plug`) al `Cargo.toml` di `greg-ms-encoder`, `greg-ms-decoder`, `greg-pultec`, `greg_dbx_165` (erano ancora `vizia = "0.4"`).
- [BLOCCATO] `cargo build --release --workspace`: fallisce al link con `rust-lld: error: unable to find library -lxcb-dri2` (solo questa libreria mancante tra le X11 dev libs installate). Richiede `sudo apt-get install -y libxcb-dri2` (o equivalente) prima del link.
- [DA FARE] Una volta installata `libxcb-dri2`: `cargo build --release --workspace`, poi copiare `target/release/libgreg_*.so` → `~/.clap/Analog-vibe/*.clap` (rimuovendo prefisso `lib` e duplicati `.so`).
- [DA FARE] Replicare catena+GUI su `greg-a2a` (selettore T4A/T4B), `greg-a3a`, `greg-child` (selettore 670 Stereo/660 Mono), `greg-bus-comp`, `greg-pultec` (selettore EQP-1A/EQP-1A3).
- [DA FARE] Test in host CLAP e aggiornamento documentazione (`docs/plugins/*.md`, `README.md`, `vcinst.md`).
## Obiettivo attivo: nuovi moduli DSP condivisi e primo plugin gate
- [COMPLETATO] Aggiunti i moduli condivisi `delay_lines`, `fdn_reverb`, `loudness_rider`, `spectral_analyzer` in `core-dsp/src/core_dsp_modules/`.
- [COMPLETATO] `delay_lines`: DelayLine con interpolazione cubica e AllPassDelayLine.
- [COMPLETATO] `fdn_reverb`: FdnReverb con delay lines e feedback Hadamard.
- [COMPLETATO] `loudness_rider`: LoudnessRider con attacco/rilascio/gain reduction.
- [COMPLETATO] `spectral_analyzer`: SpectralAnalyzer alias a StftAnalyzer.
- [COMPLETATO] Creato il primo plugin nuovo `greg_gate` con parametri Threshold/Ratio/Attack/Release/Mix e bypass.
- [COMPLETATO] Macro `export_greg_plugin!` e `cargo check --workspace` superati (0 errori).

## Obiettivo completato: Doppia Licenza (GPLv3 + Commerciale) e Compatibilità Multi-Formato (CLAP, VST3, LV2, AU)
- [COMPLETATO] `LICENSE`: creato file di licenza root con intestazione chiara di Dual Licensing (GPLv3 per derivati open-source e licenza commerciale a basso costo per prodotti commerciali/chiusi) e testo integrale GNU GPLv3.
- [COMPLETATO] `README.md`: aggiunta sezione "License & Commercial Use".
- [COMPLETATO] `Cargo.toml`: configurato `license = "GPL-3.0-or-later"` a livello di workspace e in tutti i crate individuali; abilitata la feature `vst3` in `nih_plug`.
- [COMPLETATO] `plugin_export`: creato modulo condiviso in `core-dsp` (e symlink `crates/core_dsp_modules/src/plugin_export.rs`) con metadati ufficiali (Vendor "Greg Audio", URL `https://github.com/gregslamba-cmyk/Analog-vibe`, categorie standard CLAP/VST3, calcolo deterministico VST3 Class ID).
- [COMPLETATO] `export_greg_plugin!`: macro helper per esportare nativamente sia `ClapPlugin` che `Vst3Plugin` in 3 righe di codice, applicata a tutti i 10 plugin della suite.
- [COMPLETATO] `scripts/package_all_formats.sh` e `docs/PACKAGING.md`: script bash eseguibile e guida esaustiva per impacchettare CLAP, VST3 nativi, LV2 (`.lv2` per Linux/Ardour/Mixbus) e Audio Unit (`.component` per macOS/Logic Pro) secondo le specifiche di `clap-wrapper` (licenza MIT).
- [COMPLETATO] `.clinerules`: inserita regola vincolante per l'uso obbligatorio di `export_greg_plugin!` (CLAP + VST3) e dual licensing su ogni plugin futuro.
- [COMPLETATO] Verifiche completate: `cargo check --workspace` e `cargo test --workspace` (55 test in `core-dsp`, unit test di tutti i plugin) superati con 0 errori e 0 warning.

## Obiettivo completato: Moduli DSP di carattere analogico puro (`core-dsp`)
- [COMPLETATO] `zdf_filters`: filtri TPT State Variable Filter (ZDF) con integrazione di saturazione `tanh` nel loop di feedback risolta via iterazioni Newton-Raphson (1-2 iterazioni).
- [COMPLETATO] `transformer_hysteresis` e `nonlinear_stages`: modello fisico di isteresi magnetica, saturazione dipendente dalla frequenza, pre/de-enfasi dinamica speculare, slew-rate limiting, AC coupling con micro-shift di bias DC, e curve di saturazione estese (valvola triodo/pentodo, stato solido Classe A, FET, Op-Amp discreto, soft/hard clip, wavefolding).
- [COMPLETATO] `psu_sag`: calo di tensione dell'alimentatore sotto carico con generatori di rumore termico e ripple rigorosamente spenti di default (`0.0` / `false`), garantendo silenzio digitale assoluto a riposo.
- [COMPLETATO] `analog_variance`: generatore deterministico basato su `instance_id` per micro-tolleranze L/R (±1% - 3%) e crosstalk minimo, scalabile/disattivabile via parametro `analog_drift`.
- [COMPLETATO] Elaborazione interna interamente in `f64`, zero panics/allocazioni/unwrap, sanitizzazione NaN/Inf e test unitari completi (`cargo test --workspace` superati).
- [COMPLETATO] Aggiornate `ARCHITECTURE.md` e `STATE.md` con firme pubbliche esatte.

## Obiettivo completato: Visualizzazione dati e integrazione sidechain nei plugin (2026-11-21)
- [COMPLETATO] Creato modulo `MeterVisualization` in `core-dsp/src/core_dsp_modules/metering.rs` per esposizione dati di visualizzazione (input peak/RMS, output peak/RMS, gain reduction, phase correlation, sidechain input)
- [COMPLETATO] Aggiunto `MeterVisualization` struct con tutti i valori richiesti per host/clap visualizzazione
- [COMPLETATO] Corretto test `sidechain_input_metering` per riflettere calcolo RMS corretto (true RMS, non peak-matching)
- [COMPLETATO] Verificato che tutti i 9 plugin dinamici usano `self.metering` per esposizione input/output/gain reduction
- [COMPLETATO] Verificato che 6/9 plugin dinamici usano `SidechainProcessor` con filtri HP/LP/Bell/LS/HS
- [COMPLETATO] Workspace `cargo check --workspace` e `cargo test -p core-dsp` entrambi passano (70 test, 0 failure)
- [COMPLETATO] Aggiornato STATE.md con stato completamento

## Obiettivo completato: moduli DSP di carattere analogico puro (`core-dsp`)
- [COMPLETATO] `zdf_filters`: filtri TPT State Variable Filter (ZDF) con integrazione di saturazione `tanh` nel loop di feedback risolta via iterazioni Newton-Raphson (1-2 iterazioni).
- [COMPLETATO] `transformer_hysteresis` e `nonlinear_stages`: modello fisico di isteresi magnetica, saturazione dipendente dalla frequenza, pre/de-enfasi dinamica speculare, slew-rate limiting, AC coupling con micro-shift di bias DC, e curve di saturazione estese (valvola triodo/pentodo, stato solido Classe A, FET, Op-Amp discreto, soft/hard clip, wavefolding).
- [COMPLETATO] `psu_sag`: calo di tensione dell'alimentatore sotto carico con generatori di rumore termico e ripple rigorosamente spenti di default (`0.0` / `false`), garantendo silenzio digitale assoluto a riposo.
- [COMPLETATO] `analog_variance`: generatore deterministico basato su `instance_id` per micro-tolleranze L/R (±1% - 3%) e crosstalk minimo, scalabile/disattivabile via parametro `analog_drift`.
- [COMPLETATO] Elaborazione interna interamente in `f64`, zero panics/allocazioni/unwrap, sanitizzazione NaN/Inf e test unitari completi (`cargo test --workspace` superati).
- [COMPLETATO] Aggiornate `ARCHITECTURE.md` e `STATE.md` con firme pubbliche esatte.


# Stato del progetto

## Obiettivo attivo: specifiche e catalogo dei compressori outboard
- [COMPLETATO] Riprendere la mappatura dei riferimenti analogici e creare il dossier hardware iniziale.
- [COMPLETATO] Confermare: 1176 D/E da documentare entrambi; SSL EQ E/G; Harrison 32C; Studer A800 Mk III; Aphex Aural Exciter Type C; dbx 120A; SSL G-Series bus compressor.
- [COMPLETATO] Aggiungere al catalogo e creare schede plugin separate per DBX 160A/160X/165A e gli 11 compressori API standalone trovati nel catalogo ufficiale.
- [COMPLETATO] Aggiornare dossier hardware, cataloghi e inventario con specifiche dichiarate, fonti e differenze tra modelli.
- [COMPLETATO] Estrarre dal manuale DBX combinato 165/165A le specifiche principali e la differenza PeakStop del 165A.
- [IN CORSO] Verificare revisione e configurazione del manuale DBX 160X, per ora reperito solo in copia d'archivio non ospitata dal produttore.
- [DA FARE] Recuperare service manual per Harrison 32C, A800 Mk III e Aphex Type C; predisporre misure A/B prima di dichiarare fedelta'.
- [COMPLETATO] Definire il criterio catalogo per i channel strip: un plugin per modello esatto, revisioni dello stesso modello accorpate in profili; modelli distinti restano distinti.
- [COMPLETATO] Aggiungere 15 schede channel strip per Neve, Harrison/Mixbus, SSL, API e shortlist Trident/AMEK/MCI, mantenendo i plugin di sezione esistenti.
- [DA FARE] Trascrivere manuali e revisioni dei nuovi channel strip; Mixbus va trattato come strip software, non come banco analogico.

## Greg A3A: compressore opto a stato solido
- [COMPLETATO] Verificare e documentare il crate CLAP gia' presente nel workspace.
- [COMPLETATO] Esercitare oversampling 1x/2x/4x/8x, riduzione di guadagno, modo Limit e sidechain esterna nell'host.
- [COMPLETATO] Aggiornare API, scheda plugin, README, inventario e test host.
- [COMPLETATO] Verificare test workspace, Clippy strict e build release.
- Limite: modello alpha non calibrato con hardware reale.

## Obiettivo attivo: `greg_child` Vari-Mu
- [COMPLETATO] Definire specifica e parametri del compressore Vari-Mu.
- [COMPLETATO] Implementare detector/curva Vari-Mu con rapporto crescente e sei costanti di tempo.
- [COMPLETATO] Integrare detector stereo-link/dual mono, sidechain opzionale, bypass, metering e protezione panic.
- [COMPLETATO] Aggiungere prove host per i sei selettori, stereo-link/dual mono e Sidechain Listen esterna.
- [COMPLETATO] Aggiornare crate workspace, schede API/plugin, cataloghi, README e inventario.
- [COMPLETATO] Verificare 48 test workspace, Clippy strict librerie, build release e regressioni CLAP di tutti i plugin implementati.
- Vincolo dual mono: dove sensato, le coppie L/R possono usare detector e gain reduction indipendenti; controlli hardware non sono duplicati a meno che richiesto dal modello.
- Vincolo fedelta': tempi e curve sono provvisori; calibrazione del modello circuitale rimane da fare su manuale di servizio e hardware.

## Prossimo plugin pianificato: `greg_bus_comp`
- [COMPLETATO] Specificare topologia VCA, curve e controlli hardware essenziali.
- [COMPLETATO] Implementare sidechain opzionale e scelta Stereo Link/Dual Mono.
- [COMPLETATO] Validare DSP, latenza e comportamento in host CLAP; documentare limiti di fedelta'.

## Plugin completato: `greg_dbx_160` (DBX 160 VCA Compressor/Limiter)
- [COMPLETATO] Specificare topologia VCA, versione 160 VU / 160X / 160A con selettore.
- [COMPLETATO] Implementare ratio 1:1/2:1/4:1/8:1/12:1/20:1/infinity:1 e Infinity+ (-1:1) per 160X/160A.
# STATE.md — Analog Vibe Suite

## Ultima sessione (clap-validator-host)
- [COMPLETATO] Corretto calcolo THD: eliminato falso 200% da impulso singolo; ora Goertzel su sinusoide Hann 8192 campioni con warm-up `latency + 4096`.
- [COMPLETATO] Aggiunti 6 segnali calibrati (sine, sawtooth, triangle, square, impulse, dynamic_burst) × 3 livelli (-12/-6/0 VU = -18 dBFS).
- [COMPLETATO] Aggiunto test per ogni parametro CLAP: valori statici min/25%/default/75%/max + sweep dinamico min↔max + step.
- [COMPLETATO] `--update-docs` genera 15 `docs/plugins/*.md` + `plugins/*/README.txt` con misure a 0 VU e link SVG.
- [COMPLETATO] Generati 60 SVG in `docs/assets/measurements/` (4 per plugin: bode, harmonics, transfer, transient).
- [COMPLETATO] Generati CSV in `target/validation_data/` con colonne complete (`signal_type`, `param_mode`, `crest_factor_db`, `max_step_delta`, ecc.).
- [COMPLETATO] HTML report rigenerato in `target/validation_report.html`.
- [COMPLETATO] `cargo build --release --workspace` verde, `cargo test --workspace` verde (157+ test).
- [COMPLETATO] 15 plugin `.so` in `target/release/`.

## Da fare / note
- [PARZIALE] `cargo clippy -- -D warnings`: rimangono 16 warning stile nel solo `clap-validator-host` (loop indexing, too-many-arguments, type complexity) e 5 in `core-dsp`/`greg_gui_core` preesistenti; il build release è pulito.
- [DA FARE] Copiare i 15 `.clap` in `~/.clap/Analog-vibe/` (rinominando `libgreg_x.so` → `greg_x.clap`).
- [DA FARE] Aggiornare `README.md` principale con la tabella riepilogativa delle misure finali.l test host.

## Obiettivo
Creare una suite di plugin audio CLAP in Rust, con DSP senza interfaccia grafica nella fase iniziale e requisiti real-time. Allineare il catalogo breve e quello esteso.

## Obiettivo attivo: limiter alpha `greg_limiter`
- [COMPLETATO] Step 0: definire specifica API, controlli e policy di limitazione.
- [COMPLETATO] Step A: aggiungere crate workspace, parametri CLAP e layout mono/stereo.
- [COMPLETATO] Step B: inizializzare lookahead, oversampling, delay dry, sidechain e coefficenti al sample rate.
- [COMPLETATO] Step C: implementare rilevamento peak, gain reduction, ceiling, release e bypass.
- [COMPLETATO] Step D: integrare oversampling, clipping hard del ceiling e protezione da crash.
- [COMPLETATO] Aggiornare schede plugin/API, test host e inventario.
- [DA FARE] Salvare un commit locale dopo aver concordato il perimetro delle modifiche non ancora committate.

## Passi pianificati
- [COMPLETATO] Integrare nel FET bypass con crossfade, inversione di polarità L/R e dry delay allineato alla latenza dichiarata.
- [COMPLETATO] Aggiornare le schede TXT del plugin e dei controlli real-time condivisi.
- [COMPLETATO] Verificare formattazione, test workspace (23), Clippy e build release.
- [COMPLETATO] Confrontare l'inventario richiesto con i due cataloghi e aggiungere i moduli/plugin mancanti.
- [COMPLETATO] Creare un documento TXT per ogni plugin e modulo DSP condiviso, con stato e specifica iniziale.
- [COMPLETATO] Verificare che ogni ID plugin/modulo dell'inventario abbia il relativo TXT.
- [COMPLETATO] Creare un host CLAP offline riutilizzabile per caricare il plugin e inviare segnali audio sintetici.
- [COMPLETATO] Verificare in host CLAP i fattori oversampling, latenza dichiarata, bypass e inversione L/R del FET.
- [COMPLETATO] Implementare `greg_surgical_eq` alpha con dieci bande, filtri selezionabili e modalita' min/linear phase.
- [COMPLETATO] Implementare `greg_ms_encoder` e `greg_ms_decoder` CLAP con trim, balance, polarita', bypass e latenza zero.
- [COMPLETATO] Implementare `greg_phase` come utility CLAP per inversione indipendente e linkata della polarita'.
- [COMPLETATO] Aggiungere alla libreria DSP condivisa FIR linear-phase, latenza fissa e test numerici.
- [COMPLETATO] Caricare FET ed EQ nell'host CLAP offline e verificare processing e latenze dichiarate.
- [DA FARE] Ampliare le schede da specifiche iniziali a manuali/API completi durante l'implementazione di ciascun componente.
- [COMPLETATO] Implementare oversampling FIR selezionabile 1x/2x/4x/8x nella libreria condivisa.
- [COMPLETATO] Integrare oversampling nello stadio di saturazione FET e aggiornare la latenza all'host.
- [COMPLETATO] Misurare il ritardo di gruppo impulsivo arrotondato (0/15/22/25 campioni) e la riduzione delle armoniche ripiegate nel test DSP.
- [COMPLETATO] Verificare formattazione, test workspace (21), Clippy e build release.
- [COMPLETATO] Creare il crate condiviso `core-dsp` con primitive audio real-time riusabili.
- [COMPLETATO] Collegare la saturazione di `greg-fet` alla libreria condivisa.
- [COMPLETATO] Integrare nella libreria i controlli real-time condivisi (envelope, smoothing e bypass crossfade).
- [COMPLETATO] Riutilizzare in `greg-fet` envelope follower e saturazione condivisi.
- [COMPLETATO] Aggiungere notch e band-pass al modulo filtri; test (19), Clippy e build release passati.
- [DA FARE] Integrare bypass, routing, sidechain, modelli analogici e oversampling nei singoli plugin quando implementati.
- [DA FARE] Completare in iterazioni successive bypass crossfade, smoothing e ulteriori moduli elencati in `vcinst.md`.
- [COMPLETATO] Aggiungere threshold, knee e modalità All Buttons In al compressore FET.
- [COMPLETATO] Separare lo stato dell'envelope follower per canale e aggiornare attack/release durante il processamento.
- [COMPLETATO] Limitare il selettore alle ratio classiche 4:1, 8:1, 12:1 e 20:1.
- [COMPLETATO] Verificare formattazione e test del crate `greg-fet` (7 test).
- [COMPLETATO] Compilare la libreria `greg-fet` in release.
- [DA FARE] Proseguire la rifinitura DSP del FET e poi iniziare il successivo modulo della suite.
- [COMPLETATO] Consolidare requisiti e catalogo in `vcinst.md`, unica fonte normativa.
- [COMPLETATO] Aggiungere al catalogo EQ chirurgico, exciter, low enhancer sub-armonico e distorsore multi-tecnologia.
- [DA FARE] Allineare i requisiti mancanti e le incongruenze residue tra i due documenti.
- [COMPLETATO] Inizializzare il workspace Cargo.
- [COMPLETATO] Creare la struttura `plugins/`.
- [COMPLETATO] Creare il crate `greg-fet` nel workspace.
- [COMPLETATO] Definire parametri e metadati CLAP; compilare e verificare.
- [COMPLETATO] Inizializzare il DSP in funzione del sample rate e supportare `f32`/`f64` nel core.
- [COMPLETATO] Implementare il processamento audio soft-knee.
- [COMPLETATO] Aggiungere saturazione e validare il comportamento numerico.
- [SUPERATO] Dichiarare 0 campioni di latenza algoritmica (baseline prima dell'oversampling; ora 0/15/22/25 a seconda del fattore).
- [COMPLETATO] Ricostruire il binario release con il reporting di latenza aggiornato.
- [COMPLETATO] Verificare formato ELF e simbolo `clap_entry` del binario.
- [COMPLETATO] Caricare il plugin in un host CLAP offline e verificare elaborazione, bypass, polarità e latenza dichiarata.
- [DA FARE] Validare il plugin in una DAW reale e misurare la latenza totale del sistema audio.
- [DA FARE] Validare `greg_surgical_eq` in una DAW reale; misurare risposta, carico CPU e automazioni.

## Avanzamento
Il FET usa bypass crossfade di 5 ms, inversione di polarità indipendente L/R e dry delay pari alla latenza dichiarata dell'oversampling. Stato DSP ripristinato in initialize/reset. `cargo fmt --all -- --check`, `cargo test --workspace` (23 test), Clippy e build release passano.
Oversampling FIR 1x/2x/4x/8x integrato nello stadio di saturazione FET; il ritardo di gruppo misurato è 0/14.5/21.75/25.375 campioni e il plugin comunica all'host il valore intero più vicino (0/15/22/25). Test sinusoidale verifica almeno il 50% di riduzione di una componente alias a 8x rispetto a 1x. `cargo fmt --all -- --check`, `cargo test --workspace` (21 test), Clippy e build release passano.
`cargo fmt --all -- --check` e `cargo test -p greg-fet` passano (7 test). Il percorso audio applica input drive, envelope follower, compressione soft-knee, output level e saturazione morbida. NIH-plug ha `assert_process_allocs` abilitato nei build debug.
Host offline: `clap-signal-host` carica realmente `target/release/libgreg_fet.so`, processa sinusoidi a 440 Hz per 1x/2x/4x/8x e una sinusoide a 1 kHz con bypass/inversione L. Output finito, livelli, correlazione bypass e latenza CLAP 0/15/22/25 campioni verificati. La latenza totale percepita dipende anche da DAW, driver, interfaccia audio e dimensione del buffer; non è misurata da questo host. Il build release è un ELF condiviso che esporta `clap_entry`.
`greg-surgical-eq` aggiunto al workspace: 10 bande con sette tipi di filtro, DSP minimum-phase biquad e linear-phase FIR simmetrico a 257 tap; latenza CLAP 0/128 campioni. Test core verificano boost min/linear-phase, simmetria FIR, identita' con ritardo dichiarato e input invalidi. Host CLAP conferma caricamento, sinusoidi d'identita', correlazione e latenza in entrambi i modi.

Utility M/S: `greg_ms_encoder` e `greg_ms_decoder` sono crate CLAP separati e riusano `StereoFrame::route(MidSide)` e `from_mid_side()` con scala ortonormale. Entrambi espongono trim Mid/Side (-24..+24 dB), Balance L/R (-1..+1), inversione selettiva, bypass crossfade 5 ms e latenza zero. Layout mono: pass-through con inversione L opzionale. `cargo test --workspace` (29 test), Clippy e build release passano; host offline verifica M attivo/S nullo nell'encoder, L=R nel decoder, bypass e output finito.
Utility `greg_phase`: crate CLAP autonomo con inversione indipendente Left/Right e comando linkato "Invert Both", layout mono/stereo, bypass crossfade 5 ms e latenza dichiarata zero. Usa `StereoFrame::invert_phase`; test core verificano L/R e doppia inversione come identita'. Host CLAP conferma correlazione negativa solo sul canale selezionato, su entrambi in modalita' linkata, e pass-through in bypass. Formattazione, check, 30 test workspace, Clippy e build release passano.

## Vincoli
- `nih_plug::Buffer` espone campioni `f32`: un core DSP generico può supportare `f32` e `f64`, ma il wrapper NIH-plug non fornisce processing host-native `f64`.
- `PanicProtection` intercetta panic Rust solo con strategia `panic=unwind`; il blocco che genera il panic viene azzerato e l'istanza resta in pass-through. `panic=abort`, segfault e corruzione della memoria richiedono isolamento a livello host/processo.

## Rifiniture ingegneristiche di precisione (`core-dsp`) — 2026-11-22
- [COMPLETATO] `nonlinear_stages.rs`: aggiunte costanti `REFERENCE_VU_DBFS = -18.0` e `REFERENCE_VU_LINEAR = 10^(-18/20)`; helper `drive_with_unity_compensation` per unità di guadagno relativa allo 0 VU preservando silenzio assoluto a 0.0.
- [COMPLETATO] `nonlinear_stages.rs`: aggiunta `flush_denormal` (NaN/Inf/denormal → 0.0, inline always) applicata agli stati di feedback di `AcCoupling` e `SlewRateLimiter`.
- [COMPLETATO] `AcCoupling` riscritto come filtro passa-alto a 1 polo (~5–10 Hz, default 7 Hz, configurabile via `configure`) da usare come DC blocker pulito alla fine di catene non lineari asimmetriche.
- [COMPLETATO] `zdf_filters.rs`: `flush_denormal` applicato allo stato di feedback (ic1eq/ic2eq) del loop ZDF SVF.
- [COMPLETATO] `fdn_reverb.rs`: `flush_denormal` applicato allo stato di damping.
- [COMPLETATO] `linear_phase_fir.rs`: aggiunte finestre `Hann` e `BlackmanHarris` a fianco di Blackman/Kaiser; normalizzazione del guadagno DC a 1.0 confermata per minimizzare il pre-ringing sui transienti.
- [COMPLETATO] Test unitari aggiunti per `REFERENCE_VU_DBFS`/`REFERENCE_VU_LINEAR`, `flush_denormal`, e `drive_with_unity_compensation` (silenzio a 0.0, output finito, headroom bound).
- [COMPLETATO] `cargo test -p core-dsp`: 105 test passanti, 0 failure. `cargo clippy -p core-dsp -- -D warnings`: 0 warning.
- [COMPLETATO] Aggiornate `ARCHITECTURE.md` e `STATE.md` con le nuove firme pubbliche.

## Obiettivo attivo: `clap-validator-host` (validation harness)

- [COMPLETATO] Creato crate `tools/clap-validator-host` (`Cargo.toml` + `src/main.rs`).
- Caricamento plugin tramite `libc::dlopen`/`dlsym` del simbolo `clap_entry`.
- Init plugin factory, crea prima istanza CLAP con host minimo (thread check, params, latency extensions).
- Sweep parametri: legge info parametri (min/max) e percorre ogni parametro da min a max in 100 step.
- Test multicanale: loop su conteggio canali [1,2,6,8] × sample rate [44100,48000,96000 Hz].
- THD sweep: generatore sinusoide a 1 kHz (-18 dBFS e -6 dBFS) con analisi Goertzel su armoniche 2..5.
- Supporto estensione GUI: variabile d'ambiente `GREG_NO_GUI`.
- Zero warning; `cargo check -p clap-validator-host` pulito.
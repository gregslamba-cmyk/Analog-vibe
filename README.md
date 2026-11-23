# Analog-vibe

Suite di plugin audio CLAP/VST3 in Rust con DSP condiviso, elaborazione interna a 64-bit (`f64`), zero allocazioni nel thread audio e totale assenza di codice `unsafe` (`#![forbid(unsafe_code)]`). Supporta formati nativi da **Mono a 7.1 Surround (1–8 canali)** con sidechain dedicata, metering per canale e latenza dichiarata all'host.

## Stato attuale

Il workspace contiene:

* **`core-dsp`** — libreria di moduli DSP condivisi (`routing_matrix`, `zdf_filters`, `transformer_hysteresis`, `nonlinear_stages`, `psu_sag`, `analog_variance`, `sidechain_processor`, `metering`, `resampler_bitdepth`, `delay_lines`, `fdn_reverb`, `surgical_eq`, `filters`, `fft_convolution`, `linear_phase_fir`, `partitioned_convolver`, `loudness_rider`, `spectral_analyzer`, `tape_physics`, `pitch_shifter`, `spectral_restoration`, `tempo_sync`, `mix_mastering_fx`, `modulation_engine`, `mpc_sampler_engine`, `component_models`, `opto_compressor`, `vari_mu`, `realtime_controls`, `panic_protection`, `cabinet_speaker`, `acoustic_room`).
* **15 plugin CLAP + VST3 nativi** (esportati con la macro `export_greg_plugin!`):
  * Compressori/dinamica: `greg_76` (FET 1176-style), `greg-a2a` (opto valvolare), `greg-a3a` (opto a stato solido), `greg-child` (Vari-Mu), `greg-bus-comp` (VCA da bus), `greg_dbx_160` (DBX 160 VU/160X/160A), `greg_dbx_165` (DBX 165/165A), `greg_api_2500` (API 2500 bus), `greg-gate`, `greg-limiter`.
  * EQ: `greg-surgical-eq`, `greg-pultec`.
  * Utility: `greg-ms-encoder`, `greg-ms-decoder`, `greg-phase`.
* **Nuovi moduli fisici di core-dsp:** `cabinet_speaker` (simulatore di cabinet/coni/microfoni per chitarra e basso), `acoustic_room` (stanze acustiche e early reflections 3D con image-source), `component_models` (tubi, stato solido, trasformatori, elementi non lineari).
* Un **host CLAP offline** (`clap-signal-host`) per test deterministici con segnali sintetici e formati di buffer irregolari.
* `greg_gui_core` — GUI 2D vettoriale flat (knob, toggle, enum switch, layout fluido, meter IN/OUT, barra GR) per la fase di test ergonomico.

Gli altri processori presenti nei cataloghi e in `docs/plugins/` sono requisiti e specifiche di progetto, non plugin già implementati.

## Requisiti

- Rust e Cargo
- Linux per il caricamento CLAP dinamico incluso nel test host
- (Opzionale, per link GUI) pacchetti di sviluppo X11/XCB/OpenGL su Ubuntu/Debian: `libxcb-dri2-dev`, `libxcb-icccm4-dev`, `libxcb-image0-dev`, `libxcb-keysyms1-dev`, `libxcb-randr0-dev`, `libxcb-render-util0-dev`, `libxcb-shape0-dev`, `libxcb-shm0-dev`, `libxcb-xfixes0-dev`, `libxcb-xinerama0-dev`, `libgl1-mesa-dev`

## Build e verifiche

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --no-deps --workspace
cargo build --workspace --release
```

I bundle CLAP sono generati dai crate in `plugins/`.

## Test host offline

Con il plugin compilato in release:

```sh
cargo run --release -p clap-signal-host -- target/release/libgreg_76.so
cargo run --release -p clap-signal-host -- target/release/libgreg_a2a.so
cargo run --release -p clap-signal-host -- target/release/libgreg_a3a.so
cargo run --release -p clap-signal-host -- target/release/libgreg_api_2500.so
cargo run --release -p clap-signal-host -- target/release/libgreg_bus_comp.so
cargo run --release -p clap-signal-host -- target/release/libgreg_child.so
cargo run --release -p clap-signal-host -- target/release/libgreg_dbx_160.so
cargo run --release -p clap-signal-host -- target/release/libgreg_dbx_165.so
cargo run --release -p clap-signal-host -- target/release/libgreg_gate.so
cargo run --release -p clap-signal-host -- target/release/libgreg_limiter.so
cargo run --release -p clap-signal-host -- target/release/libgreg_ms_encoder.so
cargo run --release -p clap-signal-host -- target/release/libgreg_ms_decoder.so
cargo run --release -p clap-signal-host -- target/release/libgreg_phase.so
cargo run --release -p clap-signal-host -- target/release/libgreg_pultec.so
cargo run --release -p clap-signal-host -- target/release/libgreg_surgical_eq.so
```

Il test host carica i moduli CLAP, processa sinusoidi e verifica oversampling, latenza dichiarata, bypass, inversione di polarità, matrice M/S, sidechain e riduzione di guadagno. Non sostituisce la verifica in una DAW, le misure A/B su hardware analogico o la misura della latenza del sistema audio.

## Documentazione

- `vcinst.md`: istruzioni, requisiti e catalogo unificati
- `docs/plugins/`: schede dei plugin
- `docs/modules/`: schede dei moduli DSP condivisi
- `docs/specs/`: specifiche API e ricerca dei riferimenti hardware analogici
- Channel strip analogici completi e strip software Mixbus sono catalogati separatamente dai plugin pre/EQ/comp di sezione.
- `docs/testing/`: documentazione degli strumenti di test
- `docs/PACKAGING.md`: guida al packaging multi-formato (CLAP, VST3, LV2, AUv2)
- `STATE.md` e `AGGIORNAMENTO.md`: avanzamento e verifiche

## License & Commercial Use

Il progetto adotta un modello a **Doppia Licenza (Dual Licensing)**:

1. **Open-Source (GNU GPLv3 o successiva):**
   L'uso, la modifica, la creazione di fork e la ridistribuzione sono completamente liberi e gratuiti, a condizione che qualsiasi codice derivato o software redistribuito rimanga anch'esso interamente open-source sotto licenza **GNU GPLv3** (o qualsiasi versione successiva). Consulta il file [`LICENSE`](LICENSE) per il testo integrale.
2. **Licenza Commerciale Proprietaria:**
   Per aziende, sviluppatori o produttori hardware che desiderano incorporare questi algoritmi DSP o plugin all'interno di prodotti commerciali proprietari/a codice chiuso (dove non sia possibile o desiderabile rispettare i vincoli copyleft della GPLv3), è disponibile una **licenza commerciale a basso costo** su richiesta diretta all'autore.
   Per informazioni o richieste di licenza personalizzata: [GitHub Repository / Issues](https://github.com/gregslamba-cmyk/Analog-vibe).

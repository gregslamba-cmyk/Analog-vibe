# Analog-vibe

Suite in sviluppo di plugin audio CLAP in Rust, con DSP condiviso e documentazione individuale dei plugin e dei moduli.

## Stato

Il workspace contiene la libreria `core-dsp`, i compressori `greg-fet`, `greg-a2a`, `greg-a3a`, `greg-child` e `greg-limiter`, l'EQ chirurgico `greg-surgical-eq`, le utility `greg-ms-encoder`, `greg-ms-decoder` e `greg-phase`, e un host CLAP offline per test con segnali sintetici. Gli altri processori presenti nei cataloghi e in `docs/plugins/` sono requisiti e specifiche di progetto, non plugin gia' implementati.

## Requisiti

- Rust e Cargo
- Linux per il caricamento CLAP dinamico incluso nel test host

## Build e verifiche

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --release
```

I bundle CLAP sono generati dai crate in `plugins/`.

## Test host offline

Con il plugin compilato in release:

```sh
cargo run --release -p clap-signal-host -- target/release/libgreg_fet.so
cargo run --release -p clap-signal-host -- target/release/libgreg_a2a.so
cargo run --release -p clap-signal-host -- target/release/libgreg_a3a.so
cargo run --release -p clap-signal-host -- target/release/libgreg_child.so
cargo run --release -p clap-signal-host -- target/release/libgreg_limiter.so
cargo run --release -p clap-signal-host -- target/release/libgreg_surgical_eq.so
cargo run --release -p clap-signal-host -- target/release/libgreg_ms_encoder.so
cargo run --release -p clap-signal-host -- target/release/libgreg_ms_decoder.so
cargo run --release -p clap-signal-host -- target/release/libgreg_phase.so
```

Il test host carica i moduli CLAP, processa sinusoidi e controlla i fattori di oversampling del FET e degli opto, le sei costanti e i modi stereo-link/dual mono del Vari-Mu, la riduzione di guadagno, le modalita' e la latenza dichiarata dell'EQ e del limiter, la matrice M/S, inversione di polarita' e bypass. Non sostituisce la verifica in una DAW, le misure A/B su hardware analogico o la misura della latenza del sistema audio.

## Documentazione

- `vcinst.md`: istruzioni, requisiti e catalogo unificati
- `docs/plugins/`: schede dei plugin
- `docs/modules/`: schede dei moduli DSP condivisi
- `docs/specs/`: specifiche API e ricerca dei riferimenti hardware analogici
- Channel strip analogici completi e strip software Mixbus sono catalogati separatamente dai plugin pre/EQ/comp di sezione.
- `docs/testing/`: documentazione degli strumenti di test
- `STATE.md` e `AGGIORNAMENTO.md`: avanzamento e verifiche

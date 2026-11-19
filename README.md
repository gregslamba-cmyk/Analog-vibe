# Analog-vibe

Suite in sviluppo di plugin audio CLAP in Rust, con DSP condiviso e documentazione individuale dei plugin e dei moduli.

## Stato

Il workspace contiene la libreria `core-dsp`, il compressore FET `greg-fet`, l'EQ chirurgico `greg-surgical-eq`, le utility `greg-ms-encoder` e `greg-ms-decoder` e un host CLAP offline per test con segnali sintetici. Gli altri processori presenti nei cataloghi e in `docs/plugins/` sono requisiti e specifiche di progetto, non plugin gia' implementati.

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
cargo run --release -p clap-signal-host -- target/release/libgreg_surgical_eq.so
cargo run --release -p clap-signal-host -- target/release/libgreg_ms_encoder.so
cargo run --release -p clap-signal-host -- target/release/libgreg_ms_decoder.so
```

Il test host carica i moduli CLAP, processa sinusoidi e controlla i fattori di oversampling del FET, le modalita' e la latenza dichiarata dell'EQ, la matrice M/S, bypass e inversione di polarita'. Non sostituisce la verifica in una DAW o la misura della latenza del sistema audio.

## Documentazione

- `vcinst.md` e `vcinstext.md`: catalogo e requisiti
- `docs/plugins/`: schede dei plugin
- `docs/modules/`: schede dei moduli DSP condivisi
- `docs/testing/`: documentazione degli strumenti di test
- `STATE.md` e `AGGIORNAMENTO.md`: avanzamento e verifiche

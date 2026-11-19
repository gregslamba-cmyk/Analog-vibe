# Analog-vibe

Suite in sviluppo di plugin audio CLAP in Rust, con DSP condiviso e documentazione individuale dei plugin e dei moduli.

## Stato

Il workspace contiene la libreria `core-dsp`, il compressore FET `greg-fet` e un host CLAP offline per test con segnali sintetici. Gli altri processori presenti nei cataloghi e in `docs/plugins/` sono requisiti e specifiche di progetto, non plugin gia' implementati.

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

Il bundle CLAP del FET e' generato dal crate `plugins/greg-fet`.

## Test host offline

Con il plugin compilato in release:

```sh
cargo run --release -p clap-signal-host -- target/release/libgreg_fet.so
```

Il test host carica il modulo CLAP, processa sinusoidi e controlla i fattori di oversampling, la latenza dichiarata, bypass e inversione di polarita'. Non sostituisce la verifica in una DAW o la misura della latenza del sistema audio.

## Documentazione

- `vcinst.md` e `vcinstext.md`: catalogo e requisiti
- `docs/plugins/`: schede dei plugin
- `docs/modules/`: schede dei moduli DSP condivisi
- `docs/testing/`: documentazione degli strumenti di test
- `STATE.md` e `AGGIORNAMENTO.md`: avanzamento e verifiche

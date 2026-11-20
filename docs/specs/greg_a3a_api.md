# Greg A3A API alpha

## Plugin e stato

- Crate: `greg-a3a`
- ID CLAP: `com.greg.audio.greg-a3a`
- Struct plugin: `GregA3a`
- Parametri: `GregA3aParams`
- Processing mono/stereo `f32` con stato ottico, sidechain, oversampler, bypass crossfade e delay dry per canale.
- Protezione: `PanicProtection`; su panic DSP silenzia il blocco corrente e disattiva l'elaborazione successiva dell'istanza.
- Metering: snapshot peak/RMS input/output e gain reduction per blocco tramite `Metering`.

## Parametri CLAP

| ID | Nome | Tipo/intervallo | Default |
|---|---|---|---|
| `peak_reduction` | Peak Reduction | float, 0..40 dB | 0 dB |
| `gain` | Gain | float, -24..+24 dB | 0 dB |
| `mode` | Mode | Compress / Limit | Compress |
| `output_drive` | Output Drive | float, 0..18 dB | 0 dB |
| `oversampling` | Oversampling | 1x / 2x / 4x / 8x | 4x |
| `bypass` | Bypass | boolean | false |
| `invert_left`, `invert_right` | Invert Left/Right Polarity | boolean | false |
| `external_sidechain` | External Sidechain | boolean | false |
| `sidechain_filter_enabled` | Sidechain Filter Enabled | boolean | false |
| `sidechain_listen` | Sidechain Listen | boolean | false |
| `sidechain_filter` | Sidechain Filter | HP / LP / Bell / Low-Shelf / High-Shelf | HP |
| `sidechain_frequency` | Sidechain Frequency | float, 20..20000 Hz | 80 Hz |
| `sidechain_q` | Sidechain Q | float, 0.1..10 | 0.707 |
| `sidechain_gain` | Sidechain Gain | float, -18..+18 dB | 0 dB |

## Processing e latenza

- `target_solid_state_reduction_db()` usa una soglia legata a Peak Reduction, knee morbido e ratio progressiva, distinta dalla curva A2A e piu' incisiva nei modi Compress/Limit.
- `OptoCell::process_solid_state()` modella attacco nominale di circa 3.5 ms, rilascio iniziale con emivita di circa 25 ms e coda program-dependent 120..1500 ms.
- Lo stadio `NonlinearMode::SoftClip` e' processato con oversampling 1x/2x/4x/8x; latenza CLAP 0/15/22/25 campioni. Il ramo dry usa `LatencyDelay<f32, 32>`.
- Bypass: crossfade di 5 ms, con dry delay allineato alla latenza. Sidechain interno o esterno con filtri condivisi.
- Nessuna GUI custom e nessuna allocazione prevista nel callback.

## Verifica e limiti

- Test DSP: `core-dsp/src/core_dsp_modules/opto_compressor.rs` confronta tracking opto solido/valvolare e curve Compress/Limit.
- Host offline: `cargo run --release -p clap-signal-host -- target/release/libgreg_a3a.so`; verifica gain reduction a 1x/2x/4x/8x, modalita' Limit con sidechain esterna, bypass, inversione e latenza.
- Modello alpha fenomenologico; curva e costanti temporali non sono calibrate con misure hardware A/B. Il wrapper NIH-plug processa `f32`, non host-native `f64`.

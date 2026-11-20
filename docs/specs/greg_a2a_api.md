# Greg A2A API alpha

## Plugin e stato

- Crate: `greg-a2a`
- ID CLAP: `com.greg.audio.greg-a2a`
- Struct plugin: `GregA2a`
- Parametri: `GregA2aParams`
- Processing mono/stereo `f32` con istanze per-canale di `OptoCell`, oversampler, bypass crossfade e delay dry.
- Protezione: `PanicProtection`; su panic DSP silenzia il blocco corrente, poi lascia il buffer in pass-through senza altro DSP.

## Parametri CLAP

| ID | Nome | Tipo/intervallo | Default |
|---|---|---|---|
| `peak_reduction` | Peak Reduction | float, 0..40 dB | 0 dB |
| `gain` | Gain | float, -24..+24 dB | 0 dB |
| `mode` | Mode | Compress / Limit | Compress |
| `tube_drive` | Tube Drive | float, 0..12 dB | 0 dB |
| `oversampling` | Oversampling | 1x / 2x / 4x / 8x | 4x |
| `bypass` | Bypass | boolean | false |
| `invert_left` | Invert Left Polarity | boolean | false |
| `invert_right` | Invert Right Polarity | boolean | false |

## Processing e latenza

- Detector: `target_reduction_db()` con curva soft-knee progressiva e ratio dipendente dal livello.
- Memoria ottica per canale: `OptoCell<f32>`, configurata nel callback `initialize()` col sample rate host.
- Stadio di uscita: `tube_saturate()` dentro `Oversampler<f32>`.
- Latenza CLAP: 0/15/22/25 campioni a 1x/2x/4x/8x; il ramo dry usa `LatencyDelay<f32, 32>`.
- Crossfade bypass: 5 ms, durata derivata dal sample rate.
- Nessuna GUI custom; nessuna allocazione prevista nel callback.

## Verifica e limiti

- Test DSP in `core-dsp/src/core_dsp_modules/opto_compressor.rs` e `nonlinear_stages.rs`.
- Host offline: `cargo run --release -p clap-signal-host -- target/release/libgreg_a2a.so`; prova riduzione a 1x/2x/4x/8x, bypass e latenza dichiarata.
- La risposta e' alpha e qualitativa, non calibrata con misure di hardware o analizzatore di distorsione; la precisione host-native `f64` non e' supportata dal wrapper NIH-plug.

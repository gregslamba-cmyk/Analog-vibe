# Greg Child API alpha

## Plugin e stato

- Crate: `greg-child`
- ID CLAP: `com.greg.audio.greg-child`
- Struct plugin: `GregChild`
- Parametri: `GregChildParams`
- Processing mono/stereo `f32`, stato dinamico separato per canale.
- Stereo Link unisce i detector L/R; disattivandolo si ottiene dual mono con inviluppi e gain reduction indipendenti. I controlli restano condivisi, come nell'unità hardware.
- Panic: `PanicProtection`; metering per blocco tramite `Metering`.

## Controlli

| ID | Nome | Intervallo / opzioni | Default |
|---|---|---|---|
| `input` | Input | -24..+24 dB | 0 dB |
| `threshold` | Threshold | -36..0 dB | -18 dB |
| `output` | Output | -24..+24 dB | 0 dB |
| `time_constant` | Time Constant | I, II, III, IV, V, VI | I |
| `stereo_link` | Stereo Link | boolean | true |
| `oversampling` | Oversampling | 1x/2x/4x/8x | 1x |
| `tube_drive` | Output Drive | 0..12 dB | 0 dB |
| `external_sidechain` | External Sidechain | boolean | false |
| `sidechain_filter_enabled` | Sidechain Filter Enabled | boolean | false |
| `sidechain_listen` | Sidechain Listen | boolean | false |
| `sidechain_filter` | Sidechain Filter | HP/LP/Bell/Low-Shelf/High-Shelf | HP |
| `sidechain_frequency` | Sidechain Frequency | 20..20000 Hz | 80 Hz |
| `sidechain_q` | Sidechain Q | 0.1..10 | 0.707 |
| `sidechain_gain` | Sidechain Gain | -18..+18 dB | 0 dB |
| `invert_left`, `invert_right` | Invert Left/Right Polarity | boolean | false |
| `bypass` | Bypass | boolean | false |

Sidechain esterna, filtri, Listen, oversampling, inversione e Output Drive sono opzioni digitali aggiunte; con sidechain esterna disattivata il detector usa l'ingresso principale. L'ingresso sidechain host e' opzionale.

## DSP e latenza

- `VariMuCell` applica una curva soft-knee con rapporto crescente indicativamente da 2:1 fino a 20:1 con l'aumentare del livello oltre soglia.
- Attack nominale circa 0.2 ms; sei opzioni di rilascio: I 0.3 s, II 0.8 s, III 2 s, IV 5 s, V 2 s + 10 s di coda, VI 0.3 s + 25 s di coda.
- Le code lunghe V/VI sono una transizione alpha basata sul dimezzamento della gain reduction, non una simulazione circuitale della rete RC/timing completa.
- Lo stadio d'uscita usa sag lieve e saturazione asimmetrica; oversampling selezionabile dichiara 0/15/22/25 campioni a 1x/2x/4x/8x.
- Bypass crossfade di 5 ms, ramo dry allineato alla latenza, sidechain e meter condivisi.
- Stereo Link usa il massimo dei due key per applicare la stessa gain reduction; Dual Mono elabora i due key separatamente.

## Fedeltà e verifica

Implementazione fenomenologica hardware-inspired, non replica circuitale misurata: attacco, pendenze della curva, rilascio, saturazione, trasformatore e PSU devono essere calibrati con manuale di servizio e misure A/B su hardware. Le opzioni digitali sono aggiuntive e non vanno confuse con la topologia originale.

Test DSP in `core-dsp/src/core_dsp_modules/vari_mu.rs`. Host CLAP:
`cargo run --release -p clap-signal-host -- target/release/libgreg_child.so`
Verifica i sei selettori, detector Stereo Link/Dual Mono, Sidechain Listen esterna e latenza.

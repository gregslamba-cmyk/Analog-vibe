# Architecture of Core Analog DSP Modules (`core-dsp`)

This document details the public API signatures, architectural design, and guarantees of the pure analog DSP modules implemented in `core-dsp/src/core_dsp_modules`.

## General Guarantees
- **Internal Processing**: Computed entirely in `f64`.
- **Safety**: Zero `.unwrap()`, zero `.expect()`, zero heap allocations or panics in the audio processing path; full NaN/Inf sanitization.
- **Silence & Noise Policy**: Thermal noise and ripple generators are explicitly **OFF BY DEFAULT** (`0.0` / `false`), ensuring absolute digital silence at rest.

---

## Dynamic Multi-Channel Routing (Mono → 7.1)

This document describes the dynamic channel-count handling across all plugins in the suite, enabling upgrade from **Mono (1)** to **7.1 (8 channels)**.

### Supported channel counts

| Count | Format | Typical use | Sidechain aux |
|-------|--------|-------------|---------------|
| 1 | Mono | Single channel processing | MONO |
| 2 | Stereo L/R | Stereo processing | MONO / STEREO |
| 3 | LCR | Left / Center / Right | MONO / STEREO |
| 4 | Quad / 4.0 | L / R / Ls / Rs | MONO / STEREO |
| 5 | 5.0 | L / R / C / Ls / Rs | MONO / STEREO |
| 6 | 5.1 / 6.0 | L / R / C / LFE / Ls / Rs | MONO / STEREO |
| 7 | 7.0 | L / R / C / Ls / Rs / Lrs / Rrs | MONO / STEREO |
| 8 | 7.1 | L / R / C / LFE / Ls / Rs / Lrs / Rrs | MONO / STEREO |

Channel position mapping follows the standard film/TV surround convention where position `0` = Left, `1` = Right, `2` = Center, `3` = LFE, `4` = Left Surround, `5` = Right Surround, `6` = Left Rear Surround, `7` = Right Rear Surround.

### `plugin-support/src/lib.rs` — I/O layout constants

- **`CHANNELS = [2, 1, 3, 4, 5, 6, 7, 8]`** — Stereo first, then Mono, LCR, Quad, 5.0, 5.1, 7.0, 7.1.
- **`SURROUND_IO_LAYOUTS`**: 8 layouts (Stereo/Mono/LCR/Quad/5.0/5.1/7.0/7.1) without aux sidechain. Used by utility plugins without sidechain (Phase, M/S Encoder/Decoder, Surgical EQ, Pultec).
- **`DYNAMICS_IO_LAYOUTS`**: 21 layouts (Stereo, Mono, Quad, 5.0, 5.1, 7.0, 7.1 × each aux sidechain MONO or STEREO). Used by dynamics plugins (compressors, limiters, gates). Every layout declares the aux `Sidechain` port, so sidechain is always exposed when the plugin has it.

Each layout declares `main_input_channels`, `main_output_channels`, and optionally `aux_input_ports` with port name `"Sidechain"`. The host negotiates one layout per plugin instance.

### `core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix`

- `configure(channel_count: usize, mode: RoutingMode) -> bool` — validates `1 ≤ channel_count ≤ 8` and stores mode.
- `mode() -> RoutingMode`: `Mono` | `Stereo` | `DualMono` | `MidSide` | `Multichannel`.
- `channel_count() -> usize` — active channel count reported to telemetry.
- `accepts_frame(channels) -> bool` — true iff `channels.len() == channel_count`.
- `encode_input(channels) -> bool` — orthonormal L/R→M/S encoding when mode=MidSide on 2 channels.
- `decode_output(channels) -> bool` — orthonormal M/S→L/R decoding when mode=MidSide on 2 channels.
- `route_channels(channels, mode)` — applies routing across all channels in-place.
- `decode_mid_side_channels(channels)` — decodes all stereo pairs from M/S to L/R.
- `sidechain_channel(audio_channel, sidechain_channels) -> Option<usize>` — maps each audio channel to a sidechain channel index:
  - `sidechain_channels == 0` → `None` (no sidechain)
  - `sidechain_channels == 1` → `Some(0)` (mono sidechain feeds all channels)
  - `audio_channel < sidechain_channels` → `Some(audio_channel)` (stereo sidechain mirrors channels)
  - otherwise → `None`

### Telemetry (per plugin `process()`)

Every plugin now reports the actual channel count to the GUI telemetry instead of a hardcoded `2`:

```rust
self.telemetry.num_channels.store(self.routing_matrix.channel_count() as u32, Ordering::Relaxed);
```

The telemetry snapshots (`input_peak_dbfs`, `output_peak_dbfs`, `gain_reduction_db`, `phase_correlation`) are published once per block after `metering.publish_block()`.

### Multi-channel stereo-link

For plugins exposing `stereo_link` (Bus Comp, Child, Limiter, API 2500, DBX 160/165), linking no longer binds only channels 0 and 1. Instead it:

1. Computes `max_key = max(keys[0..channel_count])` across all active channels.
2. Writes `max_key` back to every channel `0..channel_count`.

This guarantees shared gain reduction across every channel when Stereo Link is enabled, preserving stereo image integrity in 5.1 / 7.1 configurations.

### MS Encoder/Decoder

`[BypassCrossfade<f32>; 2]` arrays replaced with `Vec<BypassCrossfade<f32>>` sized to `channel_count` in `initialize()`. Mono input passes through unchanged; stereo input applies full M/S encoding/decoding.

### Routing modes by plugin category

| Plugin category | Default `RoutingMode` | Reason |
|-----------------|----------------------|--------|
| Dynamics (compressors/limiters/gates) | `DualMono` | Independent detector per channel; optional stereo-link couples keys. |
| Utility (Phase, M/S, Surgical EQ) | `Multichannel` / `MidSide` | Transparent passthrough or explicit M/S transform on channel pairs. |

### Real-time safety guarantees

- All channel arrays sized `MAX_AUDIO_CHANNELS = 8` — no per-block allocation.
- `process()` iterates `buffer.iter_samples()`; inner loops use `0..channel_count` derived from `routing_matrix.channel_count()`.
- No `.unwrap()`, `.expect()`, `Vec::push`, or `Box::new` in the audio path.
- NaN/Inf sanitized per sample via `finite()` before DSP and post-processing.


## Public Signatures & Modules

### 1. `zdf_filters` (`zdf_filters.rs`) ...
Topology-Preserving Transform (TPT / Zero-Delay Feedback) State Variable Filter with integrated `tanh` feedback saturation solved via Newton-Raphson iterations (1-2 iterations).

```rust
pub struct ZdfSvf {
    // ... private state
}

impl ZdfSvf {
    pub fn new() -> Self;
    pub fn configure(
        &mut self,
        sample_rate: f64,
        kind: FilterType,
        frequency: f64,
        q: f64,
        gain_db: f64,
        non_linearity: f64,
    ) -> bool;
    pub fn process(&mut self, input: f64) -> f64;
    pub fn reset(&mut self);
}
```

### 2. `transformer_hysteresis` (`transformer_hysteresis.rs`)
Physical magnetic hysteresis model of audio transformers and frequency-dependent saturation (eddy current damping).

```rust
pub struct TransformerHysteresis {
    // ... private state
}

impl TransformerHysteresis {
    pub fn new() -> Self;
    pub fn process(
        &mut self,
        input: f64,
        drive: f64,
        hysteresis: f64,
        frequency_dependency: f64,
    ) -> f64;
    pub fn reset(&mut self);
}
```

### 3. `nonlinear_stages` (`nonlinear_stages.rs`)
Nonlinear saturation modes, dynamic mirror pre/de-emphasis, physical slew-rate limiting, and inter-stage AC coupling.

```rust
pub const REFERENCE_VU_DBFS: f64 = -18.0;
pub const REFERENCE_VU_LINEAR: f64 = 0.125_892_541_179_416_7; // 10^(-18/20)

pub fn flush_denormal(x: f64) -> f64; // NaN/Inf/sfdenormal → 0.0
pub fn drive_with_unity_compensation(sample: f64, drive: f64, mode: NonlinearMode) -> f64;
pub fn process_nonlinear(sample: f64, drive: f64, mode: NonlinearMode) -> f64;
pub fn soft_clip<T: Float>(sample: T, drive: T) -> T;
pub fn tube_saturate<T: Float>(sample: T, drive: T) -> T;

pub struct SlewRateLimiter { /* ... */ }
pub struct AcCoupling { /* 1-pole HPF DC blocker ~5–10 Hz */ }
impl AcCoupling {
    pub fn configure(&mut self, sample_rate: f64, cutoff_hz: Option<f64>) -> bool;
    pub fn reset(&mut self);
    pub fn process(&mut self, sample: f64) -> f64;
}
```
`AcCoupling` is a clean one-pole high-pass (DC blocker) with a sample-rate-dependent cutoff (default ~7 Hz) and must be configured via `configure` before processing; call it at the end of any asymmetric non-linear chain to avoid DC-offset accumulation. `flush_denormal` is applied to feedback states throughout the core-dsp loop (ZDF SVF, transformer/AC-coupling states, FDN damping states).

### 4. `psu_sag` (`psu_sag.rs`)
Power supply voltage sag under heavy load transients with zero noise/ripple by default.

```rust
pub struct PsuSag<T: Float> {
    // ...
}

impl<T: Float> PsuSag<T> {
    pub fn new() -> Self;
    pub fn initialize(&mut self, sample_rate: T, recovery_seconds: T, depth: T) -> bool;
    pub fn set_noise_and_ripple(&mut self, noise_level: T, ripple_enabled: bool);
    pub fn gain(&mut self, input_level: T) -> T;
    pub fn reset(&mut self);
}
```

### 5. `analog_variance` (`analog_variance.rs`)
Deterministic instance-based L/R micro-tolerance and crosstalk generator.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalogVariance {
    pub left_gain: f64,
    pub right_gain: f64,
    pub left_time_scale: f64,
    pub right_time_scale: f64,
    pub crosstalk: f64,
}

impl AnalogVariance {
    pub fn from_seed(seed: u64, tolerance: f64, crosstalk_max: f64, analog_drift: f64) -> Self;
}
```

### 6. `plugin_export` (`plugin_export.rs`)
Unified metadata constants, categories, deterministic VST3 Class ID calculation (UUID RFC 4122 v4), and the `export_greg_plugin!` macro exporting natively both CLAP and VST3.

```rust
pub const VENDOR: &str = "Greg Audio";
pub const URL: &str = "https://github.com/gregslamba-cmyk/Analog-vibe";
pub const SUPPORT_URL: Option<&str> = Some("https://github.com/gregslamba-cmyk/Analog-vibe/issues");
pub const MANUAL_URL: Option<&str> = Some("https://github.com/gregslamba-cmyk/Analog-vibe");
pub const EMAIL: &str = "support@example.invalid";

pub mod categories {
    pub const DYNAMICS_COMPRESSOR_CLAP: &[ClapFeature];
    pub const DYNAMICS_COMPRESSOR_VST3: &[Vst3SubCategory];
    pub const DYNAMICS_LIMITER_CLAP: &[ClapFeature];
    pub const DYNAMICS_LIMITER_VST3: &[Vst3SubCategory];
    pub const EQUALIZER_CLAP: &[ClapFeature];
    pub const EQUALIZER_VST3: &[Vst3SubCategory];
    pub const UTILITY_CLAP: &[ClapFeature];
    pub const UTILITY_VST3: &[Vst3SubCategory];
}

pub const fn make_vst3_cid(id: &str) -> [u8; 16];

#[macro_export]
macro_rules! export_greg_plugin {
    // ($plugin_ty, $clap_id, $description)
    // ($plugin_ty, $clap_id, $description, compressor | limiter | equalizer | utility)
    // ($plugin_ty, $clap_id, $description, $clap_features, $vst3_subcategories)
    // ($plugin_ty, $clap_id, $description, $clap_features, $vst3_subcategories, $vst3_cid)
}
```

---

## Multi-Format Pipeline (CLAP, VST3, LV2, AUv2)

- **CLAP & VST3**: Dual-exported natively from the same compiled `cdylib` binary using `export_greg_plugin!`, exporting `clap_entry` alongside `GetPluginFactory` / `ModuleEntry`.
- **LV2 & AUv2**: Packaged via `scripts/package_all_formats.sh` conforming to `clap-wrapper` (MIT) specifications into standard `.lv2` (Linux) and `.component` (macOS) bundles.
- **Licensing**: Dual-licensed under GNU GPLv3 or later (open-source) and low-cost proprietary commercial license on request.

---

## 40. `spatial_3d` (`spatial_3d.rs`)

3D spatial positioning with binaural HRIR rendering, first-order Ambisonics, and VBAP panning.

```rust
pub struct SpeakerPosition { azimuth: f64, elevation: f64 }
pub struct HrtfCoefficients { gain_l: f64, gain_r: f64, itd_sec: f64, ild_db: f64 }
pub struct AmbisonicCoefficients { w: f64, x: f64, y: f64, z: f64 }
pub struct VbapState { positions: [SpeakerPosition; 64], num_speakers: usize, ... }

pub struct Spatial3dConfig {
    pub ambi_norm: bool,
    pub binaural: bool,
    pub sample_rate: f64,
    pub global_gain: f64,
}

impl Spatial3dConfig {
    pub fn new(sample_rate: f64) -> Self;
    pub fn defaults() -> Self;
}

pub struct Spatial3d {
    // private
}

impl Spatial3d {
    pub fn new() -> Self;
    pub fn configure(&mut self, config: Spatial3dConfig) -> bool;
    pub fn reset(&mut self);
    pub fn set_sample_rate(&mut self, sr: f64);

    pub fn process_sample(&mut self, input: f64, az: f64, el: f64, dist_m: f64) -> f64;
    pub fn process_stereo(&mut self, left: f64, right: f64, az: f64, el: f64, dist_m: f64) -> (f64, f64);
    pub fn process_ambisonic(&mut self, w: f64, x: f64, y: f64, z: f64) -> (f64, f64);

    pub fn hrtf_coefficients(az: f64, el: f64, dist_m: f64) -> HrtfCoefficients;
    pub fn ambisonic_from_direction(az: f64, el: f64) -> AmbisonicCoefficients;
    pub fn ambisonic_to_stereo(coeffs: AmbisonicCoefficients) -> (f64, f64);
}

impl VbapState {
    pub fn new() -> Self;
    pub fn configure_speakers(&mut self, speakers: &[SpeakerPosition]);
    pub fn vbap_weights(&mut self, az: f64, el: f64) -> (Option<[usize; 3]>, Vec<f64>);
}
```

Zero allocations in `process_*` after `configure()`. All f64 internal; NaN/Inf sanitized to 0.0.

---

## 41. `loudness_metering` (`loudness_metering.rs`)

EBU R128 / ITU-R BS.1770-4 loudness metering with K-weighting filter bank, momentary/short-term/integrated LUFS, absolute (-70 LUFS) and relative (-10 LU) gating, LRA, true-peak dBTP estimation (1.25× inter-sample factor), goniometer ring buffer, and lock-free `MasteringTelemetrySnapshot` publisher.

```rust
pub const REF_LUFS: f64 = -23.0;
pub const ABSOLUTE_GATE_LUFS: f64 = -70.0;
pub const RELATIVE_GATE_OFFSET_LU: f64 = -10.0;
pub const MOMENTARY_BLOCK_SEC: f64 = 0.4;
pub const SHORTTERM_BLOCK_SEC: f64 = 3.0;

pub struct KWeightHighShelf;
impl KWeightHighShelf { pub fn configure(&mut self, sample_rate: f64); pub fn process(&mut self, x: f64) -> f64; pub fn reset(&mut self); }

pub struct KWeightHighPass; // same API as KWeightHighShelf

pub struct KWeightFilter { hs: KWeightHighShelf, hp: KWeightHighPass };
impl KWeightFilter { pub fn configure(&mut self, sample_rate: f64); pub fn process(&mut self, x: f64) -> f64; pub fn reset(&mut self); }

pub struct GoniometerRing;
impl GoniometerRing { pub fn new(capacity: usize) -> Self; pub fn push(&mut self, left: f64, right: f64); pub fn snapshot(&self) -> (Vec<f64>, Vec<f64>); }

pub struct TruePeakDetector;
impl TruePeakDetector { pub fn new() -> Self; pub fn reset(&mut self); pub fn process(&mut self, x: f64); pub fn true_peak_amplitude(&self) -> f64; }

pub struct LoudnessEngine {
    k_weight: KWeightFilter,
    true_peak: TruePeakDetector,
    goniometer: GoniometerRing,
    /* sliding-window sums + integrated block history */
}

impl LoudnessEngine {
    pub fn new(sample_rate: f64, goniometer_capacity: usize) -> Self;
    pub fn configure_sample_rate(&mut self, sample_rate: f64);
    pub fn reset(&mut self);
    pub fn process_sample(&mut self, left: f64, right: f64);
    pub fn add_block(&mut self, ms_sum_sq: f64);
    pub fn momentary_lufs(&self) -> f64;
    pub fn short_term_lufs(&self) -> f64;
    pub fn integrated_lufs(&self) -> f64;
    pub fn lra(&self) -> f64;
    pub fn true_peak_dbtp(&self) -> f64;
    pub fn goniometer_snapshot(&self) -> (Vec<f64>, Vec<f64>);
}

pub struct MasteringTelemetrySnapshot {
    pub momentary_lufs: f64, pub short_term_lufs: f64, pub integrated_lufs: f64,
    pub lra_lu: f64, pub true_peak_dbtp: f64, pub crest_factor_db: f64,
    pub psr_db: f64, pub phase_correlation: f64, pub dominant_peak_hz: f64,
}

impl MasteringTelemetrySnapshot { /* publishable atomic snapshot */ }

pub struct MasteringTelemetryPublisher; // atomic [AtomicU64; 9] Relaxed snapshot
impl MasteringTelemetryPublisher {
    pub fn new() -> Self;
    pub fn publish(&self, s: &MasteringTelemetrySnapshot);
    pub fn snapshot(&self) -> MasteringTelemetrySnapshot;
}
```

Thread-safe audio→UI communication via `[AtomicU64; 9]` stores with `Ordering::Relaxed`. Zero locks, zero heap in `process_sample`. Silence-at-rest: all meters read `-120 dBFS` / `-70 LUFS` before signal present.

---

## 42. `dsp_graph` — Topological Audio Graph & Automatic PDC Engine

### Guarantees
- Real-time execution via `CompiledAudioGraph::process()`: zero heap allocations (`Vec::new()`, `Box::new()`), zero locks, pre-allocated `AudioEdgeBuffer` up to `MAX_AUDIO_CHANNELS = 8` channels.
- Cycle validation: cycle detection during `AudioGraphBuilder::compile()` forbids illegal un-delayed loops, while allowing explicit feedback loops via `FeedbackDelayZ1Node`.
- Automatic Plugin Delay Compensation (PDC): calculates cumulative branch latency across parallel execution paths and automatically injects sample-accurate delay compensation nodes before summing nodes.
- Full `NaN`/`Inf` sanitization and `PanicProtection` per-node execution boundary.

### Public Signatures

```rust
pub const MAX_AUDIO_CHANNELS: usize = 8;

pub trait DspGraphNode {
    fn initialize(&mut self, sample_rate: f64, max_block_size: usize);
    fn reset(&mut self);
    fn latency_samples(&self) -> usize;
    fn process_node(
        &mut self,
        inputs: &[&[f64]],
        outputs: &mut [&mut [f64]],
        num_channels: usize,
        num_samples: usize,
    );
    fn offload_eligible(&self) -> bool;
    fn node_name(&self) -> &'static str;
}

// Built-in Nodes
pub struct GainPanNode; // smoothed gain + pan node
pub struct SplitterNode; // 1-to-N parallel signal splitter
pub struct SummerMixerNode; // N-to-1 summing mixer
pub struct ParallelDryWetNode; // phase-aligned parallel dry/wet crossfader
pub struct FeedbackDelayZ1Node; // 1-sample feedback delay for closed loops

pub struct AudioGraphBuilder {
    pub fn new() -> Self;
    pub fn add_node(&mut self, node: Box<dyn DspGraphNode + Send>) -> usize;
    pub fn connect(&mut self, from_node: usize, from_port: usize, to_node: usize, to_port: usize);
    pub fn compile(self, sample_rate: f64, max_block_size: usize) -> Result<CompiledAudioGraph, GraphError>;
}

pub struct CompiledAudioGraph {
    pub fn process(&mut self, inputs: &[&[f64]], outputs: &mut [&mut [f64]], num_channels: usize, num_samples: usize);
    pub fn reset(&mut self);
    pub fn total_latency_samples(&self) -> usize;
}

#[derive(Debug)]
pub enum GraphError {
    CycleDetected,
    InvalidNodeIndex(usize),
    NoNodes,
    ChannelCountExceeded,
}
```

---

## 43. `analog_summing` — Physical Non-Linear Console Summing & Crosstalk

### Guarantees
- `AdjacentCrosstalkMatrix`: models physical capacitive and inductive bleeding between adjacent console channels (`N ↔ N+1`) and stereo pairs (`L ↔ R`) via one-pole high-pass coupling filters with calibrated attenuation in dB. Zero heap in `process()`.
- `VirtualEarthSummer`: models the non-linear physics of a 32-channel (16 stereo pairs) virtual earth summing mix bus. Dynamically computes instantaneous power supply rail droop (PSU sag) proportional to total summed current, bus impedance compression, and smooth saturation before the master output stage. Zero `NaN`/`Inf`.

### Public Signatures

```rust
pub struct AdjacentCrosstalkMatrix {
    pub fn new(crosstalk_db: f64, sample_rate: f64) -> Self;
    pub fn set_crosstalk_db(&mut self, db: f64);
    pub fn initialize(&mut self, sample_rate: f64);
    pub fn reset(&mut self);
    pub fn process(&mut self, channels: &mut [&mut [f64]], num_samples: usize);
}

pub struct VirtualEarthSummer {
    pub fn new(max_channels: usize, sample_rate: f64) -> Self;
    pub fn set_bus_drive(&mut self, drive_db: f64);
    pub fn set_psu_sag(&mut self, sag_amount: f64);
    pub fn initialize(&mut self, sample_rate: f64);
    pub fn reset(&mut self);
    pub fn process_sum(
        &mut self,
        stereo_pairs: &[(&[f64], &[f64])],
        out_left: &mut [f64],
        out_right: &mut [f64],
        num_samples: usize,
    );
}
```

---

## 44. `synth_engine` — Universal Polyphonic Synthesis Engine (MPE + Step Sequencer)

### Guarantees
- `f64` internal processing, `#![forbid(unsafe_code)]`, zero heap allocation in `process()`.
- PolyBLEP-antialiased oscillators (sine, square, saw, triangle, deterministic noise).
- Multimode ZDF state-variable filter (LowPass / HighPass / BandPass / Notch).
- ADSR envelope with sample-rate-scaled attack/decay/release and gate hysteresis.
- MPE (MIDI Polyphonic Expression) manager: per-note pitch-bend, timbre (CC74), and pressure with configurable zones.
- Step sequencer with per-step trigger, roll, gate, probability, and BPM-synced timing via `tempo_sync`.
- NaN/Inf sanitized to `0.0`; digital silence at rest (`0.0 in -> 0.0 out`).

### Public Signatures

```rust
pub struct Oscillator { /* phase, waveform, seed */ }
pub enum OscillatorWaveform { Sine, Square, Saw, Triangle, Noise }
impl Oscillator {
    pub fn new(waveform: OscillatorWaveform) -> Self;
    pub fn reset(&mut self);
    pub fn process(&mut self, freq_hz: f64, sample_rate: f64) -> f64; // BLEP
}

pub enum FilterMode { LowPass, HighPass, BandPass, Notch }
pub struct MultiModeFilter { /* ZdfSvf + mode */ }
impl MultiModeFilter {
    pub fn new() -> Self;
    pub fn configure(&mut self, sample_rate: f64, freq_hz: f64, q: f64, mode: FilterMode) -> bool;
    pub fn set_mode(&mut self, mode: FilterMode);
    pub fn reset(&mut self);
    pub fn process(&mut self, input: f64) -> f64;
}

pub enum EnvelopeStage { Idle, Attack, Decay, Sustain, Release }
pub struct AdsrEnvelope { /* stage, value, coeffs, gate */ }
impl AdsrEnvelope {
    pub fn new() -> Self;
    pub fn configure(&mut self, sample_rate: f64, attack_ms: f64, decay_ms: f64, sustain: f64, release_ms: f64) -> bool;
    pub fn set_gate(&mut self, gate: bool);
    pub fn reset(&mut self);
    pub fn process(&mut self) -> f64;
    pub fn is_idle(&self) -> bool;
}

pub struct Voice { /* oscillator + filter + envelope + params */ }
impl Voice {
    pub fn new(note_number: u8, velocity: f64, sample_rate: f64) -> Self;
    pub fn note_on(&mut self, note_number: u8, velocity: f64, sample_rate: f64);
    pub fn note_off(&mut self, sample_rate: f64);
    pub fn set_waveform(&mut self, waveform: OscillatorWaveform);
    pub fn set_filter_mode(&mut self, mode: FilterMode);
    pub fn reset(&mut self);
    pub fn process(&mut self) -> f64;
}

pub struct MpeZone { lower_channel: u8, upper_channel: u8, pitch_bend_range_semitones: f64, per_note_pitch_bend: bool }
impl MpeZone {
    pub fn new(lower: u8, upper: u8, range: f64) -> Self;
    pub fn default_lower() -> Self;
    pub fn default_upper() -> Self;
    pub fn contains_channel(&self, ch: u8) -> bool;
}

pub struct MpeManager { lower_zone: MpeZone, upper_zone: MpeZone, note_states: [MpeNoteState; 128] }
impl MpeManager {
    pub fn new() -> Self;
    pub fn configure_zones(&mut self, lower: MpeZone, upper: MpeZone);
    pub fn handle_note_on(&mut self, channel: u8, note: u8, velocity: f64);
    pub fn handle_note_off(&mut self, channel: u8, note: u8);
    pub fn handle_pitch_bend(&mut self, channel: u8, value: i16);
    pub fn handle_cc74(&mut self, channel: u8, value: u8);
    pub fn handle_pressure(&mut self, channel: u8, value: u8);
    pub fn get_note_state(&self, note: u8) -> MpeNoteState;
    pub fn reset(&mut self);
}

pub fn pitch_bend_to_ratio(bend: f64, range_semitones: f64) -> f64;
pub fn timbre_to_filter_mod(timbre: f64, base_freq: f64, max_mod: f64) -> f64;
pub fn pressure_to_amp_mod(pressure: f64) -> f64;

pub struct SynthEngine { voices: [Voice; 64], num_voices: usize, sample_rate: f64, mpe: MpeManager }
impl SynthEngine {
    pub fn new(sample_rate: f64, max_voices: usize) -> Self;
    pub fn reset(&mut self);
    pub fn set_sample_rate(&mut self, sample_rate: f64);
    pub fn note_on(&mut self, note: u8, velocity: f64) -> bool;
    pub fn note_off(&mut self, note: u8);
}
```

---

## Nuovi Moduli Console & Routing (`core-dsp`)
- **`aux_send` (`core_dsp_modules::aux_send`):** Deviazione di segnale multi-mandata Pre/Post-Fader a zero allocazioni heap, con smussamento anti-zipper del livello di mandata (`-∞ .. +6 dB`) e instradamento verso bus interni o porte Aux esterne CLAP/VST3.
- **`fader_console` (`core_dsp_modules::fader_console`):** Stadio fader professionale da console analogica (`-∞ .. +10 dB` con legge balistica reale da banco), leggi di Pan costante (`-3.0 dB`, `-4.5 dB`, `-6.0 dB`), `Mute` click-free e gestione `Solo / Solo-Safe`.

---

## Pattern Multicanale a Grafo (`MonoChannelProcessor` + `MultiChannelGraph<P>`)
- **Principio:** Ogni plugin istanzia un array pre-allocato in `initialize()` di `N` motori mono indipendenti (`[P; MAX_AUDIO_CHANNELS]`, dove `P: MonoChannelProcessor`) gestiti da un grafo interno (`MultiChannelGraph<P>`).
- **Flusso del Grafo:**
  1. **Pre-Routing (`RoutingMatrix`):** Pass-through diretto `0..N` oppure codifica ortonormale `Mid/Side` (`L/R → M/S`) prima dei motori mono.
  2. **Detector Link Opzionale:** Per i processori dinamici in modalità `Stereo Link` / `Surround Link`, il grafo calcola la gain reduction condivisa tra i canali attivi lasciando indipendenti gli stadi fisici (trasformatori, valvole, filtri).
  3. **Post-Routing & PDC:** Decodifica `M/S → L/R` (se attiva), compensazione latenza Dry/Wet al singolo campione e `BypassCrossfade` click-free.
  4. **Detector Multicanale Adattivo (`N` canali):**
     - Rilevamento automatico dei canali attivi (`buffer.channels()`, sovrascrivibile tramite `Channel Setup`).
     - Supporto a 4 logiche di link del detector: **Unlinked (Multi-Mono)**, **All-Linked (Stereo/Bus Link)**, **Surround 5.1/7.1 Stem Link** (esclude il canale LFE `.1` dal trigger comune per evitare pumping da sub) e **Mid/Side (`M/S`)**.

---

## Evoluzioni v2.0 Previste per `core-dsp` (Neural, Object-Based 128ch, Smart Sampler)
- **`neural_capture` (`core_dsp_modules::neural_capture`):** Inferenza real-time zero-alloc in Rust puro per modelli già addestrati (`.nam` / `RTNeural` / `LSTM` / `WaveNet` caricati in `initialize()`) + identificazione matematica rapida *Hammerstein-Wiener* senza addestramento GPU.
- **`spatial_3d` & `routing_matrix` (128 Canali Object-Based):** Estensione dei buffer statici da `8` a `128` canali (7.1.4, 9.1.6, Atmos object-based) con `AudioObjectPanner` (Azimuth, Elevation, Distance, Spread) smussato e downmix HRIR binaurale simultaneo.
- **`mpc_sampler_engine` & `pitch_shifter` (Smart Multisample & Round-Robin):** Generazione di round-robin organici anti-mitragliatrice (micro-variazioni di fase, risonanza ZDF e transiente), interpolazione continua su 127 velocity layer da 2–4 sample reali e trasposizione *formant-preserving*.

---

## Crate Host Multi-Formato (`crates/greg_plugin_host`)
- **Motivazione:** Mantiene `core-dsp` puro (`#![forbid(unsafe_code)]`), isolando il caricamento dinamico FFI (`libloading`) in una crate dedicata riutilizzabile da `greg_bay`, `greg_nl_mixer` e dai Channel Strip.
- **Formati supportati:** **CLAP** (priorità 1, multi-piattaforma), **LV2 / LADSPA** (Linux nativo), **VST3** (COM ABI) e **AUv2** (`#[cfg(target_os = "macos")]`).
- **Integrazione Grafo:** Ogni istanza implementerà il trait `DspGraphNode` di `core-dsp`, garantendo PDC automatico al singolo campione, isolamento nel thread non-audio per allocazioni/scansione e serializzazione dello stato binario.

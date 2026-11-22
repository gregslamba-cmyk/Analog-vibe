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


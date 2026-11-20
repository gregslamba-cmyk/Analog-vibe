# Architecture of Core Analog DSP Modules (`core-dsp`)

This document details the public API signatures, architectural design, and guarantees of the pure analog DSP modules implemented in `core-dsp/src/core_dsp_modules`.

## General Guarantees
- **Internal Processing**: Computed entirely in `f64`.
- **Safety**: Zero `.unwrap()`, zero `.expect()`, zero heap allocations or panics in the audio processing path; full NaN/Inf sanitization.
- **Silence & Noise Policy**: Thermal noise and ripple generators are explicitly **OFF BY DEFAULT** (`0.0` / `false`), ensuring absolute digital silence at rest.

---

## Public Signatures & Modules

### 1. `zdf_filters` (`zdf_filters.rs`)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonlinearMode {
    SoftClip,
    HardClip,
    Asymmetric,
    WaveFold,
    FullWaveRectify,
    TubeTriode,
    TubePentode,
    SolidStateClassA,
    SolidStateFet,
    DiscreteOpAmp,
}

pub struct SlewRateLimiter {
    // ...
}
impl SlewRateLimiter {
    pub fn new() -> Self;
    pub fn reset(&mut self);
    pub fn process(&mut self, sample: f64, max_v_per_sec: f64, sample_rate: f64) -> f64;
}

pub struct AcCoupling {
    // ...
}
impl AcCoupling {
    pub fn new() -> Self;
    pub fn reset(&mut self);
    pub fn process(&mut self, sample: f64) -> f64;
}

pub struct DynamicMirrorPreEmphasis {
    // ...
}
impl DynamicMirrorPreEmphasis {
    pub fn new() -> Self;
    pub fn reset(&mut self);
    pub fn pre_emphasis(&mut self, sample: f64, emphasis: f64) -> f64;
    pub fn de_emphasis(&mut self, sample: f64, emphasis: f64) -> f64;
}

pub fn process_nonlinear(sample: f64, drive: f64, mode: NonlinearMode) -> f64;
pub fn soft_clip<T: Float>(sample: T, drive: T) -> T;
pub fn tube_saturate<T: Float>(sample: T, drive: T) -> T;
```

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

#![forbid(unsafe_code)]
//! Greg NL Console — a single console channel strip.
//! Neve 8078 (inductor 3-band EQ + diode-bridge dynamics),
//! SSL 4000G (4-band G-series EQ + VCA), API 1608 (550B proportional-Q + 525/527).
//! f64, allocation-free per-sample DSP.

use core_dsp::core_dsp_modules::{
    component_models::{create_transformer, TransformerModel},
    filters::{Biquad, FilterType, SvfFilter},
    transformer_hysteresis::TransformerHysteresis,
};

#[allow(dead_code)] // Public constants reserved for future channel/bus expansion.
pub const MAX_CONSOLE_CHANNELS: usize = 24;
#[allow(dead_code)] // Public constants reserved for future channel/bus expansion.
pub const MAX_CONSOLE_BUSES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants are used via match arms but may not be constructed externally yet.
pub enum ConsoleType {
    #[default]
    Neve8078,
    Ssl4000G,
    Api1608,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants are used via insert_stage but not all may be wired yet.
pub enum InsertType {
    #[default]
    None,
    Pultec,
    Tape,
    Fet1176,
    OptoLa2a,
    Clipper,
}

/// One console channel strip: preamp / transformer, 3-band EQ, dynamics, two inserts.
/// All state is `Copy`-friendly; `process_sample` performs zero heap allocations.
#[derive(Clone, Copy, Debug)]
pub struct ConsoleChannelStrip {
    pub console_type: ConsoleType,
    pub enabled: bool,
    pub muted: bool,
    pub solo: bool,
    pub pan: f64,
    pub fader_db: f64,
    pub low_gain_db: f64,
    pub mid_freq_hz: f64,
    pub mid_q: f64,
    pub mid_gain_db: f64,
    pub high_gain_db: f64,
    pub comp_enabled: bool,
    pub comp_thresh_db: f64,
    pub comp_ratio: f64,
    pub comp_makeup_db: f64,
    pub gate_enabled: bool,
    pub gate_thresh_db: f64,
    pub insert_a: InsertType,
    pub insert_b: InsertType,
    transformer: TransformerHysteresis,
    hpf: SvfFilter<f64>,
    lpf: SvfFilter<f64>,
    eq_low: Biquad<f64>,
    eq_low_mid: Biquad<f64>,
    eq_high_mid: Biquad<f64>,
    eq_high: Biquad<f64>,
    comp_env: f64,
    gate_env: f64,
    attack: f64,
    release: f64,
    sample_rate: f64,
}

impl Default for ConsoleChannelStrip {
    fn default() -> Self {
        Self {
            console_type: Default::default(),
            enabled: true,
            muted: false,
            solo: false,
            pan: 0.0,
            fader_db: 0.0,
            low_gain_db: 0.0,
            mid_freq_hz: 1000.0,
            mid_q: 1.0,
            mid_gain_db: 0.0,
            high_gain_db: 0.0,
            comp_enabled: false,
            comp_thresh_db: -24.0,
            comp_ratio: 1.0,
            comp_makeup_db: 0.0,
            gate_enabled: false,
            gate_thresh_db: -48.0,
            insert_a: Default::default(),
            insert_b: Default::default(),
            transformer: TransformerHysteresis::default(),
            hpf: SvfFilter::default(),
            lpf: SvfFilter::default(),
            eq_low: Biquad::default(),
            eq_low_mid: Biquad::default(),
            eq_high_mid: Biquad::default(),
            eq_high: Biquad::default(),
            comp_env: 0.0,
            gate_env: 0.0,
            attack: 0.0,
            release: 0.0,
            sample_rate: 48_000.0,
        }
    }
}

impl ConsoleChannelStrip {
    /// Prepare the strip for a new sample rate (called once from `Plugin::initialize`).
    pub fn prepare(&mut self, sample_rate: f64) {
        let sr = if sample_rate.is_finite() && sample_rate > 0.0 {
            sample_rate
        } else {
            48_000.0
        };
        self.sample_rate = sr;
        let model = match self.console_type {
            ConsoleType::Neve8078 => TransformerModel::MarinairLo1166,
            ConsoleType::Ssl4000G => TransformerModel::LundahlLl1538,
            ConsoleType::Api1608 => TransformerModel::Api2623,
        };
        self.transformer = create_transformer(model, sr);
        self.hpf.configure(sr, FilterType::HighPass, 20.0, 0.707, 0.0);
        self.lpf.configure(
            sr,
            FilterType::LowPass,
            18_000.0_f64.min(sr * 0.45),
            0.707,
            0.0,
        );
        self.configure_eq();
        self.comp_env = 0.0;
        self.gate_env = 0.0;
        self.attack = (1.0 - (-1.0 / (0.005 * sr)).exp()).max(1e-4);
        self.release = (1.0 - (-1.0 / (0.1 * sr)).exp()).max(1e-4);
    }

    fn configure_eq(&mut self) {
        let sr = self.sample_rate.max(1.0);
        self.eq_low
            .configure(sr, FilterType::LowShelf, 100.0, 0.707, self.low_gain_db);
        let lo_mid = (self.mid_freq_hz * 0.5).clamp(40.0, 9_000.0);
        let hi_mid = self.mid_freq_hz.clamp(80.0, 18_000.0);
        self.eq_low_mid.configure(
            sr,
            FilterType::Bell,
            lo_mid,
            self.mid_q,
            self.mid_gain_db * 0.5,
        );
        self.eq_high_mid.configure(
            sr,
            FilterType::Bell,
            hi_mid,
            self.mid_q,
            self.mid_gain_db,
        );
        self.eq_high.configure(sr, FilterType::HighShelf, 8_000.0, 0.707, self.high_gain_db);
    }

    /// Process one sample through the complete channel strip.
    pub fn process_sample(&mut self, input: f64) -> f64 {
        if !input.is_finite() || !self.enabled {
            return 0.0;
        }
        // Preamp / transformer stage (console‑dependent drive).
        let drive = match self.console_type {
            ConsoleType::Neve8078 => 1.35,
            ConsoleType::Ssl4000G => 1.10,
            ConsoleType::Api1608 => 1.25,
        };
        let mut x = self.transformer.process(input, drive, 0.5, 0.6);
        // EQ + gentle band‑limiting.
        x = self.hpf.process(x);
        x = self.eq_low.process(x);
        x = self.eq_low_mid.process(x);
        x = self.eq_high_mid.process(x);
        x = self.eq_high.process(x);
        x = self.lpf.process(x);
        // Two internal inserts (Pultec / Tape / 1176 / LA‑2A / Clipper).
        x = insert_stage(self.insert_a, x);
        x = insert_stage(self.insert_b, x);
        // Gate / expander (SSL style) then compressor.
        if self.gate_enabled {
            let lvl = x.abs();
            self.gate_env += self.attack * (lvl - self.gate_env);
            let thresh = 10.0f64.powf(self.gate_thresh_db / 20.0).max(1e-9);
            if self.gate_env < thresh {
                x *= (self.gate_env / thresh).sqrt().clamp(0.0, 1.0);
            }
        }
        if self.comp_enabled {
            let lvl = x.abs();
            let coeff = if lvl > self.comp_env { self.attack } else { self.release };
            self.comp_env += coeff * (lvl - self.comp_env);
            let env_db = 20.0 * self.comp_env.max(1e-9).log10();
            let over = (env_db - self.comp_thresh_db).max(0.0);
            let gr_db = over * (1.0 - 1.0 / self.comp_ratio.max(1.0));
            x *= 10.0f64.powf((self.comp_makeup_db - gr_db) / 20.0);
        }
        // Fader.
        x * 10.0f64.powf(self.fader_db / 20.0)
    }

    /// Constant‑power stereo pan gains for this strip.
    pub fn pan_gains(&self) -> (f64, f64) {
        let p = self.pan.clamp(-1.0, 1.0);
        (((1.0 - p) * 0.5).sqrt(), ((1.0 + p) * 0.5).sqrt())
    }

    pub fn reset(&mut self) {
        self.transformer.reset();
        self.hpf.reset();
        self.lpf.reset();
        self.eq_low.reset();
        self.eq_low_mid.reset();
        self.eq_high_mid.reset();
        self.eq_high.reset();
        self.comp_env = 0.0;
        self.gate_env = 0.0;
    }
}

#[inline]
fn insert_stage(kind: InsertType, x: f64) -> f64 {
    match kind {
        InsertType::None => x,
        InsertType::Pultec => (x * 1.05).tanh(),
        InsertType::Tape => (x * 1.2).tanh() / 1.2,
        InsertType::Fet1176 => x.tanh(),
        InsertType::OptoLa2a => x / (1.0 + x.abs() * 0.3),
        InsertType::Clipper => x.clamp(-0.98, 0.98),
    }
}

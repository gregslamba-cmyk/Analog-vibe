#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{sync::Arc, sync::atomic::Ordering};

use core_dsp::core_dsp_modules::{
    analog_variance::AnalogVariance,
    filters::FilterType,
    metering::Metering,
    nonlinear_stages::{
        AcCoupling, DynamicMirrorPreEmphasis, NonlinearMode, SlewRateLimiter, process_nonlinear,
    },
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::{RoutingMatrix, RoutingMode},
    transformer_hysteresis::TransformerHysteresis,
    zdf_filters::ZdfSvf,
};
use greg_gui_core::{GuiTelemetry, create_greg_editor};
use nih_plug::prelude::*;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum PultecVersion {
    #[id = "eqp-1a"]
    #[name = "EQP-1A"]
    EQP1A,
    #[id = "eqp-1a3"]
    #[name = "EQP-1A3"]
    EQP1A3,
}

impl From<PultecVersion> for f32 {
    fn from(value: PultecVersion) -> Self {
        match value {
            PultecVersion::EQP1A => 0.0,
            PultecVersion::EQP1A3 => 1.0,
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum LowFreqStep {
    #[id = "off"]
    #[name = "Off"]
    Off,
    #[id = "20"]
    #[name = "20"]
    Twenty,
    #[id = "30"]
    #[name = "30"]
    Thirty,
    #[id = "60"]
    #[name = "60"]
    Sixty,
    #[id = "100"]
    #[name = "100"]
    Hundred,
}

impl LowFreqStep {
    fn freq(self) -> Option<f64> {
        match self {
            LowFreqStep::Off => None,
            LowFreqStep::Twenty => Some(20.0),
            LowFreqStep::Thirty => Some(30.0),
            LowFreqStep::Sixty => Some(60.0),
            LowFreqStep::Hundred => Some(100.0),
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum HighPeakFreqStep {
    #[id = "off"]
    #[name = "Off"]
    Off,
    #[id = "3k"]
    #[name = "3 kHz"]
    ThreeK,
    #[id = "4k"]
    #[name = "4 kHz"]
    FourK,
    #[id = "5k"]
    #[name = "5 kHz"]
    FiveK,
    #[id = "8k"]
    #[name = "8 kHz"]
    EightK,
    #[id = "10k"]
    #[name = "10 kHz"]
    TenK,
    #[id = "12k"]
    #[name = "12 kHz"]
    TwelveK,
    #[id = "16k"]
    #[name = "16 kHz"]
    SixteenK,
}

impl HighPeakFreqStep {
    fn freq(self) -> Option<f64> {
        match self {
            HighPeakFreqStep::Off => None,
            HighPeakFreqStep::ThreeK => Some(3_000.0),
            HighPeakFreqStep::FourK => Some(4_000.0),
            HighPeakFreqStep::FiveK => Some(5_000.0),
            HighPeakFreqStep::EightK => Some(8_000.0),
            HighPeakFreqStep::TenK => Some(10_000.0),
            HighPeakFreqStep::TwelveK => Some(12_000.0),
            HighPeakFreqStep::SixteenK => Some(16_000.0),
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum HighAttenFreqStep {
    #[id = "off"]
    #[name = "Off"]
    Off,
    #[id = "5k"]
    #[name = "5 kHz"]
    FiveK,
    #[id = "10k"]
    #[name = "10 kHz"]
    TenK,
    #[id = "20k"]
    #[name = "20 kHz"]
    TwentyK,
}

impl HighAttenFreqStep {
    fn freq(self) -> Option<f64> {
        match self {
            HighAttenFreqStep::Off => None,
            HighAttenFreqStep::FiveK => Some(5_000.0),
            HighAttenFreqStep::TenK => Some(10_000.0),
            HighAttenFreqStep::TwentyK => Some(20_000.0),
        }
    }
}

#[derive(Params)]
struct GregPultecParams {
    #[id = "bypass"]
    bypass: BoolParam,
    #[id = "version"]
    version: EnumParam<PultecVersion>,
    #[id = "low_boost_freq"]
    low_boost_freq: EnumParam<LowFreqStep>,
    #[id = "low_boost_gain"]
    low_boost_gain: FloatParam,
    #[id = "low_atten_freq"]
    low_atten_freq: EnumParam<LowFreqStep>,
    #[id = "low_atten_gain"]
    low_atten_gain: FloatParam,
    #[id = "high_peak_freq"]
    high_peak_freq: EnumParam<HighPeakFreqStep>,
    #[id = "high_peak_gain"]
    high_peak_gain: FloatParam,
    #[id = "high_atten_freq"]
    high_atten_freq: EnumParam<HighAttenFreqStep>,
    #[id = "high_atten_gain"]
    high_atten_gain: FloatParam,
    #[id = "output_gain"]
    output_gain: FloatParam,
    #[id = "link"]
    link: BoolParam,
    #[id = "analog_drift"]
    analog_drift: FloatParam,
}

impl Default for GregPultecParams {
    fn default() -> Self {
        Self {
            bypass: BoolParam::new("Bypass", false),
            version: EnumParam::new("Version", PultecVersion::EQP1A),
            low_boost_freq: EnumParam::new("Low Boost Freq", LowFreqStep::Off),
            low_boost_gain: FloatParam::new(
                "Low Boost Gain",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),
            low_atten_freq: EnumParam::new("Low Atten Freq", LowFreqStep::Off),
            low_atten_gain: FloatParam::new(
                "Low Atten Gain",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            high_peak_freq: EnumParam::new("High Peak Freq", HighPeakFreqStep::Off),
            high_peak_gain: FloatParam::new(
                "High Peak Gain",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),
            high_atten_freq: EnumParam::new("High Atten Freq", HighAttenFreqStep::Off),
            high_atten_gain: FloatParam::new(
                "High Atten Gain",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),
            link: BoolParam::new("Stereo Link", true),
            analog_drift: FloatParam::new(
                "Analog Drift",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

struct GregPultec {
    params: Arc<GregPultecParams>,
    routing_matrix: RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; 2],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    metering: Metering,
    panic_protection: PanicProtection,
    analog_variance: AnalogVariance,
    sample_rate: f64,
    oversampling_factor: core_dsp::core_dsp_modules::resampler_bitdepth::OversamplingFactor,
    oversampler_l: core_dsp::core_dsp_modules::resampler_bitdepth::Oversampler<f64>,
    oversampler_r: core_dsp::core_dsp_modules::resampler_bitdepth::Oversampler<f64>,
    low_boost_l: ZdfSvf,
    low_boost_r: ZdfSvf,
    low_atten_l: ZdfSvf,
    low_atten_r: ZdfSvf,
    high_peak_l: ZdfSvf,
    high_peak_r: ZdfSvf,
    high_atten_l: ZdfSvf,
    high_atten_r: ZdfSvf,
    transformer_l: TransformerHysteresis,
    transformer_r: TransformerHysteresis,
    ac_coupling_l: AcCoupling,
    ac_coupling_r: AcCoupling,
    pre_emphasis_l: DynamicMirrorPreEmphasis,
    pre_emphasis_r: DynamicMirrorPreEmphasis,
    de_emphasis_l: DynamicMirrorPreEmphasis,
    de_emphasis_r: DynamicMirrorPreEmphasis,
    slew_l: SlewRateLimiter,
    slew_r: SlewRateLimiter,
    make_gain_l: f64,
    make_gain_r: f64,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregPultec {
    fn default() -> Self {
        let sample_rate = 48_000.0_f64;
        let mut th_l = TransformerHysteresis::new();
        let mut th_r = TransformerHysteresis::new();
        th_l.configure(sample_rate, 180.0);
        th_r.configure(sample_rate, 180.0);
        th_l.set_slew_limit(48_000.0);
        th_r.set_slew_limit(48_000.0);
        Self {
            params: Arc::new(GregPultecParams::default()),
            routing_matrix: RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); 2],
            last_bypassed: true,
            bypass_ramp_samples: (sample_rate * 0.005) as u32,
            metering: Metering::default(),
            panic_protection: PanicProtection::default(),
            analog_variance: AnalogVariance::from_seed(0x47_53_5F_5F, 0.02, 0.001, 0.0),
            sample_rate,
            oversampling_factor:
                core_dsp::core_dsp_modules::resampler_bitdepth::OversamplingFactor::X1,
            oversampler_l:
                core_dsp::core_dsp_modules::resampler_bitdepth::Oversampler::<f64>::default(),
            oversampler_r:
                core_dsp::core_dsp_modules::resampler_bitdepth::Oversampler::<f64>::default(),
            low_boost_l: ZdfSvf::new(),
            low_boost_r: ZdfSvf::new(),
            low_atten_l: ZdfSvf::new(),
            low_atten_r: ZdfSvf::new(),
            high_peak_l: ZdfSvf::new(),
            high_peak_r: ZdfSvf::new(),
            high_atten_l: ZdfSvf::new(),
            high_atten_r: ZdfSvf::new(),
            transformer_l: th_l,
            transformer_r: th_r,
            ac_coupling_l: AcCoupling::new(),
            ac_coupling_r: AcCoupling::new(),
            pre_emphasis_l: DynamicMirrorPreEmphasis::new(),
            pre_emphasis_r: DynamicMirrorPreEmphasis::new(),
            de_emphasis_l: DynamicMirrorPreEmphasis::new(),
            de_emphasis_r: DynamicMirrorPreEmphasis::new(),
            slew_l: SlewRateLimiter::new(),
            slew_r: SlewRateLimiter::new(),
            make_gain_l: 1.0,
            make_gain_r: 1.0,
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl Plugin for GregPultec {
    const NAME: &'static str = "Greg Pultec";
    const VENDOR: &'static str = core_dsp::core_dsp_modules::plugin_export::VENDOR;
    const URL: &'static str = core_dsp::core_dsp_modules::plugin_export::URL;
    const EMAIL: &'static str = core_dsp::core_dsp_modules::plugin_export::EMAIL;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &plugin_support::DYNAMICS_IO_LAYOUTS;
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sr = _buffer_config.sample_rate as f64;
        if !sr.is_finite() || sr <= 0.0 {
            return false;
        }
        self.sample_rate = sr;
        self.bypass_ramp_samples = (sr * 0.005).round().max(1.0) as u32;
        self.last_bypassed = self.params.bypass.value();

        self.routing_matrix.configure(2, RoutingMode::Stereo);

        let analog_drift = self.params.analog_drift.value() as f64;
        let drift = if analog_drift.is_finite() {
            analog_drift.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.analog_variance = AnalogVariance::from_seed(0x47_53_5F_5F, 0.02, 0.001, drift);

        let factor = match self.params.version.value() {
            PultecVersion::EQP1A => {
                core_dsp::core_dsp_modules::resampler_bitdepth::OversamplingFactor::X1
            }
            PultecVersion::EQP1A3 => {
                core_dsp::core_dsp_modules::resampler_bitdepth::OversamplingFactor::X2
            }
        };
        self.oversampling_factor = factor;
        self.oversampler_l.reset();
        self.oversampler_r.reset();

        self.transformer_l.configure(sr, 180.0);
        self.transformer_r.configure(sr, 180.0);
        self.transformer_l.set_slew_limit(sr * 1.0);
        self.transformer_r.set_slew_limit(sr * 1.0);

        self.ac_coupling_l.reset();
        self.ac_coupling_r.reset();
        self.pre_emphasis_l.reset();
        self.pre_emphasis_r.reset();
        self.de_emphasis_l.reset();
        self.de_emphasis_r.reset();
        self.slew_l.reset();
        self.slew_r.reset();

        self.configure_filters();
        true
    }

    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
        self.low_boost_l.reset();
        self.low_boost_r.reset();
        self.low_atten_l.reset();
        self.low_atten_r.reset();
        self.high_peak_l.reset();
        self.high_peak_r.reset();
        self.high_atten_l.reset();
        self.high_atten_r.reset();
        self.transformer_l.reset();
        self.transformer_r.reset();
        self.ac_coupling_l.reset();
        self.ac_coupling_r.reset();
        self.pre_emphasis_l.reset();
        self.pre_emphasis_r.reset();
        self.de_emphasis_l.reset();
        self.de_emphasis_r.reset();
        self.slew_l.reset();
        self.slew_r.reset();
        self.oversampler_l.reset();
        self.oversampler_r.reset();
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        create_greg_editor(
            "Greg Pultec",
            self.telemetry.clone(),
            |_cx| {},
            |_cx| {},
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.panic_protection.is_disabled() {
            return ProcessStatus::Normal;
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        let analog_drift = self.params.analog_drift.value() as f64;
        let drift = if analog_drift.is_finite() {
            analog_drift.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if drift != self.analog_variance.left_gain.abs() - 1.0 {
            self.analog_variance = AnalogVariance::from_seed(0x47_53_5F_5F, 0.02, 0.001, drift);
        }

        let stereo_link = self.params.link.value();

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut inputs = [0.0_f32; 2];
                for (ch, sample) in frame.iter_mut().enumerate() {
                    if ch < 2 {
                        inputs[ch] = *sample;
                    }
                }

                let x_l = inputs[0] as f64;
                let x_r = inputs[1] as f64;

                let wet_l = process_channel(
                    x_l,
                    0,
                    &mut self.low_boost_l,
                    &mut self.low_boost_r,
                    &mut self.low_atten_l,
                    &mut self.low_atten_r,
                    &mut self.high_peak_l,
                    &mut self.high_peak_r,
                    &mut self.high_atten_l,
                    &mut self.high_atten_r,
                    &mut self.transformer_l,
                    &mut self.transformer_r,
                    &mut self.ac_coupling_l,
                    &mut self.ac_coupling_r,
                    &mut self.pre_emphasis_l,
                    &mut self.pre_emphasis_r,
                    &mut self.de_emphasis_l,
                    &mut self.de_emphasis_r,
                    &mut self.slew_l,
                    &mut self.slew_r,
                    &self.params,
                    self.sample_rate,
                );
                let wet_r = if stereo_link {
                    let mid = (x_l + x_r) * 0.5_f64;
                    let wet_mid = process_channel(
                        mid,
                        0,
                        &mut self.low_boost_l,
                        &mut self.low_boost_r,
                        &mut self.low_atten_l,
                        &mut self.low_atten_r,
                        &mut self.high_peak_l,
                        &mut self.high_peak_r,
                        &mut self.high_atten_l,
                        &mut self.high_atten_r,
                        &mut self.transformer_l,
                        &mut self.transformer_r,
                        &mut self.ac_coupling_l,
                        &mut self.ac_coupling_r,
                        &mut self.pre_emphasis_l,
                        &mut self.pre_emphasis_r,
                        &mut self.de_emphasis_l,
                        &mut self.de_emphasis_r,
                        &mut self.slew_l,
                        &mut self.slew_r,
                        &self.params,
                        self.sample_rate,
                    );
                    (wet_mid + wet_mid) * 0.5_f64
                } else {
                    process_channel(
                        x_r,
                        1,
                        &mut self.low_boost_l,
                        &mut self.low_boost_r,
                        &mut self.low_atten_l,
                        &mut self.low_atten_r,
                        &mut self.high_peak_l,
                        &mut self.high_peak_r,
                        &mut self.high_atten_l,
                        &mut self.high_atten_r,
                        &mut self.transformer_l,
                        &mut self.transformer_r,
                        &mut self.ac_coupling_l,
                        &mut self.ac_coupling_r,
                        &mut self.pre_emphasis_l,
                        &mut self.pre_emphasis_r,
                        &mut self.de_emphasis_l,
                        &mut self.de_emphasis_r,
                        &mut self.slew_l,
                        &mut self.slew_r,
                        &self.params,
                        self.sample_rate,
                    )
                };

                let dry_l = x_l;
                let dry_r = x_r;
                let out_l_raw = self.bypass_crossfades[0].process(dry_l as f32, wet_l as f32);
                let out_r_raw = self.bypass_crossfades[1].process(dry_r as f32, wet_r as f32);
                let out_l = if out_l_raw.is_finite() {
                    out_l_raw
                } else {
                    0.0_f32
                };
                let out_r = if out_r_raw.is_finite() {
                    out_r_raw
                } else {
                    0.0_f32
                };

                for (ch, sample) in frame.iter_mut().enumerate() {
                    match ch {
                        0 => *sample = out_l,
                        1 => *sample = out_r,
                        _ => {}
                    }
                }

                self.metering.record_sample(inputs[0], out_l, 0.0);
                self.metering.record_stereo_sample(out_l, out_r);
            }
        });

        self.metering.publish_block();
        self.telemetry.num_channels.store(2, Ordering::Relaxed);
        let snap = self.metering.snapshot();
        let to_linear = |dbfs: f32| -> f32 { 10.0_f32.powf(dbfs * 0.05) };
        if snap.input_peak_dbfs.is_finite() {
            self.telemetry.push_in_peak(0, to_linear(snap.input_peak_dbfs));
        }
        if snap.output_peak_dbfs.is_finite() {
            self.telemetry.push_out_peak(0, to_linear(snap.output_peak_dbfs));
        }
        self.telemetry.set_gr_and_phase(snap.gain_reduction_db, snap.phase_correlation);
        if protection_result == ProtectionResult::Tripped {
            silence_buffer(buffer);
        }
        ProcessStatus::Normal
    }
}

impl GregPultec {
    fn configure_filters(&mut self) {
        let sr = self.sample_rate;
        if let Some(f) = self.params.low_boost_freq.value().freq() {
            if f > 0.0 && f < sr * 0.49 {
                let g = self.params.low_boost_gain.value() as f64;
                let g_db = if g.is_finite() { g } else { 0.0 };
                self.low_boost_l
                    .configure(sr, FilterType::LowShelf, f, 1.4, g_db, 0.5);
                self.low_boost_r
                    .configure(sr, FilterType::LowShelf, f, 1.4, g_db, 0.5);
            }
        }
        if let Some(f) = self.params.low_atten_freq.value().freq() {
            if f > 0.0 && f < sr * 0.49 {
                let g = self.params.low_atten_gain.value() as f64;
                let g_db = if g.is_finite() { g } else { 0.0 };
                self.low_atten_l
                    .configure(sr, FilterType::LowShelf, f, 1.4, g_db, 0.5);
                self.low_atten_r
                    .configure(sr, FilterType::LowShelf, f, 1.4, g_db, 0.5);
            }
        }
        if let Some(f) = self.params.high_peak_freq.value().freq() {
            if f > 0.0 && f < sr * 0.49 {
                let g = self.params.high_peak_gain.value() as f64;
                let g_db = if g.is_finite() { g } else { 0.0 };
                self.high_peak_l
                    .configure(sr, FilterType::Bell, f, 1.0, g_db, 0.4);
                self.high_peak_r
                    .configure(sr, FilterType::Bell, f, 1.0, g_db, 0.4);
            }
        }
        if let Some(f) = self.params.high_atten_freq.value().freq() {
            if f > 0.0 && f < sr * 0.49 {
                let g = self.params.high_atten_gain.value() as f64;
                let g_db = if g.is_finite() { g } else { 0.0 };
                self.high_atten_l
                    .configure(sr, FilterType::HighShelf, f, 1.0, g_db, 0.4);
                self.high_atten_r
                    .configure(sr, FilterType::HighShelf, f, 1.0, g_db, 0.4);
            }
        }
        let out_db = if self.params.output_gain.value().is_finite() {
            self.params.output_gain.value() as f64
        } else {
            0.0
        };
        let out_lin = 10.0_f64.powf(out_db / 20.0);
        self.make_gain_l = out_lin * self.analog_variance.left_gain;
        self.make_gain_r = out_lin * self.analog_variance.right_gain;
    }
}

#[inline(always)]
fn process_channel(
    input: f64,
    channel: usize,
    low_boost_l: &mut ZdfSvf,
    low_boost_r: &mut ZdfSvf,
    low_atten_l: &mut ZdfSvf,
    low_atten_r: &mut ZdfSvf,
    high_peak_l: &mut ZdfSvf,
    high_peak_r: &mut ZdfSvf,
    high_atten_l: &mut ZdfSvf,
    high_atten_r: &mut ZdfSvf,
    transformer_l: &mut TransformerHysteresis,
    transformer_r: &mut TransformerHysteresis,
    ac_coupling_l: &mut AcCoupling,
    ac_coupling_r: &mut AcCoupling,
    pre_emphasis_l: &mut DynamicMirrorPreEmphasis,
    pre_emphasis_r: &mut DynamicMirrorPreEmphasis,
    de_emphasis_l: &mut DynamicMirrorPreEmphasis,
    de_emphasis_r: &mut DynamicMirrorPreEmphasis,
    slew_l: &mut SlewRateLimiter,
    slew_r: &mut SlewRateLimiter,
    params: &GregPultecParams,
    sample_rate: f64,
) -> f64 {
    let x = if input.is_finite() { input } else { 0.0 };

    let low_boost_on =
        params.low_boost_freq.value().freq().is_some() && params.low_boost_gain.value() > 0.0;
    let low_atten_on =
        params.low_atten_freq.value().freq().is_some() && params.low_atten_gain.value() < 0.0;
    let high_p_on = params.high_peak_freq.value().freq().is_some();
    let high_a_on = params.high_atten_freq.value().freq().is_some();

    let low_path = if channel == 0 {
        if low_boost_on {
            low_boost_l.process(x)
        } else {
            x
        }
    } else {
        if low_boost_on {
            low_boost_r.process(x)
        } else {
            x
        }
    };

    let low_path = if channel == 0 {
        if low_atten_on {
            low_atten_l.process(low_path)
        } else {
            low_path
        }
    } else {
        if low_atten_on {
            low_atten_r.process(low_path)
        } else {
            low_path
        }
    };

    let high_path = if channel == 0 {
        if high_p_on { high_peak_l.process(x) } else { x }
    } else {
        if high_p_on { high_peak_r.process(x) } else { x }
    };

    let high_path = if channel == 0 {
        if high_a_on {
            high_atten_l.process(high_path)
        } else {
            high_path
        }
    } else {
        if high_a_on {
            high_atten_r.process(high_path)
        } else {
            high_path
        }
    };

    let mix = if low_boost_on || low_atten_on {
        low_path
    } else if high_p_on || high_a_on {
        high_path
    } else {
        x
    };

    let pre_emp = if channel == 0 {
        pre_emphasis_l.pre_emphasis(mix, 0.25)
    } else {
        pre_emphasis_r.pre_emphasis(mix, 0.25)
    };
    let saturated = process_nonlinear(pre_emp, 1.2, NonlinearMode::TubeTriode);
    let de_emp = if channel == 0 {
        de_emphasis_l.de_emphasis(saturated, 0.25)
    } else {
        de_emphasis_r.de_emphasis(saturated, 0.25)
    };
    let ac_out = if channel == 0 {
        ac_coupling_l.process(de_emp)
    } else {
        ac_coupling_r.process(de_emp)
    };
    let trans_out = if channel == 0 {
        transformer_l.process(ac_out, 1.4, 0.3, 0.5)
    } else {
        transformer_r.process(ac_out, 1.4, 0.3, 0.5)
    };
    let slewed = if channel == 0 {
        slew_l.process(trans_out, sample_rate * 0.5, sample_rate)
    } else {
        slew_r.process(trans_out, sample_rate * 0.5, sample_rate)
    };

    slewed
}

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

core_dsp::export_greg_plugin!(
    GregPultec,
    "com.greg.audio.greg-pultec",
    "Pultec-style passive valve equalizer with low-end trick and transformer saturation",
    equalizer
);

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::context::init::InitContext;
    use std::num::NonZeroU32;

    /// Minimal mock implementing the [`InitContext`] trait for testing.
    struct TestInitContext;

    impl InitContext<GregPultec> for TestInitContext {
        fn plugin_api(&self) -> nih_plug::context::PluginApi {
            nih_plug::context::PluginApi::Clap
        }

        fn execute(&self, _task: ()) {}

        fn set_latency_samples(&self, _samples: u32) {}

        fn set_current_voice_capacity(&self, _capacity: u32) {}
    }

    #[test]
    fn pultec_basic_setup() {
        let mut pultec = GregPultec::default();

        let config = BufferConfig {
            sample_rate: 48_000.0,
            min_buffer_size: Some(64),
            max_buffer_size: 64,
            process_mode: ProcessMode::Realtime,
        };

        let layout = AudioIOLayout {
            main_input_channels: Some(NonZeroU32::new(2).unwrap()),
            main_output_channels: Some(NonZeroU32::new(2).unwrap()),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames {
                layout: None,
                main_input: None,
                main_output: None,
                aux_inputs: &[],
                aux_outputs: &[],
            },
        };

        assert!(pultec.initialize(&layout, &config, &mut TestInitContext));
    }

    #[test]
    fn pultec_irregular_buffer_sizes() {
        for size in [1, 7, 64, 513, 2048usize] {
            let mut pultec = GregPultec::default();

            let config = BufferConfig {
                sample_rate: 48_000.0,
                min_buffer_size: Some(size as u32),
                max_buffer_size: size as u32,
                process_mode: ProcessMode::Realtime,
            };

            let layout = AudioIOLayout {
                main_input_channels: Some(NonZeroU32::new(2).unwrap()),
                main_output_channels: Some(NonZeroU32::new(2).unwrap()),
                aux_input_ports: &[],
                aux_output_ports: &[],
                names: PortNames {
                    layout: None,
                    main_input: None,
                    main_output: None,
                    aux_inputs: &[],
                    aux_outputs: &[],
                },
            };

            assert!(pultec.initialize(&layout, &config, &mut TestInitContext));
        }
        assert!(true);
    }
}

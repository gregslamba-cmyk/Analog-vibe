#![forbid(unsafe_code)]
//! greg_dbx_160 — DBX 160 VCA Compressor/Limiter family
//! Consolidates DBX 160 VU / 160X / 160A with version selector.
//!
//! Implements a DC-coupled RMS detector, OverEasy/Hard Knee soft-knee curve,
//! program-dependent attack/release (versions 160X / 160A), Infinity+ ratio
//! (-1:1 expansion), external sidechain, stereo-link detection and
//! click-free bypass crossfades with oversampling delay compensation.

#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};
use std::sync::atomic::Ordering;

use core_dsp::core_dsp_modules::{
    analog_variance::{AnalogVariance, ThermalDrift},
    component_models::{SolidStateModel, SolidStateStage},
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, EnvelopeFollower, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
};
use plugin_support::CommonChannelParams;
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum Dbx160Version {
    #[id = "160_vu"]
    #[name = "160 VU"]
    Vu,
    #[id = "160_x"]
    #[name = "160X"]
    X,
    #[id = "160_a"]
    #[name = "160A"]
    A,
}

impl Dbx160Version {
    /// Program-dependent attack multiplier per hardware version.
    /// 160 VU uses fixed classical times; 160X/A scale dynamically with level.
    fn attack_scale(self) -> f32 {
        match self {
            Dbx160Version::Vu => 1.0,
            Dbx160Version::X => 0.9,
            Dbx160Version::A => 1.05,
        }
    }

    /// Program-dependent release multiplier per hardware version.
    fn release_scale(self) -> f32 {
        match self {
            Dbx160Version::Vu => 1.0,
            Dbx160Version::X => 0.95,
            Dbx160Version::A => 1.1,
        }
    }

    /// Whether Infinity+ (negative ratio -1:1 expansion) is available.
    fn supports_infinity_plus(self) -> bool {
        matches!(self, Dbx160Version::X | Dbx160Version::A)
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum DbxRatio {
    #[id = "1:1"]
    #[name = "1:1"]
    R1,
    #[id = "2:1"]
    #[name = "2:1"]
    R2,
    #[id = "4:1"]
    #[name = "4:1"]
    R4,
    #[id = "8:1"]
    #[name = "8:1"]
    R8,
    #[id = "12:1"]
    #[name = "12:1"]
    R12,
    #[id = "20:1"]
    #[name = "20:1"]
    R20,
    #[id = "inf:1"]
    #[name = "infinity:1"]
    RInf,
    #[id = "infinity_plus"]
    #[name = "Infinity+ (-1:1)"]
    RInfPlus,
}

impl DbxRatio {
    fn value(self) -> f32 {
        match self {
            DbxRatio::R1 => 1.0,
            DbxRatio::R2 => 2.0,
            DbxRatio::R4 => 4.0,
            DbxRatio::R8 => 8.0,
            DbxRatio::R12 => 12.0,
            DbxRatio::R20 => 20.0,
            DbxRatio::RInf => 100.0,
            DbxRatio::RInfPlus => -1.0,
        }
    }
}

impl From<DbxRatio> for f32 {
    fn from(value: DbxRatio) -> Self {
        value.value()
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum KneeMode {
    #[id = "over_easy"]
    #[name = "OverEasy"]
    OverEasy,
    #[id = "hard_knee"]
    #[name = "Hard Knee"]
    HardKnee,
}

impl KneeMode {
    /// Soft-knee width in dB.
    fn knee_width(self) -> f32 {
        match self {
            KneeMode::OverEasy => 12.0,
            KneeMode::HardKnee => 1.5,
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum Oversampling {
    #[id = "1x"]
    #[name = "1x"]
    X1,
    #[id = "2x"]
    #[name = "2x"]
    X2,
    #[id = "4x"]
    #[name = "4x"]
    X4,
    #[id = "8x"]
    #[name = "8x"]
    X8,
}

impl From<Oversampling> for OversamplingFactor {
    fn from(value: Oversampling) -> Self {
        match value {
            Oversampling::X1 => Self::X1,
            Oversampling::X2 => Self::X2,
            Oversampling::X4 => Self::X4,
            Oversampling::X8 => Self::X8,
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum ScFilter {
    #[id = "high_pass"]
    #[name = "High-Pass"]
    HighPass,
    #[id = "low_pass"]
    #[name = "Low-Pass"]
    LowPass,
    #[id = "bell"]
    #[name = "Bell"]
    Bell,
    #[id = "low_shelf"]
    #[name = "Low-Shelf"]
    LowShelf,
    #[id = "high_shelf"]
    #[name = "High-Shelf"]
    HighShelf,
}

impl From<ScFilter> for SidechainFilter {
    fn from(value: ScFilter) -> Self {
        match value {
            ScFilter::HighPass => Self::HighPass,
            ScFilter::LowPass => Self::LowPass,
            ScFilter::Bell => Self::Bell,
            ScFilter::LowShelf => Self::LowShelf,
            ScFilter::HighShelf => Self::HighShelf,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SidechainSettings {
    filter: ScFilter,
    frequency: f32,
    q: f32,
    gain: f32,
}

#[derive(Params)]
struct GregDbx160Params {
    #[id = "input"]
    input: FloatParam,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "ratio"]
    ratio: EnumParam<DbxRatio>,
    #[id = "knee"]
    knee: EnumParam<KneeMode>,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "output_gain"]
    output_gain: FloatParam,
    #[id = "stereo_link"]
    stereo_link: BoolParam,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "bypass"]
    bypass: BoolParam,
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[id = "invert_right"]
    invert_right: BoolParam,
    #[id = "version"]
    version: EnumParam<Dbx160Version>,
    #[id = "external_sidechain"]
    external_sidechain: BoolParam,
    #[id = "sc_enable"]
    sc_enable: BoolParam,
    #[id = "sc_listen"]
    sc_listen: BoolParam,
    #[id = "sc_filter"]
    sc_filter: EnumParam<ScFilter>,
    #[id = "sc_freq"]
    sc_freq: FloatParam,
    #[id = "sc_q"]
    sc_q: FloatParam,
    #[id = "sc_gain"]
    sc_gain: FloatParam,
    #[nested]
    channel_setup: CommonChannelParams,
}

impl Default for GregDbx160Params {
    fn default() -> Self {
        Self {
            input: FloatParam::new(
                "Input",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            threshold: FloatParam::new(
                "Threshold",
                -10.0,
                FloatRange::Linear {
                    min: -40.0,
                    max: 20.0,
                },
            )
            .with_unit(" dB"),
            ratio: EnumParam::new("Ratio", DbxRatio::R4),
            knee: EnumParam::new("Knee", KneeMode::OverEasy),
            attack: FloatParam::new(
                "Attack",
                5.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 50.0,
                },
            )
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 2000.0,
                },
            )
            .with_unit(" ms"),
            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear {
                    min: -20.0,
                    max: 20.0,
                },
            )
            .with_unit(" dB"),
            stereo_link: BoolParam::new("Stereo Link", true),
            oversampling: EnumParam::new("Oversampling", Oversampling::X1),
            bypass: BoolParam::new("Bypass", false),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
            version: EnumParam::new("Version", Dbx160Version::A),
            external_sidechain: BoolParam::new("External Sidechain", false),
            sc_enable: BoolParam::new("SC Enable", false),
            sc_listen: BoolParam::new("SC Listen", false),
            sc_filter: EnumParam::new("SC Filter", ScFilter::HighPass),
            sc_freq: FloatParam::new(
                "SC Frequency",
                100.0,
                FloatRange::Linear {
                    min: 20.0,
                    max: 20_000.0,
                },
            )
            .with_unit(" Hz"),
            sc_q: FloatParam::new(
                "SC Q",
                0.707,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            ),
            sc_gain: FloatParam::new(
                "SC Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            channel_setup: CommonChannelParams::default(),
        }
    }
}

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregDbx160Params>,
}

impl Model for EditorData {}

struct GregDbx160 {
    params: Arc<GregDbx160Params>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    envelopes: [EnvelopeFollower<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_oversampling: Oversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
    output_stages: [SolidStateStage; plugin_support::MAX_AUDIO_CHANNELS],
    analog_variance: AnalogVariance,
    thermal_drift: ThermalDrift,
}

impl Default for GregDbx160 {
    fn default() -> Self {
        Self {
            params: Arc::new(GregDbx160Params::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            envelopes: [EnvelopeFollower::default(); plugin_support::MAX_AUDIO_CHANNELS],
            sidechains: array::from_fn(|_| SidechainProcessor::default()),
            last_sidechain_settings: SidechainSettings {
                filter: ScFilter::HighPass,
                frequency: 80.0,
                q: 0.707,
                gain: 0.0,
            },
            oversamplers: [Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
            sample_rate: 0.0,
            last_oversampling: Oversampling::X1,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
            output_stages: array::from_fn(|_| SolidStateStage::new(
                SolidStateModel::Dbx202GoldCan,
                48_000.0,
            )),
            analog_variance: AnalogVariance::from_seed(0x22_44_66_88, 0.02, 0.001, 0.0),
            thermal_drift: ThermalDrift::new(),
        }
    }
}

impl GregDbx160 {
    fn sidechain_settings(&self) -> SidechainSettings {
        SidechainSettings {
            filter: self.params.sc_filter.value(),
            frequency: self.params.sc_freq.value(),
            q: self.params.sc_q.value(),
            gain: self.params.sc_gain.value(),
        }
    }

    fn configure_sidechains(&mut self, settings: SidechainSettings) -> bool {
        let configured = self.sidechains.iter_mut().all(|sidechain| {
            sidechain.configure(
                self.sample_rate,
                SidechainFilter::from(settings.filter).into(),
                settings.frequency.min(self.sample_rate * 0.49),
                settings.q,
                settings.gain,
            )
        });
        if configured {
            self.last_sidechain_settings = settings;
        }
        configured
    }
}

impl Plugin for GregDbx160 {
    const NAME: &'static str = "Greg DBX 160";
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
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        let Some(channel_count) = audio_io_layout
            .main_input_channels
            .map(|channels| channels.get() as usize)
        else {
            return false;
        };
        if !self.routing_matrix.configure(
            channel_count,
            core_dsp::core_dsp_modules::routing_matrix::RoutingMode::DualMono,
        ) {
            return false;
        }

        self.sample_rate = buffer_config.sample_rate;
        if !self.sample_rate.is_finite() || self.sample_rate <= 0.0 {
            return false;
        }

        let version = self.params.version.value();
        let attack_ms = self.params.attack.value() * version.attack_scale();
        let release_ms = self.params.release.value() * version.release_scale();
        let attack = (attack_ms * 0.001).max(1e-4);
        let release = (release_ms * 0.001).max(1e-4);

        for env in &mut self.envelopes {
            env.initialize(self.sample_rate, attack, release);
        }

        let sidechains_ok = self.configure_sidechains(self.sidechain_settings());

        let oversampling = self.params.oversampling.value();
        let latency = Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
        let delays_ok = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency as usize));

        if sidechains_ok && delays_ok {
            self.last_oversampling = oversampling;
            self.last_bypassed = self.params.bypass.value();
            self.bypass_ramp_samples = (self.sample_rate * 0.005).round().max(1.0) as u32;
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(self.last_bypassed, 0);
            }
            for oversampler in &mut self.oversamplers {
                oversampler.reset();
            }
            for delay in &mut self.dry_delays {
                delay.reset();
            }
            context.set_latency_samples(latency);
        }

        // Initialize physical analog stages
        self.thermal_drift.configure(self.sample_rate as f64);
        for stage in &mut self.output_stages {
            stage.configure(self.sample_rate as f64);
        }
        sidechains_ok && delays_ok
    }

    fn reset(&mut self) {
        for env in &mut self.envelopes {
            env.reset();
        }
        for sidechain in &mut self.sidechains {
            sidechain.reset();
        }
        for oversampler in &mut self.oversamplers {
            oversampler.reset();
        }
        for delay in &mut self.dry_delays {
            delay.reset();
        }
        for stage in &mut self.output_stages {
            stage.reset();
        }
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
        self.last_sidechain_settings = self.sidechain_settings();
        self.routing_matrix.configure(
            2,
            core_dsp::core_dsp_modules::routing_matrix::RoutingMode::DualMono,
        );
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params_main = self.params.clone();
        let param_defs: &'static [greg_gui_core::ParamDef] =
            Box::leak(vec![
                greg_gui_core::ParamDef::knob("Input", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.input);
                })),
                greg_gui_core::ParamDef::knob("Threshold", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.threshold);
                })),
                greg_gui_core::ParamDef::enum_switch("Ratio", Box::new(|cx| {
                    EnumSwitch::new(cx, EditorData::params, |p| &p.ratio);
                })),
                greg_gui_core::ParamDef::enum_switch("Knee", Box::new(|cx| {
                    EnumSwitch::new(cx, EditorData::params, |p| &p.knee);
                })),
                greg_gui_core::ParamDef::knob("Attack", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.attack);
                })),
                greg_gui_core::ParamDef::knob("Release", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.release);
                })),
                greg_gui_core::ParamDef::knob("Output Gain", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.output_gain);
                })),
                greg_gui_core::ParamDef::toggle("Stereo Link", Box::new(|cx| {
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.stereo_link);
                })),
                greg_gui_core::ParamDef::enum_switch("Oversampling", Box::new(|cx| {
                    EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
                })),
            ].into_boxed_slice(),
        );
        create_greg_editor_with_params(
            "Greg DBX 160",
            self.telemetry.clone(),
            move |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "Main Parameters").font_size(12.0).color(Color::rgb(210, 215, 225)).bottom(Pixels(6.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.input).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.threshold).width(Pixels(115.0)).height(Pixels(24.0));
                    EnumSwitch::new(cx, EditorData::params, |p| &p.ratio).width(Pixels(115.0)).height(Pixels(24.0));
                    EnumSwitch::new(cx, EditorData::params, |p| &p.knee).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.attack).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.release).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.output_gain).width(Pixels(115.0)).height(Pixels(24.0));
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.stereo_link).width(Pixels(115.0)).height(Pixels(24.0));
                    EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling).width(Pixels(115.0)).height(Pixels(24.0));
                })
                .child_space(Pixels(10.0));
            },
            move |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "Advanced Parameters").font_size(12.0).color(Color::rgb(210, 215, 225)).bottom(Pixels(6.0));
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass).width(Pixels(115.0)).height(Pixels(24.0));
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.external_sidechain).width(Pixels(115.0)).height(Pixels(24.0));
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.sc_enable).width(Pixels(115.0)).height(Pixels(24.0));
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.sc_listen).width(Pixels(115.0)).height(Pixels(24.0));
                    EnumSwitch::new(cx, EditorData::params, |p| &p.sc_filter).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.sc_freq).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.sc_q).width(Pixels(115.0)).height(Pixels(24.0));
                    RotaryKnob::new(cx, EditorData::params, |p| &p.sc_gain).width(Pixels(115.0)).height(Pixels(24.0));
                })
                .child_space(Pixels(10.0));
            },
            Some(param_defs),
            move |cx| EditorData { params: params_main.clone() }.build(cx),
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.panic_protection.is_disabled() {
            return ProcessStatus::Normal;
        }

        let oversampling = self.params.oversampling.value();
        if oversampling != self.last_oversampling {
            let latency =
                Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
            for oversampler in &mut self.oversamplers {
                oversampler.reset();
            }
            for delay in &mut self.dry_delays {
                delay.reset();
                if !delay.set_delay_samples(latency as usize) {
                    return ProcessStatus::Error("Greg DBX 160 latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            context.set_latency_samples(latency);
        }

        let settings = self.sidechain_settings();
        if settings != self.last_sidechain_settings && !self.configure_sidechains(settings) {
            return ProcessStatus::Error("invalid Greg DBX 160 sidechain configuration");
        }
        for sidechain in &mut self.sidechains {
            sidechain.set_filter_enabled(self.params.sc_enable.value());
            sidechain.set_listen(self.params.sc_listen.value());
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        let version = self.params.version.value();
        let attack_scale = version.attack_scale();
        let release_scale = version.release_scale();
        let program_dependent = version != Dbx160Version::Vu;
        let invert_left = self.params.invert_left.value();
        let invert_right = self.params.invert_right.value();
        let stereo_link = self.params.stereo_link.value();
        let use_external = self.params.external_sidechain.value();
        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());
        let input_gain = util::db_to_gain(self.params.input.value());
        let output_gain = util::db_to_gain(self.params.output_gain.value());
        let threshold_db = self.params.threshold.value();
        let ratio_value: f32 = self.params.ratio.value().into();
        let infinity_plus = ratio_value < 0.0 && version.supports_infinity_plus();
        let knee_width = self.params.knee.value().knee_width();
        let factor = OversamplingFactor::from(oversampling);

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for (frame_index, mut frame) in buffer.iter_samples().enumerate() {
                let channel_count = frame.len();
                let mut raw = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                let mut driven = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                for (channel, sample) in frame.iter_mut().enumerate() {
                    if channel >= plugin_support::MAX_AUDIO_CHANNELS {
                        break;
                    }
                    raw[channel] = finite(*sample);
                    let polarity =
                        if (channel == 0 && invert_left) || (channel == 1 && invert_right) {
                            -1.0
                        } else {
                            1.0
                        };
                    driven[channel] = raw[channel] * input_gain * polarity;
                }

                // Update program-dependent envelope times per block
                if program_dependent {
                    let level = if stereo_link && channel_count >= 2 {
                        driven[0].max(driven[1])
                    } else {
                        driven[0]
                    };
                    let over = (level.abs() - 0.1).clamp(0.0, 1.0);
                    let atk =
                        (self.params.attack.value() * attack_scale * (1.0 - 0.7 * over)).max(0.5);
                    let rel =
                        (self.params.release.value() * release_scale * (1.0 + 1.5 * over)).max(5.0);
                    let atk_s = atk * 0.001;
                    let rel_s = rel * 0.001;
                    for env in self.envelopes.iter_mut().take(channel_count) {
                        env.set_times(atk_s, rel_s);
                    }
                }

                let mut keys = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                for channel in 0..channel_count {
                    let external_key = external_sidechain.as_ref().and_then(|channels| {
                        self.routing_matrix
                            .sidechain_channel(channel, channels.len())
                            .and_then(|sc_ch| channels.get(sc_ch))
                            .and_then(|samples| samples.get(frame_index))
                            .copied()
                            .filter(|value| value.is_finite())
                    });
                    keys[channel] = self.sidechains[channel].process_key(
                        driven[channel],
                        external_key,
                        use_external,
                    );
                    self.metering.record_sidechain_input(keys[channel]);
                }

                if stereo_link && channel_count >= 2 {
                    // Link all active channels: find max key across all channels
                    let mut max_key = 0.0_f32;
                    for &k in &keys[0..channel_count] {
                        if k > max_key {
                            max_key = k;
                        }
                    }
                    for key in keys.iter_mut().take(channel_count) {
                        *key = max_key;
                    }
                }

                for (channel, sample) in frame.iter_mut().enumerate() {
                    let dry = self.dry_delays[channel].process(driven[channel]);
                    if self.params.sc_listen.value() {
                        *sample = keys[channel];
                        self.metering.record_sample(raw[channel], *sample, 0.0);
                        self.telemetry.push_in_peak(channel, raw[channel]);
                        self.telemetry.push_out_peak(channel, *sample);
                        continue;
                    }

                    let level_db = 20.0 * keys[channel].abs().max(1.0e-12).log10();
                    let mut target_reduction =
                        target_reduction_db(level_db, threshold_db, ratio_value, knee_width);

                    if infinity_plus {
                        // Infinity+ expands below threshold by inverting gain
                        target_reduction = if level_db < threshold_db {
                            (threshold_db - level_db) * 0.5
                        } else {
                            target_reduction
                        };
                    } else if matches!(version, Dbx160Version::Vu) {
                        // 160 VU has no soft-knee expansion below threshold
                        target_reduction = target_reduction.max(0.0);
                    }

                    let reduction_db = self.envelopes[channel].next(target_reduction);
                    let gain = util::db_to_gain(-reduction_db);
                    let compressed = driven[channel] * gain * output_gain;
                    // Physical output stage (DBX 202 Gold Can VCA with asymmetric nonlinearity)
                    let stage_out = self.output_stages[channel].process_sample(compressed as f64, 0.0) as f32;
                    let oversampler = &mut self.oversamplers[channel];
                    let wet = oversampler.process(finite(stage_out), factor, |value| value);
                    // Analog variance drift per channel (±1% component tolerance)
                    let drift = if channel == 0 {
                        self.analog_variance.left_gain
                    } else {
                        self.analog_variance.right_gain
                    };
                    *sample = finite(self.bypass_crossfades[channel].process(dry, wet * drift as f32));
                    self.metering
                        .record_sample(raw[channel], *sample, reduction_db);
                    self.telemetry.push_in_peak(channel, raw[channel]);
                    self.telemetry.push_out_peak(channel, *sample);
                }

                if stereo_link && channel_count >= 2 {
                    self.metering.record_stereo_sample(raw[0], raw[1]);
                }
            }
        });

        self.metering.publish_block();
        self.telemetry.num_channels.store(self.routing_matrix.channel_count() as u32, Ordering::Relaxed);
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

fn target_reduction_db(input_db: f32, threshold_db: f32, ratio: f32, knee_width: f32) -> f32 {
    if !input_db.is_finite() || !threshold_db.is_finite() || !ratio.is_finite() {
        return 0.0;
    }
    let over = input_db - threshold_db;
    if over <= 0.0 {
        return 0.0;
    }
    if ratio >= 1.0 {
        let knee = knee_width.max(0.1);
        if over < knee {
            let progress = (over / knee).clamp(0.0, 1.0);
            over * (1.0 - 1.0 / ratio) * progress * progress
        } else {
            over * (1.0 - 1.0 / ratio)
        }
    } else {
        // ratio < 0 not handled here (Infinity+ handled in process loop)
        0.0
    }
}

#[inline]
fn finite(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

core_dsp::export_greg_plugin!(
    GregDbx160,
    "com.greg.audio.greg-db-x-160",
    "DBX 160 VCA compressor family (160 VU / 160X / 160A) with version selector",
    compressor
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_values_are_correct() {
        assert_eq!(f32::from(DbxRatio::R1), 1.0);
        assert_eq!(f32::from(DbxRatio::R2), 2.0);
        assert_eq!(f32::from(DbxRatio::R4), 4.0);
        assert_eq!(f32::from(DbxRatio::R8), 8.0);
        assert_eq!(f32::from(DbxRatio::R12), 12.0);
        assert_eq!(f32::from(DbxRatio::R20), 20.0);
        assert_eq!(f32::from(DbxRatio::RInf), 100.0);
        assert_eq!(f32::from(DbxRatio::RInfPlus), -1.0);
    }

    #[test]
    fn version_default_is_160a() {
        let params = GregDbx160Params::default();
        assert_eq!(params.version.value(), Dbx160Version::A);
    }

    #[test]
    fn envelope_follower_works() {
        let mut env = EnvelopeFollower::default();
        assert!(env.initialize(48_000.0, 0.005, 0.1));
        let rising = env.next(1.0);
        assert!(rising > 0.0 && rising < 1.0);
        let falling = env.next(0.0);
        assert!(falling < rising);
    }

    #[test]
    fn knee_width_values() {
        assert_eq!(KneeMode::OverEasy.knee_width(), 12.0);
        assert_eq!(KneeMode::HardKnee.knee_width(), 1.5);
    }

    #[test]
    fn version_supports_infinity_plus() {
        assert!(!Dbx160Version::Vu.supports_infinity_plus());
        assert!(Dbx160Version::X.supports_infinity_plus());
        assert!(Dbx160Version::A.supports_infinity_plus());
    }
}

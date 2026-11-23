#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};
use std::sync::atomic::Ordering;

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, EnvelopeFollower, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
};

use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregBusCompParams>,
}

impl Model for EditorData {}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum RatioSelection {
    #[id = "1_5:1"]
    #[name = "1.5:1"]
    R1_5,
    #[id = "2:1"]
    #[name = "2:1"]
    R2,
    #[id = "4:1"]
    #[name = "4:1"]
    R4,
    #[id = "10:1"]
    #[name = "10:1"]
    R10,
    #[id = "inf:1"]
    #[name = "∞:1"]
    RInf,
}

impl RatioSelection {
    fn value(self) -> f32 {
        match self {
            RatioSelection::R1_5 => 1.5,
            RatioSelection::R2 => 2.0,
            RatioSelection::R4 => 4.0,
            RatioSelection::R10 => 10.0,
            RatioSelection::RInf => 100.0,
        }
    }
}

impl From<RatioSelection> for f32 {
    fn from(value: RatioSelection) -> Self {
        value.value()
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
struct GregBusCompParams {
    #[id = "input"]
    input: FloatParam,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "ratio"]
    ratio: EnumParam<RatioSelection>,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "makeup"]
    makeup: FloatParam,
    #[id = "stereo_link"]
    stereo_link: BoolParam,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "vca_drive"]
    vca_drive: FloatParam,
    #[id = "bypass"]
    bypass: BoolParam,
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[id = "invert_right"]
    invert_right: BoolParam,
    #[id = "external_sidechain"]
    external_sidechain: BoolParam,
    #[id = "sidechain_filter_enabled"]
    sidechain_filter_enabled: BoolParam,
    #[id = "sidechain_listen"]
    sidechain_listen: BoolParam,
    #[id = "sidechain_filter"]
    sidechain_filter: EnumParam<ScFilter>,
    #[id = "sidechain_frequency"]
    sidechain_frequency: FloatParam,
    #[id = "sidechain_q"]
    sidechain_q: FloatParam,
    #[id = "sidechain_gain"]
    sidechain_gain: FloatParam,
}

impl Default for GregBusCompParams {
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
                -20.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            ratio: EnumParam::new("Ratio", RatioSelection::R2),
            attack: FloatParam::new(
                "Attack",
                0.001,
                FloatRange::Skewed {
                    min: 0.0001,
                    max: 0.1,
                    factor: FloatRange::skew_factor(0.001),
                },
            )
            .with_unit(" s"),
            release: FloatParam::new(
                "Release",
                0.5,
                FloatRange::Skewed {
                    min: 0.05,
                    max: 5.0,
                    factor: FloatRange::skew_factor(0.5),
                },
            )
            .with_unit(" s"),
            makeup: FloatParam::new(
                "Makeup",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            stereo_link: BoolParam::new("Stereo Link", true),
            oversampling: EnumParam::new("Oversampling", Oversampling::X1),
            vca_drive: FloatParam::new(
                "VCA Drive",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),
            bypass: BoolParam::new("Bypass", false),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
            external_sidechain: BoolParam::new("External Sidechain", false),
            sidechain_filter_enabled: BoolParam::new("Sidechain Filter Enabled", false),
            sidechain_listen: BoolParam::new("Sidechain Listen", false),
            sidechain_filter: EnumParam::new("Sidechain Filter", ScFilter::HighPass),
            sidechain_frequency: FloatParam::new(
                "Sidechain Frequency",
                80.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(20.0),
                },
            )
            .with_unit(" Hz"),
            sidechain_q: FloatParam::new(
                "Sidechain Q",
                0.707,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 10.0,
                    factor: FloatRange::skew_factor(0.1),
                },
            ),
            sidechain_gain: FloatParam::new(
                "Sidechain Gain",
                0.0,
                FloatRange::Linear {
                    min: -18.0,
                    max: 18.0,
                },
            )
            .with_unit(" dB"),
        }
    }
}

struct GregBusComp {
    params: Arc<GregBusCompParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    envelopes: [EnvelopeFollower<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_ratio: RatioSelection,
    last_oversampling: Oversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregBusComp {
    fn default() -> Self {
        Self {
            params: Arc::new(GregBusCompParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            envelopes: array::from_fn(|_| EnvelopeFollower::default()),
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
            last_ratio: RatioSelection::R2,
            last_oversampling: Oversampling::X1,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl GregBusComp {
    fn sidechain_settings(&self) -> SidechainSettings {
        SidechainSettings {
            filter: self.params.sidechain_filter.value(),
            frequency: self.params.sidechain_frequency.value(),
            q: self.params.sidechain_q.value(),
            gain: self.params.sidechain_gain.value(),
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

    #[inline]
    fn target_reduction_db(input_db: f32, threshold_db: f32, ratio: f32) -> f32 {
        if !input_db.is_finite() || !threshold_db.is_finite() || !ratio.is_finite() {
            return 0.0;
        }
        let over = input_db - threshold_db;
        if over <= 0.0 {
            return 0.0;
        }
        let knee = 2.0_f32;
        if over < knee {
            let progress = (over / knee).clamp(0.0, 1.0);
            over * (1.0 - 1.0 / ratio) * progress * progress
        } else {
            over * (1.0 - 1.0 / ratio)
        }
    }
}

impl Plugin for GregBusComp {
    const NAME: &'static str = "Greg Bus Comp";
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

        let attack = self.params.attack.value();
        let release = self.params.release.value();
        let envelopes_initialized = self
            .envelopes
            .iter_mut()
            .all(|env| env.initialize(self.sample_rate, attack, release));
        let sidechains_initialized = self.configure_sidechains(self.sidechain_settings());
        let oversampling = self.params.oversampling.value();
        let latency = Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
        let delays_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency as usize));
        let initialized = envelopes_initialized && sidechains_initialized && delays_initialized;
        if initialized {
            self.last_ratio = self.params.ratio.value();
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
        initialized
    }

    fn reset(&mut self) {
        for envelope in &mut self.envelopes {
            envelope.reset();
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
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params_main = self.params.clone();
        let params_adv = self.params.clone();
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
                greg_gui_core::ParamDef::knob("Attack", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.attack);
                })),
                greg_gui_core::ParamDef::knob("Release", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.release);
                })),
                greg_gui_core::ParamDef::knob("Makeup", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.makeup);
                })),
                greg_gui_core::ParamDef::knob("VCA Drive", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.vca_drive);
                })),
                greg_gui_core::ParamDef::enum_switch("Oversampling", Box::new(|cx| {
                    EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
                })),
                greg_gui_core::ParamDef::toggle("Stereo Link", Box::new(|cx| {
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.stereo_link);
                })),
                greg_gui_core::ParamDef::toggle("Bypass", Box::new(|cx| {
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
                })),
            ].into_boxed_slice(),
        );
        create_greg_editor_with_params(
            "Greg Bus Comp",
            self.telemetry.clone(),
            move |cx| {
                let editor_data = EditorData { params: params_main.clone() };
                editor_data.build(cx);
            },
            move |cx| {
                let editor_data = EditorData { params: params_adv.clone() };
                editor_data.build(cx);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_right);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.external_sidechain);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.sidechain_filter_enabled);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.sidechain_listen);
                EnumSwitch::new(cx, EditorData::params, |p| &p.sidechain_filter);
                RotaryKnob::new(cx, EditorData::params, |p| &p.sidechain_frequency);
                RotaryKnob::new(cx, EditorData::params, |p| &p.sidechain_q);
                RotaryKnob::new(cx, EditorData::params, |p| &p.sidechain_gain);
            },
            Some(param_defs),
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
                    return ProcessStatus::Error("Greg Bus Comp latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            context.set_latency_samples(latency);
        }

        let bypass = self.params.bypass.value();
        if bypass != self.last_bypassed {
            let ramp = (self.sample_rate * 0.005).round().max(1.0) as u32;
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypass, ramp);
            }
            self.last_bypassed = bypass;
        }

        let stereo_link = self.params.stereo_link.value();
        let external = self.params.external_sidechain.value();
        let listen = self.params.sidechain_listen.value();
        let filter_enabled = self.params.sidechain_filter_enabled.value();
        let settings = self.sidechain_settings();
        if settings != self.last_sidechain_settings {
            self.configure_sidechains(settings);
            for sidechain in &mut self.sidechains {
                sidechain.set_filter_enabled(filter_enabled);
            }
        }

        let threshold_db = self.params.threshold.value();
        let ratio_value = f32::from(self.params.ratio.value());
        let input_gain = db_to_gain(self.params.input.value());
        let output_gain = db_to_gain(self.params.makeup.value());
        let drive = db_to_gain(self.params.vca_drive.value());
        let invert_l = self.params.invert_left.value();
        let invert_r = self.params.invert_right.value();

        let factor = OversamplingFactor::from(oversampling);
        let use_external = external;
        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());

        self.metering.begin_block();

        let protection_result = self.panic_protection.run(|| {
            for (frame_index, mut channel_samples) in buffer.iter_samples().enumerate() {
                let channel_count = channel_samples.len();
                let mut driven = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                let mut raw_values = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];

                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    raw_values[channel] = finite(*sample);
                    let polarity = if (channel == 0 && invert_l) || (channel == 1 && invert_r) {
                        -1.0
                    } else {
                        1.0
                    };
                    driven[channel] = raw_values[channel] * input_gain * polarity;
                }

                let mut keys = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                for channel in 0..channel_count {
                    let external_key = external_sidechain.as_ref().and_then(|channels| {
                        self.routing_matrix
                            .sidechain_channel(channel, channels.len())
                            .and_then(|sidechain_channel| channels.get(sidechain_channel))
                            .and_then(|samples| samples.get(frame_index))
                            .copied()
                            .filter(|value| value.is_finite())
                    });
                    let key_use_external = use_external && external_key.is_some();
                    keys[channel] = self.sidechains[channel].process_key(
                        driven[channel],
                        external_key,
                        key_use_external,
                    );
                }
                if stereo_link && channel_count >= 2 {
                    // Link all active channels: find max key across all channels
                    let mut max_key = 0.0_f32;
                    for &k in &keys[0..channel_count] {
                        if k > max_key {
                            max_key = k;
                        }
                    }
                    for ch in 0..channel_count {
                        keys[ch] = max_key;
                    }
                }

                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    let dry = self.dry_delays[channel].process(driven[channel]);
                    if listen {
                        *sample = keys[channel];
                        self.metering.record_sample(driven[channel], *sample, 0.0);
                        continue;
                    }

                    let level_db = 20.0 * keys[channel].abs().max(1.0e-12).log10();
                    let target_reduction =
                        GregBusComp::target_reduction_db(level_db, threshold_db, ratio_value);

                    let envelope = &mut self.envelopes[channel];
                    let reduction_db = envelope.next(target_reduction);

                    let compressed = driven[channel] * db_to_gain(-reduction_db) * output_gain;
                    let oversampler = &mut self.oversamplers[channel];
                    let wet = if drive > 1.0 {
                        oversampler.process(compressed, factor, |value| {
                            nonlinear_stages_tanh(value, drive)
                        })
                    } else {
                        oversampler.process(compressed, factor, |value| value)
                    };
                    *sample = finite(self.bypass_crossfades[channel].process(dry, wet));
                    self.metering
                        .record_sample(driven[channel], *sample, reduction_db);
                    self.telemetry.push_in_peak(channel, driven[channel]);
                    self.telemetry.push_out_peak(channel, *sample);
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

#[inline]
fn db_to_gain(db: f32) -> f32 {
    if db.is_finite() {
        10.0_f32.powf(db / 20.0)
    } else {
        0.0
    }
}

#[inline]
fn finite(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

#[inline]
fn nonlinear_stages_tanh(sample: f32, drive: f32) -> f32 {
    let driven = sample * drive;
    driven.tanh() / drive
}

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

core_dsp::export_greg_plugin!(
    GregBusComp,
    "com.greg.audio.greg-bus-comp",
    "Stereo bus VCA compressor with filtered sidechain, stereo-link or dual-mono detection, and optional oversampling",
    compressor
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_reduction_is_zero_below_threshold() {
        assert_eq!(GregBusComp::target_reduction_db(-30.0, -20.0, 2.0), 0.0);
    }

    #[test]
    fn target_reduction_scales_with_ratio() {
        let low = GregBusComp::target_reduction_db(-10.0, -20.0, 2.0);
        let high = GregBusComp::target_reduction_db(-10.0, -20.0, 10.0);
        assert!(low > 0.0);
        assert!(high > low);
    }

    #[test]
    fn envelope_follower_tracks_attack_and_release() {
        let mut env = EnvelopeFollower::default();
        assert!(env.initialize(48_000.0, 0.001, 0.5));
        let rising = env.next(1.0);
        assert!(rising > 0.0 && rising < 1.0);
        let falling = env.next(0.0);
        assert!(falling < rising);
    }

    #[test]
    fn ratio_values_are_positive() {
        for ratio in [
            RatioSelection::R1_5,
            RatioSelection::R2,
            RatioSelection::R4,
            RatioSelection::R10,
            RatioSelection::RInf,
        ] {
            assert!(f32::from(ratio) > 0.0);
        }
    }
}

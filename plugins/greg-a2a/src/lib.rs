#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};
use std::sync::atomic::Ordering;

use core_dsp::core_dsp_modules::{
    analog_variance::AnalogVariance,
    component_models::{create_transformer, TubeModel, TubeStage, TransformerModel},
    metering::Metering,
    nonlinear_stages::{AcCoupling, DynamicMirrorPreEmphasis, tube_saturate},
    opto_compressor::{OptoCell, OptoMode, target_reduction_db},
    panic_protection::{PanicProtection, ProtectionResult},
    psu_sag::PsuSag,
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
    transformer_hysteresis::TransformerHysteresis,
};
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;

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

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    #[id = "compress"]
    #[name = "Compress"]
    Compress,
    #[id = "limit"]
    #[name = "Limit"]
    Limit,
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

struct GregA2a {
    params: Arc<GregA2aParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    optical_cells: [OptoCell<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    input_transformers: [TransformerHysteresis; plugin_support::MAX_AUDIO_CHANNELS],
    tube_stages_ecc83: [Option<TubeStage>; plugin_support::MAX_AUDIO_CHANNELS],
    tube_stages_ecc81: [Option<TubeStage>; plugin_support::MAX_AUDIO_CHANNELS],
    output_transformers: [TransformerHysteresis; plugin_support::MAX_AUDIO_CHANNELS],
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    power_sags: [PsuSag<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    transformers: [TransformerHysteresis; plugin_support::MAX_AUDIO_CHANNELS],
    ac_couplings: [AcCoupling; plugin_support::MAX_AUDIO_CHANNELS],
    pre_emphases: [DynamicMirrorPreEmphasis; plugin_support::MAX_AUDIO_CHANNELS],
    analog_variance: AnalogVariance,
    last_sidechain_settings: SidechainSettings,
    sample_rate: f32,
    last_oversampling: Oversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

#[derive(Params)]
struct GregA2aParams {
    #[id = "peak_reduction"]
    peak_reduction: FloatParam,
    #[id = "gain"]
    gain: FloatParam,
    #[id = "mode"]
    mode: EnumParam<Mode>,
    #[id = "tube_drive"]
    tube_drive: FloatParam,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
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
    #[id = "analog_drift"]
    analog_drift: FloatParam,
}

impl Default for GregA2aParams {
    fn default() -> Self {
        Self {
            peak_reduction: FloatParam::new(
                "Peak Reduction",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 40.0,
                },
            )
            .with_unit(" dB"),
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            mode: EnumParam::new("Mode", Mode::Compress),
            tube_drive: FloatParam::new(
                "Tube Drive",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),
            oversampling: EnumParam::new("Oversampling", Oversampling::X4),
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
            analog_drift: FloatParam::new(
                "Analog Drift",
                0.2,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
        }
    }
}

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregA2aParams>,
}

impl Model for EditorData {}

impl Default for GregA2a {
    fn default() -> Self {
        Self {
            params: Arc::new(GregA2aParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            optical_cells: [OptoCell::default(); plugin_support::MAX_AUDIO_CHANNELS],
            input_transformers: [TransformerHysteresis::default(); plugin_support::MAX_AUDIO_CHANNELS],
            tube_stages_ecc83: [None; plugin_support::MAX_AUDIO_CHANNELS],
            tube_stages_ecc81: [None; plugin_support::MAX_AUDIO_CHANNELS],
            output_transformers: [TransformerHysteresis::default(); plugin_support::MAX_AUDIO_CHANNELS],
            oversamplers: [Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
            sidechains: array::from_fn(|_| SidechainProcessor::default()),
            power_sags: [PsuSag::default(); plugin_support::MAX_AUDIO_CHANNELS],
            transformers: [TransformerHysteresis::default(); plugin_support::MAX_AUDIO_CHANNELS],
            ac_couplings: [AcCoupling::default(); plugin_support::MAX_AUDIO_CHANNELS],
            pre_emphases: [DynamicMirrorPreEmphasis::default(); plugin_support::MAX_AUDIO_CHANNELS],
            analog_variance: AnalogVariance::from_seed(12345, 0.03, 0.01, 0.2),
            last_sidechain_settings: SidechainSettings {
                filter: ScFilter::HighPass,
                frequency: 80.0,
                q: 0.707,
                gain: 0.0,
            },
            sample_rate: 0.0,
            last_oversampling: Oversampling::X4,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl GregA2a {
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
}

impl Plugin for GregA2a {
    const NAME: &'static str = "Greg A2A";
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

        let initialized = self
            .optical_cells
            .iter_mut()
            .all(|cell| cell.initialize(self.sample_rate));
        let sidechains_initialized = self.configure_sidechains(self.sidechain_settings());
        let oversampling = self.params.oversampling.value();
        let factor = OversamplingFactor::from(oversampling);
        let latency_samples = Oversampler::<f32>::latency_samples(factor);
        let delays_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency_samples as usize));

        let variance =
            AnalogVariance::from_seed(13579, 0.03, 0.01, self.params.analog_drift.value() as f64);
        self.analog_variance = variance;
        for sag in &mut self.power_sags {
            sag.initialize(self.sample_rate, 0.1, 0.3);
        }
        for stage in &mut self.tube_stages_ecc83 {
            *stage = Some(TubeStage::new(
                TubeModel::Ecc83_12ax7,
                self.sample_rate as f64,
            ));
        }
        for stage in &mut self.tube_stages_ecc81 {
            *stage = Some(TubeStage::new(
                TubeModel::Ecc81_12at7,
                self.sample_rate as f64,
            ));
        }
        for tx in &mut self.input_transformers {
            *tx = create_transformer(TransformerModel::UtcHa100x, self.sample_rate as f64);
        }
        for tx in &mut self.output_transformers {
            *tx = create_transformer(TransformerModel::UtcA10, self.sample_rate as f64);
        }
        for t in &mut self.transformers {
            t.reset();
        }
        for ac in &mut self.ac_couplings {
            ac.reset();
        }
        for pe in &mut self.pre_emphases {
            pe.reset();
        }
        if initialized && delays_initialized {
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
            context.set_latency_samples(latency_samples);
        }
        initialized && sidechains_initialized && delays_initialized
    }

    fn reset(&mut self) {
        for cell in &mut self.optical_cells {
            cell.reset();
        }
        for stage in &mut self.tube_stages_ecc83 {
            if let Some(s) = stage.as_mut() { s.reset(); }
        }
        for stage in &mut self.tube_stages_ecc81 {
            if let Some(s) = stage.as_mut() { s.reset(); }
        }
        for tx in &mut self.input_transformers {
            tx.reset();
        }
        for tx in &mut self.output_transformers {
            tx.reset();
        }
        for oversampler in &mut self.oversamplers {
            oversampler.reset();
        }
        for delay in &mut self.dry_delays {
            delay.reset();
        }
        for sag in &mut self.power_sags {
            sag.reset();
        }
        for t in &mut self.transformers {
            t.reset();
        }
        for ac in &mut self.ac_couplings {
            ac.reset();
        }
        for pe in &mut self.pre_emphases {
            pe.reset();
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
                greg_gui_core::ParamDef::knob("Peak Reduction", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.peak_reduction);
                })),
                greg_gui_core::ParamDef::knob("Gain", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.gain);
                })),
                greg_gui_core::ParamDef::enum_switch("Mode", Box::new(|cx| {
                    EnumSwitch::new(cx, EditorData::params, |p| &p.mode);
                })),
                greg_gui_core::ParamDef::knob("Tube Drive", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.tube_drive);
                })),
                greg_gui_core::ParamDef::enum_switch("Oversampling", Box::new(|cx| {
                    EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
                })),
            ].into_boxed_slice(),
        );
        create_greg_editor_with_params(
            "Greg A2A",
            self.telemetry.clone(),
            move |cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.peak_reduction);
                RotaryKnob::new(cx, EditorData::params, |p| &p.gain);
                EnumSwitch::new(cx, EditorData::params, |p| &p.mode);
                RotaryKnob::new(cx, EditorData::params, |p| &p.tube_drive);
                EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
            },
            move |cx| {
                ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_right);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.external_sidechain);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.sidechain_filter_enabled);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.sidechain_listen);
                EnumSwitch::new(cx, EditorData::params, |p| &p.sidechain_filter);
                RotaryKnob::new(cx, EditorData::params, |p| &p.sidechain_frequency);
                RotaryKnob::new(cx, EditorData::params, |p| &p.sidechain_q);
                RotaryKnob::new(cx, EditorData::params, |p| &p.sidechain_gain);
                RotaryKnob::new(cx, EditorData::params, |p| &p.analog_drift);
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
            let factor = OversamplingFactor::from(oversampling);
            let latency_samples = Oversampler::<f32>::latency_samples(factor);
            for oversampler in &mut self.oversamplers {
                oversampler.reset();
            }
            for delay in &mut self.dry_delays {
                delay.reset();
                if !delay.set_delay_samples(latency_samples as usize) {
                    return ProcessStatus::Error("A2A latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            context.set_latency_samples(latency_samples);
        }
        let sidechain_settings = self.sidechain_settings();
        if sidechain_settings != self.last_sidechain_settings
            && !self.configure_sidechains(sidechain_settings)
        {
            return ProcessStatus::Error("invalid A2A sidechain filter configuration");
        }
        for sidechain in &mut self.sidechains {
            sidechain.set_filter_enabled(self.params.sidechain_filter_enabled.value());
            sidechain.set_listen(self.params.sidechain_listen.value());
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        let factor = OversamplingFactor::from(oversampling);
        let use_external_sidechain = self.params.external_sidechain.value();
        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());
        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            let mut frame_index = 0;
            for mut channel_samples in buffer.iter_samples() {
                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    let input = finite(*sample);
                    let invert = if (channel == 0 && self.params.invert_left.value())
                        || (channel == 1 && self.params.invert_right.value())
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    let phase_input = input * invert;
                    let dry = self.dry_delays[channel].process(phase_input);
                    let external_key = external_sidechain.as_ref().and_then(|channels| {
                        self.routing_matrix
                            .sidechain_channel(channel, channels.len())
                            .and_then(|sidechain_channel| channels.get(sidechain_channel))
                            .and_then(|samples| samples.get(frame_index))
                            .copied()
                            .filter(|value| value.is_finite())
                    });
                    let key = self.sidechains[channel].process_key(
                        phase_input,
                        external_key,
                        use_external_sidechain,
                    );
                    if self.params.sidechain_listen.value() {
                        *sample = finite(key);
                        self.metering.record_sample(input, *sample, 0.0);
                        continue;
                    }
                    let level_db = 20.0 * key.max(1.0e-12).log10();
                    let mode = match self.params.mode.value() {
                        Mode::Compress => OptoMode::Compress,
                        Mode::Limit => OptoMode::Limit,
                    };
                    let target_reduction =
                        target_reduction_db(level_db, self.params.peak_reduction.value(), mode);
                    let reduction_db = self.optical_cells[channel].process(target_reduction);
                    let makeup_gain = util::db_to_gain(self.params.gain.value() - reduction_db);
                    let drive_db = self.params.tube_drive.value();
                    let tube_drive = 1.0 + 0.2 * (util::db_to_gain(drive_db) - 1.0);
                    let compressed = phase_input * makeup_gain;
                    let drift = if channel == 0 {
                        self.analog_variance.left_gain
                    } else {
                        self.analog_variance.right_gain
                    };
                    let sag_gain = self.power_sags[channel].gain(compressed);
                    let pre_emph = self.pre_emphases[channel]
                        .pre_emphasis((compressed as f64) * drift * (sag_gain as f64), 0.3, channel);
                    let oversampler = &mut self.oversamplers[channel];
                    let saturated = oversampler.process(pre_emph as f32, factor, |value| {
                        tube_saturate(value, tube_drive)
                    });
                    let de_emph = self.pre_emphases[channel].de_emphasis(saturated as f64, 0.3, channel);
                    let ac_coupled = self.ac_couplings[channel].process(de_emph, channel);
                    // Input transformer (UtcHa100x)
                    let input_txfm_out = self.input_transformers[channel]
                        .process(ac_coupled, 1.0, 0.5, 0.2);
                    // Tube stages: Ecc83 (12AX7) then Ecc81 (cathode follower)
                    let tube_ecc83_out = self.tube_stages_ecc83[channel]
                        .as_mut()
                        .map(|s| s.process_sample(input_txfm_out, 0.0))
                        .unwrap_or(input_txfm_out);
                    let tube_ecc81_out = self.tube_stages_ecc81[channel]
                        .as_mut()
                        .map(|s| s.process_sample(tube_ecc83_out, 0.0))
                        .unwrap_or(tube_ecc83_out);
                    // Output transformer (UtcA10)
                    let transformer_out = self.output_transformers[channel]
                        .process(tube_ecc81_out as f64, 1.0, 0.5, 0.2) as f32;
                    *sample = finite(
                        self.bypass_crossfades[channel].process(dry, transformer_out as f32),
                    );
                    self.metering.record_sample(input, *sample, reduction_db);
                    self.telemetry.push_in_peak(channel, input);
                    self.telemetry.push_out_peak(channel, *sample);
                }
                frame_index += 1;
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
    GregA2a,
    "com.greg.audio.greg-a2a",
    "Program-dependent opto compressor with an asymmetric tube output stage",
    compressor
);

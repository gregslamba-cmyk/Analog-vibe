#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};

use core_dsp::core_dsp_modules::{
    analog_variance::{AnalogVariance, ThermalDrift},
    metering::Metering,
    nonlinear_stages::{
        AcCoupling, DynamicMirrorPreEmphasis, MillerRollOff, SlewRateLimiter, tube_saturate,
    },
    panic_protection::{PanicProtection, ProtectionResult},
    psu_sag::PsuSag,
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
    transformer_hysteresis::TransformerHysteresis,
};
use greg_gui_core::{GuiTelemetry, create_greg_editor};
use nih_plug::prelude::*;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum FetRatio {
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
}

impl From<FetRatio> for f32 {
    fn from(value: FetRatio) -> Self {
        match value {
            FetRatio::R4 => 4.0,
            FetRatio::R8 => 8.0,
            FetRatio::R12 => 12.0,
            FetRatio::R20 => 20.0,
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
struct GregFet76Params {
    #[id = "input"]
    input: FloatParam,
    #[id = "output"]
    output: FloatParam,
    #[id = "ratio"]
    ratio: EnumParam<FetRatio>,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "output_gain"]
    output_gain: FloatParam,
    #[id = "transformer_iron"]
    transformer_iron: FloatParam,
    #[id = "analog_drift"]
    analog_drift: FloatParam,
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
}

impl Default for GregFet76Params {
    fn default() -> Self {
        Self {
            input: FloatParam::new(
                "Input",
                0.0,
                FloatRange::Linear { min: -24.0, max: 24.0 },
            )
            .with_unit(" dB"),
            output: FloatParam::new(
                "Output",
                0.0,
                FloatRange::Linear { min: -24.0, max: 24.0 },
            )
            .with_unit(" dB"),
            ratio: EnumParam::new("Ratio", FetRatio::R4),
            threshold: FloatParam::new(
                "Threshold",
                -10.0,
                FloatRange::Linear { min: -40.0, max: 20.0 },
            )
            .with_unit(" dB"),
            attack: FloatParam::new(
                "Attack",
                0.005,
                FloatRange::Linear { min: 0.001, max: 10.0 },
            )
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                0.1,
                FloatRange::Linear { min: 0.01, max: 10.0 },
            )
            .with_unit(" ms"),
            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear { min: -12.0, max: 12.0 },
            )
            .with_unit(" dB"),
            transformer_iron: FloatParam::new(
                "Transformer Iron",
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.5 },
            ),
            analog_drift: FloatParam::new(
                "Analog Drift",
                0.2,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
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
                FloatRange::Linear { min: -18.0, max: 18.0 },
            )
            .with_unit(" dB"),
        }
    }
}
struct GregFet76 {
    params: Arc<GregFet76Params>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    transformers: [TransformerHysteresis; plugin_support::MAX_AUDIO_CHANNELS],
    ac_couplings: [AcCoupling; plugin_support::MAX_AUDIO_CHANNELS],
    slew_limiters: [SlewRateLimiter; plugin_support::MAX_AUDIO_CHANNELS],
    miller_rolloffs: [MillerRollOff; plugin_support::MAX_AUDIO_CHANNELS],
    pre_emphases: [DynamicMirrorPreEmphasis; plugin_support::MAX_AUDIO_CHANNELS],
    power_sags: [PsuSag<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    analog_variance: AnalogVariance,
    thermal_drift: ThermalDrift,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_oversampling: Oversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregFet76 {
    fn default() -> Self {
        Self {
            params: Arc::new(GregFet76Params::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            sidechains: array::from_fn(|_| SidechainProcessor::default()),
            last_sidechain_settings: SidechainSettings {
                filter: ScFilter::HighPass,
                frequency: 80.0,
                q: 0.707,
                gain: 0.0,
            },
            oversamplers: [Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            transformers: [TransformerHysteresis::default(); plugin_support::MAX_AUDIO_CHANNELS],
            ac_couplings: [AcCoupling::default(); plugin_support::MAX_AUDIO_CHANNELS],
            slew_limiters: [SlewRateLimiter::default(); plugin_support::MAX_AUDIO_CHANNELS],
            miller_rolloffs: [MillerRollOff::default(); plugin_support::MAX_AUDIO_CHANNELS],
            pre_emphases: [DynamicMirrorPreEmphasis::default(); plugin_support::MAX_AUDIO_CHANNELS],
            power_sags: [PsuSag::default(); plugin_support::MAX_AUDIO_CHANNELS],
            analog_variance: AnalogVariance::from_seed(76_76, 0.03, 0.01, 0.2),
            thermal_drift: ThermalDrift::default(),
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
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

impl GregFet76 {
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
impl Plugin for GregFet76 {
    const NAME: &'static str = "Greg 76";
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

        let sidechains_initialized = self.configure_sidechains(self.sidechain_settings());
        let oversampling = self.params.oversampling.value();
        let latency_samples =
            Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
        let delays_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency_samples as usize));

        let variance = AnalogVariance::from_seed(
            76_76,
            0.03,
            0.01,
            self.params.analog_drift.value() as f64,
        );
        self.analog_variance = variance;
        self.thermal_drift.configure(self.sample_rate as f64);

        for sag in &mut self.power_sags {
            sag.initialize(self.sample_rate, 0.1, 0.3);
        }
        for t in &mut self.transformers {
            t.reset();
        }
        for ac in &mut self.ac_couplings {
            ac.reset();
        }
        for sl in &mut self.slew_limiters {
            sl.reset();
        }
        for m in &mut self.miller_rolloffs {
            m.reset();
            m.configure(self.sample_rate as f64);
        }
        for pe in &mut self.pre_emphases {
            pe.reset();
        }

        let initialized = sidechains_initialized && delays_initialized;
        if initialized {
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
        initialized
    }

    fn reset(&mut self) {
        for sidechain in &mut self.sidechains {
            sidechain.reset();
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
        for sl in &mut self.slew_limiters {
            sl.reset();
        }
        for m in &mut self.miller_rolloffs {
            m.reset();
        }
        for pe in &mut self.pre_emphases {
            pe.reset();
        }
        self.thermal_drift.reset();
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        create_greg_editor(
            "Greg 76",
            self.telemetry.clone(),
            |_cx| {},
            |_cx| {},
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
            let latency_samples =
                Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
            for oversampler in &mut self.oversamplers {
                oversampler.reset();
            }
            for delay in &mut self.dry_delays {
                delay.reset();
                if !delay.set_delay_samples(latency_samples as usize) {
                    return ProcessStatus::Error("76 latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            context.set_latency_samples(latency_samples);
        }

        let settings = self.sidechain_settings();
        if settings != self.last_sidechain_settings && !self.configure_sidechains(settings) {
            return ProcessStatus::Error("invalid 76 sidechain filter configuration");
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        for sidechain in &mut self.sidechains {
            sidechain.set_filter_enabled(self.params.sidechain_filter_enabled.value());
            sidechain.set_listen(self.params.sidechain_listen.value());
        }

        let input_gain = util::db_to_gain(self.params.input.value());
        let threshold_db = self.params.threshold.value();
        let ratio: f32 = self.params.ratio.value().into();
        let transformer_iron: f64 = self.params.transformer_iron.value() as f64;
        let factor = OversamplingFactor::from(oversampling);
        let use_external = self.params.external_sidechain.value();
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
                    let phase_input = input * invert * input_gain;
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
                        use_external,
                    );
                    if self.params.sidechain_listen.value() {
                        *sample = finite(key);
                        self.metering.record_sample(input, *sample, 0.0);
                        continue;
                    }

                    let level_db = 20.0 * key.max(1.0e-12).log10();
                    let gr = fet_gain_reduction(level_db, threshold_db, ratio);
                    let compressed = phase_input * util::db_to_gain(-gr);
                    let compressed_f64 = compressed as f64;

                    let drift = if channel == 0 {
                        self.analog_variance.left_gain
                    } else {
                        self.analog_variance.right_gain
                    };
                    let sag_gain = self.power_sags[channel].gain(compressed);
                    let pre_emph = self.pre_emphases[channel]
                        .pre_emphasis(compressed_f64 * drift * (sag_gain as f64), 0.3);
                    let oversampler = &mut self.oversamplers[channel];
                    let saturated = oversampler.process(pre_emph as f32, factor, |value| {
                        tube_saturate(value as f64, 1.0 + transformer_iron * 0.5) as f32
                    });
                    let de_emph = self.pre_emphases[channel].de_emphasis(saturated as f64, 0.3);
                    let ac_coupled = self.ac_couplings[channel].process(de_emph);
                    let transformer_out =
                        self.transformers[channel].process(ac_coupled, transformer_iron, 0.5, 0.2);
                    let slew_out = self.slew_limiters[channel].process(
                        transformer_out,
                        self.sample_rate as f64 * 2.0,
                        self.sample_rate as f64,
                    );
                    let miller_out = self.miller_rolloffs[channel].process(
                        slew_out,
                        1.0 + transformer_iron,
                        channel,
                    );
                    let wet = miller_out * (util::db_to_gain(self.params.output_gain.value()) as f64);

                    *sample = finite(self.bypass_crossfades[channel].process(dry, wet as f32));
                    self.metering.record_sample(input, *sample, gr);
                }
                frame_index += 1;
            }
        });

        self.metering.publish_block();
        if protection_result == ProtectionResult::Tripped {
            silence_buffer(buffer);
        }
        ProcessStatus::Normal
    }
}

#[inline]
fn fet_gain_reduction(level_db: f32, threshold_db: f32, ratio: f32) -> f32 {
    if level_db <= threshold_db {
        return 0.0;
    }
    let over = level_db - threshold_db;
    let gr_db = over - over / ratio.max(1.0);
    gr_db.clamp(0.0, 40.0)
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
    GregFet76,
    "com.greg.audio.greg-76",
    "Greg 1176 FET compressor emulation with full analog chain",
    compressor
);

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::context::init::InitContext;
    use std::num::NonZeroU32;

    struct TestInitContext;

    impl InitContext<GregFet76> for TestInitContext {
        fn plugin_api(&self) -> nih_plug::context::PluginApi {
            nih_plug::context::PluginApi::Clap
        }

        fn execute(&self, _task: ()) {}

        fn set_latency_samples(&self, _samples: u32) {}

        fn set_current_voice_capacity(&self, _capacity: u32) {}
    }

    #[test]
    fn fet76_basic_setup() {
        let mut fet = GregFet76::default();

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

        assert!(fet.initialize(&layout, &config, &mut TestInitContext));
    }
}

// End of greg_76 plugin source

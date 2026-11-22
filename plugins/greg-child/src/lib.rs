#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};
use std::sync::atomic::Ordering;

use core_dsp::core_dsp_modules::{
    metering::Metering,
    nonlinear_stages::tube_saturate,
    panic_protection::{PanicProtection, ProtectionResult},
    psu_sag::PsuSag,
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
    vari_mu::{TimeConstant, VariMuCell},
};

use greg_gui_core::{GuiTelemetry, create_greg_editor};
use nih_plug::prelude::*;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum TimeConstantSelection {
    #[id = "i"]
    #[name = "I"]
    I,
    #[id = "ii"]
    #[name = "II"]
    II,
    #[id = "iii"]
    #[name = "III"]
    III,
    #[id = "iv"]
    #[name = "IV"]
    IV,
    #[id = "v"]
    #[name = "V"]
    V,
    #[id = "vi"]
    #[name = "VI"]
    VI,
}

impl From<TimeConstantSelection> for TimeConstant {
    fn from(value: TimeConstantSelection) -> Self {
        match value {
            TimeConstantSelection::I => Self::I,
            TimeConstantSelection::II => Self::II,
            TimeConstantSelection::III => Self::III,
            TimeConstantSelection::IV => Self::IV,
            TimeConstantSelection::V => Self::V,
            TimeConstantSelection::VI => Self::VI,
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
struct GregChildParams {
    #[id = "input"]
    input: FloatParam,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "output"]
    output: FloatParam,
    #[id = "time_constant"]
    time_constant: EnumParam<TimeConstantSelection>,
    #[id = "stereo_link"]
    stereo_link: BoolParam,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "tube_drive"]
    tube_drive: FloatParam,
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

impl Default for GregChildParams {
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
                -18.0,
                FloatRange::Linear {
                    min: -36.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            output: FloatParam::new(
                "Output",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            time_constant: EnumParam::new("Time Constant", TimeConstantSelection::I),
            stereo_link: BoolParam::new("Stereo Link", true),
            oversampling: EnumParam::new("Oversampling", Oversampling::X1),
            tube_drive: FloatParam::new(
                "Output Drive",
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

struct GregChild {
    params: Arc<GregChildParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    cells: [VariMuCell<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    power_sags: [PsuSag<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_time_constant: TimeConstantSelection,
    last_oversampling: Oversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregChild {
    fn default() -> Self {
        Self {
            params: Arc::new(GregChildParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            cells: [VariMuCell::default(); plugin_support::MAX_AUDIO_CHANNELS],
            sidechains: array::from_fn(|_| SidechainProcessor::default()),
            last_sidechain_settings: SidechainSettings {
                filter: ScFilter::HighPass,
                frequency: 80.0,
                q: 0.707,
                gain: 0.0,
            },
            oversamplers: [Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            power_sags: [PsuSag::default(); plugin_support::MAX_AUDIO_CHANNELS],
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
            sample_rate: 0.0,
            last_time_constant: TimeConstantSelection::I,
            last_oversampling: Oversampling::X1,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl GregChild {
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

impl Plugin for GregChild {
    const NAME: &'static str = "Greg Child";
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

        let time_constant = TimeConstant::from(self.params.time_constant.value());
        let cells_initialized = self
            .cells
            .iter_mut()
            .all(|cell| cell.initialize(self.sample_rate, time_constant));
        let sidechains_initialized = self.configure_sidechains(self.sidechain_settings());
        let sag_initialized = self
            .power_sags
            .iter_mut()
            .all(|sag| sag.initialize(self.sample_rate, 0.12, 0.025));
        let oversampling = self.params.oversampling.value();
        let latency = Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
        let delays_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency as usize));
        let initialized =
            cells_initialized && sidechains_initialized && sag_initialized && delays_initialized;
        if initialized {
            self.last_time_constant = self.params.time_constant.value();
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
        for cell in &mut self.cells {
            cell.reset();
        }
        for sidechain in &mut self.sidechains {
            sidechain.reset();
        }
        for sag in &mut self.power_sags {
            sag.reset();
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
        create_greg_editor(
            "Greg Child",
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

        let time_constant = self.params.time_constant.value();
        if time_constant != self.last_time_constant {
            for cell in &mut self.cells {
                cell.set_time_constant(TimeConstant::from(time_constant));
            }
            self.last_time_constant = time_constant;
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
                    return ProcessStatus::Error("Greg Child latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            context.set_latency_samples(latency);
        }

        let settings = self.sidechain_settings();
        if settings != self.last_sidechain_settings && !self.configure_sidechains(settings) {
            return ProcessStatus::Error("invalid Greg Child sidechain configuration");
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

        let input_gain = util::db_to_gain(self.params.input.value());
        let output_gain = util::db_to_gain(self.params.output.value());
        let threshold_db = self.params.threshold.value();
        let factor = OversamplingFactor::from(oversampling);
        let drive = 1.0 + 0.2 * (util::db_to_gain(self.params.tube_drive.value()) - 1.0);
        let use_external = self.params.external_sidechain.value();
        let stereo_link = self.params.stereo_link.value();
        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for (frame_index, mut frame) in buffer.iter_samples().enumerate() {
                let channel_count = frame.len();
                let mut raw = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                let mut driven = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                for (channel, sample) in frame.iter_mut().enumerate() {
                    raw[channel] = finite(*sample);
                    let polarity = if (channel == 0 && self.params.invert_left.value())
                        || (channel == 1 && self.params.invert_right.value())
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    driven[channel] = raw[channel] * input_gain * polarity;
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
                    keys[channel] = self.sidechains[channel].process_key(
                        driven[channel],
                        external_key,
                        use_external,
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

                for (channel, sample) in frame.iter_mut().enumerate() {
                    let dry = self.dry_delays[channel].process(driven[channel]);
                    if self.params.sidechain_listen.value() {
                        *sample = keys[channel];
                        self.metering.record_sample(raw[channel], *sample, 0.0);
                        continue;
                    }

                    let reduction = self.cells[channel].process(keys[channel], threshold_db);
                    let compressed = driven[channel] * util::db_to_gain(-reduction) * output_gain;
                    let sag_gain = self.power_sags[channel].gain(compressed);
                    let oversampler = &mut self.oversamplers[channel];
                    let wet = oversampler.process(compressed * sag_gain, factor, |value| {
                        tube_saturate(value, drive)
                    });
                    *sample = finite(self.bypass_crossfades[channel].process(dry, wet));
                    self.metering
                        .record_sample(raw[channel], *sample, reduction);
                    self.telemetry.push_in_peak(channel, raw[channel]);
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
    GregChild,
    "com.greg.audio.greg-child",
    "Variable-mu tube compressor with six time constants and stereo-link or dual-mono detection",
    compressor
);

#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};

use nih_plug::prelude::*;

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
};

mod dsp;

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

#[derive(Enum, Debug, PartialEq, Eq)]
enum FetRatio {
    #[id = "4-to-1"]
    #[name = "4:1"]
    Four,
    #[id = "8-to-1"]
    #[name = "8:1"]
    Eight,
    #[id = "12-to-1"]
    #[name = "12:1"]
    Twelve,
    #[id = "20-to-1"]
    #[name = "20:1"]
    Twenty,
}

#[derive(Enum, Debug, PartialEq, Eq)]
enum FetOversampling {
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

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum FetRevision {
    #[id = "rev-d"]
    #[name = "Rev D"]
    RevD,
    #[id = "rev-e"]
    #[name = "Rev E"]
    RevE,
}

struct GregFet {
    params: Arc<GregFetParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    envelopes: [dsp::EnvelopeFollower<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    oversamplers: [dsp::Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [dsp::BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [dsp::LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    sample_rate: f32,
    last_attack_seconds: f32,
    last_release_seconds: f32,
    last_oversampling: FetOversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
}

#[derive(Params)]
struct GregFetParams {
    #[id = "revision"]
    revision: EnumParam<FetRevision>,
    #[id = "input"]
    input: FloatParam,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "ratio"]
    ratio: EnumParam<FetRatio>,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "knee"]
    knee: FloatParam,
    #[id = "all_buttons_in"]
    all_buttons_in: BoolParam,
    #[id = "oversampling"]
    oversampling: EnumParam<FetOversampling>,
    #[id = "bypass"]
    bypass: BoolParam,
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[id = "invert_right"]
    invert_right: BoolParam,
    #[id = "output"]
    output: FloatParam,
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

impl Default for GregFet {
    fn default() -> Self {
        Self {
            params: Arc::new(GregFetParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            envelopes: [dsp::EnvelopeFollower::default(); plugin_support::MAX_AUDIO_CHANNELS],
            oversamplers: [dsp::Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            bypass_crossfades: [dsp::BypassCrossfade::default();
                plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [dsp::LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
            sidechains: array::from_fn(|_| SidechainProcessor::default()),
            last_sidechain_settings: SidechainSettings {
                filter: ScFilter::HighPass,
                frequency: 80.0,
                q: 0.707,
                gain: 0.0,
            },
            sample_rate: 0.0,
            last_attack_seconds: 0.0,
            last_release_seconds: 0.0,
            last_oversampling: FetOversampling::X1,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
        }
    }
}

impl Default for GregFetParams {
    fn default() -> Self {
        Self {
            revision: EnumParam::new("Revision", FetRevision::RevD),
            input: FloatParam::new(
                "Input",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 36.0,
                },
            )
            .with_unit(" dB"),
            attack: FloatParam::new(
                "Attack",
                0.2,
                FloatRange::Skewed {
                    min: dsp::FET_ATTACK_MIN_MS,
                    max: dsp::FET_ATTACK_MAX_MS,
                    factor: FloatRange::skew_factor(dsp::FET_ATTACK_MIN_MS),
                },
            )
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                250.0,
                FloatRange::Skewed {
                    min: dsp::FET_RELEASE_MIN_MS,
                    max: dsp::FET_RELEASE_MAX_MS,
                    factor: FloatRange::skew_factor(dsp::FET_RELEASE_MIN_MS),
                },
            )
            .with_unit(" ms"),
            ratio: EnumParam::new("Ratio", FetRatio::Four),
            threshold: FloatParam::new(
                "Threshold",
                -18.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            knee: FloatParam::new(
                "Knee",
                6.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 18.0,
                },
            )
            .with_unit(" dB"),
            all_buttons_in: BoolParam::new("All Buttons In", false),
            oversampling: EnumParam::new("Oversampling", FetOversampling::X1),
            bypass: BoolParam::new("Bypass", false),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
            output: FloatParam::new(
                "Output",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
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

impl Plugin for GregFet {
    const NAME: &'static str = "Greg FET";
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
        self.last_attack_seconds = self.params.attack.value() * 0.001;
        self.last_release_seconds = self.params.release.value() * 0.001;
        self.sample_rate = buffer_config.sample_rate;
        let initialized = self.envelopes.iter_mut().all(|envelope| {
            envelope.initialize(
                buffer_config.sample_rate,
                self.last_attack_seconds,
                self.last_release_seconds,
            )
        });

        let latency_samples = dsp::Oversampler::<f32>::latency_samples(self.oversampling_factor());
        let delays_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency_samples as usize));
        let sidechains_initialized = self.configure_sidechains(self.sidechain_settings());
        if initialized && delays_initialized && sidechains_initialized {
            self.last_oversampling = self.params.oversampling.value();
            self.last_bypassed = self.params.bypass.value();
            self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
            for envelope in &mut self.envelopes {
                envelope.reset();
            }
            for oversampler in &mut self.oversamplers {
                oversampler.reset();
            }
            for delay in &mut self.dry_delays {
                delay.reset();
            }
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(self.last_bypassed, 0);
            }
            context.set_latency_samples(dsp::Oversampler::<f32>::latency_samples(
                self.oversampling_factor(),
            ));
        }

        initialized && delays_initialized && sidechains_initialized
    }

    fn reset(&mut self) {
        for envelope in &mut self.envelopes {
            envelope.reset();
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

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.panic_protection.is_disabled() {
            return ProcessStatus::Normal;
        }

        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());
        let panic_protection = std::mem::take(&mut self.panic_protection);
        let protection_result = panic_protection.run(|| {
            self.metering.begin_block();
            let oversampling = self.params.oversampling.value();
            let oversampling_factor = match oversampling {
                FetOversampling::X1 => dsp::OversamplingFactor::X1,
                FetOversampling::X2 => dsp::OversamplingFactor::X2,
                FetOversampling::X4 => dsp::OversamplingFactor::X4,
                FetOversampling::X8 => dsp::OversamplingFactor::X8,
            };
            if oversampling != self.last_oversampling {
                let latency_samples = dsp::Oversampler::<f32>::latency_samples(oversampling_factor);
                for oversampler in &mut self.oversamplers {
                    oversampler.reset();
                }
                for delay in &mut self.dry_delays {
                    delay.reset();
                    let _ = delay.set_delay_samples(latency_samples as usize);
                }
                self.last_oversampling = oversampling;
                context.set_latency_samples(latency_samples);
            }
            let sidechain_settings = self.sidechain_settings();
            if sidechain_settings != self.last_sidechain_settings
                && !self.configure_sidechains(sidechain_settings)
            {
                return ProcessStatus::Error("invalid FET sidechain filter configuration");
            }
            for sidechain in &mut self.sidechains {
                sidechain.set_filter_enabled(self.params.sidechain_filter_enabled.value());
                sidechain.set_listen(self.params.sidechain_listen.value());
            }

            let input_gain = util::db_to_gain(self.params.input.value());
            let output_gain = util::db_to_gain(self.params.output.value());
            let threshold_db = self.params.threshold.value();
            let all_buttons_in = self.params.all_buttons_in.value();
            let revision = self.params.revision.value();
            let ratio = if all_buttons_in {
                dsp::FET_RATIO_20
            } else {
                match self.params.ratio.value() {
                    FetRatio::Four => dsp::FET_RATIO_4,
                    FetRatio::Eight => dsp::FET_RATIO_8,
                    FetRatio::Twelve => dsp::FET_RATIO_12,
                    FetRatio::Twenty => dsp::FET_RATIO_20,
                }
            };
            let knee_db = if all_buttons_in {
                self.params.knee.value().max(12.0)
            } else {
                self.params.knee.value()
            };
            let use_external_sidechain = self.params.external_sidechain.value();
            let mut frame_index = 0;
            for mut channel_samples in buffer.iter_samples() {
                // Capture first two channels for phase correlation
                let mut stereo_samples = [0.0_f32, 0.0_f32];
                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    if channel < 2 {
                        stereo_samples[channel] = if sample.is_finite() { *sample } else { 0.0 };
                    }
                    let bypassed = self.params.bypass.value();
                    if bypassed != self.last_bypassed {
                        for crossfade in &mut self.bypass_crossfades {
                            crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
                        }
                        self.last_bypassed = bypassed;
                    }

                    let attack_seconds = self.params.attack.value() * 0.001;
                    let release_seconds = self.params.release.value() * 0.001;
                    if attack_seconds != self.last_attack_seconds
                        || release_seconds != self.last_release_seconds
                    {
                        self.last_attack_seconds = attack_seconds;
                        self.last_release_seconds = release_seconds;
                        for envelope in &mut self.envelopes {
                            envelope.set_times(attack_seconds, release_seconds);
                        }
                    }

                    let input_sample = stereo_samples[channel];
                    let invert_left = self.params.invert_left.value();
                    let invert_right = self.params.invert_right.value();
                    let polarity =
                        if (channel == 0 && invert_left) || (channel == 1 && invert_right) {
                            -1.0
                        } else {
                            1.0
                        };
                    let phase_input = input_sample * polarity;
                    let dry = self.dry_delays[channel].process(phase_input);
                    let driven_sample = phase_input * input_gain;
                    let external_key = external_sidechain.as_ref().and_then(|channels| {
                        self.routing_matrix
                            .sidechain_channel(channel, channels.len())
                            .and_then(|sidechain_channel| channels.get(sidechain_channel))
                            .and_then(|samples| samples.get(frame_index))
                            .copied()
                            .filter(|value| value.is_finite())
                    });
                    let key = self.sidechains[channel].process_key(
                        driven_sample,
                        external_key,
                        use_external_sidechain,
                    );
                    // Record sidechain input for metering
                    self.metering.record_sidechain_input(key);
                    
                    if self.params.sidechain_listen.value() {
                        *sample = key;
                        self.metering.record_sample(input_sample, *sample, 0.0);
                        continue;
                    }
                    let envelope = self.envelopes[channel].next(key);
                    let level_db = 20.0 * envelope.max(1.0e-12).log10();
                    let gain_reduction_db =
                        dsp::gain_reduction_db(level_db, threshold_db, ratio, knee_db);

                    let compressed_sample =
                        driven_sample * util::db_to_gain(gain_reduction_db) * output_gain;
                    let wet = self.oversamplers[channel].process(
                        compressed_sample,
                        oversampling_factor,
                        |oversampled_sample| {
                            let drive = match revision {
                                FetRevision::RevD => 0.75,
                                FetRevision::RevE => 0.75,
                            };
                            dsp::soft_saturate(oversampled_sample, drive)
                        },
                    );
                    *sample = self.bypass_crossfades[channel].process(dry, wet);
                    self.metering
                        .record_sample(input_sample, *sample, gain_reduction_db);
                }
                // Record stereo pair for phase correlation (channels 0 and 1)
                let frame_channel_count = channel_samples.len();
                if frame_channel_count >= 2 {
                    self.metering.record_stereo_sample(stereo_samples[0], stereo_samples[1]);
                }
                frame_index += 1;
            }
            ProcessStatus::Normal
        });
        self.metering.publish_block();
        self.panic_protection = panic_protection;
        match protection_result {
            ProtectionResult::Processed(status) => status,
            ProtectionResult::Tripped => {
                silence_buffer(buffer);
                ProcessStatus::Normal
            }
            ProtectionResult::Disabled => ProcessStatus::Normal,
        }
    }
}

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

impl GregFet {
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

    fn oversampling_factor(&self) -> dsp::OversamplingFactor {
        match self.params.oversampling.value() {
            FetOversampling::X1 => dsp::OversamplingFactor::X1,
            FetOversampling::X2 => dsp::OversamplingFactor::X2,
            FetOversampling::X4 => dsp::OversamplingFactor::X4,
            FetOversampling::X8 => dsp::OversamplingFactor::X8,
        }
    }
}

core_dsp::export_greg_plugin!(
    GregFet,
    "com.greg.audio.greg-fet",
    "FET-style dynamics processor",
    compressor
);

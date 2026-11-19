use std::{num::NonZeroU32, sync::Arc};

use nih_plug::prelude::*;

mod dsp;

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

struct GregFet {
    params: Arc<GregFetParams>,
    envelopes: [dsp::EnvelopeFollower<f32>; 2],
    oversamplers: [dsp::Oversampler<f32>; 2],
    bypass_crossfades: [dsp::BypassCrossfade<f32>; 2],
    dry_delays: [dsp::LatencyDelay<f32, 32>; 2],
    last_attack_seconds: f32,
    last_release_seconds: f32,
    last_oversampling: FetOversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
}

#[derive(Params)]
struct GregFetParams {
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
}

impl Default for GregFet {
    fn default() -> Self {
        Self {
            params: Arc::new(GregFetParams::default()),
            envelopes: [dsp::EnvelopeFollower::default(); 2],
            oversamplers: [dsp::Oversampler::default(); 2],
            bypass_crossfades: [dsp::BypassCrossfade::default(); 2],
            dry_delays: [dsp::LatencyDelay::default(); 2],
            last_attack_seconds: 0.0,
            last_release_seconds: 0.0,
            last_oversampling: FetOversampling::X1,
            last_bypassed: false,
            bypass_ramp_samples: 1,
        }
    }
}

impl Default for GregFetParams {
    fn default() -> Self {
        Self {
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
                    min: 0.02,
                    max: 0.8,
                    factor: FloatRange::skew_factor(0.02),
                },
            )
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                250.0,
                FloatRange::Skewed {
                    min: 50.0,
                    max: 1100.0,
                    factor: FloatRange::skew_factor(50.0),
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
        }
    }
}

impl Plugin for GregFet {
    const NAME: &'static str = "Greg FET";
    const VENDOR: &'static str = "Greg Audio";
    const URL: &'static str = "https://example.invalid";
    const EMAIL: &'static str = "support@example.invalid";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

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
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.last_attack_seconds = self.params.attack.value() * 0.001;
        self.last_release_seconds = self.params.release.value() * 0.001;
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
        if initialized && delays_initialized {
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

        initialized && delays_initialized
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
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
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

        let input_gain = util::db_to_gain(self.params.input.value());
        let output_gain = util::db_to_gain(self.params.output.value());
        let threshold_db = self.params.threshold.value();
        let all_buttons_in = self.params.all_buttons_in.value();
        let ratio = if all_buttons_in {
            20.0
        } else {
            match self.params.ratio.value() {
                FetRatio::Four => 4.0,
                FetRatio::Eight => 8.0,
                FetRatio::Twelve => 12.0,
                FetRatio::Twenty => 20.0,
            }
        };
        let knee_db = if all_buttons_in {
            self.params.knee.value().max(12.0)
        } else {
            self.params.knee.value()
        };
        for mut channel_samples in buffer.iter_samples() {
            for (channel, sample) in channel_samples.iter_mut().enumerate() {
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

                let input_sample = if sample.is_finite() { *sample } else { 0.0 };
                let invert_left = self.params.invert_left.value();
                let invert_right = self.params.invert_right.value();
                let polarity = if (channel == 0 && invert_left) || (channel == 1 && invert_right) {
                    -1.0
                } else {
                    1.0
                };
                let phase_input = input_sample * polarity;
                let dry = self.dry_delays[channel].process(phase_input);
                let driven_sample = phase_input * input_gain;
                let envelope = self.envelopes[channel].next(driven_sample.abs());
                let level_db = 20.0 * envelope.max(1.0e-12).log10();
                let gain_reduction_db =
                    dsp::gain_reduction_db(level_db, threshold_db, ratio, knee_db);

                let compressed_sample =
                    driven_sample * util::db_to_gain(gain_reduction_db) * output_gain;
                let wet = self.oversamplers[channel].process(
                    compressed_sample,
                    oversampling_factor,
                    |oversampled_sample| dsp::soft_saturate(oversampled_sample, 0.75),
                );
                *sample = self.bypass_crossfades[channel].process(dry, wet);
            }
        }

        ProcessStatus::Normal
    }
}

impl GregFet {
    fn oversampling_factor(&self) -> dsp::OversamplingFactor {
        match self.params.oversampling.value() {
            FetOversampling::X1 => dsp::OversamplingFactor::X1,
            FetOversampling::X2 => dsp::OversamplingFactor::X2,
            FetOversampling::X4 => dsp::OversamplingFactor::X4,
            FetOversampling::X8 => dsp::OversamplingFactor::X8,
        }
    }
}

impl ClapPlugin for GregFet {
    const CLAP_ID: &'static str = "com.greg.audio.greg-fet";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("FET-style dynamics processor");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Compressor,
        ClapFeature::Mono,
        ClapFeature::Stereo,
    ];
}

nih_export_clap!(GregFet);

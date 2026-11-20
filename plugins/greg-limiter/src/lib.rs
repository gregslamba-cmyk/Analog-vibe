use std::{array, sync::Arc};

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
};
use nih_plug::prelude::*;

const LOOKAHEAD_MAX_SAMPLES: usize = 2048;

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
struct GregLimiterParams {
    #[id = "input"]
    input: FloatParam,
    #[id = "ceiling"]
    ceiling: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "lookahead"]
    lookahead: FloatParam,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "stereo_link"]
    stereo_link: BoolParam,
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
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[id = "invert_right"]
    invert_right: BoolParam,
    #[id = "bypass"]
    bypass: BoolParam,
}

impl Default for GregLimiterParams {
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
            ceiling: FloatParam::new(
                "Ceiling",
                -0.1,
                FloatRange::Linear {
                    min: -12.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(10.0),
                },
            )
            .with_unit(" ms"),
            lookahead: FloatParam::new(
                "Lookahead",
                5.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_unit(" ms"),
            oversampling: EnumParam::new("Oversampling", Oversampling::X4),
            stereo_link: BoolParam::new("Stereo Link", true),
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
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PeakLimiter {
    input_ring: [f32; LOOKAHEAD_MAX_SAMPLES],
    key_ring: [f32; LOOKAHEAD_MAX_SAMPLES],
    cursor: usize,
    lookahead_samples: usize,
    gain: f32,
    release_coefficient: f32,
}

impl Default for PeakLimiter {
    fn default() -> Self {
        Self {
            input_ring: [0.0; LOOKAHEAD_MAX_SAMPLES],
            key_ring: [0.0; LOOKAHEAD_MAX_SAMPLES],
            cursor: 0,
            lookahead_samples: 0,
            gain: 1.0,
            release_coefficient: 0.999,
        }
    }
}

impl PeakLimiter {
    fn initialize(&mut self, sample_rate: f32, lookahead_ms: f32, release_ms: f32) -> bool {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return false;
        }
        self.lookahead_samples =
            ((lookahead_ms * 0.001 * sample_rate).round() as usize).min(LOOKAHEAD_MAX_SAMPLES - 1);
        self.release_coefficient = (-1.0 / (release_ms * 0.001 * sample_rate).max(1.0e-6)).exp();
        self.reset();
        true
    }

    fn set_lookahead(&mut self, sample_rate: f32, lookahead_ms: f32) {
        self.lookahead_samples =
            ((lookahead_ms * 0.001 * sample_rate).round() as usize).min(LOOKAHEAD_MAX_SAMPLES - 1);
    }

    fn set_release(&mut self, sample_rate: f32, release_ms: f32) {
        self.release_coefficient = (-1.0 / (release_ms * 0.001 * sample_rate).max(1.0e-6)).exp();
    }

    fn reset(&mut self) {
        self.input_ring.fill(0.0);
        self.key_ring.fill(0.0);
        self.cursor = 0;
        self.gain = 1.0;
    }

    fn process(&mut self, input: f32, key: f32, ceiling: f32) -> f32 {
        let input = finite(input);
        let key = finite(key);
        self.input_ring[self.cursor] = input;
        self.key_ring[self.cursor] = key.abs();

        let read_index =
            (self.cursor + LOOKAHEAD_MAX_SAMPLES - self.lookahead_samples) % LOOKAHEAD_MAX_SAMPLES;

        let mut peak = 0.0_f32;
        let scan_samples = self.lookahead_samples + 1;
        for offset in 0..scan_samples {
            let index = (read_index + offset) % LOOKAHEAD_MAX_SAMPLES;
            peak = peak.max(self.key_ring[index]);
        }

        let target = if peak > ceiling {
            (ceiling / peak).clamp(0.0, 1.0)
        } else {
            1.0
        };
        if target < self.gain {
            self.gain = target;
        } else {
            self.gain = target + self.release_coefficient * (self.gain - target);
        }

        let delayed = self.input_ring[read_index];
        self.cursor = (self.cursor + 1) % LOOKAHEAD_MAX_SAMPLES;
        delayed * self.gain
    }
}

struct GregLimiter {
    params: Arc<GregLimiterParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    limiters: [PeakLimiter; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 4096>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_oversampling: Oversampling,
    last_lookahead: f32,
    last_release: f32,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
}

impl Default for GregLimiter {
    fn default() -> Self {
        Self {
            params: Arc::new(GregLimiterParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            limiters: [PeakLimiter::default(); plugin_support::MAX_AUDIO_CHANNELS],
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
            last_oversampling: Oversampling::X4,
            last_lookahead: 5.0,
            last_release: 100.0,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
        }
    }
}

impl GregLimiter {
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

impl Plugin for GregLimiter {
    const NAME: &'static str = "Greg Limiter";
    const VENDOR: &'static str = "Greg Audio";
    const URL: &'static str = "https://example.invalid";
    const EMAIL: &'static str = "support@example.invalid";
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
        let lookahead = self.params.lookahead.value();
        let release = self.params.release.value();
        let initialized = self
            .limiters
            .iter_mut()
            .all(|limiter| limiter.initialize(self.sample_rate, lookahead, release));
        let sidechains_initialized = self.configure_sidechains(self.sidechain_settings());
        let oversampling = self.params.oversampling.value();
        let oversampling_latency =
            Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
        let lookahead_latency = (lookahead * 0.001 * self.sample_rate)
            .round()
            .clamp(0.0, 2047.0) as usize;
        let total_latency = oversampling_latency as usize + lookahead_latency;
        let delays_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(total_latency));
        if initialized && sidechains_initialized && delays_initialized {
            self.last_oversampling = oversampling;
            self.last_lookahead = lookahead;
            self.last_release = release;
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
            context.set_latency_samples(total_latency as u32);
        }
        initialized && sidechains_initialized && delays_initialized
    }

    fn reset(&mut self) {
        for limiter in &mut self.limiters {
            limiter.reset();
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
        let lookahead = self.params.lookahead.value();
        let release = self.params.release.value();
        if oversampling != self.last_oversampling || (lookahead - self.last_lookahead).abs() > 0.001
        {
            let oversampling_latency =
                Oversampler::<f32>::latency_samples(OversamplingFactor::from(oversampling));
            let lookahead_latency = (lookahead * 0.001 * self.sample_rate)
                .round()
                .clamp(0.0, 2047.0) as usize;
            let total_latency = oversampling_latency as usize + lookahead_latency;
            for oversampler in &mut self.oversamplers {
                oversampler.reset();
            }
            for limiter in &mut self.limiters {
                limiter.set_lookahead(self.sample_rate, lookahead);
            }
            for delay in &mut self.dry_delays {
                delay.reset();
                if !delay.set_delay_samples(total_latency) {
                    return ProcessStatus::Error("Limiter latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            self.last_lookahead = lookahead;
            context.set_latency_samples(total_latency as u32);
        }
        if (release - self.last_release).abs() > 0.001 {
            for limiter in &mut self.limiters {
                limiter.set_release(self.sample_rate, release);
            }
            self.last_release = release;
        }
        let settings = self.sidechain_settings();
        if settings != self.last_sidechain_settings && !self.configure_sidechains(settings) {
            return ProcessStatus::Error("invalid limiter sidechain filter configuration");
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

        let factor = OversamplingFactor::from(oversampling);
        let use_external = self.params.external_sidechain.value();
        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());
        let input_gain = util::db_to_gain(self.params.input.value());
        let ceiling = util::db_to_gain(self.params.ceiling.value());
        let stereo_link = self.params.stereo_link.value();
        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for (frame_index, mut channel_samples) in buffer.iter_samples().enumerate() {
                let mut processed = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                let channel_count = channel_samples.len();
                let mut channel_values = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                let mut raw_values = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    raw_values[channel] = finite(*sample);
                    channel_values[channel] = raw_values[channel] * input_gain;
                }

                let mut keys = [0.0_f32; plugin_support::MAX_AUDIO_CHANNELS];
                for channel in 0..channel_count {
                    let polarity = if (channel == 0 && self.params.invert_left.value())
                        || (channel == 1 && self.params.invert_right.value())
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    let phase_input = channel_values[channel] * polarity;
                    let external_key = external_sidechain.as_ref().and_then(|channels| {
                        self.routing_matrix
                            .sidechain_channel(channel, channels.len())
                            .and_then(|sidechain_channel| channels.get(sidechain_channel))
                            .and_then(|samples| samples.get(frame_index))
                            .copied()
                            .filter(|value| value.is_finite())
                    });
                    keys[channel] = self.sidechains[channel].process_key(
                        phase_input,
                        external_key,
                        use_external,
                    );
                }
                if stereo_link && channel_count >= 2 {
                    let linked = keys[0].abs().max(keys[1].abs());
                    keys[0] = linked;
                    keys[1] = linked;
                }

                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    let phase_input = channel_values[channel]
                        * if (channel == 0 && self.params.invert_left.value())
                            || (channel == 1 && self.params.invert_right.value())
                        {
                            -1.0
                        } else {
                            1.0
                        };
                    let dry = self.dry_delays[channel].process(phase_input);
                    if self.params.sidechain_listen.value() {
                        processed[channel] = finite(keys[channel]);
                        self.metering
                            .record_sample(raw_values[channel], processed[channel], 0.0);
                    } else {
                        let limited =
                            self.limiters[channel].process(phase_input, keys[channel], ceiling);
                        let gain_reduction_db = if self.limiters[channel].gain > 0.0 {
                            -20.0 * self.limiters[channel].gain.log10()
                        } else {
                            120.0
                        };
                        let oversampler = &mut self.oversamplers[channel];
                        let wet = oversampler.process(limited, factor, |value| {
                            finite(value).clamp(-ceiling, ceiling)
                        });
                        processed[channel] =
                            finite(self.bypass_crossfades[channel].process(dry, wet));
                        self.metering.record_sample(
                            raw_values[channel],
                            processed[channel],
                            gain_reduction_db,
                        );
                    }
                    *sample = processed[channel];
                }
            }
        });
        self.metering.publish_block();
        if protection_result == ProtectionResult::Tripped {
            silence_buffer(buffer);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for GregLimiter {
    const CLAP_ID: &'static str = "com.greg.audio.greg-limiter";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Stereo and mono lookahead peak limiter with filtered sidechain");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Limiter,
        ClapFeature::Mono,
        ClapFeature::Stereo,
    ];
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

#[cfg(test)]
mod tests {
    use super::PeakLimiter;

    #[test]
    fn limiter_reduces_and_releases_with_valid_samples() {
        let mut limiter = PeakLimiter::default();
        assert!(limiter.initialize(48_000.0, 1.0, 100.0));

        let muted = limiter.process(1.0, 1.0, 0.5);
        assert!(muted.is_finite());
        assert!(muted <= 0.50001);

        let released = limiter.process(0.0, 0.0, 0.5);
        assert!(released.is_finite());
        assert!(released >= 0.0);
    }

    #[test]
    fn limiter_handles_invalid_configuration() {
        let mut limiter = PeakLimiter::default();
        assert!(!limiter.initialize(0.0, 1.0, 100.0));
        assert!(limiter.initialize(48_000.0, 1.0, 100.0));

        assert_eq!(limiter.process(f32::NAN, f32::INFINITY, 0.5), 0.0);
    }
}

nih_export_clap!(GregLimiter);

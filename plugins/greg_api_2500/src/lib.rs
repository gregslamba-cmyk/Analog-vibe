#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, EnvelopeFollower, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    sidechain_processor::{SidechainFilter, SidechainProcessor},
};
use nih_plug::prelude::*;
use greg_gui_core::GuiTelemetry;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum ApiVersion {
    #[id = "2500"]
    #[name = "2500"]
    V2500,
    #[id = "2500plus"]
    #[name = "2500+"]
    V2500Plus,
}

impl ApiVersion {
    fn supports_mix(self) -> bool {
        matches!(self, ApiVersion::V2500Plus)
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum ApiRatio {
    #[id = "1.5:1"]
    #[name = "1.5:1"]
    R15,
    #[id = "2:1"]
    #[name = "2:1"]
    R2,
    #[id = "3:1"]
    #[name = "3:1"]
    R3,
    #[id = "4:1"]
    #[name = "4:1"]
    R4,
    #[id = "6:1"]
    #[name = "6:1"]
    R6,
    #[id = "10:1"]
    #[name = "10:1"]
    R10,
    #[id = "inf:1"]
    #[name = "infinity:1"]
    RInf,
}

impl ApiRatio {
    fn value(self) -> f32 {
        match self {
            ApiRatio::R15 => 1.5,
            ApiRatio::R2 => 2.0,
            ApiRatio::R3 => 3.0,
            ApiRatio::R4 => 4.0,
            ApiRatio::R6 => 6.0,
            ApiRatio::R10 => 10.0,
            ApiRatio::RInf => 100.0,
        }
    }
}

impl From<ApiRatio> for f32 {
    fn from(value: ApiRatio) -> Self {
        value.value()
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum KneeMode {
    #[id = "soft"]
    #[name = "Soft"]
    Soft,
    #[id = "medium"]
    #[name = "Medium"]
    Medium,
    #[id = "hard"]
    #[name = "Hard"]
    Hard,
}

impl KneeMode {
    fn knee_width(self) -> f32 {
        match self {
            KneeMode::Soft => 12.0,
            KneeMode::Medium => 6.0,
            KneeMode::Hard => 1.5,
        }
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum ToneType {
    #[id = "old"]
    #[name = "Old (Feedback)"]
    Old,
    #[id = "new"]
    #[name = "New (Feed-Forward)"]
    New,
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum Thrust {
    #[id = "normal"]
    #[name = "Normal"]
    Normal,
    #[id = "medium"]
    #[name = "Medium"]
    Medium,
    #[id = "loud"]
    #[name = "Loud"]
    Loud,
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum MixBlend {
    #[id = "off"]
    #[name = "OFF"]
    Off,
    #[id = "crossfade"]
    #[name = "Crossfade"]
    Crossfade,
    #[id = "parallel"]
    #[name = "Parallel"]
    Parallel,
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
struct GregApi2500Params {
    #[id = "input"]
    input: FloatParam,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "ratio"]
    ratio: EnumParam<ApiRatio>,
    #[id = "knee"]
    knee: EnumParam<KneeMode>,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "makeup_gain"]
    makeup_gain: FloatParam,
    #[id = "version"]
    version: EnumParam<ApiVersion>,
    #[id = "tone_type"]
    tone_type: EnumParam<ToneType>,
    #[id = "thrust"]
    thrust: EnumParam<Thrust>,
    #[id = "mix_blend"]
    mix_blend: EnumParam<MixBlend>,
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
}

impl Default for GregApi2500Params {
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
                    min: -20.0,
                    max: 10.0,
                },
            )
            .with_unit(" dBu"),
            ratio: EnumParam::new("Ratio", ApiRatio::R2),
            knee: EnumParam::new("Knee", KneeMode::Hard),
            attack: FloatParam::new(
                "Attack",
                1.0,
                FloatRange::Linear {
                    min: 0.03,
                    max: 30.0,
                },
            )
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                1.0,
                FloatRange::Linear {
                    min: 0.05,
                    max: 2.0,
                },
            )
            .with_unit(" s"),
            makeup_gain: FloatParam::new(
                "Makeup Gain",
                0.0,
                FloatRange::Linear {
                    min: -20.0,
                    max: 20.0,
                },
            )
            .with_unit(" dB"),
            version: EnumParam::new("Version", ApiVersion::V2500),
            tone_type: EnumParam::new("Tone", ToneType::Old),
            thrust: EnumParam::new("Thrust", Thrust::Normal),
            mix_blend: EnumParam::new("Mix Blend", MixBlend::Off),
            stereo_link: BoolParam::new("Stereo Link", true),
            oversampling: EnumParam::new("Oversampling", Oversampling::X1),
            bypass: BoolParam::new("Bypass", false),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
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
        }
    }
}

struct GregApi2500 {
    params: Arc<GregApi2500Params>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    envelopes: [EnvelopeFollower<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    sidechains: [SidechainProcessor<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_sidechain_settings: SidechainSettings,
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    last_oversampling: Oversampling,
    panic_protection: PanicProtection,
    metering: Metering,
    sample_rate: f32,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregApi2500 {
    fn default() -> Self {
        Self {
            params: Arc::new(GregApi2500Params::default()),
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
            last_bypassed: false,
            bypass_ramp_samples: 1,
            last_oversampling: Oversampling::X1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            sample_rate: 0.0,
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl GregApi2500 {
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

impl Plugin for GregApi2500 {
    const NAME: &'static str = "Greg API 2500";
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

        let attack_ms = self.params.attack.value();
        let release_ms = self.params.release.value() * 1000.0;
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

    fn editor(
        &mut self,
        _async_executor: AsyncExecutor<Self>,
    ) -> Option<Box<dyn Editor>> {
        None
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
                    return ProcessStatus::Error("Greg API 2500 latency exceeds dry delay storage");
                }
            }
            self.last_oversampling = oversampling;
            context.set_latency_samples(latency);
        }

        let version = self.params.version.value();
        let stereo_link = self.params.stereo_link.value();
        let use_external = self.params.external_sidechain.value();
        let external_sidechain = aux
            .inputs
            .first_mut()
            .map(|sidechain| sidechain.as_slice_immutable());
        let invert_left = self.params.invert_left.value();
        let invert_right = self.params.invert_right.value();
        let input_gain = util::db_to_gain(self.params.input.value());
        let output_gain = util::db_to_gain(self.params.makeup_gain.value());
        let threshold_db = self.params.threshold.value();
        let ratio_value: f32 = self.params.ratio.value().into();
        let knee_width = self.params.knee.value().knee_width();
        let tone_type = self.params.tone_type.value();
        let thrust = self.params.thrust.value();
        let mix_blend = self.params.mix_blend.value();
        let factor = OversamplingFactor::from(oversampling);
        let thrust_offset_db = thrust_detector_offset(thrust);
        let tone_scale = tone_reduction_scale(tone_type);
        let mix_active = version.supports_mix();

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
                        use_external || external_key.is_some(),
                    );
                    self.metering.record_sidechain_input(keys[channel]);
                }

                if stereo_link && channel_count >= 2 {
                    let linked = keys[0].abs().max(keys[1].abs());
                    keys[0] = linked;
                    keys[1] = linked;
                }

                for (channel, sample) in frame.iter_mut().enumerate() {
                    let dry = self.dry_delays[channel].process(driven[channel]);
                    if self.params.sc_listen.value() {
                        *sample = keys[channel];
                        self.metering.record_sample(raw[channel], *sample, 0.0);
                        continue;
                    }

                    let level_db =
                        20.0 * keys[channel].abs().max(1.0e-12).log10() + thrust_offset_db;
                    let target_reduction =
                        target_reduction_db(level_db, threshold_db, ratio_value, knee_width)
                            * tone_scale;

                    let reduction_db = self.envelopes[channel].next(target_reduction);
                    let gain = util::db_to_gain(-reduction_db);
                    let wet_signal = driven[channel] * gain * output_gain;
                    let oversampler = &mut self.oversamplers[channel];
                    let wet = oversampler.process(finite(wet_signal), factor, |value| value);

                    let mixed = if mix_active {
                        match mix_blend {
                            MixBlend::Off => wet,
                            MixBlend::Crossfade => {
                                let wet_amount = 1.0 - (-reduction_db / 20.0).exp().clamp(0.0, 1.0);
                                dry * (1.0 - wet_amount) + wet * wet_amount
                            }
                            MixBlend::Parallel => finite(dry + wet),
                        }
                    } else {
                        wet
                    };

                    *sample = finite(self.bypass_crossfades[channel].process(dry, mixed));
                    self.metering
                        .record_sample(raw[channel], *sample, reduction_db);
                }

                if stereo_link && channel_count >= 2 {
                    self.metering.record_stereo_sample(raw[0], raw[1]);
                }
            }
        });

        self.metering.publish_block();
        self.telemetry.push_in_peak(0, 0.0);
        self.telemetry.push_out_peak(0, 0.0);
        self.telemetry.set_gr_and_phase(0.0, 1.0);
        if protection_result == ProtectionResult::Tripped {
            silence_buffer(buffer);
        }
        ProcessStatus::Normal
    }
}

fn thrust_detector_offset(thrust: Thrust) -> f32 {
    match thrust {
        Thrust::Normal => 0.0,
        Thrust::Medium => 2.0,
        Thrust::Loud => 4.0,
    }
}

fn tone_reduction_scale(tone_type: ToneType) -> f32 {
    match tone_type {
        ToneType::Old => 1.1,
        ToneType::New => 1.0,
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
    GregApi2500,
    "com.greg.audio.greg-api-2500",
    "API 2500-style VCA compressor (2500 / 2500+) with Old/New tone, Thrust and mix blend",
    compressor
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_values_are_correct() {
        assert_eq!(f32::from(ApiRatio::R15), 1.5);
        assert_eq!(f32::from(ApiRatio::R2), 2.0);
        assert_eq!(f32::from(ApiRatio::R3), 3.0);
        assert_eq!(f32::from(ApiRatio::R4), 4.0);
        assert_eq!(f32::from(ApiRatio::R6), 6.0);
        assert_eq!(f32::from(ApiRatio::R10), 10.0);
        assert_eq!(f32::from(ApiRatio::RInf), 100.0);
    }

    #[test]
    fn tone_type_enum() {
        assert_eq!(ToneType::Old as u32, 0);
        assert_eq!(ToneType::New as u32, 1);
    }

    #[test]
    fn thrust_enum() {
        assert_eq!(Thrust::Normal as u32, 0);
        assert_eq!(Thrust::Medium as u32, 1);
        assert_eq!(Thrust::Loud as u32, 2);
    }

    #[test]
    fn mix_blend_enum() {
        assert_eq!(MixBlend::Off as u32, 0);
        assert_eq!(MixBlend::Crossfade as u32, 1);
        assert_eq!(MixBlend::Parallel as u32, 2);
    }

    #[test]
    fn envelope_follower_works() {
        let mut env = EnvelopeFollower::default();
        assert!(env.initialize(48_000.0, 0.01, 0.1));
        let rising = env.next(1.0);
        assert!(rising > 0.0 && rising < 1.0);
        let falling = env.next(0.0);
        assert!(falling < rising);
    }

    #[test]
    fn knee_width_values() {
        assert_eq!(KneeMode::Soft.knee_width(), 12.0);
        assert_eq!(KneeMode::Medium.knee_width(), 6.0);
        assert_eq!(KneeMode::Hard.knee_width(), 1.5);
    }

    #[test]
    fn thrust_detector_offsets() {
        assert_eq!(thrust_detector_offset(Thrust::Normal), 0.0);
        assert_eq!(thrust_detector_offset(Thrust::Medium), 2.0);
        assert_eq!(thrust_detector_offset(Thrust::Loud), 4.0);
    }

    #[test]
    fn tone_reduction_scales() {
        assert!((tone_reduction_scale(ToneType::Old) - 1.1).abs() < 1e-6);
        assert!((tone_reduction_scale(ToneType::New) - 1.0).abs() < 1e-6);
    }
}

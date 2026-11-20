use std::{array, sync::Arc};

use core_dsp::core_dsp_modules::{
    filters::FilterType,
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    surgical_eq::{EqBand, EqPhaseMode, SurgicalEq},
};
use nih_plug::prelude::*;

const BAND_COUNT: usize = 10;
const LINEAR_PHASE_TAPS: usize = 257;
const MAX_DRY_DELAY: usize = 256;

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum EqFilterKind {
    #[id = "bell"]
    #[name = "Bell"]
    Bell,
    #[id = "low_shelf"]
    #[name = "Low Shelf"]
    LowShelf,
    #[id = "high_shelf"]
    #[name = "High Shelf"]
    HighShelf,
    #[id = "high_pass"]
    #[name = "High Pass"]
    HighPass,
    #[id = "low_pass"]
    #[name = "Low Pass"]
    LowPass,
    #[id = "notch"]
    #[name = "Notch"]
    Notch,
    #[id = "band_pass"]
    #[name = "Band Pass"]
    BandPass,
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum PhaseMode {
    #[id = "minimum_phase"]
    #[name = "Minimum Phase"]
    MinimumPhase,
    #[id = "linear_phase"]
    #[name = "Linear Phase"]
    LinearPhase,
}

#[derive(Params)]
struct EqBandParams {
    #[id = "enabled"]
    enabled: BoolParam,
    #[id = "type"]
    filter_type: EnumParam<EqFilterKind>,
    #[id = "frequency"]
    frequency: FloatParam,
    #[id = "gain"]
    gain: FloatParam,
    #[id = "q"]
    q: FloatParam,
}

impl Default for EqBandParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Enabled", false),
            filter_type: EnumParam::new("Type", EqFilterKind::Bell),
            frequency: FloatParam::new(
                "Frequency",
                1_000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(20.0),
                },
            )
            .with_unit(" Hz"),
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            q: FloatParam::new(
                "Q",
                0.707,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 18.0,
                    factor: FloatRange::skew_factor(0.1),
                },
            ),
        }
    }
}

#[derive(Params)]
struct GregSurgicalEqParams {
    #[id = "phase_mode"]
    phase_mode: EnumParam<PhaseMode>,
    #[id = "bypass"]
    bypass: BoolParam,
    #[nested(array, group = "Bands")]
    bands: [EqBandParams; BAND_COUNT],
}

impl Default for GregSurgicalEqParams {
    fn default() -> Self {
        Self {
            phase_mode: EnumParam::new("Phase Mode", PhaseMode::MinimumPhase),
            bypass: BoolParam::new("Bypass", false),
            bands: array::from_fn(|_| EqBandParams::default()),
        }
    }
}

struct GregSurgicalEq {
    params: Arc<GregSurgicalEqParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    filters: [SurgicalEq<BAND_COUNT, LINEAR_PHASE_TAPS, f32>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, MAX_DRY_DELAY>; plugin_support::MAX_AUDIO_CHANNELS],
    last_bands: [EqBand<f32>; BAND_COUNT],
    last_phase_mode: PhaseMode,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    sample_rate: f32,
    initialized: bool,
    panic_protection: PanicProtection,
    metering: Metering,
}

impl Default for GregSurgicalEq {
    fn default() -> Self {
        Self {
            params: Arc::new(GregSurgicalEqParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            filters: array::from_fn(|_| SurgicalEq::default()),
            bypass_crossfades: array::from_fn(|_| BypassCrossfade::default()),
            dry_delays: array::from_fn(|_| LatencyDelay::default()),
            last_bands: [EqBand::default(); BAND_COUNT],
            last_phase_mode: PhaseMode::MinimumPhase,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            sample_rate: 0.0,
            initialized: false,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
        }
    }
}

impl GregSurgicalEq {
    fn band_settings(&self) -> [EqBand<f32>; BAND_COUNT] {
        array::from_fn(|index| {
            let band = &self.params.bands[index];
            EqBand {
                enabled: band.enabled.value(),
                kind: match band.filter_type.value() {
                    EqFilterKind::Bell => FilterType::Bell,
                    EqFilterKind::LowShelf => FilterType::LowShelf,
                    EqFilterKind::HighShelf => FilterType::HighShelf,
                    EqFilterKind::HighPass => FilterType::HighPass,
                    EqFilterKind::LowPass => FilterType::LowPass,
                    EqFilterKind::Notch => FilterType::Notch,
                    EqFilterKind::BandPass => FilterType::BandPass,
                },
                frequency_hz: band.frequency.value().min(self.sample_rate * 0.49).max(1.0),
                q: band.q.value(),
                gain_db: band.gain.value(),
            }
        })
    }

    fn configure_filters(
        &mut self,
        sample_rate: f32,
        bands: &[EqBand<f32>; BAND_COUNT],
        phase_mode: PhaseMode,
    ) -> bool {
        let dsp_phase_mode = match phase_mode {
            PhaseMode::MinimumPhase => EqPhaseMode::MinimumPhase,
            PhaseMode::LinearPhase => EqPhaseMode::LinearPhase,
        };
        for filter in &mut self.filters {
            if !filter.configure(sample_rate, bands) {
                return false;
            }
            filter.set_phase_mode(dsp_phase_mode);
        }
        self.last_bands = *bands;
        self.last_phase_mode = phase_mode;
        true
    }

    fn latency(phase_mode: PhaseMode) -> u32 {
        match phase_mode {
            PhaseMode::MinimumPhase => 0,
            PhaseMode::LinearPhase => {
                SurgicalEq::<BAND_COUNT, LINEAR_PHASE_TAPS, f32>::latency_samples(
                    EqPhaseMode::LinearPhase,
                )
            }
        }
    }
}

impl Plugin for GregSurgicalEq {
    const NAME: &'static str = "Greg Surgical EQ";
    const VENDOR: &'static str = "Greg Audio";
    const URL: &'static str = "https://example.invalid";
    const EMAIL: &'static str = "support@example.invalid";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &plugin_support::SURROUND_IO_LAYOUTS;

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
            core_dsp::core_dsp_modules::routing_matrix::RoutingMode::Multichannel,
        ) {
            return false;
        }
        self.sample_rate = buffer_config.sample_rate;
        let bands = self.band_settings();
        let phase_mode = self.params.phase_mode.value();
        let latency_samples = Self::latency(phase_mode);
        let delay_initialized = self
            .dry_delays
            .iter_mut()
            .all(|delay| delay.set_delay_samples(latency_samples as usize));
        let filters_initialized =
            self.configure_filters(buffer_config.sample_rate, &bands, phase_mode);
        if !delay_initialized || !filters_initialized {
            return false;
        }

        self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
        self.last_bypassed = self.params.bypass.value();
        for delay in &mut self.dry_delays {
            delay.reset();
        }
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
        context.set_latency_samples(latency_samples);
        self.initialized = true;
        true
    }

    fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
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
        if self.panic_protection.is_disabled() {
            return ProcessStatus::Normal;
        }
        if !self.initialized {
            return ProcessStatus::Error("plugin has not been initialized");
        }

        let bands = self.band_settings();
        let phase_mode = self.params.phase_mode.value();
        if bands != self.last_bands || phase_mode != self.last_phase_mode {
            if !self.configure_filters(self.sample_rate, &bands, phase_mode) {
                return ProcessStatus::Error("invalid EQ configuration");
            }
            let latency_samples = Self::latency(phase_mode);
            for delay in &mut self.dry_delays {
                delay.reset();
                if !delay.set_delay_samples(latency_samples as usize) {
                    return ProcessStatus::Error("EQ latency exceeds dry delay storage");
                }
            }
            context.set_latency_samples(latency_samples);
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut channel_samples in buffer.iter_samples() {
                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    let input = if sample.is_finite() { *sample } else { 0.0 };
                    let dry = self.dry_delays[channel].process(input);
                    let wet = self.filters[channel].process(input);
                    *sample = self.bypass_crossfades[channel].process(dry, wet);
                    self.metering.record_sample(input, *sample, 0.0);
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

impl ClapPlugin for GregSurgicalEq {
    const CLAP_ID: &'static str = "com.greg.audio.greg-surgical-eq";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Ten-band precision equalizer with minimum and linear phase modes");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Equalizer,
        ClapFeature::Mono,
        ClapFeature::Stereo,
    ];
}

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

nih_export_clap!(GregSurgicalEq);

#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{array, sync::Arc};
use std::sync::atomic::Ordering;

use core_dsp::core_dsp_modules::{
    filters::FilterType,
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    surgical_eq::{EqBand, EqPhaseMode, SurgicalEq},
};
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params};
use greg_gui_core::prelude::*;
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

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregSurgicalEqParams>,
}

impl Model for EditorData {}

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
    telemetry: Arc<GuiTelemetry>,
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
            telemetry: GuiTelemetry::new(true),
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
    const VENDOR: &'static str = core_dsp::core_dsp_modules::plugin_export::VENDOR;
    const URL: &'static str = core_dsp::core_dsp_modules::plugin_export::URL;
    const EMAIL: &'static str = core_dsp::core_dsp_modules::plugin_export::EMAIL;
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params_main = self.params.clone();
        let params_adv = self.params.clone();
        let mut param_defs: Vec<greg_gui_core::ParamDef> = vec![
            greg_gui_core::ParamDef::enum_switch("Phase Mode", Box::new(|cx| {
                EnumSwitch::new(cx, EditorData::params, |p| &p.phase_mode);
            })),
            greg_gui_core::ParamDef::toggle("Bypass", Box::new(|cx| {
                ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
            })),
        ];
        for band_idx in 0..BAND_COUNT {
            let idx = band_idx;
            param_defs.push(greg_gui_core::ParamDef::toggle(
                format!("Band {} Enabled", idx + 1).leak(),
                Box::new(move |cx| {
                    ToggleSwitch::new(cx, EditorData::params, move |p| &p.bands[idx].enabled);
                }),
            ));
            param_defs.push(greg_gui_core::ParamDef::enum_switch(
                format!("Band {} Type", idx + 1).leak(),
                Box::new(move |cx| {
                    EnumSwitch::new(cx, EditorData::params, move |p| &p.bands[idx].filter_type);
                }),
            ));
            param_defs.push(greg_gui_core::ParamDef::knob(
                format!("Band {} Frequency", idx + 1).leak(),
                Box::new(move |cx| {
                    RotaryKnob::new(cx, EditorData::params, move |p| &p.bands[idx].frequency);
                }),
            ));
            param_defs.push(greg_gui_core::ParamDef::knob(
                format!("Band {} Gain", idx + 1).leak(),
                Box::new(move |cx| {
                    RotaryKnob::new(cx, EditorData::params, move |p| &p.bands[idx].gain);
                }),
            ));
            param_defs.push(greg_gui_core::ParamDef::knob(
                format!("Band {} Q", idx + 1).leak(),
                Box::new(move |cx| {
                    RotaryKnob::new(cx, EditorData::params, move |p| &p.bands[idx].q);
                }),
            ));
        }
        let param_defs: &'static [greg_gui_core::ParamDef] = Box::leak(param_defs.into_boxed_slice());
        create_greg_editor_with_params(
            "Greg Surgical EQ",
            self.telemetry.clone(),
            move |cx| {
                let editor_data = EditorData { params: params_main.clone() };
                editor_data.build(cx);
            },
            move |cx| {
                let editor_data = EditorData { params: params_adv.clone() };
                editor_data.build(cx);
            },
            Some(param_defs),
        )
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
                    self.telemetry.push_in_peak(channel, input);
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

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

core_dsp::export_greg_plugin!(
    GregSurgicalEq,
    "com.greg.audio.greg-surgical-eq",
    "Ten-band precision equalizer with minimum and linear phase modes",
    equalizer
);

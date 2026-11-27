//! Copyright (c) 2026 Alberto Gregorio Ferraris. All rights reserved.
//! Licensed under GNU GPLv3 or later + Proprietary Commercial License.
//
//! # Greg Tape — Master Tape 7.5/15/30 IPS NAB/CCIR
//!
//! Simulazione di nastro magnetico con `TapePhysics` (isteresi magnetica,
//! pre/de-emphasis, head bump, gap loss, wow & flutter), `TransformerHysteresis`
//! e oversampling polifase. DSP interno a f64, zero-alloc nel thread audio,
//! NaN/Inf sanitizzati a 0.0.

#![forbid(unsafe_code)]
#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::array;
use std::sync::{Arc, atomic::Ordering};

use core_dsp::core_dsp_modules::{
    tape_physics::{TapePhysics, TapeSpeed, TapeStandard},
    transformer_hysteresis::TransformerHysteresis,
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    plugin_export::{VENDOR, URL, EMAIL},
};
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params, greg_static_params};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;
use plugin_support::CommonChannelParams;

const MAX_DRY_DELAY: usize = 256;
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
enum TapeMode {
    #[id = "7_5_ips"]
    #[name = "7.5 IPS"]
    Ips7_5,
    #[id = "15_ips"]
    #[name = "15 IPS"]
    Ips15,
    #[id = "30_ips"]
    #[name = "30 IPS"]
    Ips30,
}

impl Default for TapeMode {
    fn default() -> Self {
        TapeMode::Ips15
    }
}

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum EqCurve {
    #[id = "nab"]
    #[name = "NAB"]
    Nab,
    #[id = "ccir"]
    #[name = "CCIR"]
    Ccir,
}

impl Default for EqCurve {
    fn default() -> Self {
        EqCurve::Nab
    }
}

#[derive(Params)]
struct GregTapeParams {
    #[id = "input_drive"]
    input_drive: FloatParam,
    #[id = "tape_speed"]
    tape_speed: EnumParam<TapeMode>,
    #[id = "eq_curve"]
    eq_curve: EnumParam<EqCurve>,
    #[id = "bias"]
    bias: FloatParam,
    #[id = "saturation"]
    saturation: FloatParam,
    #[id = "output_trim"]
    output_trim: FloatParam,
    #[id = "dry_wet"]
    dry_wet: FloatParam,
    #[id = "head_bump"]
    head_bump: FloatParam,
    #[id = "hf_rolloff"]
    hf_rolloff: FloatParam,
    #[id = "wow_flutter"]
    wow_flutter: FloatParam,
    #[id = "crosstalk"]
    crosstalk: FloatParam,
    #[allow(dead_code)]
    #[id = "bypass"]
    bypass: BoolParam,
    #[allow(dead_code)]
    channel_setup: CommonChannelParams,

    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "invert_left"]
    invert_left: BoolParam,
}

impl Default for GregTapeParams {
    fn default() -> Self {
        Self {
            input_drive: FloatParam::new("Input Drive", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 }).with_unit(" dB"),
            tape_speed: EnumParam::new("Tape Speed", TapeMode::Ips15),
            eq_curve: EnumParam::new("EQ Curve", EqCurve::Nab),
            bias: FloatParam::new("Bias", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            saturation: FloatParam::new("Saturation/Iron", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            output_trim: FloatParam::new("Output Trim", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 }).with_unit(" dB"),
            dry_wet: FloatParam::new("Dry/Wet", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            head_bump: FloatParam::new("Head Bump", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            hf_rolloff: FloatParam::new("HF Roll-off", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            wow_flutter: FloatParam::new("Wow & Flutter", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            crosstalk: FloatParam::new("Crosstalk", 0.0, FloatRange::Linear { min: 0.0, max: 0.1 }),
            bypass: BoolParam::new("Bypass", false),
            channel_setup: CommonChannelParams::default(),
            oversampling: EnumParam::new("Oversampling", Oversampling::X1),
            invert_left: BoolParam::new("Invert Left Polarity", false),
        }
    }
}

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregTapeParams>,
}

impl Model for EditorData {}

struct GregTape {
    params: Arc<GregTapeParams>,
    tapes: [TapePhysics; plugin_support::MAX_AUDIO_CHANNELS],
    transformers: [TransformerHysteresis; plugin_support::MAX_AUDIO_CHANNELS],
    oversamplers: [Oversampler<f64>; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, MAX_DRY_DELAY>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_oversampling: Oversampling,
    _last_dry_wet: f32,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    initialized: bool,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregTape {
    fn default() -> Self {
        Self {
            params: Arc::new(GregTapeParams::default()),
            tapes: array::from_fn(|_| TapePhysics::new()),
            transformers: array::from_fn(|_| TransformerHysteresis::new()),
            oversamplers: array::from_fn(|_| Oversampler::<f64>::default()),
            bypass_crossfades: array::from_fn(|_| BypassCrossfade::default()),
            dry_delays: array::from_fn(|_| LatencyDelay::default()),
            sample_rate: 0.0,
            last_oversampling: Oversampling::X2,
            _last_dry_wet: 0.0,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            initialized: false,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl GregTape {
    fn configure_tapes(&mut self, sample_rate: f64) {
        for tape in &mut self.tapes {
            let ok = tape.configure(sample_rate, TapeSpeed::Ips15, TapeStandard::Nab);
            let _ = ok;
        }
    }

    fn set_latency(&mut self, latency_samples: u32, _buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        let d = latency_samples as usize;
        let ok = self.dry_delays.iter_mut().all(|delay| delay.set_delay_samples(d));
        if ok {
            _context.set_latency_samples(latency_samples);
        }
        ok
    }
}

impl Plugin for GregTape {
    const NAME: &'static str = "Greg Tape";
    const VENDOR: &'static str = VENDOR;
    const URL: &'static str = URL;
    const EMAIL: &'static str = EMAIL;
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
        let Some(_channel_count) = audio_io_layout
            .main_input_channels
            .map(|channels| channels.get() as usize)
        else {
            return false;
        };
        self.sample_rate = buffer_config.sample_rate;

        let sr = buffer_config.sample_rate as f64;
        let speed = match self.params.tape_speed.value() {
            TapeMode::Ips7_5 => TapeSpeed::Ips7_5,
            TapeMode::Ips30 => TapeSpeed::Ips30,
            _ => TapeSpeed::Ips15,
        };
        let standard = match self.params.eq_curve.value() {
            EqCurve::Ccir => TapeStandard::Ccir,
            _ => TapeStandard::Nab,
        };
        self.configure_tapes(sr);
        for tape in &mut self.tapes {
            tape.configure(sr, speed, standard);
            tape.set_drive(0.0, 0.0, 0.0);
            tape.set_wow_flutter(0.0, 0.0);
        }

        let latency = Oversampler::<f64>::latency_samples(self.params.oversampling.value().into());
        if !self.set_latency(latency, buffer_config, context) {
            return false;
        }
        for delay in &mut self.dry_delays {
            delay.reset();
        }
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.params.bypass.value(), 0);
        }
        self.last_bypassed = self.params.bypass.value();
        self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
        self.initialized = true;
        true
    }

    fn reset(&mut self) {
        for tape in &mut self.tapes {
            tape.reset();
        }
        for transformer in &mut self.transformers {
            transformer.reset();
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
        let param_defs: Vec<greg_gui_core::ParamDef> = vec![
            greg_gui_core::ParamDef::knob("Input Drive", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.input_drive);
            })),
            greg_gui_core::ParamDef::enum_switch("Tape Speed", Box::new(|cx| {
                EnumSwitch::new(cx, EditorData::params, |p| &p.tape_speed);
            })),
            greg_gui_core::ParamDef::enum_switch("EQ Curve", Box::new(|cx| {
                EnumSwitch::new(cx, EditorData::params, |p| &p.eq_curve);
            })),
            greg_gui_core::ParamDef::knob("Bias", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.bias);
            })),
            greg_gui_core::ParamDef::knob("Saturation/Iron", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.saturation);
            })),
            greg_gui_core::ParamDef::knob("Output Trim", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.output_trim);
            })),
            greg_gui_core::ParamDef::knob("Dry/Wet", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.dry_wet);
            })),
            greg_gui_core::ParamDef::toggle("Bypass", Box::new(|cx| {
                ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
            })),
            greg_gui_core::ParamDef::enum_switch("Oversampling", Box::new(|cx| {
                EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
            })),
            greg_gui_core::ParamDef::toggle("Invert Left Polarity", Box::new(|cx| {
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
            })),
        ];
        let param_defs: &'static [greg_gui_core::ParamDef] = greg_static_params!(param_defs);
        create_greg_editor_with_params(
            "Greg Tape",
            self.telemetry.clone(),
            move |_| {},
            move |_cx| {},
            Some(param_defs),
            move |cx| EditorData { params: params_main.clone() }.build(cx),
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.panic_protection.is_disabled() {
            return ProcessStatus::Normal;
        }
        if !self.initialized {
            return ProcessStatus::Error("plugin has not been initialized");
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        let dry_wet = self.params.dry_wet.value();
        let dry_wet_lin = dry_wet;
        let invert_left = self.params.invert_left.value();

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut channel_samples in buffer.iter_samples() {
                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    let input = if sample.is_finite() { *sample } else { 0.0 };
                    let input = if invert_left && channel == 0 { -input } else { input };
                    let dry = self.dry_delays[channel].process(input);
                    let tape_in = if input.is_finite() { input } else { 0.0 } as f64;
                    let tape_out = self.tapes[channel].process_sample(tape_in);
                    let transformer_out = self.transformers[channel].process(tape_out, 1.0 + self.params.saturation.value() as f64, 0.5, 0.1);
                    let wet = transformer_out as f32;
                    let mix = (1.0 - dry_wet_lin) * dry + dry_wet_lin * wet;
                    *sample = self.bypass_crossfades[channel].process(dry, mix);
                    self.metering.record_sample(input, *sample, 0.0);
                    self.telemetry.push_in_peak(channel, input);
                    self.telemetry.push_out_peak(channel, *sample);
                }
            }
        });
        self.metering.publish_block();
        self.telemetry.num_channels.store(0u32, Ordering::Relaxed);
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
            for mut samples in buffer.iter_samples() {
                for sample in samples.iter_mut() {
                    *sample = 0.0;
                }
            }
        }

        ProcessStatus::Normal
    }
}

core_dsp::export_greg_plugin!(
    GregTape,
    "com.greg.audio.greg-tape",
    "Master tape emulation: 7.5/15/30 IPS, NAB/CCIR EQ, bias and saturation"
);

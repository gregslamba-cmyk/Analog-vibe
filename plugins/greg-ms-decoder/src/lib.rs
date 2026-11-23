#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{sync::Arc, sync::atomic::Ordering};

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::StereoFrame,
};
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;

#[derive(Params)]
struct MidSideDecoderParams {
    #[id = "mid_trim"]
    mid_trim: FloatParam,
    #[id = "side_trim"]
    side_trim: FloatParam,
    #[id = "balance"]
    balance: FloatParam,
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[id = "invert_right"]
    invert_right: BoolParam,
    #[id = "bypass"]
    bypass: BoolParam,
}

impl Default for MidSideDecoderParams {
    fn default() -> Self {
        Self {
            mid_trim: FloatParam::new(
                "Mid Trim",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            side_trim: FloatParam::new(
                "Side Trim",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
            balance: FloatParam::new(
                "Balance",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<MidSideDecoderParams>,
}

impl Model for EditorData {}

struct GregMsDecoder {
    params: Arc<MidSideDecoderParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; 2],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregMsDecoder {
    fn default() -> Self {
        Self {
            params: Arc::new(MidSideDecoderParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); 2],
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl Plugin for GregMsDecoder {
    const NAME: &'static str = "Greg M/S Decoder";
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
            core_dsp::core_dsp_modules::routing_matrix::RoutingMode::MidSide,
        ) {
            return false;
        }
        self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
        context.set_latency_samples(0);
        true
    }

    fn reset(&mut self) {
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
                greg_gui_core::ParamDef::knob("Mid Trim", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.mid_trim);
                })),
                greg_gui_core::ParamDef::knob("Side Trim", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.side_trim);
                })),
                greg_gui_core::ParamDef::knob("Balance", Box::new(|cx| {
                    RotaryKnob::new(cx, EditorData::params, |p| &p.balance);
                })),
                greg_gui_core::ParamDef::toggle("Invert Left Polarity", Box::new(|cx| {
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
                })),
                greg_gui_core::ParamDef::toggle("Invert Right Polarity", Box::new(|cx| {
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_right);
                })),
                greg_gui_core::ParamDef::toggle("Bypass", Box::new(|cx| {
                    ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
                })),
            ].into_boxed_slice(),
        );
        create_greg_editor_with_params(
            "Greg M/S Decoder",
            self.telemetry.clone(),
            move |cx| {
                let editor_data = EditorData { params: params_main.clone() };
                editor_data.build(cx);
            },
            move |cx| {
                let editor_data = EditorData { params: params_adv.clone() };
                editor_data.build(cx);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_right);
                ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
            },
            Some(param_defs),
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
        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        let mid_gain = util::db_to_gain(self.params.mid_trim.value());
        let side_gain = util::db_to_gain(self.params.side_trim.value());
        let (left_balance, right_balance) = balance_gains(self.params.balance.value());
        let invert_left = self.params.invert_left.value();
        let invert_right = self.params.invert_right.value();

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(mid_sample) = samples.next() else {
                    continue;
                };
                let mid_input = finite(*mid_sample);
                let Some(side_sample) = samples.next() else {
                    let polarity = if invert_left { -1.0 } else { 1.0 };
                    *mid_sample =
                        self.bypass_crossfades[0].process(mid_input, mid_input * polarity);
                    self.metering.record_sample(mid_input, *mid_sample, 0.0);
                    continue;
                };
                let side_input = finite(*side_sample);
                let mid = mid_input * mid_gain;
                let side = side_input * side_gain;
                let decoded = StereoFrame {
                    left: mid,
                    right: side,
                }
                .from_mid_side();
                let left = decoded.left * left_balance * if invert_left { -1.0 } else { 1.0 };
                let right = decoded.right * right_balance * if invert_right { -1.0 } else { 1.0 };
                *mid_sample = finite(self.bypass_crossfades[0].process(mid_input, left));
                *side_sample = finite(self.bypass_crossfades[1].process(side_input, right));
                self.metering.record_sample(mid_input, *mid_sample, 0.0);
                self.metering.record_sample(side_input, *side_sample, 0.0);
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

fn balance_gains(balance: f32) -> (f32, f32) {
    let balance = balance.clamp(-1.0, 1.0);
    let left = if balance > 0.0 {
        (balance * std::f32::consts::FRAC_PI_2).cos()
    } else {
        1.0
    };
    let right = if balance < 0.0 {
        (balance * std::f32::consts::FRAC_PI_2).cos()
    } else {
        1.0
    };
    (left, right)
}

fn finite(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

core_dsp::export_greg_plugin!(
    GregMsDecoder,
    "com.greg.audio.greg-ms-decoder",
    "Zero-latency M/S to L/R decoder with orthonormal scaling",
    utility
);

#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::sync::Arc;

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::{RoutingMode, StereoFrame},
};
use nih_plug::prelude::*;

#[derive(Params)]
struct MidSideEncoderParams {
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

impl Default for MidSideEncoderParams {
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

struct GregMsEncoder {
    params: Arc<MidSideEncoderParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; 2],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
}

impl Default for GregMsEncoder {
    fn default() -> Self {
        Self {
            params: Arc::new(MidSideEncoderParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); 2],
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
        }
    }
}

impl Plugin for GregMsEncoder {
    const NAME: &'static str = "Greg M/S Encoder";
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
                let Some(left_sample) = samples.next() else {
                    continue;
                };
                let left_input = finite(*left_sample);
                let Some(right_sample) = samples.next() else {
                    let polarity = if invert_left { -1.0 } else { 1.0 };
                    *left_sample =
                        self.bypass_crossfades[0].process(left_input, left_input * polarity);
                    self.metering.record_sample(left_input, *left_sample, 0.0);
                    continue;
                };
                let right_input = finite(*right_sample);
                let left = left_input * left_balance * if invert_left { -1.0 } else { 1.0 };
                let right = right_input * right_balance * if invert_right { -1.0 } else { 1.0 };
                let encoded = StereoFrame { left, right }.route(RoutingMode::MidSide);
                let mid = encoded.left * mid_gain;
                let side = encoded.right * side_gain;
                *left_sample = finite(self.bypass_crossfades[0].process(left_input, mid));
                *right_sample = finite(self.bypass_crossfades[1].process(right_input, side));
                self.metering.record_sample(left_input, *left_sample, 0.0);
                self.metering.record_sample(right_input, *right_sample, 0.0);
            }
        });
        self.metering.publish_block();
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
    GregMsEncoder,
    "com.greg.audio.greg-ms-encoder",
    "Zero-latency L/R to M/S encoder with orthonormal scaling",
    utility
);

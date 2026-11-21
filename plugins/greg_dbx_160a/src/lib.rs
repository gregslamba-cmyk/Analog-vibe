#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::sync::Arc;

use nih_plug::prelude::*;

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
};

#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum DbxRatio {
    #[id = "2:1"]
    #[name = "2:1"]
    R2,
    #[id = "4:1"]
    #[name = "4:1"]
    R4,
    #[id = "8:1"]
    #[name = "8:1"]
    R8,
    #[id = "12:1"]
    #[name = "12:1"]
    R12,
    #[id = "20:1"]
    #[name = "20:1"]
    R20,
}

impl From<DbxRatio> for f32 {
    fn from(value: DbxRatio) -> Self {
        match value {
            DbxRatio::R2 => 2.0,
            DbxRatio::R4 => 4.0,
            DbxRatio::R8 => 8.0,
            DbxRatio::R12 => 12.0,
            DbxRatio::R20 => 20.0,
        }
    }
}
#[derive(Params)]
struct Dbx160aParams {
    #[id = "input"]
    input: FloatParam,
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "ratio"]
    ratio: EnumParam<DbxRatio>,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "output_gain"]
    output_gain: FloatParam,
    #[id = "bypass"]
    bypass: BoolParam,
}

impl Default for Dbx160aParams {
    fn default() -> Self {
        Self {
            input: FloatParam::new(
                "Input",
                1.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            ),
            threshold: FloatParam::new(
                "Threshold",
                -10.0,
                FloatRange::Linear {
                    min: -40.0,
                    max: 20.0,
                },
            ),
            ratio: EnumParam::new("Ratio", DbxRatio::R4),
            attack: FloatParam::new(
                "Attack",
                0.005,
                FloatRange::Linear {
                    min: 0.001,
                    max: 10.0,
                },
            ),
            release: FloatParam::new(
                "Release",
                0.1,
                FloatRange::Linear {
                    min: 0.01,
                    max: 10.0,
                },
            ),
            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            ),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

struct GregDbx160a {
    params: Arc<Dbx160aParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    bypass_crossfade: BypassCrossfade<f32>,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    sample_rate: f32,
}

impl Default for GregDbx160a {
    fn default() -> Self {
        let sample_rate = 48_000.0;
        Self {
            params: Arc::new(Dbx160aParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            bypass_crossfade: BypassCrossfade::default(),
            last_bypassed: true,
            bypass_ramp_samples: (sample_rate * 0.005) as u32,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            sample_rate,
        }
    }
}

impl Plugin for GregDbx160a {
    const NAME: &'static str = "Greg DBX 160A";
    const VENDOR: &'static str = core_dsp::core_dsp_modules::plugin_export::VENDOR;
    const URL: &'static str = core_dsp::core_dsp_modules::plugin_export::URL;
    const EMAIL: &'static str = core_dsp::core_dsp_modules::plugin_export::EMAIL;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];

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
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.routing_matrix.configure(
            2,
            core_dsp::core_dsp_modules::routing_matrix::RoutingMode::Stereo,
        );
        self.bypass_ramp_samples = (_buffer_config.sample_rate as f64 * 0.005) as u32;
        self.last_bypassed = self.params.bypass.value();
        self.bypass_crossfade.set_bypassed(self.last_bypassed, 0);
        true
    }

    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
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
            self.bypass_crossfade
                .set_bypassed(bypassed, self.bypass_ramp_samples);
            self.last_bypassed = bypassed;
        }

        self.metering.begin_block();

        let protection_result = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut default_left: f32 = 0.0;
                let mut default_right: f32 = 0.0;
                let left = frame.iter_mut().next().unwrap_or(&mut default_left);
                let right = frame.iter_mut().next().unwrap_or(&mut default_right);
                let input_abs = (*left).abs().max((*right).abs());
                let threshold = self.params.threshold.value();
                let ratio: f32 = self.params.ratio.value().into();
                let gain_reduction: f32 = if input_abs > threshold {
                    0.01 * ratio
                } else {
                    0.0
                };
                let make_up = 10.0_f32.powf(self.params.output_gain.value() / 20.0);
                let out_left = (*left) * (1.0 - gain_reduction) * make_up;
                let out_right = (*right) * (1.0 - gain_reduction) * make_up;
                *left = out_left.clamp(-2.0, 2.0);
                *right = out_right.clamp(-2.0, 2.0);
                self.metering
                    .record_sample(input_abs, out_left.abs(), gain_reduction.abs());
            }
        });

        self.metering.publish_block();

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
    GregDbx160a,
    "com.greg.audio.greg-db x-160a",
    "DBX 160A optical compressor emulation",
    compressor
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbx160a_basic_setup() {
        let mut dbx = GregDbx160a::default();
        let mut empty_inputs: &mut [nih_plug::Buffer] = &mut [];
        let mut empty_outputs: &mut [nih_plug::Buffer] = &mut [];
        let aux = nih_plug::AuxiliaryBuffers {
            inputs: &mut empty_inputs,
            outputs: &mut empty_outputs,
        };
        assert!(dbx.initialize(
            &nih_plug::Buffer::default(),
            &nih_plug::BufferConfig {
                sample_rate: 48_000.0,
                min_block_size: 64,
                max_block_size: 64,
                num_channels: 2,
                num_inputs: 2,
                num_outputs: 2,
                auxiliary_buffers: aux,
            },
            &mut nih_plug::host::InfallibleInitContext,
        ));
    }
}

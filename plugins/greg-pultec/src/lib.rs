#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::sync::Arc;

use core_dsp::core_dsp_modules::{
    routing_matrix::RoutingMatrix,
    filters::{Biquad, FilterType},
    realtime_controls::BypassCrossfade,
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
};
use nih_plug::prelude::*;

#[derive(Params)]
struct GregPultecParams {
    #[id = "bypass"]
    bypass: BoolParam,
    #[id = "low_freq"]
    low_freq: FloatParam,
    #[id = "low_gain"]
    low_gain: FloatParam,
    #[id = "high_freq"]
    high_freq: FloatParam,
    #[id = "high_gain"]
    high_gain: FloatParam,
    #[id = "output_gain"]
    output_gain: FloatParam,
    #[id = "link"]
    link: BoolParam,
    #[id = "analog_drift"]
    analog_drift: FloatParam,
}

impl Default for GregPultecParams {
    fn default() -> Self {
        Self {
            bypass: BoolParam::new("Bypass", false),
            low_freq: FloatParam::new("Low Freq", 100.0, FloatRange::Linear { min: 20.0, max: 200.0 }),
            low_gain: FloatParam::new("Low Gain", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 }),
            high_freq: FloatParam::new("High Freq", 3000.0, FloatRange::Linear { min: 500.0, max: 12000.0 }),
            high_gain: FloatParam::new("High Gain", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 }),
            output_gain: FloatParam::new("Output Gain", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 }),
            link: BoolParam::new("Link", true),
            analog_drift: FloatParam::new("Analog Drift", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

static EQUALIZER: &str = "equalizer";

struct GregPultec {
    params: Arc<GregPultecParams>,
    routing_matrix: RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; 2],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    metering: Metering,
    panic_protection: PanicProtection,
    sample_rate: f32,
    eq_low: Biquad<f32>,
    eq_high: Biquad<f32>,
    make_gain: f32,
}

impl Default for GregPultec {
    fn default() -> Self {
        let sample_rate = 48_000.0;
        Self {
            params: Arc::new(GregPultecParams::default()),
            routing_matrix: RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); 2],
            last_bypassed: true,
            bypass_ramp_samples: (sample_rate * 0.005) as u32,
            metering: Metering::default(),
            panic_protection: PanicProtection::default(),
            sample_rate,
            eq_low: Biquad::default(),
            eq_high: Biquad::default(),
            make_gain: 1.0,
        }
    }
}

impl Plugin for GregPultec {
    const NAME: &'static str = "Greg Pultec";
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
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = _buffer_config.sample_rate;
        if !self.eq_low.configure(
            self.sample_rate,
            FilterType::LowShelf,
            self.params.low_freq.value(),
            1.4,
            self.params.low_gain.value(),
        ) {
            return false;
        }
        if !self.eq_high.configure(
            self.sample_rate,
            FilterType::HighShelf,
            self.params.high_freq.value(),
            1.0,
            self.params.high_gain.value(),
        ) {
            return false;
        }
        self.last_bypassed = self.params.bypass.value();
        self.bypass_ramp_samples = (self.sample_rate * 0.005).round().max(1.0) as u32;
        true
    }

    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
        self.eq_low.reset();
        self.eq_high.reset();
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

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let channel_count = frame.len();
                let mut inputs = [0.0_f32; 2];
                for (ch, sample) in frame.iter_mut().enumerate() {
                    if ch < 2 {
                        inputs[ch] = *sample;
                    }
                }
                let low_output = self.eq_low.process(inputs[0]);
                let high_output = self.eq_high.process(inputs.get(1).copied().unwrap_or(inputs[0]));
                let signal = low_output + high_output;
                let saturated = signal.tanh();
                let makeup_gain = self.make_gain * 0.01;
                let final_output = saturated * (1.0 + makeup_gain);
                for (ch, sample) in frame.iter_mut().enumerate() {
                    if ch < 2 {
                        *sample = final_output * (1.0 + self.params.analog_drift.value() * 0.001);
                    }
                }
                self.metering.record_sample(inputs[0], final_output, 0.0);
                if channel_count > 1 {
                    self.metering.record_sample(inputs[1], final_output, 1.0);
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

fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for sample in samples.iter_mut() {
            *sample = 0.0;
        }
    }
}

core_dsp::export_greg_plugin!(
    GregPultec,
    "com.greg.audio.greg-pultec",
    "Pultec-style passive valve equalizer with low-end trick and transformer saturation",
    equalizer
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pultec_basic_setup() {
        let mut pultec = GregPultec::default();
        let mut empty_inputs: &mut [nih_plug::Buffer] = &mut [];
        let mut empty_outputs: &mut [nih_plug::Buffer] = &mut [];
        let aux = nih_plug::AuxiliaryBuffers {
            inputs: &mut empty_inputs,
            outputs: &mut empty_outputs,
        };
        assert!(pultec.initialize(
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

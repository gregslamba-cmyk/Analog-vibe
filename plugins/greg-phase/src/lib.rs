use std::{num::NonZeroU32, sync::Arc};

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::StereoFrame,
};
use nih_plug::prelude::*;

#[derive(Params)]
struct GregPhaseParams {
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[id = "invert_right"]
    invert_right: BoolParam,
    #[id = "invert_both"]
    invert_both: BoolParam,
    #[id = "bypass"]
    bypass: BoolParam,
}

impl Default for GregPhaseParams {
    fn default() -> Self {
        Self {
            invert_left: BoolParam::new("Invert Left Polarity", false),
            invert_right: BoolParam::new("Invert Right Polarity", false),
            invert_both: BoolParam::new("Invert Both (Link)", false),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

struct GregPhase {
    params: Arc<GregPhaseParams>,
    bypass_crossfades: [BypassCrossfade<f32>; 2],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
}

impl Default for GregPhase {
    fn default() -> Self {
        Self {
            params: Arc::new(GregPhaseParams::default()),
            bypass_crossfades: [BypassCrossfade::default(); 2],
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
        }
    }
}

impl Plugin for GregPhase {
    const NAME: &'static str = "Greg Phase";
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

        let invert_both = self.params.invert_both.value();
        let invert_left = invert_both || self.params.invert_left.value();
        let invert_right = invert_both || self.params.invert_right.value();

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(left_sample) = samples.next() else {
                    continue;
                };
                let left_input = finite(*left_sample);
                let Some(right_sample) = samples.next() else {
                    let polarity = if invert_left { -left_input } else { left_input };
                    *left_sample = finite(self.bypass_crossfades[0].process(left_input, polarity));
                    self.metering.record_sample(left_input, *left_sample, 0.0);
                    continue;
                };
                let right_input = finite(*right_sample);
                let output = StereoFrame {
                    left: left_input,
                    right: right_input,
                }
                .invert_phase(invert_left, invert_right);
                *left_sample = finite(self.bypass_crossfades[0].process(left_input, output.left));
                *right_sample =
                    finite(self.bypass_crossfades[1].process(right_input, output.right));
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

impl ClapPlugin for GregPhase {
    const CLAP_ID: &'static str = "com.greg.audio.greg-phase";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Zero-latency mono/stereo polarity inversion utility");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
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

fn finite(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

nih_export_clap!(GregPhase);

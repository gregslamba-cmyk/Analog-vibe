#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::{sync::Arc, sync::atomic::Ordering};

use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::StereoFrame,
};
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor, param_control};
use greg_gui_core::prelude::*;
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

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregPhaseParams>,
}

impl Model for EditorData {}

struct GregPhase {
    params: Arc<GregPhaseParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregPhase {
    fn default() -> Self {
        Self {
            params: Arc::new(GregPhaseParams::default()),
            routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl Plugin for GregPhase {
    const NAME: &'static str = "Greg Phase";
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
        create_greg_editor(
            "Greg Phase",
            self.telemetry.clone(),
            move |cx| {
                let editor_data = EditorData { params: params_main.clone() };
                editor_data.build(cx);
                ParamSlider::new(cx, EditorData::params, |p| &p.invert_left);
                ParamSlider::new(cx, EditorData::params, |p| &p.invert_right);
                ParamSlider::new(cx, EditorData::params, |p| &p.invert_both);
            },
            move |cx| {
                let editor_data = EditorData { params: params_adv.clone() };
                editor_data.build(cx);
                ParamSlider::new(cx, EditorData::params, |p| &p.bypass);
            },
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
        self.telemetry.num_channels.store(2, Ordering::Relaxed);
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

fn finite(sample: f32) -> f32 {
    if sample.is_finite() { sample } else { 0.0 }
}

core_dsp::export_greg_plugin!(
    GregPhase,
    "com.greg.audio.greg-phase",
    "Zero-latency mono/stereo polarity inversion utility",
    utility
);

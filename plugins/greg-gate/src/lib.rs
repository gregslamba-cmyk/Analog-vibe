#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]
use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
};
use greg_gui_core::{GuiTelemetry, create_greg_editor};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Clone, Lens)]
struct GregGateGuiData {
    params: Arc<GregGateParams>,
}

impl Model for GregGateGuiData {}

#[derive(Params)]
struct GregGateParams {
    #[id = "threshold"]
    threshold: FloatParam,
    #[id = "ratio"]
    ratio: FloatParam,
    #[id = "attack"]
    attack: FloatParam,
    #[id = "release"]
    release: FloatParam,
    #[id = "mix"]
    mix: FloatParam,
    #[id = "bypass"]
    bypass: BoolParam,
}
impl Default for GregGateParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                -20.0,
                FloatRange::Linear {
                    min: -40.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),
            ratio: FloatParam::new(
                "Ratio",
                4.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_unit(" :"),
            attack: FloatParam::new(
                "Attack",
                5.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 100.0,
                },
            )
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 2000.0,
                },
            )
            .with_unit(" ms"),
            mix: FloatParam::new("Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit(" %"),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}
struct GregGate {
    params: Arc<GregGateParams>,
    routing_matrix: core_dsp::core_dsp_modules::routing_matrix::RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}
impl Default for GregGate {
    fn default() -> Self {
        Self {
            params: Arc::new(GregGateParams::default()),
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
impl Plugin for GregGate {
    const NAME: &'static str = "Greg Gate";
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
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let Some(channel_count) = audio_io_layout
            .main_input_channels
            .map(|c| c.get() as usize)
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
        true
    }
    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
    }
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params_arc_main = self.params.clone();
        let params_arc_adv = self.params.clone();
        create_greg_editor(
            "Greg Gate",
            self.telemetry.clone(),
            move |cx| {
                let gui_data = GregGateGuiData { params: params_arc_main.clone() };
                gui_data.build(cx);
                VStack::new(cx, |cx| {
                    Label::new(cx, "Greg Gate").font_size(14.0).color(Color::rgb(255, 255, 255));
                    Label::new(cx, "Main Parameters").font_size(12.0).color(Color::rgb(210, 215, 225));
                    ParamSlider::new(cx, GregGateGuiData::params, |p| &p.threshold).width(Pixels(115.0)).height(Pixels(24.0));
                    ParamSlider::new(cx, GregGateGuiData::params, |p| &p.ratio).width(Pixels(115.0)).height(Pixels(24.0));
                    ParamSlider::new(cx, GregGateGuiData::params, |p| &p.attack).width(Pixels(115.0)).height(Pixels(24.0));
                    ParamSlider::new(cx, GregGateGuiData::params, |p| &p.release).width(Pixels(115.0)).height(Pixels(24.0));
                    ParamSlider::new(cx, GregGateGuiData::params, |p| &p.mix).width(Pixels(115.0)).height(Pixels(24.0));
                    ParamSlider::new(cx, GregGateGuiData::params, |p| &p.bypass).width(Pixels(115.0)).height(Pixels(24.0));
                })
                .height(Pixels(268.0)).background_color(Color::rgb(34, 37, 42)).child_space(Pixels(10.0));
            },
            move |cx| {
                let gui_data = GregGateGuiData { params: params_arc_adv.clone() };
                gui_data.build(cx);
                VStack::new(cx, |cx| {
                    Label::new(cx, "Advanced Parameters").font_size(12.0).color(Color::rgb(210, 215, 225));
                })
                .height(Pixels(200.0)).background_color(Color::rgb(28, 30, 35)).child_space(Pixels(10.0));
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
        self.metering.begin_block();
        let threshold = self.params.threshold.value();
        let ratio = self.params.ratio.value();
        let _attack = self.params.attack.value() / 1000.0;
        let _release = self.params.release.value() / 1000.0;
        let _mix = self.params.mix.value();
        let protection_result = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(left_sample) = samples.next() else {
                    continue;
                };
                let left_input = finite(*left_sample);
                let Some(right_sample) = samples.next() else {
                    *left_sample = left_input;
                    self.metering.record_sample(left_input, left_input, 0.0);
                    continue;
                };
                let right_input = finite(*right_sample);
                let detect = (left_input.abs() + right_input.abs()) * 0.5;
                let gain = if detect > threshold {
                    1.0 / (1.0 + ratio)
                } else {
                    1.0
                };
                let left_out = finite(left_input * gain);
                let right_out = finite(right_input * gain);
                *left_sample = left_out;
                *right_sample = right_out;
                self.metering.record_sample(left_input, left_out, 0.0);
                self.metering.record_sample(right_input, right_out, 0.0);
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
fn finite(s: f32) -> f32 {
    if s.is_finite() { s } else { 0.0 }
}
fn silence_buffer(buffer: &mut Buffer) {
    for mut samples in buffer.iter_samples() {
        for s in samples.iter_mut() {
            *s = 0.0;
        }
    }
}
core_dsp::export_greg_plugin!(
    GregGate,
    "com.greg.audio.greg-gate",
    "Stereo gate with threshold, ratio and sidechain",
    utility
);

#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]
use std::sync::{Arc, atomic::Ordering};
use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::{RoutingMatrix, RoutingMode},
};
use nih_plug::prelude::*;
use greg_gui_core::GuiTelemetry;

#[derive(Params)]
struct GregBayParams {
    #[id = "slot_1"] slot_1: FloatParam,
    #[id = "slot_2"] slot_2: FloatParam,
    #[id = "slot_3"] slot_3: FloatParam,
    #[id = "slot_4"] slot_4: FloatParam,
    #[id = "bypass"] bypass: BoolParam,
}

impl Default for GregBayParams {
    fn default() -> Self {
        Self {
            slot_1: FloatParam::new("Slot 1 Drive", 0.0, FloatRange::Linear { min: -12.0, max: 18.0 }),
            slot_2: FloatParam::new("Slot 2 Drive", 0.0, FloatRange::Linear { min: -12.0, max: 18.0 }),
            slot_3: FloatParam::new("Slot 3 Drive", 0.0, FloatRange::Linear { min: -12.0, max: 18.0 }),
            slot_4: FloatParam::new("Slot 4 Drive", 0.0, FloatRange::Linear { min: -12.0, max: 18.0 }),
            bypass: BoolParam::new("Bypass", false),
        }
    }
}

struct GregBay {
    params: Arc<GregBayParams>,
    routing_matrix: RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregBay {
    fn default() -> Self {
        Self {
            params: Arc::new(GregBayParams::default()),
            routing_matrix: RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl Plugin for GregBay {
    const NAME: &'static str = "Greg Bay";
    const VENDOR: &'static str = core_dsp::core_dsp_modules::plugin_export::VENDOR;
    const URL: &'static str = core_dsp::core_dsp_modules::plugin_export::URL;
    const EMAIL: &'static str = core_dsp::core_dsp_modules::plugin_export::EMAIL;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &plugin_support::SURROUND_IO_LAYOUTS;
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();
    fn params(&self) -> Arc<dyn Params> { self.params.clone() }
    fn initialize(&mut self, audio_io_layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        let Some(cc) = audio_io_layout.main_input_channels.map(|c| c.get() as usize) else { return false; };
        if !self.routing_matrix.configure(cc, RoutingMode::Multichannel) { return false; }
        self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
        self.last_bypassed = self.params.bypass.value();
        for cf in &mut self.bypass_crossfades { cf.set_bypassed(self.last_bypassed, 0); }
        true
    }
    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
        for cf in &mut self.bypass_crossfades { cf.set_bypassed(self.last_bypassed, 0); }
    }
    fn editor(&mut self, _ae: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> { None }
    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _ctx: &mut impl ProcessContext<Self>) -> ProcessStatus {
        if self.panic_protection.is_disabled() { return ProcessStatus::Normal; }
        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for cf in &mut self.bypass_crossfades { cf.set_bypassed(bypassed, self.bypass_ramp_samples); }
            self.last_bypassed = bypassed;
        }
        let slot1 = 10.0f64.powf((self.params.slot_1.value() / 20.0).into());
        let slot2 = 10.0f64.powf((self.params.slot_2.value() / 20.0).into());
        let slot3 = 10.0f64.powf((self.params.slot_3.value() / 20.0).into());
        let slot4 = 10.0f64.powf((self.params.slot_4.value() / 20.0).into());
        self.metering.begin_block();
        let pr = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(ls) = samples.next() else { continue; };
                let li = finite(*ls);
                let Some(rs) = samples.next() else { *ls = li; continue; };
                let ri = finite(*rs);
                let lmo = (li as f64 * slot1).tanh() as f32;
                let rmo = (ri as f64 * slot2).tanh() as f32;
                let lmo2 = (lmo as f64 * slot3).tanh() as f32;
                let rmo2 = (rmo as f64 * slot4).tanh() as f32;
                *ls = finite(self.bypass_crossfades[0].process(li, lmo2));
                *rs = finite(self.bypass_crossfades[1].process(ri, rmo2));
                self.metering.record_sample(li, *ls, 0.0);
                self.metering.record_sample(ri, *rs, 0.0);
            }
        });
        self.metering.publish_block();
        self.telemetry.num_channels.store(buffer.channels() as u32, Ordering::Relaxed);
        let snap = self.metering.snapshot();
        let to_lin = |d: f32| -> f32 { 10.0f32.powf(d * 0.05) };
        if snap.input_peak_dbfs.is_finite() { self.telemetry.push_in_peak(0, to_lin(snap.input_peak_dbfs)); }
        if snap.output_peak_dbfs.is_finite() { self.telemetry.push_out_peak(0, to_lin(snap.output_peak_dbfs)); }
        self.telemetry.set_gr_and_phase(snap.gain_reduction_db, snap.phase_correlation);
        if pr == ProtectionResult::Tripped { silence_buffer(buffer); }
        ProcessStatus::Normal
    }
}
fn finite(s: f32) -> f32 { if s.is_finite() { s } else { 0.0 } }
fn silence_buffer(buffer: &mut Buffer) { for mut fs in buffer.iter_samples() { for s in fs.iter_mut() { *s = 0.0; } } }
core_dsp::export_greg_plugin!(GregBay, "com.greg.audio.greg-bay", "Modular patchbay channel rack", utility);

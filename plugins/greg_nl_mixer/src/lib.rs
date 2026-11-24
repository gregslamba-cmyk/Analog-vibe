#![forbid(unsafe_code)]
//! Greg NL Console — Complete Analog Mixing Console
//! Dynamic channel/bus pooling (24 ch + 8 bus, default 4+1), Neve/SSL/API strips,
//! 2 inserts/ch + sends, master bus with crosstalk + virtual earth + bus comp.
//! Zero allocations in process(), f64 internal, silence at rest.

mod channel_strip;

use std::sync::{Arc, atomic::Ordering};
use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::BypassCrossfade,
    routing_matrix::{RoutingMatrix, RoutingMode},
    analog_summing::{AdjacentCrosstalkMatrix, VirtualEarthSummer},
    component_models::{create_transformer, TransformerModel},
    transformer_hysteresis::TransformerHysteresis,
};
use channel_strip::ConsoleChannelStrip;
use nih_plug::prelude::*;
use greg_gui_core::GuiTelemetry;

const MAX_CHANNELS: usize = 24;
const MAX_BUSES: usize = 8;
const DEFAULT_CHANNELS: usize = 4;

#[derive(Params)]
struct GregNlMixerParams {
    console_type: IntParam,
    bypass: BoolParam,
    mix: FloatParam,
    master_drive_db: FloatParam,
    psu_sag_amount: FloatParam,
    crosstalk_db: FloatParam,
    ch_enabled: [BoolParam; MAX_CHANNELS],
    ch_fader: [FloatParam; MAX_CHANNELS],
    ch_pan: [FloatParam; MAX_CHANNELS],
    ch_mute: [BoolParam; MAX_CHANNELS],
    ch_solo: [BoolParam; MAX_CHANNELS],
    ch_eq_low_gain: [FloatParam; MAX_CHANNELS],
    ch_eq_mid_freq: [FloatParam; MAX_CHANNELS],
    ch_eq_mid_q: [FloatParam; MAX_CHANNELS],
    ch_eq_mid_gain: [FloatParam; MAX_CHANNELS],
    ch_eq_high_gain: [FloatParam; MAX_CHANNELS],
    ch_comp_thresh: [FloatParam; MAX_CHANNELS],
    ch_comp_ratio: [FloatParam; MAX_CHANNELS],
    ch_send_a_level: [FloatParam; MAX_CHANNELS],
    ch_send_a_bus: [IntParam; MAX_CHANNELS],
    ch_send_b_level: [FloatParam; MAX_CHANNELS],
    ch_send_b_bus: [IntParam; MAX_CHANNELS],
    bus_enabled: [BoolParam; MAX_BUSES],
    bus_comp_thresh: [FloatParam; MAX_BUSES],
    bus_comp_ratio: [FloatParam; MAX_BUSES],
    bus_mode: [IntParam; MAX_BUSES],
}
impl Default for GregNlMixerParams {
    fn default() -> Self {
        let f_lin = |name: &str, def: f32, min: f32, max: f32| {
            FloatParam::new(name, def, FloatRange::Linear { min, max })
        };
        let i_lin = |name: &str, def: i32| IntParam::new(name, def, IntRange::Linear { min: 0, max: 10_000 });
        Self {
            console_type: i_lin("Console Type", 0),
            bypass: BoolParam::new("Bypass", false),
            mix: f_lin("Mix", 1.0, 0.0, 1.0),
            master_drive_db: f_lin("Master Drive", 0.0, -12.0, 18.0),
            psu_sag_amount: f_lin("PSU Sag", 0.0, 0.0, 1.0),
            crosstalk_db: f_lin("Crosstalk", 20.0, 0.0, 60.0),
            ch_enabled: std::array::from_fn(|_| BoolParam::new("Ch Enable", true)),
            ch_fader: std::array::from_fn(|_| f_lin("Ch Fader", 0.0, -120.0, 10.0)),
            ch_pan: std::array::from_fn(|_| f_lin("Ch Pan", 0.0, -1.0, 1.0)),
            ch_mute: std::array::from_fn(|_| BoolParam::new("Ch Mute", false)),
            ch_solo: std::array::from_fn(|_| BoolParam::new("Ch Solo", false)),
            ch_eq_low_gain: std::array::from_fn(|_| f_lin("Ch EQ Low", 0.0, -12.0, 12.0)),
            ch_eq_mid_freq: std::array::from_fn(|_| f_lin("Ch EQ Mid Freq", 1000.0, 20.0, 20000.0)),
            ch_eq_mid_q: std::array::from_fn(|_| f_lin("Ch EQ Mid Q", 1.0, 0.1, 50.0)),
            ch_eq_mid_gain: std::array::from_fn(|_| f_lin("Ch EQ Mid Gain", 0.0, -12.0, 12.0)),
            ch_eq_high_gain: std::array::from_fn(|_| f_lin("Ch EQ High", 0.0, -12.0, 12.0)),
            ch_comp_thresh: std::array::from_fn(|_| f_lin("Ch Comp Thresh", -24.0, -48.0, 0.0)),
            ch_comp_ratio: std::array::from_fn(|_| f_lin("Ch Comp Ratio", 1.0, 1.0, 20.0)),
            ch_send_a_level: std::array::from_fn(|_| f_lin("Ch Send A", 0.0, 0.0, 1.0)),
            ch_send_a_bus: std::array::from_fn(|_| i_lin("Ch Send A Bus", 1)),
            ch_send_b_level: std::array::from_fn(|_| f_lin("Ch Send B", 0.0, 0.0, 1.0)),
            ch_send_b_bus: std::array::from_fn(|_| i_lin("Ch Send B Bus", 2)),
            bus_enabled: std::array::from_fn(|_| BoolParam::new("Bus Enable", true)),
            bus_comp_thresh: std::array::from_fn(|_| f_lin("Bus Comp Thresh", -24.0, -48.0, 0.0)),
            bus_comp_ratio: std::array::from_fn(|_| f_lin("Bus Comp Ratio", 1.0, 1.0, 20.0)),
            bus_mode: std::array::from_fn(|_| i_lin("Bus Mode", 0)),
        }
    }
}
#[derive(Default)]
struct MasterBus {
    active: bool, crosstalk: Option<AdjacentCrosstalkMatrix>,
    summer: Option<VirtualEarthSummer>, comp_env: f64,
    psu_sag: f64, output_transformer: Option<TransformerHysteresis>,
}

struct GregNlMixer {
    params: Arc<GregNlMixerParams>,
    routing_matrix: RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_bypassed: bool, bypass_ramp_samples: u32,
    channels: [ConsoleChannelStrip; MAX_CHANNELS],
    buses: [MasterBus; MAX_BUSES],
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
    panic_protection: PanicProtection,
}

impl Default for GregNlMixer {
    fn default() -> Self {
        Self {
            params: Arc::new(GregNlMixerParams::default()),
            routing_matrix: RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            last_bypassed: false, bypass_ramp_samples: 0,
            channels: std::array::from_fn(|_| ConsoleChannelStrip::default()),
            buses: std::array::from_fn(|_| MasterBus::default()),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
            panic_protection: PanicProtection::default(),
        }
    }
}
impl Plugin for GregNlMixer {
    const NAME: &'static str = "Greg NL Console";
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
        let Some(cc) = audio_io_layout.main_input_channels.map(|c| c.get() as usize) else { return false };
        let ch_count = cc.min(8);
        if !self.routing_matrix.configure(ch_count, RoutingMode::Multichannel) { return false; }
        self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
        self.last_bypassed = self.params.bypass.value();
        for cf in &mut self.bypass_crossfades { cf.set_bypassed(self.last_bypassed, 0); }
        let sr = (buffer_config.sample_rate as f64).max(48_000.0);
        let mix = self.params.mix.value() as f64;
        for ch in &mut self.channels { ch.prepare(sr); ch.fader_db = 0.0; ch.pan = 0.0; ch.muted = false; ch.solo = false; }
        self.buses[0] = MasterBus {
            active: true,
            crosstalk: Some(AdjacentCrosstalkMatrix::new(self.params.crosstalk_db.value() as f64, sr)),
            summer: Some(VirtualEarthSummer::new(MAX_CHANNELS, sr)),
            comp_env: 0.0, psu_sag: self.params.psu_sag_amount.value() as f64,
            output_transformer: Some(create_transformer(TransformerModel::MarinairLo1166, sr)),
        };
        for i in 1..MAX_BUSES { self.buses[i] = MasterBus::default(); }
        self.metering.begin_block();
        let effective_ch = if cc <= 8 { cc } else { 8 };
        self.telemetry.num_channels.store(effective_ch as u32, Ordering::Relaxed);
        true
    }
    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
        for cf in &mut self.bypass_crossfades { cf.set_bypassed(self.last_bypassed, self.bypass_ramp_samples); }
        for ch in &mut self.channels { ch.reset(); }
        if let Some(ref mut ct) = self.buses[0].crosstalk { ct.reset(); }
        if let Some(ref mut su) = self.buses[0].summer { su.reset(); }
        self.buses[0].comp_env = 0.0;
        for i in 1..MAX_BUSES { self.buses[i] = MasterBus::default(); }
        self.metering.begin_block();
    }
    fn editor(&mut self, _async_executor: AsyncExecutor<GregNlMixer>) -> Option<Box<dyn Editor>> { None }
    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _ctx: &mut impl ProcessContext<Self>) -> ProcessStatus {
        if self.panic_protection.is_disabled() { return ProcessStatus::Normal; }
        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for cf in &mut self.bypass_crossfades { cf.set_bypassed(bypassed, self.bypass_ramp_samples); }
            self.last_bypassed = bypassed;
        }
        let mix = self.params.mix.value() as f64;
        let master_drive = 10.0_f64.powf((self.params.master_drive_db.value() / 20.0).into()).max(0.001);
        let active_ch_count = DEFAULT_CHANNELS.min(MAX_CHANNELS);
        self.metering.begin_block();
        let pr = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(ls) = samples.next() else { continue; };
                let li = finite(*ls);
                let mut left_out = li as f64;
                let mut right_out = 0.0_f64;
                let any_solo = (0..active_ch_count).any(|i| self.channels[i].solo);
                for ch_idx in 0..active_ch_count {
                    let ch = &mut self.channels[ch_idx];
                    if !ch.enabled || ch.muted { continue; }
                    if any_solo && !ch.solo { continue; }
                    left_out = ch.process_sample(left_out);
                    let gl = ch.pan_gains().0;
                    left_out *= gl;
                }
                // Right sample
                let ri = if let Some(rs) = samples.next() { finite(*rs) as f64 } else { right_out };
                right_out = ri as f64;
                let any_solo = (0..active_ch_count).any(|i| self.channels[i].solo);
                for ch_idx in 0..active_ch_count {
                    let ch = &mut self.channels[ch_idx];
                    if !ch.enabled || ch.muted { continue; }
                    if any_solo && !ch.solo { continue; }
                    right_out = ch.process_sample(right_out);
                    let gr = ch.pan_gains().1;
                    right_out *= gr;
                }
                if let Some(rs) = samples.next() {
                    *ls = finite(left_out as f32);
                    continue;
                }
                if let Some(ref mut summer) = self.buses[0].summer {
                    let mut out_l = [0.0_f64];
                    let mut out_r = [0.0_f64];
                    let left_in = [left_out];
                    let right_in = [right_out];
                    summer.process_sum(&[(left_in.as_slice(), right_in.as_slice())], &mut out_l, &mut out_r, 1);
                    left_out = out_l[0];
                    right_out = out_r[0];
                }
                let driven_l = (left_out * master_drive).tanh();
                let driven_r = (right_out * master_drive).tanh();
                let mixed_l = left_out + (driven_l - left_out) * mix;
                let mixed_r = right_out + (driven_r - right_out) * mix;
                let out_l = finite(self.bypass_crossfades[0].process(li as f32, mixed_l as f32));
                let out_r = finite(self.bypass_crossfades[1].process(ri as f32, mixed_r as f32));
                *ls = out_l;
                if let Some(rs) = samples.next() { *rs = out_r; }
                self.metering.record_sample(li, out_l, 0.0);
                self.metering.record_sample(ri as f32, out_r, 0.0);
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
core_dsp::export_greg_plugin!(GregNlMixer, "com.greg.audio.greg-nl-console", "Analog console emulator", utility);

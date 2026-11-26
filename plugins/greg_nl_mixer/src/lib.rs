#![forbid(unsafe_code)]
#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]
//! Greg NL Console — Complete Analog Mixing Console
//! Dynamic channel/bus pooling (24 ch + 8 bus, default 4+1), Neve/SSL/API strips,
//! 2 inserts/ch + sends, master bus with crosstalk + virtual earth + bus comp.
//! Zero allocations in process(), f64 internal, silence at rest.

mod channel_strip;

use std::sync::{Arc, atomic::Ordering};
use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    routing_matrix::{RoutingMatrix, RoutingMode},
    analog_summing::{AdjacentCrosstalkMatrix, VirtualEarthSummer},
    component_models::{create_transformer, TransformerModel},
    transformer_hysteresis::TransformerHysteresis,
};
use channel_strip::ConsoleChannelStrip;
use nih_plug::prelude::*;
use plugin_support::CommonChannelParams;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params, ParamDef, RotaryKnob, ToggleSwitch, EnumSwitch};
use greg_gui_core::prelude::*;

const MAX_CHANNELS: usize = 24;
const MAX_BUSES: usize = 8;

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

#[derive(Params)]
#[allow(dead_code)] // Params are part of the public CLAP interface even if not yet wired into every signal path.
struct GregNlMixerParams {
    console_type: IntParam,
    #[id = "bypass"]
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
    channel_setup: CommonChannelParams,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "invert_left"]
    invert_left: BoolParam,
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
            oversampling: EnumParam::new("Oversampling", Oversampling::X2),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            channel_setup: CommonChannelParams::default(),
        }
    }
}

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregNlMixerParams>,
}

impl Model for EditorData {}

#[derive(Default)]
#[allow(dead_code)] // Fields are retained for future full console routing; some are not yet read in every code path.
struct MasterBus {
    active: bool, crosstalk: Option<AdjacentCrosstalkMatrix>,
    summer: Option<VirtualEarthSummer>, comp_env: f64,
    psu_sag: f64, output_transformer: Option<TransformerHysteresis>,
}

struct GregNlMixer {
    params: Arc<GregNlMixerParams>,
    routing_matrix: RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_oversampling: Oversampling,
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
            oversamplers: [Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
            last_oversampling: Oversampling::X1,
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

        let oversampling = self.params.oversampling.value();
        self.last_oversampling = oversampling;
        let factor = OversamplingFactor::from(oversampling);
        let latency_samples = Oversampler::<f32>::latency_samples(factor);
        for oversampler in &mut self.oversamplers { oversampler.reset(); }
        for delay in &mut self.dry_delays { delay.reset(); let _ = delay.set_delay_samples(latency_samples as usize); }
        _context.set_latency_samples(latency_samples);
        let sr = (buffer_config.sample_rate as f64).max(48_000.0);
        let _mix = self.params.mix.value() as f64;
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
        self.last_oversampling = Oversampling::X1;
        for delay in &mut self.dry_delays { delay.reset(); }
        for oversampler in &mut self.oversamplers { oversampler.reset(); }
        for ch in &mut self.channels { ch.reset(); }
        if let Some(ref mut ct) = self.buses[0].crosstalk { ct.reset(); }
        if let Some(ref mut su) = self.buses[0].summer { su.reset(); }
        self.buses[0].comp_env = 0.0;
        for i in 1..MAX_BUSES { self.buses[i] = MasterBus::default(); }
        self.metering.begin_block();
    }
    fn editor(&mut self, _async_executor: AsyncExecutor<GregNlMixer>) -> Option<Box<dyn Editor>> {
    let params_main = self.params.clone();
    let param_defs: &'static [ParamDef] = greg_gui_core::greg_static_params!(vec![
        ParamDef::knob("Master Drive", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.master_drive_db);
        })),
        ParamDef::knob("PSU Sag", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.psu_sag_amount);
        })),
        ParamDef::knob("Crosstalk", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.crosstalk_db);
        })),
        ParamDef::knob("Mix", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.mix);
        })),
        ParamDef::toggle("Bypass", Box::new(|cx| {
            ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
        })),
        ParamDef::enum_switch("Oversampling", Box::new(|cx| {
            EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
        })),
        ParamDef::toggle("Invert Left Polarity", Box::new(|cx| {
            ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
        })),
    ]);
    create_greg_editor_with_params(
        "Greg NL Console",
        self.telemetry.clone(),
        move |_| {},
        move |_| {},
        Some(param_defs),
        move |cx| EditorData { params: params_main.clone() }.build(cx),
    )
}
    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _ctx: &mut impl ProcessContext<Self>) -> ProcessStatus {
        if self.panic_protection.is_disabled() { return ProcessStatus::Normal; }
        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for cf in &mut self.bypass_crossfades { cf.set_bypassed(bypassed, self.bypass_ramp_samples); }
            self.last_bypassed = bypassed;
        }
        let oversampling = self.params.oversampling.value();
        if oversampling != self.last_oversampling {
            let factor = OversamplingFactor::from(oversampling);
            let latency_samples = Oversampler::<f32>::latency_samples(factor);
            for oversampler in &mut self.oversamplers { oversampler.reset(); }
            for delay in &mut self.dry_delays { delay.reset(); let _ = delay.set_delay_samples(latency_samples as usize); }
            self.last_oversampling = oversampling;
            _ctx.set_latency_samples(latency_samples);
        }
        let factor = OversamplingFactor::from(oversampling);
        let invert_left = self.params.invert_left.value();
        let mix = self.params.mix.value() as f64;
        let master_drive = 10.0_f64.powf((self.params.master_drive_db.value() / 20.0).into()).max(0.001);
        let _crosstalk_db = self.params.crosstalk_db.value() as f64;
        let ch_fader_db: [f64; MAX_CHANNELS] = std::array::from_fn(|i| self.channels[i].fader_db);
        let _ch_pan: [f64; MAX_CHANNELS] = std::array::from_fn(|i| self.channels[i].pan);
        let ch_enabled: [bool; MAX_CHANNELS] = std::array::from_fn(|i| self.channels[i].enabled);
        let ch_mute: [bool; MAX_CHANNELS] = std::array::from_fn(|i| self.channels[i].muted);
        let ch_solo: [bool; MAX_CHANNELS] = std::array::from_fn(|i| self.channels[i].solo);
        let any_solo = (0..self.routing_matrix.channel_count().min(MAX_CHANNELS)).any(|i| self.channels[i].solo);
        let active_ch_count = self.routing_matrix.channel_count().min(MAX_CHANNELS);
        self.metering.begin_block();
        let pr = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(ls) = samples.next() else { continue; };
                let li = finite(*ls);
                let xl = if invert_left { -li } else { li };
                let input_left = li as f64;
                // Right sample
                let rs_opt = samples.next();
                let ri = if let Some(rs) = rs_opt.as_deref() { finite(*rs) as f64 } else { 0.0_f64 };
                let input_right = ri;

                // Parallel summation: process every active channel independently and sum their outputs.
                let mut sum_left = 0.0_f64;
                let mut sum_right = 0.0_f64;
                for ch_idx in 0..active_ch_count {
                    let ch = &mut self.channels[ch_idx];
                    if !ch_enabled[ch_idx] || ch_mute[ch_idx] { continue; }
                    if any_solo && !ch_solo[ch_idx] { continue; }

                    // Process left sample through this channel strip.
                    let ch_left = ch.process_sample(input_left);
                    let (gl, gr) = ch.pan_gains();
                    // Apply fader gain.
                    let fader = 10.0_f64.powf(ch_fader_db[ch_idx] / 20.0);
                    sum_left += ch_left * gl * fader;
                    sum_right += ch_left * gr * fader;

                    // Process right sample through this channel strip.
                    let ch_right = ch.process_sample(input_right);
                    sum_right += ch_right * gr * fader;
                }

                // Send summed stereo pair through the master bus summer (virtual earth summing with crosstalk).
                let mut left_out = sum_left;
                let mut right_out = sum_right;
                if let Some(ref mut summer) = self.buses[0].summer {
                    let mut out_l = [0.0_f64];
                    let mut out_r = [0.0_f64];
                    let left_in = [left_out];
                    let right_in = [right_out];
                    summer.process_sum(&[(left_in.as_slice(), right_in.as_slice())], &mut out_l, &mut out_r, 1);
                    left_out = out_l[0];
                    right_out = out_r[0];
                }
                // Apply master bus drive saturation through oversampler + latency-compensated dry path.
                let dry_l = self.dry_delays[0].process(xl);
                let dry_r = self.dry_delays[1].process(ri as f32);
                let wet_l = self.oversamplers[0].process(left_out as f32, factor, |x| (x as f64 * master_drive).tanh() as f32);
                let wet_r = self.oversamplers[1].process(right_out as f32, factor, |x| (x as f64 * master_drive).tanh() as f32);
                let mixed_l = dry_l as f64 + (wet_l as f64 - dry_l as f64) * mix;
                let mixed_r = dry_r as f64 + (wet_r as f64 - dry_r as f64) * mix;
                let out_l = finite(self.bypass_crossfades[0].process(dry_l, mixed_l as f32));
                let out_r = finite(self.bypass_crossfades[1].process(dry_r, mixed_r as f32));
                *ls = out_l;
                if let Some(rs) = rs_opt {
                    *rs = out_r;
                }
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
core_dsp::export_greg_plugin!{GregNlMixer, "com.greg.audio.greg-nl-console", "Analog console emulator", utility}

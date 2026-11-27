#![forbid(unsafe_code)]
#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]
use std::sync::{Arc, atomic::Ordering};
use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    routing_matrix::{RoutingMatrix, RoutingMode},
};
use nih_plug::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params, ParamDef, RotaryKnob, ToggleSwitch, EnumSwitch};
use greg_gui_core::prelude::*;
use plugin_support::CommonChannelParams;
use serde::{Serialize, Deserialize};

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregBayParams>,
}

impl Model for EditorData {}

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
struct GregBayParams {
    #[id = "slot_1"]
    slot_1: FloatParam,
    #[id = "slot_2"]
    slot_2: FloatParam,
    #[id = "slot_3"]
    slot_3: FloatParam,
    #[id = "slot_4"]
    slot_4: FloatParam,
    #[id = "routing"]
    routing: EnumParam<RoutingTopology>,
    #[id = "slot_1_module"]
    slot_1_module: EnumParam<SlotModule>,
    #[id = "slot_2_module"]
    slot_2_module: EnumParam<SlotModule>,
    #[id = "slot_3_module"]
    slot_3_module: EnumParam<SlotModule>,
    #[id = "slot_4_module"]
    slot_4_module: EnumParam<SlotModule>,
    #[id = "bypass"]
    bypass: BoolParam,
    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "invert_left"]
    invert_left: BoolParam,
    #[allow(dead_code)]
    channel_setup: CommonChannelParams,
}

impl Default for GregBayParams {
    fn default() -> Self {
        Self {
            slot_1: FloatParam::new(
                "Slot 1 Drive",
                0.0,
                FloatRange::Linear { min: -12.0, max: 18.0 },
            ),
            slot_2: FloatParam::new(
                "Slot 2 Drive",
                0.0,
                FloatRange::Linear { min: -12.0, max: 18.0 },
            ),
            slot_3: FloatParam::new(
                "Slot 3 Drive",
                0.0,
                FloatRange::Linear { min: -12.0, max: 18.0 },
            ),
            slot_4: FloatParam::new(
                "Slot 4 Drive",
                0.0,
                FloatRange::Linear { min: -12.0, max: 18.0 },
            ),
            routing: EnumParam::new("Routing Topology", RoutingTopology::Serial),
            slot_1_module: EnumParam::new("Slot 1 Module", SlotModule::None),
            slot_2_module: EnumParam::new("Slot 2 Module", SlotModule::None),
            slot_3_module: EnumParam::new("Slot 3 Module", SlotModule::None),
            slot_4_module: EnumParam::new("Slot 4 Module", SlotModule::None),
            bypass: BoolParam::new("Bypass", false),
            oversampling: EnumParam::new("Oversampling", Oversampling::X4),
            invert_left: BoolParam::new("Invert Left Polarity", false),
            channel_setup: CommonChannelParams::default(),
        }
    }
}

/// Available DSP modules that can be loaded into each slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum SlotModule {
    #[id = "none"]
    #[name = "None"]
    None,
    #[id = "greg_q-1a"]
    #[name = "greg q-1a"]
    VintageEq,
    #[id = "greg_76"]
    #[name = "greg 76"]
    FetComp,
    #[id = "greg_a2a"]
    #[name = "greg a2a"]
    OptoComp,
    #[id = "greg_73"]
    #[name = "greg 73"]
    ConsolePre,
    #[id = "tape"]
    #[name = "Tape Emulation"]
    Tape,
    #[id = "surgical_eq"]
    #[name = "Surgical EQ"]
    SurgicalEq,
    #[id = "spatial_3d"]
    #[name = "Spatial 3D"]
    Spatial3d,
}

/// Available routing topologies for the 4-slot patchbay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum RoutingTopology {
    #[id = "serial"]
    #[name = "Serial"]
    Serial,      // Slot1 -> Slot2 -> Slot3 -> Slot4
    #[id = "parallel"]
    #[name = "Parallel"]
    Parallel,   // All slots receive original input, mix outputs
    #[id = "midside"]
    #[name = "Mid/Side"]
    Midside,     // Slot1 processes M/S encoded stereo
    #[id = "crossover"]
    #[name = "Crossover 2-Band"]
    Crossover,   // Split freq bands: low=slot1, high=slot2
    #[id = "double_parallel"]
    #[name = "Double Parallel"]
    DoubleParallel, // Two parallel chains mixed together
    #[id = "invert"]
    #[name = "Inverted Phase"]
    Invert,      // Invert phase of odd slots
}

/// Serialized chain state for save/load recall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    pub topology: RoutingTopology,
    pub slot_modules: [SlotModule; 4],
    pub slot_levels: [f32; 4],
    pub slot_mixes: [f32; 4],
    pub bypass: bool,
    pub channel_count: usize,
    pub routing_mode: String,
}

impl Default for ChainState {
    fn default() -> Self {
        Self {
            topology: RoutingTopology::Serial,
            slot_modules: [SlotModule::None; 4],
            slot_levels: [1.0; 4],
            slot_mixes: [0.0; 4],
            bypass: false,
            channel_count: 2,
            routing_mode: format!("{:?}", RoutingMode::Multichannel),
        }
    }
}

/// Chain manager: handles loading/saving patchbay chains via serde_json.
/// Operates only on the non-audio thread; never allocates in process().
struct ChainManager {
    #[allow(dead_code)]
    chains_dir: std::path::PathBuf,
}

#[allow(dead_code)]
impl ChainManager {
    fn new() -> Self {
        let dir = dirs::config_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("analog-vibe")
            .join("greg_bay_chains");
        let _ = std::fs::create_dir_all(&dir);
        Self { chains_dir: dir }
    }

    fn chain_path(&self, name: &str) -> std::path::PathBuf {
        self.chains_dir.join(format!("{}.json", name))
    }

    fn save(&self, name: &str, state: &ChainState) -> std::io::Result<()> {
        let path = self.chain_path(name);
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    fn load(&self, name: &str) -> std::io::Result<ChainState> {
        let path = self.chain_path(name);
        let json = std::fs::read_to_string(path)?;
        let state: ChainState = serde_json::from_str(&json)?;
        Ok(state)
    }

    fn list(&self) -> std::io::Result<Vec<String>> {
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.chains_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json")
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    names.push(stem.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }
}

impl Default for ChainManager {
    fn default() -> Self { Self::new() }
}

struct GregBay {
    params: Arc<GregBayParams>,
    routing_matrix: RoutingMatrix,
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    oversamplers: [Oversampler<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, 32>; plugin_support::MAX_AUDIO_CHANNELS],
    last_oversampling: Oversampling,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
    #[allow(dead_code)]
    chain_manager: ChainManager,
    #[allow(dead_code)]
    current_chain: Option<String>,
    chain_state: ChainState,
}

impl Default for GregBay {
    fn default() -> Self {
        Self {
            params: Arc::new(GregBayParams::default()),
            routing_matrix: RoutingMatrix::default(),
            bypass_crossfades: [BypassCrossfade::default(); plugin_support::MAX_AUDIO_CHANNELS],
            oversamplers: [Oversampler::default(); plugin_support::MAX_AUDIO_CHANNELS],
            dry_delays: [LatencyDelay::default(); plugin_support::MAX_AUDIO_CHANNELS],
            last_oversampling: Oversampling::X1,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
            chain_manager: ChainManager::default(),
            current_chain: None,
            chain_state: ChainState::default(),
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
        self.chain_state.channel_count = cc;
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
        true
    }
    fn reset(&mut self) {
        self.last_bypassed = self.params.bypass.value();
        for cf in &mut self.bypass_crossfades { cf.set_bypassed(self.last_bypassed, 0); }
        self.last_oversampling = Oversampling::X1;
        for delay in &mut self.dry_delays { delay.reset(); }
        for oversampler in &mut self.oversamplers { oversampler.reset(); }
    }
    fn editor(&mut self, _ae: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    let params_main = self.params.clone();
    let param_defs: &'static [ParamDef] = greg_gui_core::greg_static_params!(vec![
        ParamDef::knob("Slot 1 Drive", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.slot_1);
        })),
        ParamDef::knob("Slot 2 Drive", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.slot_2);
        })),
        ParamDef::knob("Slot 3 Drive", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.slot_3);
        })),
        ParamDef::knob("Slot 4 Drive", Box::new(|cx| {
            RotaryKnob::new(cx, EditorData::params, |p| &p.slot_4);
        })),
        ParamDef::enum_switch("Routing Topology", Box::new(|cx| {
            EnumSwitch::new(cx, EditorData::params, |p| &p.routing);
        })),
        ParamDef::enum_switch("Slot 1 Module", Box::new(|cx| {
            EnumSwitch::new(cx, EditorData::params, |p| &p.slot_1_module);
        })),
        ParamDef::enum_switch("Slot 2 Module", Box::new(|cx| {
            EnumSwitch::new(cx, EditorData::params, |p| &p.slot_2_module);
        })),
        ParamDef::enum_switch("Slot 3 Module", Box::new(|cx| {
            EnumSwitch::new(cx, EditorData::params, |p| &p.slot_3_module);
        })),
        ParamDef::enum_switch("Slot 4 Module", Box::new(|cx| {
            EnumSwitch::new(cx, EditorData::params, |p| &p.slot_4_module);
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
        "Greg Bay",
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
                let xl = if invert_left { -li } else { li };
                let wet_l = self.oversamplers[0].process(xl, factor, |x| {
                    let t = (x as f64 * slot1).tanh() as f32;
                    (t as f64 * slot3).tanh() as f32
                });
                let wet_r = self.oversamplers[1].process(ri, factor, |x| {
                    let t = (x as f64 * slot2).tanh() as f32;
                    (t as f64 * slot4).tanh() as f32
                });
                let dry_l = self.dry_delays[0].process(xl);
                let dry_r = self.dry_delays[1].process(ri);
                *ls = finite(self.bypass_crossfades[0].process(dry_l, wet_l));
                *rs = finite(self.bypass_crossfades[1].process(dry_r, wet_r));
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

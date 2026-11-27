//! Copyright (c) 2026 Alberto Gregorio Ferraris. All rights reserved.
//! Licensed under GNU GPLv3 or later + Proprietary Commercial License.
//
//! # Greg Space — Riverbero Acustico 3D & FDN Studio Suite
//!
//! Acoustic-room + FDN reverb con posizionamento spaziale 3D (ITD/HRIR) per
//! mastering e post-produzione. I moduli DSP sono a f64, zero-alloc nel thread
//! audio e sanitizzano NaN/Inf a 0.0.

#![forbid(unsafe_code)]
#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]

use std::array;
use std::sync::{Arc, atomic::Ordering};

use core_dsp::core_dsp_modules::{
    fdn_reverb::FdnReverb,
    acoustic_room::AcousticRoom,
    spatial_3d::{Spatial3d, Spatial3dConfig, SpeakerPosition},
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    routing_matrix::{RoutingMatrix, RoutingMode},
    plugin_export::{VENDOR, URL, EMAIL},
    zdf_filters::{ZdfSvf},
};
use nih_plug_vizia::vizia::prelude::*;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params, greg_static_params};
use greg_gui_core::prelude::*;
use nih_plug::prelude::*;
use plugin_support::CommonChannelParams;

const MAX_DELAY_SEC: f64 = 5.0;
const MAX_DRY_DELAY: usize = 256;
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


#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
enum ReverbAlgorithm {
    #[id = "booth"]
    #[name = "Booth"]
    Booth,
    #[id = "room"]
    #[name = "Room"]
    Room,
    #[id = "chamber"]
    #[name = "Chamber"]
    Chamber,
    #[id = "plate"]
    #[name = "Plate"]
    Plate,
    #[id = "hall"]
    #[name = "Hall"]
    Hall,
    #[id = "cathedral"]
    #[name = "Cathedral"]
    Cathedral,
}

impl Default for ReverbAlgorithm {
    fn default() -> Self {
        ReverbAlgorithm::Room
    }
}

#[derive(Params)]
struct GregSpaceParams {
    #[id = "algorithm"]
    algorithm: EnumParam<ReverbAlgorithm>,
    #[id = "decay_time"]
    decay_time: FloatParam,
    #[id = "room_size"]
    room_size: FloatParam,
    #[id = "pre_delay"]
    pre_delay_ms: FloatParam,
    #[id = "hf_damping"]
    hf_damping: FloatParam,
    #[id = "early_late_mix"]
    early_late_mix: FloatParam,
    #[id = "dry_wet"]
    dry_wet: FloatParam,
    #[id = "early_density"]
    early_density: FloatParam,
    #[id = "early_spread"]
    early_spread: FloatParam,
    #[id = "tail_density"]
    tail_density: FloatParam,
    #[id = "tail_diffusion"]
    tail_diffusion: FloatParam,
    #[id = "low_decay_mult"]
    low_decay_mult: FloatParam,
    #[id = "modulation_rate"]
    modulation_rate: FloatParam,
    #[id = "modulation_depth"]
    modulation_depth: FloatParam,
    #[id = "spatial_width"]
    spatial_width: FloatParam,
    #[id = "low_cut"]
    low_cut: FloatParam,
    #[id = "high_cut"]
    high_cut: FloatParam,
    #[id = "freeze_tail"]
    freeze_tail: BoolParam,
    #[id = "output_trim"]
    output_trim: FloatParam,
    #[allow(dead_code)]
    #[id = "bypass"]
    bypass: BoolParam,
    #[allow(dead_code)]
    channel_setup: CommonChannelParams,

    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "invert_left"]
    invert_left: BoolParam,
}

impl Default for GregSpaceParams {
    fn default() -> Self {
        Self {
            algorithm: EnumParam::new("Algorithm", ReverbAlgorithm::Room),
            decay_time: FloatParam::new("Decay Time", 1.5, FloatRange::Linear { min: 0.15, max: 20.0 })
                .with_unit(" s"),
            room_size: FloatParam::new("Room Size", 0.5, FloatRange::Linear { min: 0.05, max: 1.0 })
                .with_unit(" %"),
            pre_delay_ms: FloatParam::new("Pre-Delay", 0.0, FloatRange::Linear { min: 0.0, max: 250.0 })
                .with_unit(" ms"),
            hf_damping: FloatParam::new("HF Damping", 5000.0, FloatRange::Linear { min: 1000.0, max: 20000.0 })
                .with_unit(" Hz"),
            early_late_mix: FloatParam::new("Early/Late Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            dry_wet: FloatParam::new("Dry/Wet", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            early_density: FloatParam::new("Early Density", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            early_spread: FloatParam::new("Early Spread", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            tail_density: FloatParam::new("Tail Density", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            tail_diffusion: FloatParam::new("Tail Diffusion", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            low_decay_mult: FloatParam::new("Low Decay Mult", 1.0, FloatRange::Linear { min: 0.5, max: 2.0 }),
            modulation_rate: FloatParam::new("Modulation Rate", 0.0, FloatRange::Linear { min: 0.0, max: 2.0 })
                .with_unit(" Hz"),
            modulation_depth: FloatParam::new("Modulation Depth", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            spatial_width: FloatParam::new("Spatial Width", 1.0, FloatRange::Linear { min: 0.0, max: 2.0 }),
            low_cut: FloatParam::new("Low Cut", 20.0, FloatRange::Linear { min: 20.0, max: 500.0 })
                .with_unit(" Hz"),
            high_cut: FloatParam::new("High Cut", 20000.0, FloatRange::Linear { min: 1000.0, max: 20000.0 })
                .with_unit(" Hz"),
            freeze_tail: BoolParam::new("Freeze Tail", false),
            output_trim: FloatParam::new("Output Trim", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 })
                .with_unit(" dB"),
            bypass: BoolParam::new("Bypass", false),
            channel_setup: CommonChannelParams::default(),
            oversampling: EnumParam::new("Oversampling", Oversampling::X2),
            invert_left: BoolParam::new("Invert Left Polarity", false),
        }
    }
}

#[derive(Clone, Lens)]
struct EditorData {
    params: Arc<GregSpaceParams>,
}

impl Model for EditorData {}

struct GregSpace {
    params: Arc<GregSpaceParams>,
    routing_matrix: RoutingMatrix,
    fdns: [FdnReverb; plugin_support::MAX_AUDIO_CHANNELS],
    rooms: [AcousticRoom; plugin_support::MAX_AUDIO_CHANNELS],
    spatials: [Spatial3d; plugin_support::MAX_AUDIO_CHANNELS],
    lpf: [ZdfSvf; plugin_support::MAX_AUDIO_CHANNELS],
    bypass_crossfades: [BypassCrossfade<f32>; plugin_support::MAX_AUDIO_CHANNELS],
    dry_delays: [LatencyDelay<f32, MAX_DRY_DELAY>; plugin_support::MAX_AUDIO_CHANNELS],
    sample_rate: f32,
    last_oversampling: Oversampling,
    last_algorithm: ReverbAlgorithm,
    last_dry_wet: f32,
    last_bypassed: bool,
    bypass_ramp_samples: u32,
    initialized: bool,
    panic_protection: PanicProtection,
    metering: Metering,
    telemetry: Arc<GuiTelemetry>,
}

impl Default for GregSpace {
    fn default() -> Self {
        let sr = 48000.0;
        Self {
            params: Arc::new(GregSpaceParams::default()),
            routing_matrix: RoutingMatrix::default(),
            fdns: array::from_fn(|_| FdnReverb::new(8, MAX_DELAY_SEC, sr)),
            rooms: array::from_fn(|_| AcousticRoom::new()),
            spatials: array::from_fn(|_| Spatial3d::default()),
            lpf: array::from_fn(|_| ZdfSvf::new()),
            bypass_crossfades: array::from_fn(|_| BypassCrossfade::default()),
            dry_delays: array::from_fn(|_| LatencyDelay::default()),
            sample_rate: 0.0,
            last_oversampling: Oversampling::X2,
            last_algorithm: ReverbAlgorithm::Room,
            last_dry_wet: 0.0,
            last_bypassed: false,
            bypass_ramp_samples: 1,
            initialized: false,
            panic_protection: PanicProtection::default(),
            metering: Metering::default(),
            telemetry: GuiTelemetry::new(true),
        }
    }
}

impl GregSpace {
    fn configure_spatials(&mut self, sample_rate: f64, width: f64) {
        let positions = [
            SpeakerPosition::from_degrees(-30.0, 0.0),
            SpeakerPosition::from_degrees(30.0, 0.0),
        ];
        let cfg = Spatial3dConfig::with_gain(sample_rate, true, width.clamp(0.01, 2.0));
        for sp in &mut self.spatials {
            sp.initialize(sample_rate, cfg, &positions);
        }
    }

    fn set_latency(&mut self, latency_samples: u32, _buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        let d = latency_samples as usize;
        let ok = self.dry_delays.iter_mut().all(|delay| delay.set_delay_samples(d));
        if ok {
            _context.set_latency_samples(latency_samples);
        }
        ok
    }
}

impl Plugin for GregSpace {
    const NAME: &'static str = "Greg Space";
    const VENDOR: &'static str = VENDOR;
    const URL: &'static str = URL;
    const EMAIL: &'static str = EMAIL;
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
        if !self.routing_matrix.configure(channel_count, RoutingMode::Multichannel) {
            return false;
        }
        self.sample_rate = buffer_config.sample_rate;

        let sr = buffer_config.sample_rate as f64;
        let decay = self.params.decay_time.value() as f64;
        let fb = decay.clamp(0.0, 0.999);
        for fdn in &mut self.fdns {
            fdn.set_sample_rate(sr);
            fdn.set_feedback_gain(fb);
            fdn.set_damping_freq(self.params.hf_damping.value() as f64);
        }

        let room_cfg = core_dsp::core_dsp_modules::acoustic_room::RoomConfig::default();
        for room in &mut self.rooms {
            room.initialize(sr, room_cfg);
        }
        self.configure_spatials(sr, self.params.spatial_width.value() as f64);

        let pre_delay_samples = (self.params.pre_delay_ms.value() as f64 * sr * 1e-3).round() as u32;
        let oversampling = self.params.oversampling.value();
        let os_latency = match oversampling {
            Oversampling::X1 => 0,
            Oversampling::X2 => 15,
            Oversampling::X4 => 22,
            Oversampling::X8 => 25,
        };
        if !self.set_latency(pre_delay_samples + os_latency as u32, buffer_config, context) {
            return false;
        }
        self.last_oversampling = oversampling;
        for delay in &mut self.dry_delays {
            delay.reset();
        }
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.params.bypass.value(), 0);
        }
        self.last_bypassed = self.params.bypass.value();
        self.bypass_ramp_samples = (buffer_config.sample_rate * 0.005).round().max(1.0) as u32;
        self.initialized = true;
        true
    }

    fn reset(&mut self) {
        for fdn in &mut self.fdns {
            fdn.reset();
        }
        for room in &mut self.rooms {
            room.reset();
        }
        for sp in &mut self.spatials {
            sp.reset();
        }
        for lpf in &mut self.lpf {
            lpf.reset();
        }
        for delay in &mut self.dry_delays {
            delay.reset();
        }
        self.last_bypassed = self.params.bypass.value();
        for crossfade in &mut self.bypass_crossfades {
            crossfade.set_bypassed(self.last_bypassed, 0);
        }
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params_main = self.params.clone();
        let param_defs: Vec<greg_gui_core::ParamDef> = vec![
            greg_gui_core::ParamDef::enum_switch("Algorithm", Box::new(|cx| {
                EnumSwitch::new(cx, EditorData::params, |p| &p.algorithm);
            })),
            greg_gui_core::ParamDef::knob("Decay Time", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.decay_time);
            })),
            greg_gui_core::ParamDef::knob("Room Size", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.room_size);
            })),
            greg_gui_core::ParamDef::knob("Pre-Delay", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.pre_delay_ms);
            })),
            greg_gui_core::ParamDef::knob("HF Damping", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.hf_damping);
            })),
            greg_gui_core::ParamDef::knob("Early/Late Mix", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.early_late_mix);
            })),
            greg_gui_core::ParamDef::knob("Dry/Wet", Box::new(|cx| {
                RotaryKnob::new(cx, EditorData::params, |p| &p.dry_wet);
            })),
            greg_gui_core::ParamDef::toggle("Bypass", Box::new(|cx| {
                ToggleSwitch::new(cx, EditorData::params, |p| &p.bypass);
            })),
            greg_gui_core::ParamDef::enum_switch("Oversampling", Box::new(|cx| {
                EnumSwitch::new(cx, EditorData::params, |p| &p.oversampling);
            })),
            greg_gui_core::ParamDef::toggle("Invert Left Polarity", Box::new(|cx| {
                ToggleSwitch::new(cx, EditorData::params, |p| &p.invert_left);
            })),
        ];
        let param_defs: &'static [greg_gui_core::ParamDef] = greg_static_params!(param_defs);
        create_greg_editor_with_params(
            "Greg Space",
            self.telemetry.clone(),
            move |_| {},
            move |_cx| {},
            Some(param_defs),
            move |cx| EditorData { params: params_main.clone() }.build(cx),
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
        if !self.initialized {
            return ProcessStatus::Error("plugin has not been initialized");
        }

        let algorithm = self.params.algorithm.value();
        if algorithm != self.last_algorithm {
            self.last_algorithm = algorithm;
            self.configure_spatials(self.sample_rate as f64, self.params.spatial_width.value() as f64);
        }

        let bypassed = self.params.bypass.value();
        if bypassed != self.last_bypassed {
            for crossfade in &mut self.bypass_crossfades {
                crossfade.set_bypassed(bypassed, self.bypass_ramp_samples);
            }
            self.last_bypassed = bypassed;
        }

        let dry_wet = self.params.dry_wet.value();
        if dry_wet != self.last_dry_wet {
            self.last_dry_wet = dry_wet;
        }
        let invert_left = self.params.invert_left.value();

        let oversampling = self.params.oversampling.value();
        if oversampling != self.last_oversampling {
            let os_latency = match oversampling {
                Oversampling::X1 => 0,
                Oversampling::X2 => 15,
                Oversampling::X4 => 22,
                Oversampling::X8 => 25,
            };
            for delay in &mut self.dry_delays { delay.set_delay_samples(os_latency as usize); }
            self.last_oversampling = oversampling;
            _context.set_latency_samples(os_latency as u32);
        }

        self.metering.begin_block();
        let protection_result = self.panic_protection.run(|| {
            for mut channel_samples in buffer.iter_samples() {
                for (channel, sample) in channel_samples.iter_mut().enumerate() {
                    let input = if sample.is_finite() { *sample } else { 0.0 };
                    let input = if invert_left && channel == 0 { -input } else { input };
                    let dry = self.dry_delays[channel].process(input);
                    let fdn_in = if input.is_finite() { input } else { 0.0 };
                    let (wet_l, wet_r) = self.fdns[channel].process_sample(fdn_in as f64);
                    let wet = ((wet_l + wet_r) * 0.5) as f32;
                    let mix = (1.0 - dry_wet) * dry + dry_wet * wet;
                    *sample = self.bypass_crossfades[channel].process(dry, mix);
                    self.metering.record_sample(input, *sample, 0.0);
                    self.telemetry.push_in_peak(channel, input);
                    self.telemetry.push_out_peak(channel, *sample);
                }
            }
        });
        self.metering.publish_block();
        self.telemetry.num_channels.store(self.routing_matrix.channel_count() as u32, Ordering::Relaxed);
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
            for mut samples in buffer.iter_samples() {
                for sample in samples.iter_mut() {
                    *sample = 0.0;
                }
            }
        }

        ProcessStatus::Normal
    }
}

core_dsp::export_greg_plugin!(
    GregSpace,
    "com.greg.audio.greg-space",
    "Acoustic-room and FDN reverb with spatial 3D placement"
);

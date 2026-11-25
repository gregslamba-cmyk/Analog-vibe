#![forbid(unsafe_code)]
#![allow(unknown_lints, semicolon_in_expressions_from_non_local_macros)]
use std::sync::Arc;
use core_dsp::core_dsp_modules::{
    metering::Metering,
    panic_protection::{PanicProtection, ProtectionResult},
    realtime_controls::{BypassCrossfade, LatencyDelay},
    resampler_bitdepth::{Oversampler, OversamplingFactor},
    routing_matrix::{RoutingMatrix, RoutingMode},
};
use nih_plug::prelude::*;
use plugin_support::CommonChannelParams;
use greg_gui_core::{GuiTelemetry, create_greg_editor_with_params};
use greg_gui_core::prelude::*;

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
struct GregNlChParams {
    #[id = "drive"] drive_db: FloatParam,
    #[id = "crosstalk"] crosstalk_db: FloatParam,
    #[id = "mix"] mix: FloatParam,
    #[id = "bypass"] bypass: BoolParam,
    channel_setup: CommonChannelParams,

    #[id = "oversampling"]
    oversampling: EnumParam<Oversampling>,
    #[id = "invert_left"]
    invert_left: BoolParam,
}

impl Default for GregNlChParams {
    fn default() -> Self {
        Self {
            drive_db: FloatParam::new("Drive (dB)", 0.0, FloatRange::Linear { min: -12.0, max: 18.0 }).with_unit(" dB"),
            crosstalk_db: FloatParam::new("Crosstalk (dB)", -60.0, FloatRange::Linear { min: -80.0, max: 0.0 }).with_unit(" dB"),
            mix: FloatParam::new("Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            bypass: BoolParam::new("Bypass", false),
            channel_setup: CommonChannelParams::default(),
            oversampling: EnumParam::new("Oversampling", Oversampling::X2),
            invert_left: BoolParam::new("Invert Left Polarity", false),
        }
    }
}

struct GregNlCh {
    params: Arc<GregNlChParams>,
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
}

impl Default for GregNlCh {
    fn default() -> Self {
        Self {
            params: Arc::new(GregNlChParams::default()),
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
        }
    }
}

impl Plugin for GregNlCh {
    const NAME: &'static str = "Greg NL Ch";
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
    fn editor(&mut self, _ae: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> { None }
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
        let mix = self.params.mix.value();
        let drive_lin = 10.0f64.powf((self.params.drive_db.value() / 20.0).into());
        self.metering.begin_block();
        let pr = self.panic_protection.run(|| {
            for mut frame in buffer.iter_samples() {
                let mut samples = frame.iter_mut();
                let Some(ls) = samples.next() else { continue; };
                let li = finite(*ls);
                let Some(rs) = samples.next() else { *ls = li; continue; };
                let ri = finite(*rs);
                let xl = if invert_left { -li } else { li };
                let wet_l = self.oversamplers[0].process(xl, factor, |x| (x as f64 * drive_lin).tanh() as f32);
                let wet_r = self.oversamplers[1].process(ri, factor, |x| (x as f64 * drive_lin).tanh() as f32);
                let dry_l = self.dry_delays[0].process(xl);
                let dry_r = self.dry_delays[1].process(ri);
                let lmo = dry_l + (wet_l - dry_l) * mix;
                let rmo = dry_r + (wet_r - dry_r) * mix;
                *ls = finite(self.bypass_crossfades[0].process(dry_l, lmo));
                *rs = finite(self.bypass_crossfades[1].process(dry_r, rmo));
                self.metering.record_sample(li, *ls, 0.0);
                self.metering.record_sample(ri, *rs, 0.0);
            }
        });
        self.metering.publish_block();
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
core_dsp::export_greg_plugin!(GregNlCh, "com.greg.audio.greg-nl-ch", "Non-linear console channel strip", utility);

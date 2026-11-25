//! # Multi-Channel Graph Engine
//!
//! This module provides the infrastructure for multi-channel audio processing,
//! enabling dynamic Mono→7.1 surround support with linked-sidechain detection.
//!
//! ## Architecture
//! - **`MonoChannelProcessor<T>`**: Trait for a single mono audio processor.
//! - **`MultiChannelGraph<P>`**: Pre-allocated graph managing `N` independent mono engines.
//! - **`MultiChannelDetectorLink`**: Computes shared gain reduction per link mode.
//! - **`ChannelSetup`**: Manual override for host-detected channel count.
//! - **`MultiChannelLinkMode`**: 4 detector linking strategies.

use num_traits::Float;
use core_dsp::core_dsp_modules::routing_matrix::{RoutingMatrix, RoutingMode};
use core_dsp::core_dsp_modules::realtime_controls::BypassCrossfade;
use std::sync::atomic::{AtomicBool, Ordering};
use core_dsp::core_dsp_modules::sidechain_processor::MultiChannelLinkMode;
use nih_plug::prelude::Enum;

/// Manual channel count override (`Channel Setup` parameter exposed to user).
/// - `Auto`: Use `buffer.channels()` from DAW host (default).
/// - `Mono`..`SevenDot1`: Force a specific channel count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Enum)]
pub enum ChannelSetup {
    #[default]
    #[id = "auto"]
    #[name = "Auto"]
    Auto,
    #[id = "mono_1ch"]
    #[name = "Mono (1ch)"]
    Mono,
    #[id = "stereo_2ch"]
    #[name = "Stereo (2ch)"]
    Stereo,
    #[id = "lcr_3ch"]
    #[name = "LCR (3ch)"]
    LCR,
    #[id = "quad_4ch"]
    #[name = "Quad (4ch)"]
    Quad,
    #[id = "5_0"]
    #[name = "5.0"]
    FiveDot0,
    #[id = "5_1"]
    #[name = "5.1"]
    FiveDot1,
    #[id = "7_0"]
    #[name = "7.0"]
    SevenDot0,
    #[id = "7_1"]
    #[name = "7.1"]
    SevenDot1,
}

impl ChannelSetup {
    /// Returns the channel count if not `Auto`, otherwise `None`.
    pub fn channel_count(&self) -> Option<usize> {
        match self {
            ChannelSetup::Auto => None,
            ChannelSetup::Mono => Some(1),
            ChannelSetup::Stereo => Some(2),
            ChannelSetup::LCR => Some(3),
            ChannelSetup::Quad => Some(4),
            ChannelSetup::FiveDot0 => Some(5),
            ChannelSetup::FiveDot1 => Some(6),
            ChannelSetup::SevenDot0 => Some(7),
            ChannelSetup::SevenDot1 => Some(8),
        }
    }

    /// Validate channel count is in supported range 1..=8.
    pub fn is_valid(count: usize) -> bool {
        (1..=8).contains(&count)
    }

    /// Convert from DAW layout (1..8 channels) to ChannelSetup.
    pub fn from_channel_count(count: usize) -> Self {
        match count {
            1 => ChannelSetup::Mono,
            2 => ChannelSetup::Stereo,
            3 => ChannelSetup::LCR,
            4 => ChannelSetup::Quad,
            5 => ChannelSetup::FiveDot0,
            6 => ChannelSetup::FiveDot1,
            7 => ChannelSetup::SevenDot0,
            8 => ChannelSetup::SevenDot1,
            _ => ChannelSetup::Auto,
        }
    }
}

/// Trait implemented by each mono channel's DSP engine.
/// The plugin instantiates `[P; MAX_AUDIO_CHANNELS]` where `P: MonoChannelProcessor<T>`.
pub trait MonoChannelProcessor<T: Float>: Sized + Default {
    /// Process a single audio sample through the mono engine.
    /// Must be zero-alloc, no locks, real-time safe.
    fn process(&mut self, input: T) -> T;

    /// Reset all internal state (filters, envelopes, delay lines, etc.).
    fn reset(&mut self);

    /// Set the sample rate for time-constant calculations (attack/release, filter coeffs).
    fn set_sample_rate(&mut self, sample_rate: f64);

    /// Get current gain reduction amount: `0.0` = no reduction (unity), `1.0` = full reduction (silence).
    /// Used by `MultiChannelDetectorLink` for detector linking.
    fn gain_reduction(&self) -> T;

    /// Get current input peak level for metering (absolute value).
    fn input_peak(&self) -> T;

    /// Get current output peak level for metering (absolute value).
    fn output_peak(&self) -> T;
}

/// Pre-allocated multi-channel graph managing N independent mono processors.
///
/// # Type Parameters
/// - `P`: Mono processor type implementing `MonoChannelProcessor<f64>`.
///        Must be usable in a fixed-size array `[P; MAX_AUDIO_CHANNELS]`.
///
/// # Real-Time Guarantees
/// - Zero heap allocations in `process()`.
/// - No locks or mutexes; audio↔UI via atomics (`Ordering::Relaxed`).
/// - Handles arbitrary block sizes (1..max_block_size).
pub struct MultiChannelGraph<P: MonoChannelProcessor<f64>> {
    /// Array of 8 pre-allocated mono engines (one per max channel).
    processors: [P; crate::MAX_AUDIO_CHANNELS],
    /// Currently configured channel count (1..=8).
    channel_count: usize,
    /// Routing matrix for pre/post processing (Mid/Side, Mono sum, etc.).
    routing: RoutingMatrix,
    /// Current detector link mode.
    link_mode: MultiChannelLinkMode,
    /// Bypass crossfade for click-free enable/disable (5 ms default).
    bypass_crossfade: BypassCrossfade<f64>,
    /// Atomic bypass flag (set by UI thread, read by audio thread).
    bypassed: AtomicBool,
    /// Manual channel setup override (Auto = use host detection).
    channel_setup: ChannelSetup,
    /// Current sample rate.
    sample_rate: f64,
}

impl<P: MonoChannelProcessor<f64>> Default for MultiChannelGraph<P> {
    fn default() -> Self {
        // Pre-allocate array of 8 default-constructed processors
        let processors = std::array::from_fn(|_| P::default());
        Self {
            processors,
            channel_count: 2, // Default stereo
            routing: RoutingMatrix::default(),
            link_mode: MultiChannelLinkMode::default(),
            bypass_crossfade: BypassCrossfade::default(),
            bypassed: AtomicBool::new(false),
            channel_setup: ChannelSetup::default(),
            sample_rate: 48_000.0,
        }
    }
}

impl<P: MonoChannelProcessor<f64>> MultiChannelGraph<P> {
    /// Initialize the graph with sample rate and default channel count.
    /// Called once in plugin `initialize()` before audio starts.
    pub fn initialize(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        for proc in &mut self.processors {
            proc.set_sample_rate(sample_rate);
        }
        // Default to stereo pass-through
        self.routing.configure(self.channel_count, RoutingMode::Stereo);
        // Default bypass crossfade: 5 ms
        let ramp_samples = (sample_rate * 0.005) as u32;
        self.bypass_crossfade.set_bypassed(false, ramp_samples);
    }

    /// Reconfigure for a new channel count (called from `initialize()` or when `Channel Setup` changes).
    /// Must not be called from `process()` - only from non-realtime parameter change or `reset()`.
    pub fn set_channel_count(&mut self, count: usize) {
        if ChannelSetup::is_valid(count) && count != self.channel_count {
            self.channel_count = count;
            // Keep current routing mode, just update channel count
            let mode = self.routing.mode();
            self.routing.configure(count, mode);
        }
    }

    /// Apply manual `Channel Setup` override (from UI parameter).
    /// Returns `true` if channel count actually changed and graph was reconfigured.
    pub fn set_channel_setup(&mut self, setup: ChannelSetup) -> bool {
        self.channel_setup = setup;
        if let Some(count) = setup.channel_count() {
            self.set_channel_count(count);
            true
        } else {
            false
        }
    }

    /// Get current channel count.
    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    /// Get current routing mode.
    pub fn routing_mode(&self) -> RoutingMode {
        self.routing.mode()
    }

    /// Set routing mode (e.g., MidSide for M/S processing).
    pub fn set_routing_mode(&mut self, mode: RoutingMode) {
        self.routing.configure(self.channel_count, mode);
    }

    /// Get current detector link mode.
    pub fn link_mode(&self) -> MultiChannelLinkMode {
        self.link_mode
    }

    /// Set detector link mode.
    pub fn set_link_mode(&mut self, mode: MultiChannelLinkMode) {
        self.link_mode = mode;
    }

    /// Set bypass state with click-free crossfade.
    pub fn set_bypassed(&mut self, bypassed: bool) {
        let ramp_samples = (self.sample_rate * 0.005) as u32;
        self.bypass_crossfade.set_bypassed(bypassed, ramp_samples);
        self.bypassed.store(bypassed, Ordering::Relaxed);
    }

    /// Check if currently bypassed (atomic read from audio thread).
    pub fn is_bypassed(&self) -> bool {
        self.bypassed.load(Ordering::Relaxed)
    }

    /// Compute the shared gain reduction based on link mode.
    fn compute_linked_gain_reduction(&self, max_gr: f64) -> f64 {
        match self.link_mode {
            MultiChannelLinkMode::Unlinked => max_gr,
            MultiChannelLinkMode::AllLinked => max_gr,
            MultiChannelLinkMode::SurroundStemLink => {
                if self.channel_count >= 6 {
                    let mut stem_max = 0.0;
                    let lfe_idx = if self.channel_count == 6 { 5 } else { 7 };
                    for (i, proc) in self.processors.iter().enumerate() {
                        if i < self.channel_count && i != lfe_idx {
                            stem_max = stem_max.max(proc.gain_reduction());
                        }
                    }
                    stem_max
                } else {
                    max_gr
                }
            }
            MultiChannelLinkMode::MidSide => {
                if self.channel_count >= 2 {
                    let mid_gr = self.processors[0].gain_reduction();
                    let side_gr = self.processors[1].gain_reduction();
                    mid_gr.max(side_gr)
                } else {
                    max_gr
                }
            }
        }
    }

    /// Apply the computed shared gain reduction to all linked channels.
    fn apply_linked_gain_reduction(&mut self, channels: &mut [f64], gr: f64) {
        let gain = 1.0 - gr;
        for (i, channel_samples) in channels.iter_mut().enumerate() {
            if i < self.channel_count {
                *channel_samples *= gain;
            }
        }
    }
}

impl<P: MonoChannelProcessor<f64>> MultiChannelGraph<P> {
    /// Process one block of audio (in-place on interleaved `channels`).
    /// `channels` must have length == `channel_count`.
    /// Returns the maximum gain reduction across linked channels (for metering).
    ///
    /// # Real-Time Safe
    /// - Zero allocations
    /// - No locks
    /// - In-place processing on provided buffers
    pub fn process(&mut self, channels: &mut [f64]) -> f64 {
        if self.is_bypassed() {
            return 0.0;
        }

        if self.routing.mode() == RoutingMode::MidSide && channels.len() == 2 {
            self.routing.encode_input(channels);
        }

        let mut max_gr = 0.0;
        for (i, channel_samples) in channels.iter_mut().enumerate() {
            if i >= self.channel_count {
                break;
            }
            let wet = self.processors[i].process(*channel_samples);
            max_gr = max_gr.max(self.processors[i].gain_reduction());
            *channel_samples = wet;
        }

        let linked_gr = self.compute_linked_gain_reduction(max_gr);

        if self.link_mode != MultiChannelLinkMode::Unlinked {
            self.apply_linked_gain_reduction(channels, linked_gr);
        }

        if self.routing.mode() == RoutingMode::MidSide && channels.len() == 2 {
            self.routing.decode_output(channels);
        }

        linked_gr
    }

    /// Reset all internal state.
    pub fn reset(&mut self) {
        for proc in &mut self.processors {
            proc.reset();
        }
        let ramp_samples = (self.sample_rate * 0.005) as u32;
        self.bypass_crossfade.set_bypassed(false, ramp_samples);
        self.bypassed.store(false, Ordering::Relaxed);
    }

    /// Get mutable reference to a specific channel's processor.
    pub fn processor_mut(&mut self, channel: usize) -> Option<&mut P> {
        if channel < self.channel_count {
            Some(&mut self.processors[channel])
        } else {
            None
        }
    }

    /// Get all processors for bulk parameter updates.
    pub fn processors_mut(&mut self) -> &mut [P] {
        &mut self.processors[..self.channel_count]
    }

    /// Get peak metering for all active channels.
    pub fn meter_peaks(&self) -> (Vec<f64>, Vec<f64>) {
        let mut in_peaks = Vec::with_capacity(self.channel_count);
        let mut out_peaks = Vec::with_capacity(self.channel_count);
        for proc in &self.processors[..self.channel_count] {
            in_peaks.push(proc.input_peak());
            out_peaks.push(proc.output_peak());
        }
        (in_peaks, out_peaks)
    }
}
pub struct MultiChannelDetectorLink<P: MonoChannelProcessor<f64>> {
    processors: [P; crate::MAX_AUDIO_CHANNELS],
    link_mode: MultiChannelLinkMode,
    channel_count: usize,
    channel_setup: ChannelSetup,
}

impl<P: MonoChannelProcessor<f64>> MultiChannelDetectorLink<P> {
    /// Create a new detector link with the given pre-allocated processors.
    pub fn new(processors: [P; crate::MAX_AUDIO_CHANNELS]) -> Self {
        Self {
            processors,
            link_mode: MultiChannelLinkMode::default(),
            channel_count: 2,
            channel_setup: ChannelSetup::default(),
        }
    }

    /// Set the detector link mode.
    pub fn set_link_mode(&mut self, mode: MultiChannelLinkMode) {
        self.link_mode = mode;
    }

    /// Set the active channel count (1..=8).
    pub fn set_channel_count(&mut self, count: usize) {
        if ChannelSetup::is_valid(count) {
            self.channel_count = count;
        }
    }

    /// Apply manual Channel Setup override.
    pub fn set_channel_setup(&mut self, setup: ChannelSetup) {
        self.channel_setup = setup;
        if let Some(count) = setup.channel_count() {
            self.set_channel_count(count);
        }
    }

    /// Compute shared gain reduction amount: `0.0` = no reduction, `1.0` = full reduction.
    pub fn compute_shared_gr(&self) -> f64 {
        let mut max_gr = 0.0;
        for i in 0..self.channel_count {
            max_gr = max_gr.max(self.processors[i].gain_reduction());
        }

        match self.link_mode {
            MultiChannelLinkMode::Unlinked | MultiChannelLinkMode::AllLinked => max_gr,
            MultiChannelLinkMode::SurroundStemLink => {
                if self.channel_count >= 6 {
                    let mut stem_max = 0.0;
                    let lfe_idx = if self.channel_count == 6 { 5 } else { 7 };
                    for (i, proc) in self.processors.iter().enumerate() {
                        if i < self.channel_count && i != lfe_idx {
                            stem_max = stem_max.max(proc.gain_reduction());
                        }
                    }
                    stem_max
                } else {
                    max_gr
                }
            }
            MultiChannelLinkMode::MidSide => {
                if self.channel_count >= 2 {
                    let mid_gr = self.processors[0].gain_reduction();
                    let side_gr = self.processors[1].gain_reduction();
                    mid_gr.max(side_gr)
                } else {
                    max_gr
                }
            }
        }
    }

    /// Apply shared GR to all linked channels.
    pub fn apply_shared_gr(&self, channels: &mut [f64], shared_gr: f64) {
        let gain = 1.0 - shared_gr;
        for (i, ch) in channels.iter_mut().enumerate() {
            if i < self.channel_count {
                *ch *= gain;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_dsp::core_dsp_modules::routing_matrix::RoutingMode;

    /// Minimal mono processor for testing.
    #[derive(Clone, Copy, Debug)]
    struct TestProcessor {
        gain: f64,
        sample_rate: f64,
        in_peak: f64,
        out_peak: f64,
    }

    impl MonoChannelProcessor<f64> for TestProcessor {
        fn process(&mut self, input: f64) -> f64 {
            self.in_peak = self.in_peak.max(input.abs());
            let out = input * self.gain;
            self.out_peak = self.out_peak.max(out.abs());
            out
        }

        fn reset(&mut self) {
            self.in_peak = 0.0;
            self.out_peak = 0.0;
        }

        fn set_sample_rate(&mut self, sample_rate: f64) {
            self.sample_rate = sample_rate;
        }

        fn gain_reduction(&self) -> f64 {
            1.0 - self.gain
        }

        fn input_peak(&self) -> f64 {
            self.in_peak
        }

        fn output_peak(&self) -> f64 {
            self.out_peak
        }
    }

    impl Default for TestProcessor {
        fn default() -> Self {
            Self {
                gain: 1.0,
                sample_rate: 48_000.0,
                in_peak: 0.0,
                out_peak: 0.0,
            }
        }
    }

    #[test]
    fn multichannel_graph_initializes_with_stereo_default() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        assert_eq!(graph.channel_count(), 2);
        assert_eq!(graph.routing_mode(), RoutingMode::Stereo);
        assert_eq!(graph.link_mode(), MultiChannelLinkMode::Unlinked);
    }

    #[test]
    fn multichannel_graph_configure_channel_count() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        graph.set_channel_count(6);
        assert_eq!(graph.channel_count(), 6);
        assert!(graph.routing.accepts_frame(&[0.0; 6]));
        assert!(!graph.routing.accepts_frame(&[0.0; 2]));
    }

    #[test]
    fn multichannel_graph_channel_setup_override() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        assert!(!graph.set_channel_setup(ChannelSetup::Auto));
        assert!(graph.set_channel_setup(ChannelSetup::SevenDot1));
        assert_eq!(graph.channel_count(), 8);
    }

    #[test]
    fn multichannel_graph_mono_process_0_in_0_out() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        let mut ch = [0.0_f64; 2];
        let gr = graph.process(&mut ch);
        assert_eq!(ch, [0.0, 0.0]);
        assert_eq!(gr, 0.0);
    }

    #[test]
    fn multichannel_graph_all_linked_shared_gr() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        graph.set_link_mode(MultiChannelLinkMode::AllLinked);
        // Set different gains to create different GR values
        graph.processors[0].gain = 0.5;  // GR = 0.5
        graph.processors[1].gain = 0.25; // GR = 0.75
        let mut ch = [1.0_f64, 1.0];
        let gr = graph.process(&mut ch);
        // With AllLinked: max GR = 0.75, applied on top of per-channel processing
        // ch[0] = 1.0 * 0.5 * (1 - 0.75) = 0.125
        // ch[1] = 1.0 * 0.25 * (1 - 0.75) = 0.0625
        assert!((ch[0] - 0.125).abs() < 1e-6);
        assert!((ch[1] - 0.0625).abs() < 1e-6);
        assert!((gr - 0.75).abs() < 1e-6);
    }

    #[test]
    fn multichannel_graph_surround_stem_link_excludes_lfe() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        graph.set_channel_count(6);
        graph.set_link_mode(MultiChannelLinkMode::SurroundStemLink);
        // Stem channels (0-4): GR = 0.5, LFE (5): GR = 0.99 (ignored)
        for i in 0..5 {
            graph.processors[i].gain = 0.5; // GR = 0.5
        }
        graph.processors[5].gain = 0.01; // GR = 0.99, should be ignored

        let mut ch = [1.0_f64; 6];
        let gr = graph.process(&mut ch);
        // SurroundStemLink: max GR on stem channels = 0.5, LFE ignored
        // Output: ch[i] = 1.0 * 0.5 * (1 - 0.5) = 0.25 for stem
        // LFE: ch[5] = 1.0 * 0.01 * (1 - 0.5) = 0.005
        for c in &ch[..5] {
            assert!((*c - 0.25).abs() < 1e-6);
        }
        assert!((ch[5] - 0.005).abs() < 1e-6);
        assert!((gr - 0.5).abs() < 1e-6);
    }

    #[test]
    fn multichannel_graph_mid_side_separate_detectors() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        graph.set_channel_count(2);
        graph.set_routing_mode(RoutingMode::MidSide);
        graph.set_link_mode(MultiChannelLinkMode::MidSide);
        // Input [1.0, 1.0] (L/R). After M/S encode: M=(1+1)/√2≈1.414, S=0.
        // Process with Mid/ Side gains 0.5/0.25: mid=1.414*0.5=0.707, side=0*0.25=0.
        // max_gr = max(0.5, 0.75)=0.75; apply 0.25 gain: mid=0.707*0.25=0.177, side=0.
        // Decode: L=R=(0.177)/√2=0.125
        graph.processors[0].gain = 0.5;
        graph.processors[1].gain = 0.25;
        let mut ch = [1.0_f64, 1.0];
        let gr = graph.process(&mut ch);
        assert!((gr - 0.75).abs() < 1e-6);
        assert!((ch[0] - 0.125).abs() < 1e-6);
        assert!((ch[1] - 0.125).abs() < 1e-6);
    }

    #[test]
    fn multichannel_detector_link_computes_shared_gr() {
        let procs = std::array::from_fn(|i| {
            let mut p = TestProcessor::default();
            // 5.1 stem: 0=FL, 1=FR, 2=FC, 3=SL, 4=SR (all stem, gain=0.5), 5=LFE (excluded, gain=0.1)
            p.gain = if i < 5 { 0.5 } else { 0.1 };
            p
        });
        let mut link = MultiChannelDetectorLink::new(procs);
        link.set_channel_count(6);
        link.set_link_mode(MultiChannelLinkMode::SurroundStemLink);
        let gr = link.compute_shared_gr();
        // Stem channels (0-4): GR = 0.5, LFE (5): GR = 0.9 (ignored)
        assert!((gr - 0.5).abs() < 1e-6);
    }

    #[test]
    fn multichannel_unlinked_preserves_independent_gr() {
        let mut graph = MultiChannelGraph::<TestProcessor>::default();
        graph.initialize(48_000.0);
        graph.set_channel_count(4);
        graph.set_link_mode(MultiChannelLinkMode::Unlinked);
        graph.processors[0].gain = 0.5;
        graph.processors[1].gain = 0.25;
        graph.processors[2].gain = 0.125;
        graph.processors[3].gain = 1.0;

        let mut ch = [1.0_f64; 4];
        graph.process(&mut ch);
        assert!((ch[0] - 0.5).abs() < 1e-6);
        assert!((ch[1] - 0.25).abs() < 1e-6);
        assert!((ch[2] - 0.125).abs() < 1e-6);
        assert!((ch[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn channel_setup_from_channel_count() {
        assert_eq!(ChannelSetup::from_channel_count(1), ChannelSetup::Mono);
        assert_eq!(ChannelSetup::from_channel_count(2), ChannelSetup::Stereo);
        assert_eq!(ChannelSetup::from_channel_count(6), ChannelSetup::FiveDot1);
        assert_eq!(ChannelSetup::from_channel_count(8), ChannelSetup::SevenDot1);
        assert_eq!(ChannelSetup::from_channel_count(9), ChannelSetup::Auto);
    }
}
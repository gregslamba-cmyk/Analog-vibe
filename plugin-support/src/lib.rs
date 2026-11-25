#![allow(clippy::field_reassign_with_default)]
#![allow(rustdoc::doc_list_item_overindented)]

use nih_plug::prelude::{AudioIOLayout, PortNames, new_nonzero_u32};

/// Tutti i layout di canali standard supportati (Mono → 7.1).
/// Ordine: 1=Mono, 2=Stereo, 3=LCR, 4=Quad, 5=5.0, 6=5.1/6.0, 7=7.0, 8=7.1.
const CHANNELS: [u32; 8] = [2, 1, 3, 4, 5, 6, 7, 8];

const MONO_SIDECHAIN: [std::num::NonZeroU32; 1] = [new_nonzero_u32(1)];
const STEREO_SIDECHAIN: [std::num::NonZeroU32; 1] = [new_nonzero_u32(2)];
const SIDECHAIN_NAME: &[&str] = &["Sidechain"];

const fn layout(channels: u32) -> AudioIOLayout {
    AudioIOLayout {
        main_input_channels: Some(new_nonzero_u32(channels)),
        main_output_channels: Some(new_nonzero_u32(channels)),
        ..AudioIOLayout::const_default()
    }
}

const fn layout_with_aux(
    channels: u32,
    sidechain: &'static [std::num::NonZeroU32],
) -> AudioIOLayout {
    AudioIOLayout {
        main_input_channels: Some(new_nonzero_u32(channels)),
        main_output_channels: Some(new_nonzero_u32(channels)),
        aux_input_ports: sidechain,
        names: PortNames {
            aux_inputs: SIDECHAIN_NAME,
            ..PortNames::const_default()
        },
        ..AudioIOLayout::const_default()
    }
}

/// Layout for plugin utility/surround (no sidechain) — all formats 1-8 channels.
pub const SURROUND_IO_LAYOUTS: [AudioIOLayout; 8] = [
    layout(CHANNELS[1]),
    layout(CHANNELS[0]),
    layout(CHANNELS[2]),
    layout(CHANNELS[3]),
    layout(CHANNELS[4]),
    layout(CHANNELS[5]),
    layout(CHANNELS[6]),
    layout(CHANNELS[7]),
];

/// Layout per plugin dinamici (con sidechain) — Stereo first, then Mono, Quad, 5.0, 5.1, 7.0, 7.1, each with MONO or STEREO aux sidechain.
pub const DYNAMICS_IO_LAYOUTS: [AudioIOLayout; 21] = [
    // Stereo
    layout(CHANNELS[1]),
    layout_with_aux(CHANNELS[1], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[1], &STEREO_SIDECHAIN),
    // Mono
    layout(CHANNELS[0]),
    layout_with_aux(CHANNELS[0], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[0], &STEREO_SIDECHAIN),
    // Quad
    layout(CHANNELS[3]),
    layout_with_aux(CHANNELS[3], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[3], &STEREO_SIDECHAIN),
    // 5.0
    layout(CHANNELS[4]),
    layout_with_aux(CHANNELS[4], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[4], &STEREO_SIDECHAIN),
    // 5.1
    layout(CHANNELS[5]),
    layout_with_aux(CHANNELS[5], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[5], &STEREO_SIDECHAIN),
    // 7.0
    layout(CHANNELS[6]),
    layout_with_aux(CHANNELS[6], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[6], &STEREO_SIDECHAIN),
    // 7.1
    layout(CHANNELS[7]),
    layout_with_aux(CHANNELS[7], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[7], &STEREO_SIDECHAIN),
];

pub const MAX_AUDIO_CHANNELS: usize = 128;

pub mod multichannel;
pub use multichannel::{
    ChannelSetup, MonoChannelProcessor, MultiChannelDetectorLink, MultiChannelGraph,
};
pub use core_dsp::core_dsp_modules::sidechain_processor::MultiChannelLinkMode;
use nih_plug::prelude::*;

// ============================================================================
// Inter-Plugin Bus (lock-free local telemetry bus)
// ============================================================================
// Bus telemetrico lock-free locale per visualizzare collisioni spettrali tra
// istanze e collegare `greg_nl_ch` con `greg_nl_mixer`.

use std::sync::atomic::{AtomicU64, Ordering};

/// Costante dimensione ring-buffer del bus inter-plugin (canali di telemetria).
const INTER_PLUGIN_BUS_CAPACITY: usize = 1024;

/// Canale di telemetria inter-plugin lock-free basato su ring-buffer atomico.
/// Supporta scrittura dal thread audio e lettura dal thread UI/host senza lock.
pub struct InterPluginBus {
    /// Ring buffer di valori f64 (codificati come bit-pattern u64 per atomicità).
    buffer: [AtomicU64; INTER_PLUGIN_BUS_CAPACITY],
    /// Posizione di scrittura corrente (modulo capacità).
    write_pos: AtomicU64,
    /// Posizione di lettura corrente (modulo capacità).
    read_pos: AtomicU64,
    /// Numero totale di campioni scritti (per statistiche).
    total_written: AtomicU64,
}

impl Default for InterPluginBus {
    fn default() -> Self {
        Self::new()
    }
}

impl InterPluginBus {
    /// Crea un nuovo bus inter-plugin vuoto.
    pub fn new() -> Self {
        Self {
            buffer: core::array::from_fn(|_| AtomicU64::new(0)),
            write_pos: AtomicU64::new(0),
            read_pos: AtomicU64::new(0),
            total_written: AtomicU64::new(0),
        }
    }

    /// Scrive un campione nel bus (thread-safe, lock-free).
    #[inline]
    pub fn write(&self, sample: f64) {
        let s = if sample.is_finite() { sample } else { 0.0 };
        let bits = s.to_bits();
        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) % INTER_PLUGIN_BUS_CAPACITY as u64;
        self.buffer[pos as usize].store(bits, Ordering::Relaxed);
        self.total_written.fetch_add(1, Ordering::Relaxed);
    }

    /// Legge il campione più recente dal bus (thread-safe, lock-free).
    #[inline]
    pub fn read_latest(&self) -> f64 {
        let written = self.total_written.load(Ordering::Relaxed);
        if written == 0 {
            return 0.0;
        }
        let wpos = (self.write_pos.load(Ordering::Relaxed) + INTER_PLUGIN_BUS_CAPACITY as u64 - 1)
            % INTER_PLUGIN_BUS_CAPACITY as u64;
        let bits = self.buffer[wpos as usize].load(Ordering::Relaxed);
        f64::from_bits(bits)
    }

    /// Ritorna il numero totale di campioni scritti.
    pub fn total_written(&self) -> u64 {
        self.total_written.load(Ordering::Relaxed)
    }

    /// Reset del bus (svuota buffer e contatori).
    pub fn reset(&self) {
        for slot in &self.buffer {
            slot.store(0, Ordering::Relaxed);
        }
        self.write_pos.store(0, Ordering::Relaxed);
        self.read_pos.store(0, Ordering::Relaxed);
        self.total_written.store(0, Ordering::Relaxed);
    }
}

/// Istanza globale statica del bus inter-plugin (per comunicazione tra plugin).
static INTER_PLUGIN_BUS: InterPluginBus = InterPluginBus {
    buffer: [const { AtomicU64::new(0) }; INTER_PLUGIN_BUS_CAPACITY],
    write_pos: AtomicU64::new(0),
    read_pos: AtomicU64::new(0),
    total_written: AtomicU64::new(0),
};

/// Scrive un valore di telemetria sul bus inter-plugin globale.
pub fn inter_plugin_bus_write(sample: f64) {
    INTER_PLUGIN_BUS.write(sample);
}

/// Legge l'ultimo valore di telemetria dal bus inter-plugin globale.
pub fn inter_plugin_bus_read() -> f64 {
    INTER_PLUGIN_BUS.read_latest()
}

/// Reset del bus inter-plugin globale.
pub fn inter_plugin_bus_reset() {
    INTER_PLUGIN_BUS.reset();
}

/// Source selection for sidechain input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Enum)]
pub enum SidechainSource {
    #[default]
    #[id = "internal"]
    #[name = "Internal"]
    Internal,
    #[id = "external"]
    #[name = "External"]
    External,
}

/// Parameters common to all plugins with channel setup override.
#[derive(Params)]
pub struct CommonChannelParams {
    #[id = "channel_setup"]
    pub channel_setup: EnumParam<ChannelSetup>,
}

/// Extended parameters for dynamics processors (compressors, gate, limiter).
#[derive(Params)]
pub struct DynamicsExtendedParams {
    #[id = "channel_setup"]
    pub channel_setup: EnumParam<ChannelSetup>,
    #[id = "sc_source"]
    pub sc_source: EnumParam<SidechainSource>,
    #[id = "sc_listen"]
    pub sc_listen: BoolParam,
    #[id = "sc_hpf"]
    pub sc_hpf: BoolParam,
    #[id = "sc_lpf"]
    pub sc_lpf: BoolParam,
    #[id = "channel_mode"]
    pub channel_mode: EnumParam<MultiChannelLinkMode>,
}

impl Default for CommonChannelParams {
    fn default() -> Self {
        Self {
            channel_setup: EnumParam::new("Channel Setup", ChannelSetup::Auto),
        }
    }
}

impl Default for DynamicsExtendedParams {
    fn default() -> Self {
        Self {
            channel_setup: EnumParam::new("Channel Setup", ChannelSetup::Auto),
            sc_source: EnumParam::new("SC Source", SidechainSource::Internal),
            sc_listen: BoolParam::new("SC Listen", false),
            sc_hpf: BoolParam::new("SC HPF", false),
            sc_lpf: BoolParam::new("SC LPF", false),
            channel_mode: EnumParam::new("Channel Mode", MultiChannelLinkMode::Unlinked),
        }
    }
}

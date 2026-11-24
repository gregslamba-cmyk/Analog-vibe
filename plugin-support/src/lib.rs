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

pub const MAX_AUDIO_CHANNELS: usize = 8;

pub mod multichannel;
pub use multichannel::{
    ChannelSetup, MonoChannelProcessor, MultiChannelDetectorLink, MultiChannelGraph,
};
pub use core_dsp::core_dsp_modules::sidechain_processor::MultiChannelLinkMode;

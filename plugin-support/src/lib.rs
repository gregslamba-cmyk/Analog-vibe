use nih_plug::prelude::{AudioIOLayout, PortNames, new_nonzero_u32};

const CHANNELS: [u32; 4] = [1, 2, 6, 8];
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

pub const SURROUND_IO_LAYOUTS: [AudioIOLayout; 4] = [
    layout(CHANNELS[0]),
    layout(CHANNELS[1]),
    layout(CHANNELS[2]),
    layout(CHANNELS[3]),
];

pub const DYNAMICS_IO_LAYOUTS: [AudioIOLayout; 12] = [
    layout(CHANNELS[0]),
    layout_with_aux(CHANNELS[0], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[0], &STEREO_SIDECHAIN),
    layout(CHANNELS[1]),
    layout_with_aux(CHANNELS[1], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[1], &STEREO_SIDECHAIN),
    layout(CHANNELS[2]),
    layout_with_aux(CHANNELS[2], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[2], &STEREO_SIDECHAIN),
    layout(CHANNELS[3]),
    layout_with_aux(CHANNELS[3], &MONO_SIDECHAIN),
    layout_with_aux(CHANNELS[3], &STEREO_SIDECHAIN),
];

pub const MAX_AUDIO_CHANNELS: usize = 8;

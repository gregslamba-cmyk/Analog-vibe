use std::{
    ffi::{CStr, CString, c_char, c_void},
    fs,
    mem::size_of,
    path::Path,
    ptr,
};

use clap_sys::{
    audio_buffer::clap_audio_buffer,
    entry::clap_plugin_entry,
    events::{
        CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_PARAM_VALUE, clap_event_header,
        clap_event_param_value, clap_input_events, clap_output_events,
    },
    ext::{
        audio_ports::{CLAP_EXT_AUDIO_PORTS, clap_audio_port_info, clap_plugin_audio_ports},
        audio_ports_config::{
            CLAP_EXT_AUDIO_PORTS_CONFIG, clap_audio_ports_config, clap_plugin_audio_ports_config,
        },
        latency::{CLAP_EXT_LATENCY, clap_plugin_latency},
        params::{CLAP_EXT_PARAMS, clap_param_info, clap_plugin_params},
        thread_check::{CLAP_EXT_THREAD_CHECK, clap_host_thread_check},
    },
    factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory},
    host::clap_host,
    plugin::clap_plugin,
    process::clap_process,
    version::CLAP_VERSION,
};

mod ffi_safe;

const SAMPLE_RATE: f64 = 48_000.0;
const DEFAULT_BLOCK_SIZE: usize = 256;
const SIGNAL_FRAMES: usize = 48_000;

static HOST_NAME: &CStr = c"Greg Audio Signal Test Host";
static HOST_VENDOR: &CStr = c"Greg Audio";
static HOST_URL: &CStr = c"https://example.invalid";
static HOST_VERSION: &CStr = c"0.1.0";
static SURGICAL_EQ_CLAP_ID: &CStr = c"com.greg.audio.greg-surgical-eq";
static MS_ENCODER_CLAP_ID: &CStr = c"com.greg.audio.greg-ms-encoder";
static MS_DECODER_CLAP_ID: &CStr = c"com.greg.audio.greg-ms-decoder";
static PHASE_CLAP_ID: &CStr = c"com.greg.audio.greg-phase";
static FET_CLAP_ID: &CStr = c"com.greg.audio.greg-fet";
static A2A_CLAP_ID: &CStr = c"com.greg.audio.greg-a2a";
static A3A_CLAP_ID: &CStr = c"com.greg.audio.greg-a3a";
static CHILD_CLAP_ID: &CStr = c"com.greg.audio.greg-child";
static LIMITER_CLAP_ID: &CStr = c"com.greg.audio.greg-limiter";

static HOST_THREAD_CHECK: clap_sys::ext::thread_check::clap_host_thread_check =
    clap_sys::ext::thread_check::clap_host_thread_check {
        is_main_thread: Some(clap_sys_wrapper::host::host_is_thread),
        is_audio_thread: Some(clap_sys_wrapper::host::host_is_thread),
    };

static HOST_PARAMS: clap_sys::ext::params::clap_host_params =
    clap_sys::ext::params::clap_host_params {
        rescan: Some(host_params_rescan),
        clear: Some(host_params_clear),
        request_flush: Some(host_request_flush),
    };

static HOST_LATENCY: clap_sys::ext::latency::clap_host_latency =
    clap_sys::ext::latency::clap_host_latency {
        changed: Some(host_latency_changed),
    };

static HOST: clap_host = clap_host {
    clap_version: CLAP_VERSION,
    host_data: ptr::null_mut(),
    name: HOST_NAME.as_ptr(),
    vendor: HOST_VENDOR.as_ptr(),
    url: HOST_URL.as_ptr(),
    version: HOST_VERSION.as_ptr(),
    get_extension: Some(host_get_extension),
    request_restart: Some(host_request),
    request_process: Some(host_request),
    request_callback: Some(host_request),
};

struct LoadedLibrary(*mut c_void);

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                libc::dlclose(self.0);
            }
        }
    }
}

struct PluginInstance {
    plugin: *const clap_plugin,
    active: bool,
    processing: bool,
}

impl Drop for PluginInstance {
    fn drop(&mut self) {
        // Bug fix: mai dereferenziare un plugin nullo, nemmeno in Drop.
        if self.plugin.is_null() {
            return;
        }
        unsafe {
            if self.processing
                && let Some(stop) = (*self.plugin).stop_processing
            {
                stop(self.plugin);
            }
            if self.active
                && let Some(deactivate) = (*self.plugin).deactivate
            {
                deactivate(self.plugin);
            }
            if let Some(destroy) = (*self.plugin).destroy {
                destroy(self.plugin);
            }
        }
    }
}

struct EntryGuard(*const clap_plugin_entry);

impl Drop for EntryGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(deinit) = (*self.0).deinit {
                deinit();
            }
        }
    }
}

#[derive(Clone, Copy)]
struct TestCase {
    name: &'static str,
    mode_index: f64,
    bypass: bool,
    invert_left: bool,
    boost_first_eq_band: bool,
    frequency: f64,
    amplitude: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
    Impulse,
    Noise,
    Sweep,
    Complex,
    All,
}

impl Waveform {
    fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match value {
            "sine" => Ok(Self::Sine),
            "square" => Ok(Self::Square),
            "saw" => Ok(Self::Saw),
            "triangle" => Ok(Self::Triangle),
            "impulse" => Ok(Self::Impulse),
            "noise" => Ok(Self::Noise),
            "sweep" => Ok(Self::Sweep),
            "complex" => Ok(Self::Complex),
            "all" => Ok(Self::All),
            _ => Err(format!("unsupported waveform: {value}").into()),
        }
    }

    fn sample(self, index: usize, frequency: f64, amplitude: f32) -> f32 {
        if self == Self::Complex {
            let sine = Self::Sine.sample(index, frequency, amplitude);
            let saw = Self::Saw.sample(index, frequency * 0.5, amplitude * 0.35);
            let impulse = if index % 8_000 < 24 {
                amplitude * 0.35
            } else {
                0.0
            };
            return (sine + saw + impulse).clamp(-1.0, 1.0);
        }
        let phase = if self == Self::Sweep {
            let time = index as f64 / SAMPLE_RATE;
            let end_frequency = (frequency * 32.0).min(SAMPLE_RATE * 0.45);
            std::f64::consts::TAU
                * (frequency * time + 0.5 * (end_frequency - frequency) * time * time)
        } else {
            std::f64::consts::TAU * frequency * index as f64 / SAMPLE_RATE
        };
        let cycle = phase / std::f64::consts::TAU;
        let fractional_cycle = cycle - cycle.floor();
        let value = match self {
            Self::Sine | Self::Sweep => phase.sin(),
            Self::Square => {
                if fractional_cycle < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Saw => 2.0 * fractional_cycle - 1.0,
            Self::Triangle => 1.0 - 4.0 * (fractional_cycle - 0.5).abs(),
            Self::Impulse => {
                if index.is_multiple_of(128) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Noise => {
                let mut value = index as u64 + 0x9E37_79B9_7F4A_7C15;
                value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                let value = value ^ (value >> 31);
                (value as f64 / u64::MAX as f64) * 2.0 - 1.0
            }
            Self::Complex => 0.0,
            Self::All => phase.sin(),
        };
        amplitude * value as f32
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("CLAP signal host failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_path = None;
    let mut waveform = Waveform::Sine;
    let mut block_size = DEFAULT_BLOCK_SIZE;
    let mut variable_blocks = false;
    let mut channel_count = 2_usize;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--waveform" => {
                waveform =
                    Waveform::parse(&args.next().ok_or("--waveform requires a waveform name")?)?;
            }
            "--block-size" => {
                block_size = args
                    .next()
                    .ok_or("--block-size requires a frame count")?
                    .parse()?;
                if !(1..=8192).contains(&block_size) {
                    return Err("--block-size must be between 1 and 8192".into());
                }
            }
            "--variable-blocks" => variable_blocks = true,
            "--channels" => {
                channel_count = args
                    .next()
                    .ok_or("--channels requires 1, 2, 6, or 8")?
                    .parse()?;
                if ![1, 2, 6, 8].contains(&channel_count) {
                    return Err("--channels must be 1, 2, 6, or 8".into());
                }
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown option: {value}").into());
            }
            path if plugin_path.is_none() => plugin_path = Some(path.to_owned()),
            path => return Err(format!("unexpected argument: {path}").into()),
        }
    }
    let plugin_path = plugin_path.unwrap_or_else(|| "target/release/libgreg_fet.so".to_owned());
    println!(
        "Signal: {:?}, channels: {channel_count}, max block: {block_size}, variable blocks: {variable_blocks}",
        waveform,
    );
    if !Path::new(&plugin_path).is_file() {
        return Err(format!("plugin binary not found: {plugin_path}").into());
    }

    let path = CString::new(plugin_path.as_bytes())?;
    let library = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if library.is_null() {
        let error = unsafe { CStr::from_ptr(libc::dlerror()) }.to_string_lossy();
        return Err(format!("could not load CLAP module: {error}").into());
    }
    let library = LoadedLibrary(library);

    let entry_ptr = unsafe { libc::dlsym(library.0, c"clap_entry".as_ptr()) };
    if entry_ptr.is_null() {
        return Err("module does not export clap_entry".into());
    }
    let entry = entry_ptr.cast::<clap_plugin_entry>();
    let plugin_path = path.as_ptr();
    if !unsafe { (*entry).init.expect("CLAP entry init callback missing")(plugin_path) } {
        return Err("CLAP entry init returned false".into());
    }
    let _entry_guard = EntryGuard(entry);

    let get_factory = unsafe {
        (*entry)
            .get_factory
            .expect("CLAP entry get_factory callback missing")
    };
    let factory = unsafe { get_factory(CLAP_PLUGIN_FACTORY_ID.as_ptr()) };
    if factory.is_null() {
        return Err("CLAP plugin factory is unavailable".into());
    }
    let factory = factory.cast::<clap_plugin_factory>();
    let factory_count = unsafe {
        (*factory)
            .get_plugin_count
            .expect("CLAP factory count callback missing")(factory)
    };
    if factory_count == 0 {
        return Err("CLAP module exports no plugins".into());
    }
    let descriptor = unsafe {
        (*factory)
            .get_plugin_descriptor
            .expect("CLAP descriptor callback missing")(factory, 0)
    };
    if descriptor.is_null() {
        return Err("CLAP plugin descriptor is null".into());
    }
    let plugin_id = unsafe { CStr::from_ptr((*descriptor).id) };
    println!(
        "Loaded CLAP: {} ({})",
        unsafe { CStr::from_ptr((*descriptor).name) }.to_string_lossy(),
        plugin_id.to_string_lossy()
    );

    let is_surgical_eq = plugin_id == SURGICAL_EQ_CLAP_ID;
    let is_ms_encoder = plugin_id == MS_ENCODER_CLAP_ID;
    let is_ms_decoder = plugin_id == MS_DECODER_CLAP_ID;
    let is_phase = plugin_id == PHASE_CLAP_ID;
    let is_a2a = plugin_id == A2A_CLAP_ID;
    let is_a3a = plugin_id == A3A_CLAP_ID;
    let is_child = plugin_id == CHILD_CLAP_ID;
    let is_limiter = plugin_id == LIMITER_CLAP_ID;
    let cases = if is_surgical_eq {
        vec![
            TestCase {
                name: "surgical EQ minimum-phase identity",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 440.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "surgical EQ linear-phase identity",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "surgical EQ minimum-phase first-band boost",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: true,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    } else if is_ms_encoder || is_ms_decoder {
        vec![
            TestCase {
                name: if is_ms_encoder {
                    "M/S encoder orthonormal mapping"
                } else {
                    "M/S decoder orthonormal mapping"
                },
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "M/S utility bypass",
                mode_index: 0.0,
                bypass: true,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    } else if is_phase {
        vec![
            TestCase {
                name: "phase utility left-only inversion",
                mode_index: 0.0,
                bypass: false,
                invert_left: true,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "phase utility right-only inversion",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "phase utility linked inversion",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "phase utility bypass",
                mode_index: 0.0,
                bypass: true,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
        ]
    } else if is_a2a {
        vec![
            TestCase {
                name: "opto gain reduction at 1x",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "opto gain reduction at 2x",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "opto gain reduction at 4x",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "opto gain reduction at 8x",
                mode_index: 3.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "opto external sidechain filter",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "opto compressor latency-compensated bypass",
                mode_index: 2.0,
                bypass: true,
                invert_left: true,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    } else if is_a3a {
        vec![
            TestCase {
                name: "solid-state opto gain reduction at 1x",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "solid-state opto gain reduction at 2x",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "solid-state opto gain reduction at 4x",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "solid-state opto gain reduction at 8x",
                mode_index: 3.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "solid-state limit mode external sidechain",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "solid-state opto latency-compensated bypass",
                mode_index: 2.0,
                bypass: true,
                invert_left: true,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    } else if is_child {
        vec![
            TestCase {
                name: "Greg Child time constant I",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "Greg Child time constant II",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "Greg Child time constant III",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "Greg Child time constant IV",
                mode_index: 3.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "Greg Child time constant V",
                mode_index: 4.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "Greg Child time constant VI",
                mode_index: 5.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "Greg Child linked stereo detection",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.9,
            },
            TestCase {
                name: "Greg Child dual mono detection",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.9,
            },
            TestCase {
                name: "Greg Child external sidechain listen",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    } else if is_limiter {
        vec![
            TestCase {
                name: "limiter peak ceiling at 1x",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "limiter peak ceiling at 2x",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "limiter peak ceiling at 4x",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "limiter peak ceiling at 8x",
                mode_index: 3.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "limiter latency-compensated bypass",
                mode_index: 3.0,
                bypass: true,
                invert_left: true,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    } else {
        vec![
            TestCase {
                name: "440 Hz sine at 1x",
                mode_index: 0.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 440.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "440 Hz sine at 2x",
                mode_index: 1.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 440.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "440 Hz sine at 4x",
                mode_index: 2.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 440.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "440 Hz sine at 8x",
                mode_index: 3.0,
                bypass: false,
                invert_left: false,
                boost_first_eq_band: false,
                frequency: 440.0,
                amplitude: 0.5,
            },
            TestCase {
                name: "bypassed 1 kHz sine with left polarity inversion",
                mode_index: 3.0,
                bypass: true,
                invert_left: true,
                boost_first_eq_band: false,
                frequency: 1_000.0,
                amplitude: 0.25,
            },
        ]
    };

    let waveform_suite = if waveform == Waveform::All {
        vec![
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Complex,
            Waveform::Impulse,
        ]
    } else {
        vec![waveform]
    };
    let mut results = Vec::with_capacity(cases.len() * waveform_suite.len());
    for suite_waveform in &waveform_suite {
        for case in &cases {
            results.push(run_case(
                factory,
                plugin_id,
                *case,
                *suite_waveform,
                block_size,
                variable_blocks,
                channel_count,
            )?);
        }
    }

    let expected_latencies = if is_surgical_eq {
        vec![0, 128, 0]
    } else if is_ms_encoder || is_ms_decoder {
        vec![0, 0]
    } else if is_phase {
        vec![0, 0, 0, 0]
    } else if is_a2a || is_a3a {
        vec![0, 15, 22, 25, 22, 22]
    } else if is_child {
        vec![0; 9]
    } else if is_limiter {
        vec![240, 255, 262, 265, 265]
    } else {
        vec![0, 15, 22, 25, 25]
    };
    let expected_latencies = if waveform == Waveform::All {
        let mut repeated = Vec::with_capacity(expected_latencies.len() * 4);
        for _ in 0..4 {
            repeated.extend(expected_latencies.iter().copied());
        }
        repeated
    } else {
        expected_latencies
    };
    for (result, expected_latency) in results.iter().zip(expected_latencies) {
        if result.latency != expected_latency {
            return Err(format!(
                "{} latency mismatch: expected {expected_latency}, got {}",
                result.name, result.latency
            )
            .into());
        }
    }

    if waveform != Waveform::Sine {
        for result in &results {
            if result.left_rms < 0.0001 || result.right_rms < 0.0001 {
                return Err(format!(
                    "{} produced unexpectedly low output for {:?}: RMS L/R={}/{}",
                    result.name, waveform, result.left_rms, result.right_rms
                )
                .into());
            }
        }
    } else if channel_count == 1 {
        for result in &results {
            if result.left_rms < 0.0001 {
                return Err(
                    format!("{} produced unexpectedly low mono output", result.name).into(),
                );
            }
        }
    } else if is_surgical_eq {
        for result in &results[..2] {
            if result.left_correlation < 0.98 || result.right_correlation < 0.98 {
                return Err(format!(
                    "{} identity correlation failed: L {}, R {}",
                    result.name, result.left_correlation, result.right_correlation
                )
                .into());
            }
        }
        if results[2].left_rms < 0.3 || results[2].right_rms < 0.3 {
            return Err(format!(
                "first EQ band boost too small: L {}, R {}",
                results[2].left_rms, results[2].right_rms
            )
            .into());
        }
    } else if is_ms_encoder {
        let encoded = &results[0];
        let bypass = &results[1];
        if (encoded.left_rms - 0.5).abs() > 0.01 || encoded.right_rms > 0.001 {
            return Err(format!(
                "M/S encoder mapping failed: mid RMS {}, side RMS {}",
                encoded.left_rms, encoded.right_rms
            )
            .into());
        }
        if (bypass.left_rms - bypass.right_rms).abs() > 0.001
            || bypass.left_correlation < 0.98
            || bypass.right_correlation < 0.98
        {
            return Err("M/S encoder bypass did not preserve stereo input".into());
        }
    } else if is_ms_decoder {
        let decoded = &results[0];
        let bypass = &results[1];
        if (decoded.left_rms - 0.25).abs() > 0.01
            || (decoded.right_rms - 0.25).abs() > 0.01
            || decoded.left_correlation < 0.98
            || decoded.right_correlation < 0.98
        {
            return Err(format!(
                "M/S decoder mapping failed: L/R RMS {}, {}",
                decoded.left_rms, decoded.right_rms
            )
            .into());
        }
        if bypass.left_correlation < 0.98 || bypass.right_correlation < 0.98 {
            return Err("M/S decoder bypass did not preserve M/S input".into());
        }
    } else if is_phase {
        let left_only = &results[0];
        let right_only = &results[1];
        let linked = &results[2];
        let bypass = &results[3];
        if left_only.left_correlation > -0.98 || left_only.right_correlation < 0.98 {
            return Err("left-only polarity inversion did not affect only the left channel".into());
        }
        if right_only.left_correlation < 0.98 || right_only.right_correlation > -0.98 {
            return Err(
                "right-only polarity inversion did not affect only the right channel".into(),
            );
        }
        if linked.left_correlation > -0.98 || linked.right_correlation > -0.98 {
            return Err("linked polarity inversion did not invert both channels".into());
        }
        if bypass.left_correlation < 0.98 || bypass.right_correlation < 0.98 {
            return Err("phase utility bypass did not preserve both channels".into());
        }
    } else if is_a2a {
        for compressed in &results[..5] {
            if compressed.left_rms >= 0.30 || compressed.left_rms <= 0.001 {
                return Err(format!(
                    "{} outside expected gain-reduction range: RMS {}",
                    compressed.name, compressed.left_rms
                )
                .into());
            }
        }
        let bypass = &results[5];
        if bypass.left_correlation > -0.98 || bypass.right_correlation < 0.98 {
            return Err(format!(
                "opto bypass/polarity failed: L {}, R {}",
                bypass.left_correlation, bypass.right_correlation
            )
            .into());
        }
    } else if is_a3a {
        for compressed in &results[..5] {
            if compressed.left_rms >= 0.30 || compressed.left_rms <= 0.001 {
                return Err(format!(
                    "{} outside expected gain-reduction range: RMS {}",
                    compressed.name, compressed.left_rms
                )
                .into());
            }
        }
        let bypass = &results[5];
        if bypass.left_correlation > -0.98 || bypass.right_correlation < 0.98 {
            return Err(format!(
                "A3A bypass/polarity failed: L {}, R {}",
                bypass.left_correlation, bypass.right_correlation
            )
            .into());
        }
    } else if is_child {
        for compressed in &results[..6] {
            if compressed.left_rms >= 0.35 || compressed.left_rms <= 0.001 {
                return Err(format!(
                    "{} outside expected gain-reduction range: RMS {}",
                    compressed.name, compressed.left_rms
                )
                .into());
            }
        }
        let linked_ratio = results[6].right_rms / results[6].left_rms;
        let dual_mono_ratio = results[7].right_rms / results[7].left_rms;
        if dual_mono_ratio <= linked_ratio * 1.5 {
            return Err(format!(
                "dual-mono detector did not decouple channels: linked ratio {linked_ratio}, dual-mono ratio {dual_mono_ratio}"
            )
            .into());
        }
        if (results[8].left_rms - 0.2).abs() > 0.002 {
            return Err(format!(
                "external sidechain listen produced unexpected RMS {}",
                results[8].left_rms
            )
            .into());
        }
    } else {
        let dry = &results[4];
        if dry.left_rms < 0.15 || dry.right_rms < 0.15 {
            return Err(format!(
                "bypass level too low: left RMS {}, right RMS {}",
                dry.left_rms, dry.right_rms
            )
            .into());
        }
        if dry.left_correlation > -0.98 || dry.right_correlation < 0.98 {
            return Err(format!(
                "polarity check failed: left correlation {}, right correlation {}",
                dry.left_correlation, dry.right_correlation
            )
            .into());
        }
    }

    for result in &results {
        println!(
            "PASS {:52} channels={} latency={:2} samples, channel RMS IN->OUT={:?}, output corr to ch0={:?}, input peak L/R={:.4}/{:.4}, output peak L/R={:.4}/{:.4}, input RMS L/R={:.4}/{:.4}, output RMS L/R={:.4}/{:.4}, delta RMS dB L/R={:+.2}/{:+.2}, stereo corr IN/OUT={:.3}/{:.3}, phase={:.1}deg, THD IN/OUT={:.2}/{:.2}%, centroid IN/OUT={:.1}/{:.1}Hz, est. GR={:.2}dB",
            result.name,
            result.channel_rms.len(),
            result.latency,
            result.channel_rms,
            result.channel_correlation_to_first,
            result.left_input_peak,
            result.right_input_peak,
            result.left_peak,
            result.right_peak,
            result.left_input_rms,
            result.right_input_rms,
            result.left_rms,
            result.right_rms,
            result.left_rms_difference_db,
            result.right_rms_difference_db,
            result.stereo_correlation_input,
            result.stereo_correlation_output,
            result.phase_difference_deg,
            result.thd_input,
            result.thd_output,
            result.spectral_centroid_input_hz,
            result.spectral_centroid_output_hz,
            result.estimated_gain_reduction_db
        );
    }
    write_plugin_report(
        plugin_id,
        if waveform == Waveform::All {
            "all (sine/saw/complex/impulse)"
        } else {
            match waveform {
                Waveform::Sine => "sine",
                Waveform::Square => "square",
                Waveform::Saw => "saw",
                Waveform::Triangle => "triangle",
                Waveform::Impulse => "impulse",
                Waveform::Noise => "noise",
                Waveform::Sweep => "sweep",
                Waveform::Complex => "complex",
                Waveform::All => "all",
            }
        },
        block_size,
        variable_blocks,
        &results,
    )?;
    println!("All CLAP signal-host checks passed.");
    Ok(())
}

fn write_plugin_report(
    plugin_id: &CStr,
    waveform_label: &str,
    block_size: usize,
    variable_blocks: bool,
    results: &[Metrics],
) -> Result<(), Box<dyn std::error::Error>> {
    let filename = if plugin_id == FET_CLAP_ID {
        "greg_76.txt"
    } else if plugin_id == A2A_CLAP_ID {
        "greg_a2a.txt"
    } else if plugin_id == A3A_CLAP_ID {
        "greg_a3a.txt"
    } else if plugin_id == CHILD_CLAP_ID {
        "greg_child.txt"
    } else {
        return Ok(());
    };
    let path = Path::new("docs/plugins").join(filename);
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let base = existing
        .split("## Verifica CLAP signal host")
        .next()
        .unwrap_or(&existing)
        .trim_end();

    let channel_count = results.first().map_or(0, |result| result.channel_rms.len());
    let mut report = format!(
        "{base}\n\n## Verifica CLAP signal host\n\n\
Questa sezione viene rigenerata dal host offline. Non e' una misura A/B su hardware reale: confronta il segnale d'ingresso sintetico con l'uscita del modello software.\n\n\
- Waveform/test suite: {waveform_label}\n\
- Block size massimo: {block_size} campioni\n\
- Blocchi variabili: {variable_blocks}\n\
- Canali audio: {channel_count}\n\
- Sample rate: 48 kHz\n\
\n| Test | Latenza | RMS per canale IN->OUT | Corr uscita per canale vs ch0 | Peak IN L/R | Peak OUT L/R | RMS IN L/R | RMS OUT L/R | Delta peak dB L/R | Delta RMS dB L/R | Stereo corr IN/OUT | Phase diff | THD IN/OUT % | Timbre centroid IN/OUT Hz | Est. gain reduction dB |\n\
|---|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n"
    );
    for result in results {
        report.push_str(&format!(
            "| {} | {} | {:?} | {:?} | {:.4}/{:.4} | {:.4}/{:.4} | {:.4}/{:.4} | {:.4}/{:.4} | {:+.2}/{:+.2} | {:+.2}/{:+.2} | {:.3}/{:.3} | {:.1} deg | {:.2}/{:.2} | {:.1}/{:.1} | {:.2} |\n",
            result.name,
            result.latency,
            result.channel_rms,
            result.channel_correlation_to_first,
            result.left_input_peak,
            result.right_input_peak,
            result.left_peak,
            result.right_peak,
            result.left_input_rms,
            result.right_input_rms,
            result.left_rms,
            result.right_rms,
            result.left_peak_difference_db,
            result.right_peak_difference_db,
            result.left_rms_difference_db,
            result.right_rms_difference_db,
            result.stereo_correlation_input,
            result.stereo_correlation_output,
            result.phase_difference_deg,
            result.thd_input,
            result.thd_output,
            result.spectral_centroid_input_hz,
            result.spectral_centroid_output_hz,
            result.estimated_gain_reduction_db,
        ));
    }
    fs::write(path, report)?;
    Ok(())
}

struct Metrics {
    name: &'static str,
    channel_rms: Vec<(f32, f32)>,
    channel_correlation_to_first: Vec<f64>,
    latency: u32,
    left_input_peak: f32,
    right_input_peak: f32,
    left_input_rms: f32,
    right_input_rms: f32,
    left_peak: f32,
    right_peak: f32,
    left_rms: f32,
    right_rms: f32,
    left_peak_difference_db: f64,
    right_peak_difference_db: f64,
    left_rms_difference_db: f64,
    right_rms_difference_db: f64,
    left_correlation: f64,
    right_correlation: f64,
    stereo_correlation_input: f64,
    stereo_correlation_output: f64,
    phase_difference_deg: f64,
    thd_input: f64,
    thd_output: f64,
    spectral_centroid_input_hz: f64,
    spectral_centroid_output_hz: f64,
    estimated_gain_reduction_db: f64,
}

fn run_case(
    factory: *const clap_plugin_factory,
    plugin_id: &CStr,
    case: TestCase,
    waveform: Waveform,
    block_size: usize,
    variable_blocks: bool,
    channel_count: usize,
) -> Result<Metrics, Box<dyn std::error::Error>> {
    let plugin = unsafe {
        (*factory)
            .create_plugin
            .expect("CLAP factory create callback missing")(
            factory, &HOST, plugin_id.as_ptr()
        )
    };
    if plugin.is_null() {
        return Err("CLAP factory failed to create plugin instance".into());
    }
    let mut instance = PluginInstance {
        plugin,
        active: false,
        processing: false,
    };

    let has_external_sidechain = case.name.contains("external sidechain");
    let port_configs = unsafe {
        extension::<clap_plugin_audio_ports_config>(plugin, CLAP_EXT_AUDIO_PORTS_CONFIG.as_ptr())?
    };
    let config_count = unsafe {
        port_configs
            .count
            .expect("audio-port config count callback missing")(plugin)
    };
    let get_config = port_configs
        .get
        .expect("audio-port config get callback missing");
    let mut selected_config = None;
    for index in 0..config_count {
        let mut config: clap_audio_ports_config = unsafe { std::mem::zeroed() };
        if unsafe { get_config(plugin, index, &mut config) }
            && config.has_main_input
            && config.main_input_channel_count == channel_count as u32
            && config.has_main_output
            && config.main_output_channel_count == channel_count as u32
            && config.output_port_count == 1
            && config.input_port_count == 1 + u32::from(has_external_sidechain)
        {
            selected_config = Some(config.id);
            break;
        }
    }
    let selected_config = selected_config.ok_or_else(|| {
        format!(
            "plugin has no {}-channel layout{}",
            channel_count,
            if has_external_sidechain {
                " with an external sidechain"
            } else {
                ""
            }
        )
    })?;
    if !unsafe {
        port_configs
            .select
            .expect("audio-port config select callback missing")(plugin, selected_config)
    } {
        return Err("plugin rejected the requested audio-port configuration".into());
    }
    if !unsafe { (*plugin).init.expect("plugin init callback missing")(plugin) } {
        return Err("plugin init returned false".into());
    }

    let audio_ports =
        unsafe { extension::<clap_plugin_audio_ports>(plugin, CLAP_EXT_AUDIO_PORTS.as_ptr())? };
    let input_count = unsafe {
        audio_ports
            .count
            .expect("audio ports count callback missing")(plugin, true)
    };
    let output_count = unsafe {
        audio_ports
            .count
            .expect("audio ports count callback missing")(plugin, false)
    };
    let expected_input_count = 1 + u32::from(has_external_sidechain);
    if input_count != expected_input_count || output_count != 1 {
        return Err(format!(
            "expected {expected_input_count} input port(s) and one output; got {input_count}/{output_count}"
        )
        .into());
    }

    let mut input_info = empty_audio_port_info();
    let mut output_info = empty_audio_port_info();
    let get_port = audio_ports.get.expect("audio ports get callback missing");
    if !unsafe { get_port(plugin, 0, true, &mut input_info) }
        || !unsafe { get_port(plugin, 0, false, &mut output_info) }
        || input_info.channel_count != channel_count as u32
        || output_info.channel_count != channel_count as u32
    {
        return Err(
            format!("plugin does not expose the requested {channel_count}-channel layout").into(),
        );
    }
    let mut sidechain_channel_count = 0_usize;
    if input_count == 2 {
        let mut sidechain_info = empty_audio_port_info();
        if !unsafe { get_port(plugin, 1, true, &mut sidechain_info) }
            || !(1..=2).contains(&sidechain_info.channel_count)
        {
            return Err("plugin exposes an invalid sidechain input".into());
        }
        sidechain_channel_count = sidechain_info.channel_count as usize;
    }

    let params = unsafe { extension::<clap_plugin_params>(plugin, CLAP_EXT_PARAMS.as_ptr())? };
    if plugin_id == SURGICAL_EQ_CLAP_ID {
        let parameter_count =
            unsafe { params.count.expect("parameter count callback missing")(plugin) };
        if parameter_count != 52 {
            return Err(format!(
                "ten-band surgical EQ should expose 52 parameters, got {parameter_count}"
            )
            .into());
        }
    }
    set_parameter(plugin, params, "Bypass", f64::from(case.bypass))?;
    if plugin_id == SURGICAL_EQ_CLAP_ID {
        set_parameter(plugin, params, "Phase Mode", case.mode_index)?;
        if case.boost_first_eq_band {
            set_parameter(plugin, params, "Enabled", 1.0)?;
            set_parameter(plugin, params, "Gain", 0.6875)?;
        }
    } else if plugin_id == MS_ENCODER_CLAP_ID || plugin_id == MS_DECODER_CLAP_ID {
        set_parameter(plugin, params, "Mid Trim", 0.5)?;
        set_parameter(plugin, params, "Side Trim", 0.5)?;
        set_parameter(plugin, params, "Balance", 0.5)?;
    } else if plugin_id == PHASE_CLAP_ID {
        set_parameter(
            plugin,
            params,
            "Invert Left Polarity",
            f64::from(case.invert_left),
        )?;
        set_parameter(
            plugin,
            params,
            "Invert Right Polarity",
            f64::from(case.mode_index == 1.0),
        )?;
        set_parameter(
            plugin,
            params,
            "Invert Both (Link)",
            f64::from(case.mode_index == 2.0),
        )?;
    } else if plugin_id == A2A_CLAP_ID {
        set_parameter(plugin, params, "Oversampling", case.mode_index)?;
        set_parameter(
            plugin,
            params,
            "Invert Left Polarity",
            f64::from(case.invert_left),
        )?;
        set_parameter(
            plugin,
            params,
            "Peak Reduction",
            if case.bypass {
                0.0
            } else {
                (5.0 + 30.0 * case.mode_index / 3.0) / 40.0
            },
        )?;
        set_parameter(
            plugin,
            params,
            "Gain",
            if case.bypass { 0.5 } else { 30.0 / 48.0 },
        )?;
        set_parameter(
            plugin,
            params,
            "Tube Drive",
            if case.bypass {
                0.0
            } else {
                (3.0 + 9.0 * case.mode_index / 3.0) / 12.0
            },
        )?;
        set_parameter(
            plugin,
            params,
            "External Sidechain",
            f64::from(case.name.contains("external sidechain")),
        )?;
    } else if plugin_id == A3A_CLAP_ID {
        set_parameter(plugin, params, "Oversampling", case.mode_index)?;
        set_parameter(
            plugin,
            params,
            "Invert Left Polarity",
            f64::from(case.invert_left),
        )?;
        set_parameter(
            plugin,
            params,
            "Peak Reduction",
            if case.bypass {
                0.0
            } else {
                (10.0 + 30.0 * case.mode_index / 3.0) / 40.0
            },
        )?;
        set_parameter(
            plugin,
            params,
            "Gain",
            if case.bypass { 0.5 } else { 30.0 / 48.0 },
        )?;
        set_parameter(
            plugin,
            params,
            "Output Drive",
            if case.bypass {
                0.0
            } else {
                (3.0 + 15.0 * case.mode_index / 3.0) / 18.0
            },
        )?;
        set_parameter(
            plugin,
            params,
            "External Sidechain",
            f64::from(case.name.contains("external sidechain")),
        )?;
        if case.name.contains("limit mode") {
            set_parameter(plugin, params, "Mode", 1.0)?;
        }
    } else if plugin_id == CHILD_CLAP_ID {
        set_parameter(plugin, params, "Time Constant", case.mode_index)?;
        set_parameter(
            plugin,
            params,
            "Stereo Link",
            f64::from(!case.name.contains("dual mono")),
        )?;
        set_parameter(
            plugin,
            params,
            "Invert Left Polarity",
            f64::from(case.invert_left),
        )?;
        set_parameter(
            plugin,
            params,
            "External Sidechain",
            f64::from(case.name.contains("external sidechain")),
        )?;
        set_parameter(
            plugin,
            params,
            "Sidechain Listen",
            f64::from(case.name.contains("sidechain listen")),
        )?;
    } else if plugin_id == FET_CLAP_ID {
        set_parameter(plugin, params, "Oversampling", case.mode_index)?;
        set_parameter(
            plugin,
            params,
            "Invert Left Polarity",
            f64::from(case.invert_left),
        )?;
        set_parameter(plugin, params, "Ratio", case.mode_index)?;
        set_parameter(
            plugin,
            params,
            "Output",
            if case.bypass { 0.5 } else { 30.0 / 48.0 },
        )?;
    } else {
        set_parameter(plugin, params, "Oversampling", case.mode_index)?;
        set_parameter(
            plugin,
            params,
            "Invert Left Polarity",
            f64::from(case.invert_left),
        )?;
    }

    let latency_extension =
        unsafe { extension::<clap_plugin_latency>(plugin, CLAP_EXT_LATENCY.as_ptr())? };
    let activate = unsafe {
        (*plugin)
            .activate
            .expect("plugin activate callback missing")
    };
    if !unsafe { activate(plugin, SAMPLE_RATE, 1, block_size as u32) } {
        return Err("plugin activation failed".into());
    }
    instance.active = true;
    let latency = unsafe {
        latency_extension
            .get
            .expect("latency extension get callback missing")(plugin)
    };
    let start_processing = unsafe {
        (*plugin)
            .start_processing
            .expect("plugin start_processing callback missing")
    };
    if !unsafe { start_processing(plugin) } {
        return Err("plugin start_processing failed".into());
    }
    instance.processing = true;

    let mut input_channels = vec![vec![0.0_f32; block_size]; channel_count];
    let mut sidechain_channels = vec![vec![0.0_f32; block_size]; sidechain_channel_count];
    let mut output_channels = vec![vec![0.0_f32; block_size]; channel_count];
    let mut input_channel_ptrs: Vec<_> = input_channels
        .iter_mut()
        .map(|samples| samples.as_mut_ptr())
        .collect();
    let mut sidechain_channel_ptrs: Vec<_> = sidechain_channels
        .iter_mut()
        .map(|samples| samples.as_mut_ptr())
        .collect();
    let mut output_channel_ptrs: Vec<_> = output_channels
        .iter_mut()
        .map(|samples| samples.as_mut_ptr())
        .collect();
    let input_buffer = clap_audio_buffer {
        data32: input_channel_ptrs.as_mut_ptr(),
        data64: ptr::null_mut(),
        channel_count: channel_count as u32,
        latency: 0,
        constant_mask: 0,
    };
    let mut input_buffers = vec![input_buffer];
    if sidechain_channel_count > 0 {
        input_buffers.push(clap_audio_buffer {
            data32: sidechain_channel_ptrs.as_mut_ptr(),
            data64: ptr::null_mut(),
            channel_count: sidechain_channel_count as u32,
            latency: 0,
            constant_mask: 0,
        });
    }
    let mut output_buffer = clap_audio_buffer {
        data32: output_channel_ptrs.as_mut_ptr(),
        data64: ptr::null_mut(),
        channel_count: channel_count as u32,
        latency: 0,
        constant_mask: 0,
    };
    let in_events = empty_input_events();
    let out_events = empty_output_events();
    let process = unsafe { (*plugin).process.expect("plugin process callback missing") };

    let mut left_peak = 0.0_f32;
    let mut right_peak = 0.0_f32;
    let mut left_input_peak = 0.0_f32;
    let mut right_input_peak = 0.0_f32;
    let mut left_input_square_sum = 0.0_f64;
    let mut right_input_square_sum = 0.0_f64;
    let mut left_square_sum = 0.0_f64;
    let mut right_square_sum = 0.0_f64;
    let mut left_cross = 0.0_f64;
    let mut right_cross = 0.0_f64;
    let mut left_reference_square_sum = 0.0_f64;
    let mut right_reference_square_sum = 0.0_f64;
    let mut measured_samples = 0_usize;
    let mut stereo_input_cross = 0.0_f64;
    let mut stereo_input_left_square = 0.0_f64;
    let mut stereo_input_right_square = 0.0_f64;
    let mut stereo_output_cross = 0.0_f64;
    let mut stereo_output_left_square = 0.0_f64;
    let mut stereo_output_right_square = 0.0_f64;
    let mut input_samples = Vec::with_capacity(SIGNAL_FRAMES);
    let mut output_samples = Vec::with_capacity(SIGNAL_FRAMES);
    let mut channel_input_square_sums = vec![0.0_f64; channel_count];
    let mut channel_output_square_sums = vec![0.0_f64; channel_count];
    let mut channel_output_cross_with_first = vec![0.0_f64; channel_count];

    let mut block_start = 0;
    let mut block_index = 0;
    while block_start < SIGNAL_FRAMES {
        let requested_size = if variable_blocks {
            match block_index % 3 {
                0 => block_size,
                1 => (block_size / 2).max(1),
                _ => block_size.saturating_sub(1).max(1),
            }
        } else {
            block_size
        };
        let frames = (SIGNAL_FRAMES - block_start).min(requested_size);
        for offset in 0..frames {
            let sample_index = block_start + offset;
            let sample = waveform.sample(sample_index, case.frequency, case.amplitude);
            input_channels[0][offset] = sample;
            if channel_count > 1 {
                input_channels[1][offset] = if case.name.contains("stereo phase") {
                    let phase = std::f64::consts::TAU * case.frequency * sample_index as f64
                        / SAMPLE_RATE
                        + std::f64::consts::FRAC_PI_2;
                    (phase.cos() * case.amplitude as f64).clamp(-1.0, 1.0) as f32
                } else if plugin_id == CHILD_CLAP_ID
                    && (case.name.contains("linked stereo") || case.name.contains("dual mono"))
                {
                    sample * 0.01
                } else if !case.bypass && plugin_id == MS_ENCODER_CLAP_ID {
                    sample
                } else if !case.bypass && plugin_id == MS_DECODER_CLAP_ID {
                    0.0
                } else if !case.bypass
                    && (plugin_id == FET_CLAP_ID
                        || plugin_id == A2A_CLAP_ID
                        || plugin_id == A3A_CLAP_ID)
                {
                    let phase = std::f64::consts::TAU * case.frequency * sample_index as f64
                        / SAMPLE_RATE
                        + std::f64::consts::FRAC_PI_2;
                    (phase.cos() * case.amplitude as f64).clamp(-1.0, 1.0) as f32
                } else {
                    sample
                };
            }
            #[allow(clippy::needless_range_loop)]
            for channel in 2..channel_count {
                let channel_frequency = case.frequency * (1.0 + channel as f64 * 0.11);
                input_channels[channel][offset] =
                    waveform.sample(sample_index, channel_frequency, case.amplitude * 0.7);
            }
            for (channel, sidechain) in sidechain_channels.iter_mut().enumerate() {
                sidechain[offset] = 0.2 + channel as f32 * 0.1;
            }
            for (channel, samples) in input_channels.iter().enumerate() {
                let input = samples[offset];
                channel_input_square_sums[channel] += f64::from(input).powi(2);
            }
            let right_channel = usize::from(channel_count > 1);
            left_input_peak = left_input_peak.max(input_channels[0][offset].abs());
            right_input_peak = right_input_peak.max(input_channels[right_channel][offset].abs());
            left_input_square_sum += f64::from(input_channels[0][offset]).powi(2);
            right_input_square_sum += f64::from(input_channels[right_channel][offset]).powi(2);
            for output in &mut output_channels {
                output[offset] = 0.0;
            }
        }
        let process_data = clap_process {
            steady_time: block_start as i64,
            frames_count: frames as u32,
            transport: ptr::null(),
            audio_inputs: input_buffers.as_ptr(),
            audio_outputs: &mut output_buffer,
            audio_inputs_count: input_buffers.len() as u32,
            audio_outputs_count: 1,
            in_events: &in_events,
            out_events: &out_events,
        };
        let status = unsafe { process(plugin, &process_data) };
        if !(1..=4).contains(&status) {
            return Err(format!("unexpected CLAP process status: {status}").into());
        }

        for offset in 0..frames {
            let sample_index = block_start + offset;
            let right_channel = usize::from(channel_count > 1);
            let left = output_channels[0][offset];
            let right = output_channels[right_channel][offset];
            let input_left_value = f64::from(input_channels[0][offset]);
            let input_right_value = f64::from(input_channels[right_channel][offset]);
            for channel in 0..channel_count {
                let output = output_channels[channel][offset];
                if !output.is_finite() {
                    return Err(format!(
                        "non-finite output on channel {channel} at sample {sample_index}"
                    )
                    .into());
                }
                channel_output_square_sums[channel] += f64::from(output).powi(2);
                channel_output_cross_with_first[channel] +=
                    f64::from(output) * f64::from(output_channels[0][offset]);
            }
            input_samples.push(input_left_value);
            output_samples.push(f64::from(left));
            stereo_input_cross += input_left_value * input_right_value;
            stereo_input_left_square += input_left_value * input_left_value;
            stereo_input_right_square += input_right_value * input_right_value;
            stereo_output_cross += f64::from(left) * f64::from(right);
            stereo_output_left_square += f64::from(left).powi(2);
            stereo_output_right_square += f64::from(right).powi(2);
            left_peak = left_peak.max(left.abs());
            right_peak = right_peak.max(right.abs());
            if sample_index >= latency as usize + 1_000 {
                let phase = std::f64::consts::TAU
                    * case.frequency
                    * (sample_index - latency as usize) as f64
                    / SAMPLE_RATE;
                let reference = case.amplitude as f64 * phase.sin();
                left_cross += left as f64 * reference;
                right_cross += right as f64 * reference;
                left_reference_square_sum += reference * reference;
                right_reference_square_sum += reference * reference;
                left_square_sum += (left as f64).powi(2);
                right_square_sum += (right as f64).powi(2);
                measured_samples += 1;
            }
        }
        block_start += frames;
        block_index += 1;
    }

    if measured_samples == 0 {
        return Err("no steady-state samples available for metrics".into());
    }
    let normalization = measured_samples as f64;
    let left_rms = (left_square_sum / normalization).sqrt() as f32;
    let right_rms = (right_square_sum / normalization).sqrt() as f32;
    let left_input_rms = (left_input_square_sum / SIGNAL_FRAMES as f64).sqrt() as f32;
    let right_input_rms = (right_input_square_sum / SIGNAL_FRAMES as f64).sqrt() as f32;
    let db_difference = |input: f64, output: f64| {
        20.0 * (output / input).max(1e-10).log10()
    };
    let left_peak_difference_db = db_difference(f64::from(left_input_peak), f64::from(left_peak));
    let right_peak_difference_db =
        db_difference(f64::from(right_input_peak), f64::from(right_peak));
    let left_rms_difference_db = db_difference(f64::from(left_input_rms), f64::from(left_rms));
    let right_rms_difference_db = db_difference(f64::from(right_input_rms), f64::from(right_rms));
    let left_correlation = left_cross
        / (left_square_sum * left_reference_square_sum)
            .sqrt()
            .max(f64::MIN_POSITIVE);
    let right_correlation = right_cross
        / (right_square_sum * right_reference_square_sum)
            .sqrt()
            .max(f64::MIN_POSITIVE);
    let stereo_correlation = |left_square: f64, right_square: f64, cross: f64| {
        cross / (left_square * right_square).sqrt().max(f64::MIN_POSITIVE)
    };
    let stereo_correlation_input = stereo_correlation(
        stereo_input_left_square,
        stereo_input_right_square,
        stereo_input_cross,
    );
    let stereo_correlation_output = stereo_correlation(
        stereo_output_left_square,
        stereo_output_right_square,
        stereo_output_cross,
    );
    let phase_difference_deg = (stereo_correlation_output.clamp(-1.0, 1.0).acos() * 180.0
        / std::f64::consts::PI)
        .clamp(0.0, 180.0);
    let thd_input = waveform_thd(&input_samples, case.frequency);
    let thd_output = waveform_thd(&output_samples, case.frequency);
    let spectral_centroid_input_hz = spectral_centroid(&input_samples, case.frequency);
    let spectral_centroid_output_hz = spectral_centroid(&output_samples, case.frequency);
    let estimated_gain_reduction_db =
        (-(left_rms_difference_db + right_rms_difference_db) * 0.5).max(0.0);
    let channel_rms = channel_input_square_sums
        .iter()
        .zip(&channel_output_square_sums)
        .map(|(input, output)| {
            (
                (input / SIGNAL_FRAMES as f64).sqrt() as f32,
                (output / SIGNAL_FRAMES as f64).sqrt() as f32,
            )
        })
        .collect();
    let channel_correlation_to_first = channel_output_cross_with_first
        .iter()
        .enumerate()
        .map(|(channel, cross)| {
            cross
                / (channel_output_square_sums[channel] * channel_output_square_sums[0])
                    .sqrt()
                    .max(f64::MIN_POSITIVE)
        })
        .collect();

    drop(instance);
    Ok(Metrics {
        name: case.name,
        channel_rms,
        channel_correlation_to_first,
        latency,
        left_input_peak,
        right_input_peak,
        left_input_rms,
        right_input_rms,
        left_peak,
        right_peak,
        left_rms,
        right_rms,
        left_peak_difference_db,
        right_peak_difference_db,
        left_rms_difference_db,
        right_rms_difference_db,
        left_correlation,
        right_correlation,
        stereo_correlation_input,
        stereo_correlation_output,
        phase_difference_deg,
        thd_input,
        thd_output,
        spectral_centroid_input_hz,
        spectral_centroid_output_hz,
        estimated_gain_reduction_db,
    })
}

fn goertzel_magnitude(samples: &[f64], frequency_hz: f64) -> f64 {
    if samples.is_empty() || !frequency_hz.is_finite() || frequency_hz <= 0.0 {
        return 0.0;
    }
    let coefficient = 2.0 * (std::f64::consts::TAU * frequency_hz / SAMPLE_RATE).cos();
    let mut s1 = 0.0;
    let mut s2 = 0.0;
    for sample in samples {
        let s0 = *sample + coefficient * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coefficient * s1 * s2).sqrt().max(0.0)
}

fn waveform_thd(samples: &[f64], fundamental_hz: f64) -> f64 {
    let fundamental = goertzel_magnitude(samples, fundamental_hz);
    if fundamental <= f64::MIN_POSITIVE {
        return 0.0;
    }
    let mut harmonic_power = 0.0;
    for harmonic in 2..=5 {
        let magnitude = goertzel_magnitude(samples, fundamental_hz * harmonic as f64);
        harmonic_power += magnitude * magnitude;
    }
    (harmonic_power.sqrt() / fundamental * 100.0).min(1000.0)
}

fn spectral_centroid(samples: &[f64], fundamental_hz: f64) -> f64 {
    let mut weighted = 0.0;
    let mut weight_sum = 0.0;
    for harmonic in 1..=10 {
        let frequency = fundamental_hz * harmonic as f64;
        if frequency >= SAMPLE_RATE * 0.45 {
            break;
        }
        let magnitude = goertzel_magnitude(samples, frequency);
        weighted += frequency * magnitude;
        weight_sum += magnitude;
    }
    if weight_sum > f64::MIN_POSITIVE {
        weighted / weight_sum
    } else {
        0.0
    }
}

fn set_parameter(
    plugin: *const clap_plugin,
    params: clap_plugin_params,
    parameter_name: &str,
    value: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let count = unsafe { params.count.expect("parameter count callback missing")(plugin) };
    let get_info = params.get_info.expect("parameter info callback missing");
    let mut parameter_id = None;
    for index in 0..count {
        let mut info = empty_param_info();
        if unsafe { get_info(plugin, index, &mut info) } {
            let name = unsafe { CStr::from_ptr(info.name.as_ptr()) }.to_string_lossy();
            if name == parameter_name {
                parameter_id = Some(info.id);
                break;
            }
        }
    }
    let parameter_id =
        parameter_id.ok_or_else(|| format!("plugin parameter not found: {parameter_name}"))?;

    let event = clap_event_param_value {
        header: clap_event_header {
            size: size_of::<clap_event_param_value>() as u32,
            time: 0,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            type_: CLAP_EVENT_PARAM_VALUE,
            flags: 0,
        },
        param_id: parameter_id,
        cookie: ptr::null_mut(),
        note_id: -1,
        port_index: -1,
        channel: -1,
        key: -1,
        value,
    };
    let events = clap_input_events {
        ctx: (&event as *const clap_event_param_value).cast_mut().cast(),
        size: Some(input_event_size),
        get: Some(input_event_get),
    };
    let output_events = empty_output_events();
    unsafe {
        params.flush.expect("parameter flush callback missing")(plugin, &events, &output_events);
    }
    let mut current_value = 0.0;
    if let Some(get_value) = params.get_value
        && unsafe { get_value(plugin, parameter_id, &mut current_value) }
        && (current_value - value).abs() > 1.0e-6
    {
        return Err(format!(
            "parameter {parameter_name} did not accept {value}; current value is {current_value}"
        )
        .into());
    }
    Ok(())
}

unsafe fn extension<T: Copy>(
    plugin: *const clap_plugin,
    id: *const c_char,
) -> Result<T, Box<dyn std::error::Error>> {
    let extension = unsafe {
        (*plugin)
            .get_extension
            .expect("plugin get_extension callback missing")(plugin, id)
    };
    if extension.is_null() {
        return Err(format!(
            "plugin does not expose extension {}",
            unsafe { CStr::from_ptr(id) }.to_string_lossy()
        )
        .into());
    }
    Ok(unsafe { *extension.cast::<T>() })
}

fn empty_audio_port_info() -> clap_audio_port_info {
    clap_audio_port_info {
        id: 0,
        name: [0; clap_sys::string_sizes::CLAP_NAME_SIZE],
        flags: 0,
        channel_count: 0,
        port_type: ptr::null(),
        in_place_pair: u32::MAX,
    }
}

fn empty_param_info() -> clap_param_info {
    clap_param_info {
        id: 0,
        flags: 0,
        cookie: ptr::null_mut(),
        name: [0; clap_sys::string_sizes::CLAP_NAME_SIZE],
        module: [0; clap_sys::string_sizes::CLAP_PATH_SIZE],
        min_value: 0.0,
        max_value: 0.0,
        default_value: 0.0,
    }
}

fn empty_input_events() -> clap_input_events {
    clap_input_events {
        ctx: ptr::null_mut(),
        size: Some(no_input_events),
        get: Some(no_input_event),
    }
}

fn empty_output_events() -> clap_output_events {
    clap_output_events {
        ctx: ptr::null_mut(),
        try_push: Some(accept_output_event),
    }
}

unsafe extern "C" fn host_get_extension(
    _host: *const clap_host,
    extension_id: *const c_char,
) -> *const c_void {
    if extension_id.is_null() {
        return ptr::null();
    }
    let extension_id = unsafe { CStr::from_ptr(extension_id) };
    if extension_id == CLAP_EXT_THREAD_CHECK {
        (&HOST_THREAD_CHECK as *const clap_host_thread_check).cast()
    } else if extension_id == CLAP_EXT_PARAMS {
        (&HOST_PARAMS as *const clap_sys::ext::params::clap_host_params).cast()
    } else if extension_id == CLAP_EXT_LATENCY {
        (&HOST_LATENCY as *const clap_sys::ext::latency::clap_host_latency).cast()
    } else {
        ptr::null()
    }
}

unsafe extern "C" fn host_is_thread(_host: *const clap_host) -> bool {
    true
}

unsafe extern "C" fn host_request(_host: *const clap_host) {}

unsafe extern "C" fn host_latency_changed(_host: *const clap_host) {}

unsafe extern "C" fn host_params_rescan(_host: *const clap_host, _flags: u32) {}

unsafe extern "C" fn host_params_clear(_host: *const clap_host, _parameter_id: u32, _flags: u32) {}

unsafe extern "C" fn host_request_flush(_host: *const clap_host) {}

unsafe extern "C" fn no_input_events(_events: *const clap_input_events) -> u32 {
    0
}

unsafe extern "C" fn no_input_event(
    _events: *const clap_input_events,
    _index: u32,
) -> *const clap_event_header {
    ptr::null()
}

unsafe extern "C" fn input_event_size(_events: *const clap_input_events) -> u32 {
    1
}

unsafe extern "C" fn input_event_get(
    events: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    if index != 0 || events.is_null() {
        return ptr::null();
    }
    let context = unsafe { (*events).ctx };
    context.cast::<clap_event_param_value>().cast()
}

unsafe extern "C" fn accept_output_event(
    _events: *const clap_output_events,
    _event: *const clap_event_header,
) -> bool {
    true
}

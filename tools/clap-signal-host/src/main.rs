use std::{
    ffi::{CStr, CString, c_char, c_void},
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

const SAMPLE_RATE: f64 = 48_000.0;
const BLOCK_SIZE: usize = 256;
const SIGNAL_FRAMES: usize = 48_000;

static HOST_NAME: &CStr = c"Greg Audio Signal Test Host";
static HOST_VENDOR: &CStr = c"Greg Audio";
static HOST_URL: &CStr = c"https://example.invalid";
static HOST_VERSION: &CStr = c"0.1.0";
static SURGICAL_EQ_CLAP_ID: &CStr = c"com.greg.audio.greg-surgical-eq";
static MS_ENCODER_CLAP_ID: &CStr = c"com.greg.audio.greg-ms-encoder";
static MS_DECODER_CLAP_ID: &CStr = c"com.greg.audio.greg-ms-decoder";
static PHASE_CLAP_ID: &CStr = c"com.greg.audio.greg-phase";

static HOST_THREAD_CHECK: clap_host_thread_check = clap_host_thread_check {
    is_main_thread: Some(host_is_thread),
    is_audio_thread: Some(host_is_thread),
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

fn main() {
    if let Err(error) = run() {
        eprintln!("CLAP signal host failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let plugin_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/release/libgreg_fet.so".to_owned());
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

    let mut results = Vec::with_capacity(cases.len());
    for case in cases {
        results.push(run_case(factory, plugin_id, case)?);
    }

    let expected_latencies = if is_surgical_eq {
        vec![0, 128, 0]
    } else if is_ms_encoder || is_ms_decoder {
        vec![0, 0]
    } else if is_phase {
        vec![0, 0, 0, 0]
    } else {
        vec![0, 15, 22, 25, 25]
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

    if is_surgical_eq {
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

    for result in results {
        println!(
            "PASS {:52} latency={:2} samples, peak L/R={:.4}/{:.4}, RMS L/R={:.4}/{:.4}",
            result.name,
            result.latency,
            result.left_peak,
            result.right_peak,
            result.left_rms,
            result.right_rms
        );
    }
    println!("All CLAP signal-host checks passed.");
    Ok(())
}

struct Metrics {
    name: &'static str,
    latency: u32,
    left_peak: f32,
    right_peak: f32,
    left_rms: f32,
    right_rms: f32,
    left_correlation: f64,
    right_correlation: f64,
}

fn run_case(
    factory: *const clap_plugin_factory,
    plugin_id: &CStr,
    case: TestCase,
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
    if input_count != 1 || output_count != 1 {
        return Err(format!(
            "expected one main input and output, got {input_count}/{output_count}"
        )
        .into());
    }

    let mut input_info = empty_audio_port_info();
    let mut output_info = empty_audio_port_info();
    let get_port = audio_ports.get.expect("audio ports get callback missing");
    if !unsafe { get_port(plugin, 0, true, &mut input_info) }
        || !unsafe { get_port(plugin, 0, false, &mut output_info) }
        || input_info.channel_count != 2
        || output_info.channel_count != 2
    {
        return Err("plugin does not expose the expected stereo layout".into());
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
    if !unsafe { activate(plugin, SAMPLE_RATE, 1, BLOCK_SIZE as u32) } {
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

    let mut input_left = vec![0.0_f32; BLOCK_SIZE];
    let mut input_right = vec![0.0_f32; BLOCK_SIZE];
    let mut output_left = vec![0.0_f32; BLOCK_SIZE];
    let mut output_right = vec![0.0_f32; BLOCK_SIZE];
    let mut input_channels = [input_left.as_mut_ptr(), input_right.as_mut_ptr()];
    let mut output_channels = [output_left.as_mut_ptr(), output_right.as_mut_ptr()];
    let input_buffer = clap_audio_buffer {
        data32: input_channels.as_mut_ptr(),
        data64: ptr::null_mut(),
        channel_count: 2,
        latency: 0,
        constant_mask: 0,
    };
    let mut output_buffer = clap_audio_buffer {
        data32: output_channels.as_mut_ptr(),
        data64: ptr::null_mut(),
        channel_count: 2,
        latency: 0,
        constant_mask: 0,
    };
    let in_events = empty_input_events();
    let out_events = empty_output_events();
    let process = unsafe { (*plugin).process.expect("plugin process callback missing") };

    let mut left_peak = 0.0_f32;
    let mut right_peak = 0.0_f32;
    let mut left_square_sum = 0.0_f64;
    let mut right_square_sum = 0.0_f64;
    let mut left_cross = 0.0_f64;
    let mut right_cross = 0.0_f64;
    let mut left_reference_square_sum = 0.0_f64;
    let mut right_reference_square_sum = 0.0_f64;
    let mut measured_samples = 0_usize;

    for block_start in (0..SIGNAL_FRAMES).step_by(BLOCK_SIZE) {
        let frames = (SIGNAL_FRAMES - block_start).min(BLOCK_SIZE);
        for offset in 0..frames {
            let sample_index = block_start + offset;
            let phase = std::f64::consts::TAU * case.frequency * sample_index as f64 / SAMPLE_RATE;
            let sample = case.amplitude * phase.sin() as f32;
            input_left[offset] = sample;
            input_right[offset] = if !case.bypass && plugin_id == MS_ENCODER_CLAP_ID {
                sample
            } else if !case.bypass && plugin_id == MS_DECODER_CLAP_ID {
                0.0
            } else {
                sample
            };
            output_left[offset] = 0.0;
            output_right[offset] = 0.0;
        }
        let process_data = clap_process {
            steady_time: block_start as i64,
            frames_count: frames as u32,
            transport: ptr::null(),
            audio_inputs: &input_buffer,
            audio_outputs: &mut output_buffer,
            audio_inputs_count: 1,
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
            let left = output_left[offset];
            let right = output_right[offset];
            if !left.is_finite() || !right.is_finite() {
                return Err(format!("non-finite output at sample {sample_index}").into());
            }
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
    }

    if measured_samples == 0 {
        return Err("no steady-state samples available for metrics".into());
    }
    let normalization = measured_samples as f64;
    let left_rms = (left_square_sum / normalization).sqrt() as f32;
    let right_rms = (right_square_sum / normalization).sqrt() as f32;
    let left_correlation = left_cross
        / (left_square_sum * left_reference_square_sum)
            .sqrt()
            .max(f64::MIN_POSITIVE);
    let right_correlation = right_cross
        / (right_square_sum * right_reference_square_sum)
            .sqrt()
            .max(f64::MIN_POSITIVE);

    drop(instance);
    Ok(Metrics {
        name: case.name,
        latency,
        left_peak,
        right_peak,
        left_rms,
        right_rms,
        left_correlation,
        right_correlation,
    })
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

#![allow(unsafe_code)]
// FFI boundary crate: tutte le funzioni `unsafe extern "C"` sono callback CLAP
// invocate dal plugin/host secondo l'ABI C. Il contratto di sicurezza è
// "il chiamante deve passare puntatori validi conformi all'ABI CLAP", quindi
// la documentazione `# Safety` di clippy non è applicabile a questi shim.
#![allow(clippy::missing_safety_doc)]

/// Wrapper crate for clap-sys FFI functions with unsafe code isolation.
/// Main crates should depend on this crate instead of directly on clap-sys for FFI operations.
pub mod ffi {
    pub use clap_sys::{
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
            latency::{CLAP_EXT_LATENCY, clap_plugin_latency, clap_host_latency},
            params::{CLAP_EXT_PARAMS, clap_param_info, clap_plugin_params, clap_host_params},
            thread_check::{CLAP_EXT_THREAD_CHECK, clap_host_thread_check},
        },
        factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory},
        host::clap_host,
        plugin::clap_plugin,
        process::clap_process,
        version::CLAP_VERSION,
        stream::{clap_istream, clap_ostream},
        string_sizes::{CLAP_NAME_SIZE, CLAP_PATH_SIZE},
    };
}

/// Host-side static data and unsafe callback implementations used by
/// `clap-signal-host` and `clap-validator-host`.
///
/// All FFI‑unsafe code lives here so that consumer crates stay free of
/// `unsafe extern "C"` definitions.
pub mod host {
    use std::ffi::{c_char, c_void, CStr};
    use std::ptr;

    use super::ffi::*;
    use clap_sys::ext::latency::clap_host_latency;
    use clap_sys::ext::params::clap_host_params;
    use clap_sys::ext::thread_check::clap_host_thread_check;

    // -----------------------------------------------------------------------
    // Host identity
    // -----------------------------------------------------------------------
    pub static HOST_NAME: &CStr = c"Greg Audio Signal Test Host";
    pub static HOST_VENDOR: &CStr = c"Greg Audio";
    pub static HOST_URL: &CStr = c"https://example.invalid";
    pub static HOST_VERSION: &CStr = c"0.1.0";

    // -----------------------------------------------------------------------
    // Thread‑check extension
    // -----------------------------------------------------------------------
    pub static HOST_THREAD_CHECK: clap_host_thread_check = clap_host_thread_check {
        is_main_thread: Some(host_is_thread),
        is_audio_thread: Some(host_is_thread),
    };

    // -----------------------------------------------------------------------
    // Params extension
    // -----------------------------------------------------------------------
    pub static HOST_PARAMS: clap_host_params = clap_host_params {
        rescan: Some(host_params_rescan),
        clear: Some(host_params_clear),
        request_flush: Some(host_request_flush),
    };

    // -----------------------------------------------------------------------
    // Latency extension
    // -----------------------------------------------------------------------
    pub static HOST_LATENCY: clap_host_latency = clap_host_latency {
        changed: Some(host_latency_changed),
    };

    // -----------------------------------------------------------------------
    // Unsafe extern “C” callbacks
    // -----------------------------------------------------------------------
    pub unsafe extern "C" fn host_get_extension(
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
            (&HOST_PARAMS as *const clap_host_params).cast()
        } else if extension_id == CLAP_EXT_LATENCY {
            (&HOST_LATENCY as *const clap_host_latency).cast()
        } else {
            ptr::null()
        }
    }

    pub unsafe extern "C" fn host_is_thread(_host: *const clap_host) -> bool {
        true
    }

    pub unsafe extern "C" fn host_request(_host: *const clap_host) {}

    pub unsafe extern "C" fn host_latency_changed(_host: *const clap_host) {}

    pub unsafe extern "C" fn host_params_rescan(_host: *const clap_host, _flags: u32) {}

    pub unsafe extern "C" fn host_params_clear(_host: *const clap_host, _parameter_id: u32, _flags: u32) {}

    pub unsafe extern "C" fn host_request_flush(_host: *const clap_host) {}

    // No‑input events helpers (minimal stubs kept for hosts that need them)
    pub unsafe extern "C" fn no_input_events(_events: *const clap_input_events) -> u32 {
        0
    }

    pub unsafe extern "C" fn no_input_event(
        _events: *const clap_input_events,
        _index: u32,
    ) -> *const clap_event_header {
        ptr::null()
    }

    pub unsafe extern "C" fn input_event_size(_events: *const clap_input_events) -> u32 {
        1
    }

    pub unsafe extern "C" fn input_event_get(
        events: *const clap_input_events,
        index: u32,
    ) -> *const clap_event_header {
        if index != 0 || events.is_null() {
            return ptr::null();
        }
        let context = unsafe { (*events).ctx };
        context.cast::<clap_event_param_value>().cast()
    }

    pub unsafe extern "C" fn accept_output_event(
        _events: *const clap_output_events,
        _event: *const clap_event_header,
    ) -> bool {
        true
    }
}

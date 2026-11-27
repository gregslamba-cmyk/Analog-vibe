//! # FFI Safe Wrappers per CLAP
//!
//! Questo modulo incapsula tutte le chiamate unsafe FFI verso la libreria CLAP,
//! fornendo wrapper safe che gestiscono:
//! - Null checks per puntatori
//! - RAII guards per risorse (Drop)
//! - Gestione errori
//!
//! # Filosofia
//!
//! Ogni funzione espone una interfaccia safe. Le invarianti unsafe sono
//! documentate e mantenute internamente al modulo.

#![allow(dead_code)]

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;
use std::ptr;

use libc;

use clap_sys::{
    entry::clap_plugin_entry,
    ext::{
        audio_ports::{CLAP_EXT_AUDIO_PORTS, clap_audio_port_info, clap_plugin_audio_ports},
        params::{CLAP_EXT_PARAMS, clap_plugin_params},
    },
    factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory},
    host::clap_host,
    plugin::clap_plugin,
    process::clap_process,
};

// ============================================================================
// LIBRARY LOADER
// ============================================================================

/// RAII wrapper per una libreria CLAP caricata.
///
/// Scarica la libreria al drop se il handle non è null.
pub struct LoadedLibrary {
    handle: *mut c_void,
    path: String,
}

impl LoadedLibrary {
    /// Carica una libreria CLAP dal path specificato.
    pub fn load(path: &Path) -> Result<Self, String> {
        let resolved = if path.as_os_str() == "current_exe" {
            std::env::current_exe().map_err(|e| format!("cannot resolve current executable: {e}"))?
        } else {
            path.to_path_buf()
        };

        let c_path = CString::new(resolved.as_os_str().to_string_lossy().as_bytes())
            .map_err(|e| format!("invalid path C string: {e}"))?;

        let handle = unsafe { libc::dlopen(c_path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        if handle.is_null() {
            return Err(last_dl_error("dlopen"));
        }

        Ok(Self {
            handle,
            path: resolved.to_string_lossy().into_owned(),
        })
    }

    /// Restituisce l'handle raw per dlsym.
    pub fn as_ptr(&self) -> *mut c_void {
        self.handle
    }

    /// Path della libreria caricata.
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { libc::dlclose(self.handle); }
            self.handle = ptr::null_mut();
        }
    }
}

fn last_dl_error(context: &str) -> String {
    unsafe {
        let err = libc::dlerror();
        if err.is_null() {
            format!("{context}: unknown dynamic loader error")
        } else {
            format!("{context}: {}", CStr::from_ptr(err).to_string_lossy())
        }
    }
}

// ============================================================================
// PLUGIN ENTRY POINT
// ============================================================================

/// Ottiene il puntatore a `clap_plugin_entry` dal simbolo `clap_entry`.
pub fn safe_get_plugin_entry(lib: &LoadedLibrary) -> Result<*const clap_plugin_entry, String> {
    let symbol = unsafe { libc::dlsym(lib.as_ptr(), c"clap_entry".as_ptr()) };
    if symbol.is_null() {
        return Err("module does not export clap_entry".to_string());
    }
    Ok(symbol.cast::<clap_plugin_entry>())
}

/// RAII guard per l'entry point CLAP: chiama `deinit` al drop se presente.
pub struct EntryGuard(*const clap_plugin_entry);

impl EntryGuard {
    /// Inizializza l'entry point.
    ///
    /// Chiama `entry.init(plugin_path)` se presente e verifica il risultato.
    pub fn init(entry: *const clap_plugin_entry, plugin_path: *const c_char) -> Result<Self, String> {
        if entry.is_null() {
            return Err("null clap_plugin_entry".into());
        }
        let init_fn = unsafe { (*entry).init }.ok_or("clap_plugin_entry::init is null")?;
        if !unsafe { init_fn(plugin_path) } {
            return Err("CLAP entry init returned false".into());
        }
        Ok(Self(entry))
    }

    /// Ottiene una factory dal plugin entry.
    pub fn get_factory(&self, factory_id: *const c_char) -> Result<*const clap_plugin_factory, String> {
        if self.0.is_null() {
            return Err("EntryGuard holds null entry".into());
        }
        let get_factory = unsafe { (*self.0).get_factory }.ok_or("clap_plugin_entry::get_factory is null")?;
        let factory = unsafe { get_factory(factory_id) };
        if factory.is_null() {
            return Err(format!(
                "get_factory returned null for factory id {}",
                unsafe { CStr::from_ptr(factory_id) }.to_string_lossy()
            ));
        }
        Ok(factory.cast::<clap_plugin_factory>())
    }
}

impl Drop for EntryGuard {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        unsafe {
            if let Some(deinit) = (*self.0).deinit {
                deinit();
            }
        }
    }
}

// ============================================================================
// PLUGIN FACTORY / INSTANCE
// ============================================================================

/// Crea un'istanza plugin tramite la factory.
pub fn safe_create_plugin(
    factory: *const clap_plugin_factory,
    host: *const clap_host,
    plugin_id: &CStr,
) -> Result<*const clap_plugin, String> {
    if factory.is_null() {
        return Err("null clap_plugin_factory".into());
    }
    let create_fn = unsafe { (*factory).create_plugin }.ok_or("clap_plugin_factory::create_plugin is null")?;
    let plugin = unsafe { create_fn(factory, host, plugin_id.as_ptr()) };
    if plugin.is_null() {
        return Err(format!(
            "create_plugin returned null for {}",
            plugin_id.to_string_lossy()
        ));
    }
    Ok(plugin)
}

/// Inizializza il plugin.
pub fn safe_plugin_init(plugin: *const clap_plugin) -> Result<(), String> {
    if plugin.is_null() {
        return Err("null clap_plugin".into());
    }
    let init_fn = unsafe { (*plugin).init }.ok_or("clap_plugin::init is null")?;
    if !unsafe { init_fn(plugin) } {
        return Err("clap_plugin::init returned false".into());
    }
    Ok(())
}

/// Attiva il plugin.
pub fn safe_plugin_activate(
    plugin: *const clap_plugin,
    sample_rate: f64,
    min_frames: u32,
    max_frames: u32,
) -> Result<(), String> {
    if plugin.is_null() {
        return Err("null clap_plugin".into());
    }
    let activate_fn = unsafe { (*plugin).activate }.ok_or("clap_plugin::activate is null")?;
    if !unsafe { activate_fn(plugin, sample_rate, min_frames, max_frames) } {
        return Err("clap_plugin::activate returned false".into());
    }
    Ok(())
}

/// Avvia il processing del plugin.
pub fn safe_plugin_start_processing(plugin: *const clap_plugin) -> Result<(), String> {
    if plugin.is_null() {
        return Err("null clap_plugin".into());
    }
    let Some(start) = (unsafe { (*plugin).start_processing }) else {
        return Err("clap_plugin::start_processing is null".into());
    };
    if !unsafe { start(plugin) } {
        return Err("clap_plugin::start_processing returned false".into());
    }
    Ok(())
}

/// Esegue il processing del plugin.
pub fn safe_plugin_process(plugin: *const clap_plugin, process: *const clap_process) -> Result<(), String> {
    if plugin.is_null() {
        return Err("null clap_plugin".into());
    }
    let process_fn = unsafe { (*plugin).process }.ok_or("clap_plugin::process is null")?;
    unsafe { process_fn(plugin, process) };
    Ok(())
}

/// Ottiene l'estensione audio ports.
pub fn safe_plugin_audio_ports(plugin: *const clap_plugin) -> Result<*const clap_plugin_audio_ports, String> {
    if plugin.is_null() {
        return Err("null clap_plugin".into());
    }
    let get_ext = unsafe { (*plugin).get_extension }.ok_or("clap_plugin::get_extension is null")?;
    let ext = unsafe { get_ext(plugin, CLAP_EXT_AUDIO_PORTS.as_ptr()) };
    if ext.is_null() {
        return Err("plugin does not expose clap.audio-ports".into());
    }
    Ok(ext.cast::<clap_plugin_audio_ports>())
}

/// Ottiene info su un audio port specifico.
pub fn safe_plugin_audio_ports_get(
    ports: *const clap_plugin_audio_ports,
    plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {
    if ports.is_null() || plugin.is_null() || info.is_null() {
        return false;
    }
    let get_fn = unsafe { (*ports).get };
    if get_fn.is_none() {
        return false;
    }
    unsafe { get_fn.unwrap()(plugin, index, is_input, info) }
}

/// Ottiene l'estensione params.
pub fn safe_plugin_params(plugin: *const clap_plugin) -> Option<*const clap_plugin_params> {
    if plugin.is_null() {
        return None;
    }
    let get_ext = unsafe { (*plugin).get_extension }?;
    let ext = unsafe { get_ext(plugin, CLAP_EXT_PARAMS.as_ptr()) };
    if ext.is_null() {
        return None;
    }
    Some(ext.cast::<clap_plugin_params>())
}

/// Distrugge l'istanza plugin.
pub fn safe_plugin_destroy(plugin: *const clap_plugin) {
    if plugin.is_null() {
        return;
    }
    if let Some(destroy) = unsafe { (*plugin).destroy } {
        unsafe { destroy(plugin) };
    }
}

/// Deinizializza l'entry point (chiamato da `EntryGuard::drop`).
pub fn safe_entry_deinit(entry: *const clap_plugin_entry) {
    if entry.is_null() {
        return;
    }
    if let Some(deinit) = unsafe { (*entry).deinit } {
        unsafe { deinit() };
    }
}

// ============================================================================
// UTILITY
// ============================================================================

/// Converte un C string pointer in una Rust string, se valido.
pub unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}
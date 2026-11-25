//! greg_plugin_host — Multi-format plugin hosting crate.
//!
//! Provides a uniform [`HostedPlugin`] trait backed by format-specific loaders.
//! All implementations adapt to `core_dsp::core_dsp_modules::DspGraphNode` for
//! zero-allocation graph execution with automatic PDC.

#![forbid(unsafe_code)]

pub mod clap_host;

use core_dsp::core_dsp_modules::dsp_graph::DspGraphNode;
use std::path::Path;

/// Supported plugin formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginFormat {
    Clap,
    Lv2,
    Ladspa,
    Vst3,
    AuV2,
}

/// Generic plugin metadata.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub unique_id: String,
    pub format: PluginFormat,
    pub num_audio_inputs: usize,
    pub num_audio_outputs: usize,
}

/// State container for save/load (opaque binary blob).
#[derive(Debug, Clone, Default)]
pub struct PluginState(pub Vec<u8>);

/// Uniform interface for all hosted plugins.
pub trait HostedPlugin: Send {
    fn initialize(
        &mut self,
        sample_rate: f64,
        max_block_size: usize,
        num_inputs: usize,
        num_outputs: usize,
    ) -> Result<(), PluginHostError>;

    fn reset(&mut self);

    fn process(
        &mut self,
        inputs: &[&[f64]],
        outputs: &mut [&mut [f64]],
        num_samples: usize,
    ) -> Result<(), PluginHostError>;

    fn get_state(&self) -> PluginState;

    fn set_state(&mut self, state: &PluginState) -> Result<(), PluginHostError>;

    fn latency_samples(&self) -> usize;

    fn info(&self) -> &PluginInfo;

    fn as_dsp_graph_node(&mut self) -> &mut dyn DspGraphNode;
}

/// Errors from the host.
#[derive(Debug, thiserror::Error)]
pub enum PluginHostError {
    #[error("Library load failed: {0}")]
    LibraryLoad(String),
    #[error("Entry point not found: {0}")]
    EntryPoint(String),
    #[error("Initialization failed: {0}")]
    Initialization(String),
    #[error("Invalid state data")]
    InvalidState,
    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),
    #[error("CLAP error: {0}")]
    Clap(String),
    #[error("LV2 error: {0}")]
    Lv2(String),
    #[error("VST3 error: {0}")]
    Vst3(String),
    #[error("AU error: {0}")]
    Au(String),
}

/// Entry point: scan a directory and return list of available plugin paths.
pub fn scan_plugins(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                match ext {
                    "clap" | "so" | "dll" | "dylib" => results.push(path),
                    _ => {}
                }
            }
        }
    }
    results
}

/// Factory function: load a plugin from a path, detecting format by extension.
pub fn load_plugin(path: &Path) -> Result<Box<dyn HostedPlugin>, PluginHostError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "clap" | "so" | "dll" | "dylib" => clap_host::load_clap_plugin(path),
        _ => Err(PluginHostError::UnsupportedFormat(format!(
            "Format not yet implemented: {ext}"
        ))),
    }
}
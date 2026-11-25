//! CLAP plugin loader stub.

use super::{
    HostedPlugin, PluginHostError, PluginInfo, PluginState,
};
use core_dsp::core_dsp_modules::dsp_graph::DspGraphNode;
use std::path::Path;

#[derive(Default)]
pub struct StubNode;

impl DspGraphNode for StubNode {
    fn process_node(
        &mut self,
        _inputs: &[&[f64]],
        _outputs: &mut [&mut [f64]],
        _num_channels: usize,
        _num_samples: usize,
    ) {}
}

/// Loaded CLAP plugin wrapper (stub implementation).
pub struct ClapPlugin {
    info: PluginInfo,
    sample_rate: f64,
    block_size: usize,
    latency: usize,
    stub: StubNode,
}

impl ClapPlugin {
    pub fn load(path: &Path) -> Result<Box<dyn HostedPlugin>, PluginHostError> {
        let info = PluginInfo {
            name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown CLAP").to_string(),
            vendor: "Unknown".to_string(),
            version: "0.0.0".to_string(),
            unique_id: path.to_string_lossy().to_string(),
            format: super::PluginFormat::Clap,
            num_audio_inputs: 2,
            num_audio_outputs: 2,
        };

        Ok(Box::new(Self {
            info,
            sample_rate: 48_000.0,
            block_size: 512,
            latency: 0,
            stub: StubNode,
        }))
    }
}

impl HostedPlugin for ClapPlugin {
    fn initialize(
        &mut self,
        sample_rate: f64,
        max_block_size: usize,
        num_inputs: usize,
        num_outputs: usize,
    ) -> Result<(), PluginHostError> {
        self.sample_rate = if sample_rate > 0.0 { sample_rate } else { 48_000.0 };
        self.block_size = max_block_size;
        self.info.num_audio_inputs = num_inputs;
        self.info.num_audio_outputs = num_outputs;
        Ok(())
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        _inputs: &[&[f64]],
        _outputs: &mut [&mut [f64]],
        _num_samples: usize,
    ) -> Result<(), PluginHostError> {
        Err(PluginHostError::Clap("CLAP process not implemented in stub".into()))
    }

    fn get_state(&self) -> PluginState {
        PluginState::default()
    }

    fn set_state(&mut self, _state: &PluginState) -> Result<(), PluginHostError> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.latency
    }

    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn as_dsp_graph_node(&mut self) -> &mut dyn DspGraphNode {
        &mut self.stub
    }
}

/// Public load function exported from the host crate.
pub fn load_clap_plugin(path: &Path) -> Result<Box<dyn HostedPlugin>, PluginHostError> {
    ClapPlugin::load(path)
}

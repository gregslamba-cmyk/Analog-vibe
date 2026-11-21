use std::sync::Arc;
use vizia::prelude::*;
use baseview::WindowBuilder;
use greg_bus_comp::GregBusComp;
use greg_gui_core::GregKnob;

fn main() {
    let plugin = Arc::new(greg_bus_comp::GregBusComp::default());

    let params_input = plugin.params.input;
    let params_attack = plugin.params.attack;
    let params_release = plugin.params.release;
    let params_ratio = plugin.params.ratio;
    let params_threshold = plugin.params.threshold;
    let params_knee = plugin.params.knee;
    let params_sidechain_freq = plugin.params.sidechain_frequency;
    let params_sidechain_q = plugin.params.sidechain_q;
    let params_sidechain_gain = plugin.params.sidechain_gain;

    Application::new(|cx| {
        VStack::new(cx, |cx| {
            GregKnob::new(cx, "Input", 0.0, 1.0, plugin.params.input.get().clamp(0.0, 1.0))
                .on_change(|cx, val| { params_input.set_value(cx, val); });

            GregKnob::new(cx, "Attack (ms)", 0.02, 0.8, plugin.params.attack.get().clamp(0.02, 0.8))
                .on_change(|cx, val| { params_attack.set_value(cx, val); });

            GregKnob::new(cx, "Release (ms)", 0.05, 1.1, plugin.params.release.get().clamp(0.05, 1.1))
                .on_change(|cx, val| { params_release.set_value(cx, val); });

            GregKnob::new(cx, "Ratio", 1.0, 20.0, plugin.params.ratio.get().clamp(1.0, 20.0))
                .on_change(|cx, val| { params_ratio.set_value(cx, val); });

            GregKnob::new(cx, "Threshold (dB)", -30.0, 0.0, plugin.params.threshold.get().clamp(-30.0, 0.0))
                .on_change(|cx, val| { params_threshold.set_value(cx, val); });

            GregKnob::new(cx, "Knee (dB)", 0.0, 12.0, plugin.params.knee.get().clamp(0.0, 12.0))
                .on_change(|cx, val| { params_knee.set_value(cx, val); });

            GregKnob::new(cx, "Sidechain Freq", 20.0, 20000.0, plugin.params.sidechain_frequency.get().clamp(20.0, 20000.0))
                .on_change(|cx, val| { params_sidechain_freq.set_value(cx, val); });

            GregKnob::new(cx, "Sidechain Q", 0.5, 12.0, plugin.params.sidechain_q.get().clamp(0.5, 12.0))
                .on_change(|cx, val| { params_sidechain_q.set_value(cx, val); });

            GregKnob::new(cx, "Sidechain Gain", -20.0, 20.0, plugin.params.sidechain_gain.get().clamp(-20.0, 20.0))
                .on_change(|cx, val| { params_sidechain_gain.set_value(cx, val); });
        })
        .fixed_size(350.0, 450.0);
    })
    .title("greg‑bus-comp minimal test GUI")
    .build(WindowBuilder::new()
        .with_title("greg‑bus-comp test")
        .with_inner_size(380, 500));
}

use std::sync::Arc;
use vizia::prelude::*;
use baseview::WindowBuilder;
use greg_76::Greg76;
use greg_gui_core::GregKnob;

fn main() {
    let plugin = Arc::new(greg_76::Greg76::default());

    let params_input = plugin.params.input;
    let params_attack = plugin.params.attack;
    let params_release = plugin.params.release;
    let params_ratio = plugin.params.ratio;
    let params_threshold = plugin.params.threshold;
    let params_knee = plugin.params.knee;
    let params_all_buttons = plugin.params.all_buttons_in;

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

            GregButton::selector(cx, "All‑Buttons‑In", vec!"Off", "On", if plugin.params.all_buttons_in.get() { 1 } else { 0 })
                .on_change(|cx, index| {
                    let val = match index { 0 => false, 1 => true, _ => false };
                    params_all_buttons.set_value(cx, val);
                });
        })
        .fixed_size(300.0, 400.0);
    })
    .title("greg‑76 minimal test GUI")
    .build(WindowBuilder::new()
        .with_title("greg‑76 test")
        .with_inner_size(320, 440));
}

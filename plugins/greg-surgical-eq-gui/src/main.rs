use std::sync::Arc;
use vizia::prelude::*;
use baseview::WindowBuilder;
use greg_surgical_eq::GregSurgicalEQ;
use greg_gui_core::GregKnob;

fn main() {
    let plugin = Arc::new(greg_surgical_eq::GregSurgicalEQ::default());

    // Surgical EQ has 10 bands
    let params_f1 = plugin.params.f1;
    let params_f2 = plugin.params.f2;
    let params_f3 = plugin.params.f3;
    let params_f4 = plugin.params.f4;
    let params_f5 = plugin.params.f5;
    let params_f6 = plugin.params.f6;
    let params_f7 = plugin.params.f7;
    let params_f8 = plugin.params.f8;
    let params_f9 = plugin.params.f9;
    let params_f10 = plugin.params.f10;
    let params_gain = plugin.params.gain;

    Application::new(|cx| {
        VStack::new(cx, |cx| {
            // Frequency knobs for each band
            for (i, param) in [&params_f1, &params_f2, &params_f3, &params_f4].iter().enumerate() {
                let band = i + 1;
                GregKnob::new(cx, &format!("Band {} Freq", band), 20.0, 20000.0, param.get().clamp(20.0, 20000.0))
                    .on_change(|cx, val| { param.set_value(cx, val); });
            }
            for (i, param) in [&params_f5, &params_f6, &params_f7, &params_f8].iter().enumerate() {
                let band = i + 5;
                GregKnob::new(cx, &format!("Band {} Freq", band), 20.0, 20000.0, param.get().clamp(20.0, 20000.0))
                    .on_change(|cx, val| { param.set_value(cx, val); });
            }
            for (i, param) in [&params_f9, &params_f10].iter().enumerate() {
                let band = i + 9;
                GregKnob::new(cx, &format!("Band {} Freq", band), 20.0, 20000.0, param.get().clamp(20.0, 20000.0))
                    .on_change(|cx, val| { param.set_value(cx, val); });
            }

            // Gain knob
            GregKnob::new(cx, "Output Gain (dB)", -30.0, 30.0, params_gain.get().clamp(-30.0, 30.0))
                .on_change(|cx, val| { params_gain.set_value(cx, val); });
        })
        .fixed_size(400.0, 500.0);
    })
    .title("greg‑surgical-eq minimal test GUI")
    .build(WindowBuilder::new()
        .with_title("greg‑surgical-eq test")
        .with_inner_size(440, 550));
}

# STATE.md — Analog-Vibe Suite (Aggiornato: 2026-11-27)
Copyright (c) 2026 Alberto Gregorio Ferraris. All rights reserved.

## Stato Attuale Verificato
- **Suite Compilata & Validata:** 18 plugin CLAP + VST3 nativi (`target/release/libgreg_*.so` → `~/.clap/Analog-vibe/*.clap`), `cargo test --workspace` (**270 passati, 0 falliti**), `cargo clippy --workspace --all-targets -- -D warnings` (**0 warning**), `clap-signal-host` (**18/18 PASS**).
- **Completati di Recente:**
  - [x] Modulo `psychoacoustics.rs` (ISO 226:2003, Terhardt ATH, pesature A/B/C/D/Z/ITU-R 468, Sone/Bark/ERB, mascheramento, noise-shaping dither).
  - [x] Audit tecnico `docs/CORE_DSP_AUDIT.md` & Bonifica P0/P1 Parte A (`ZdfSvf`, `CombFilter`, `ADAA1/2`, `TubeStage`, `DualStageProgramDependentRelease`, `TransformerHysteresis`, `PsuSag`, `CabinetSpeaker` con `mic_preamp` persistito, `ChannelFormat`/`TrackSpec` con isolamento `LFE`).
  - [x] Regole `.clinerules` aggiornate con lettura preventiva obbligatoria di `vcinst.md`, `ARCHITECTURE.md` e `GUI_SPEC.md`.
  - [x] Task 1: Bonifica P0/P1 Core-DSP Parte B (Zero-Alloc Real-Time, `FdnReverb` Hadamard $1/\sqrt{8}$, `AcousticRoom`, `Spatial3d` ITD, `PitchShifter` COLA, `LoudnessEngine` ITU-1770-4, `CompiledAudioGraph`).
- **Coda Attiva (`doit.txt` — Task 3..8):**
  - [x] Task 2: Re-export completo `core-dsp/src/lib.rs` (`pub use core_dsp_modules::*`) + `WavetableOscillator` band-limited mip-mapped con morphing continuo (`synth_engine/wavetable_oscillator.rs`), `PitchTracker` YIN/pYIN + estrattore formanti (`pitch_shifter.rs`), `AutoPhaseAligner` cross-correlazione/ritardo frazionario/All-Pass (`routing_matrix.rs`).
  - [x] Task 3: GUI 2D rifinita — Box::leak rimosso con `greg_static_params!`, pulsanti cliccabili, GUI console su `greg_nl_ch/mixer/bay`, ore 12 BipolarKnob, EqBandSpec, ChannelFormat/TrackSpec, barra universale A/B/Copy/Paste/SaveDefault/PresetBrowser. Commit `df93e53`, pushato su `origin main`.
  - [x] Task 4: Hardware Reference Matching (`--match-reference`) & Orecchio Percettivo in `clap-validator-host`.
  - [ ] Task 5..7: Nuovi plugin da inventario ufficiale (`greg_space`, `greg_tape`, `greg_tube_drive`, `greg_resonance_killer_eq`, `greg_1073_1081_eq`, `greg_550_560_eq` → 24/24 PASS).

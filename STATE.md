# STATE.md — Analog-Vibe Suite (Aggiornato: 2026-11-27)
Copyright (c) 2026 Alberto Gregorio Ferraris. All rights reserved.

## Stato Attuale Verificato
- **Suite Compilata & Validata:** 20 plugin CLAP + VST3 nativi (`target/release/libgreg_*.so` → `~/.clap/Analog-vibe/*.clap`), `cargo test --workspace` (**270 passati, 0 falliti**), `cargo clippy --workspace --all-targets -- -D warnings` (**0 warning**), `clap-signal-host` (**20/20 PASS**).
- **Completati di Recente:**
  - [x] Modulo `psychoacoustics.rs` (ISO 226:2003, Terhardt ATH, pesature A/B/C/D/Z/ITU-R 468, Sone/Bark/ERB, mascheramento, noise-shaping dither).
  - [x] Audit tecnico `docs/CORE_DSP_AUDIT.md` & Bonifica P0/P1 Parte A (`ZdfSvf`, `CombFilter`, `ADAA1/2`, `TubeStage`, `DualStageProgramDependentRelease`, `TransformerHysteresis`, `PsuSag`, `CabinetSpeaker` con `mic_preamp` persistito, `ChannelFormat`/`TrackSpec` con isolamento `LFE`).
  - [x] Regole `.clinerules` aggiornate con lettura preventiva obbligatoria di `vcinst.md`, `ARCHITECTURE.md` e `GUI_SPEC.md`.
  - [x] Task 1: Bonifica P0/P1 Core-DSP Parte B (Zero-Alloc Real-Time, `FdnReverb` Hadamard $1/\sqrt{8}$, `AcousticRoom`, `Spatial3d` ITD, `PitchShifter` COLA, `LoudnessEngine` ITU-1770-4, `CompiledAudioGraph`).
  - [x] Task P0-2: Isolamento FFI unsafe in Crate Wrapper (task 10 Item 1) — Bonifica/build del crate `clap-sys-wrapper` (`tools/clap-sys-wrapper/src/lib.rs`): null-check su `extension_id`, import `std::ffi::{c_char, c_void, CStr}`, callback `unsafe extern "C"` isolate nel modulo `host`, rimozione auto-riferimenti self-crate, `clippy missing_safety_doc` gestito. **Build release & clippy OK — COMPLETATO.**
  - [x] Task P1-1: Porta DSP a `f64` interno su `greg_bay`, `greg_nl_ch`, `greg_surgical_eq` (cast `f32` solo a I/O in `process()`); `.unwrap()` numeric conversion sostituiti con `.unwrap_or(...)` in `vari_mu.rs` / `filters.rs` (zero `.unwrap()` residui).
  - [x] Task P1-2: Multicanalità 1–8 canali implementata per `greg_bay` e `greg_nl_ch` (iterazione frame canali), stereo-only paths rimossi.
  - [x] Task P1-4: `set_latency_samples` spostato fuori da `process()` (invocato in `initialize()`).
  - [x] Task P1-3 (completato): `gain_db` host = `20.0 * (out_rms / in_rms).max(1e-10).log10()`; `nan_detected` (`is_nan`/`is_infinite`) in `clap-validator-host`; FFT-based latency estimator completato (`estimate_latency_fft` con finestra Hann, cross-correlazione `conj(FFT(in))*FFT(out)`, soglia noise-floor, `next_pow2`) e `bode_from_fft` con trim/pad di `latency_samples` — 3/3 test unitari PASS, clippy 0 warning, `clap-signal-host` PASS sui plugin verificati.
- **Coda Attiva (`doit.txt` — Task 3..8):**
  - [x] Task 2: Re-export completo `core-dsp/src/lib.rs` (`pub use core_dsp_modules::*`) + `WavetableOscillator` band-limited mip-mapped con morphing continuo (`synth_engine/wavetable_oscillator.rs`), `PitchTracker` YIN/pYIN + estrattore formanti (`pitch_shifter.rs`), `AutoPhaseAligner` cross-correlazione/ritardo frazionario/All-Pass (`routing_matrix.rs`).
  - [x] Task 3: GUI 2D rifinita — Box::leak rimosso con `greg_static_params!`, pulsanti cliccabili, GUI console su `greg_nl_ch/mixer/bay`, ore 12 BipolarKnob, EqBandSpec, ChannelFormat/TrackSpec, barra universale A/B/Copy/Paste/SaveDefault/PresetBrowser. Commit `df93e53`, pushato su `origin main`.
  - [x] Task 4: Hardware Reference Matching (`--match-reference`) & Orecchio Percettivo in `clap-validator-host`.
  - [x] Task 5: Nuovi plugin (`greg_space`, `greg_tape`) implementati — struttura DSP, parametri MAIN/ADVANCED, GUI con greg_static_params!, cargo check pulito. **Build release & clap-signal-host 20/20 PASS — COMPLETATO.**

## Runtime Agenti — Docker Pool (Livello 2)
- **Architettura scelta:** Container Docker leggeri per ogni agente (web-research, rust-review, dsp-architect) + container `agent-sandbox` per operazioni ad alto rischio.
- **VM Host:** ~10 GB RAM, 6 core. Limiti per agente: `agent-web-research` 1 GB RAM / 1 core; `agent-rust-review` 512 MiB RAM / 0.5 core; `agent-dsp-architect` 768 MiB RAM / 0.75 core; `agent-sandbox` 128 MiB RAM / 0.25 core.
- **Hardening sandbox:** `read_only: true`, `tmpfs: /tmp`, `network_mode: none`, `cap_drop: [ALL]`, `security_opt: no-new-privileges:true`, `mem_limit`/`cpus` stretti.
- **Config files:** `infra/agents/docker-compose.yml`, `infra/agents/Dockerfile.{web-research,rust-review,dsp-architect}`, `infra/agents/agent.sh`.
- **Attivazione on-demand (IMPORTANTE):** gli agenti NON restano accesi. Si usa `docker compose run --rm <servizio> <comando>` oppure lo wrapper `infra/agents/agent.sh <servizio> <comando>`, che crea un container effimero, esegue il task e lo distrugge. Nessun processo in background tra un task e l'altro.

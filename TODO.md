# TODO — Analog-Vibe (Suite Plugin & Release — Collegato a `ROADMAP.md`)
Copyright (c) 2026 Alberto Gregorio Ferraris. All rights reserved.

- [ ] **v1.0:** Completare test reali d'ascolto in REAPER sui 18 plugin attuali
- [ ] **v1.0:** Eseguire packaging multi-formato (`scripts/package_all_formats.sh` → CLAP, VST3, LV2, AUv2)
- [ ] **v1.1:** Implementare i 6 nuovi plugin di copertura core (`greg_space`, `greg_tape`, `greg_tube_drive`, `greg_resonance_killer_eq`, `greg_1073_1081_eq`, `greg_550_560_eq` → 24/24 PASS)
- [ ] **v1.1:** Revisione finale nomi commerciali pubblici mantenendo invariati gli ID CLAP/VST3
- [ ] **v1.1:** Implementare **`greg_remic`** (Virtual Multi-Mic 3D Close/OH/Room + Auto-Phase Sync All-Pass per batterie, chitarre, voci e bassi)

- [ ] **Audit & Fix Numerici/FFI (`clap-signal-host` & `clap-validator-host`):**
  - Null-check FFI (`clap_plugin_entry`, `plugin->init()`, `plugin->process()`, `audio_input`/`audio_output`) + commenti `// SAFETY:` sui blocchi C-ABI dei due tool host (mantenendo `#![forbid(unsafe_code)]` su `core-dsp`, `greg_gui_core` e su tutti i plugin).
  - Fix numerico `gain_db`: `20.0 * (out_rms / in_rms).max(1e-10).log10()` (clamp sull'argomento del log invece che sul risultato).
  - Rilevamento reale `nan_detected` (`f64::is_nan()` / `is_infinite()`) sull'output in `clap-validator-host`.
  - Cross-correlazione FFT con `realfft` per la misura di latenza (riduzione tempi ~90%), finestra Hann + noise floor in `thd_percent_full` e compensazione `latency_samples` in `bode_from_fft`.

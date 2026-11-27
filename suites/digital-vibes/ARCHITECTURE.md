# Digital-Vibe — Suite Audio Digitale, Chirurgica, Spaziale, Pitch & Sound Design f64
Copyright (c) 2026 Alberto Gregorio Ferraris. All rights reserved.
Dual Licensing: GNU GPLv3+ / Licenza Commerciale Proprietaria.

## Catalogo Completo Digital-Vibe (31 Plugin)
### A. EQ, Dinamica Chirurgica & Controllo Spettrale (11)
1. `greg_surgical_eq` — EQ parametrico ZDF trasparente
2. `greg_lin_eq` — EQ a fase lineare (`linear_phase_fir`)
3. `greg_resonance_killer_eq` — EQ dinamico automatico anti-risonanze 1..15 bande con Band Solo
4. `greg_deharsh` — Suppressore dinamico spettrale asprezze (2–8 kHz) per-bin FFT
5. `greg_de_esser` — De-Esser chirurgico split-band/broadband con Listen Sibilance
6. `greg_limiter` — Brickwall Limiter digitale trasparente con lookahead
7. `greg_lin_comp` — Compressore trasparente di precisione
8. `greg_multiband_comp` — Compressore & Expander multibanda a 4 bande Linkwitz-Riley
9. `greg_gate` — Noise Gate & Expander digitale
10. `greg_split_dyn` — Dinamica & saturazione separata Transiente/Sustain
11. `greg_trans_mono` & `greg_trans_multi` — Transient Designer mono/multibanda (`transient_designer.rs`)

### B. Pitch, Voce & Drum Trigger (4)
12. `greg_harmonizer` — Harmonizer Polifonico & Micro-Pitch Studio Suite a 4 voci indipendenti con Phase-Locking verticale COLA, Formant Preservation separata, protezione transienti anti-distorsione, scale `music_theory.rs` / MIDI e Ducking
13. `greg_pitch_clean` & `greg_pitch_formant` — Intonazione automatica real-time & Formant Shifter (`polyphonic_pitch.rs`)
14. `greg_trigger` — Drum Replacer & Sample Trigger multi-layer con velocity/round-robin e mini-editor layer

### C. Spazio, Riverberi & Multi-Delay (4)
15. `greg_space` — Riverbero acustico 3D & FDN Studio Suite
16. `greg_ir_reverb` — Convolutore IR partizionato zero-latency (`partitioned_convolver`)
17. `greg_delay_studio` — Multi-Engine Delay Suite a 8 motori (Pure Digital, Dual/Ping-Pong, 8-Tap Pattern, BBD, Multi-Head Tape Echo, Diffusion Cloud, Shimmer Pitch-Delay, Reverse/Freeze) con filtri ZDF nel feedback e Ducking
18. `greg_pingpong` — Delay Ping-Pong stereo leggero

### D. Modulazioni Studio & Sound Design (6)
19. `greg_mod_studio` — Suite Modulazioni (Through-Zero Flanger, Tri-Chorus 3–6 voci, Multi-Stage Phaser 2–24 poli, Rotary Doppler, Rhythmic Tremolo & Auto-Panner)
20. `greg_bitcrusher` (`greg_destroyer`) — Bit-depth reducer (`1..24 bit`), Sample-rate reducer (`100 Hz..48 kHz`), Jitter, filtri ZDF e TPDF/Noise-Shaped Dither
21. `greg_freq_shifter` — Frequency Shifter (Hilbert) & Ring Modulator
22. `greg_granular` — Granular Cloud & Spectral Freeze real-time
23. `greg_vocoder` — Vocoder multi-banda (16–32 bande) con synth carrier integrato
24. `greg_filter_motion` — Envelope Filter (Auto-Wah), LFO Filter & Morphing Formant Filter

### E. Immagine Stereo, Fase & Utility f64 (5)
25. `greg_phase` — Allineamento di fase continuo e Auto-Phase
26. `greg_remic` — Allineamento di Fase & Tempo Multi-Microfono automatico (`AutoPhaseAligner`)
27. `greg_ms_encoder` & `greg_ms_decoder` — Matrice Mid/Side pura f64
28. `greg_stereo_imager` — Stereo Imager multibanda con Mono-Maker e micro-ritardo Haas
29. `greg_resonator` — Micro-Feedback & Comb Resonator (`0.05 ms – 50.0 ms`, `Feedback Volume` bipolare `-100%..+100%`, `Damping` nel loop e limiter morbido anti-esplosione per risonanze metalliche, tubolari e Karplus-Strong)

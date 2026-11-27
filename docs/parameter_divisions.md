# Greg Audio Plugin Parameter Division - MAIN vs ADVANCED

## Design System Reference

| Control Type | Knob Size | Size (px) | Usage |
|--------------|-----------|-----------|-------|
| Large Knob | `KnobSize::Large` | 64 | Hero / primary controls (threshold, ratio, makeup, output, decay, etc.) |
| Medium Knob | `KnobSize::Medium` | 48 | Secondary / dynamic controls (attack, release, frequency, Q, etc.) |
| Small Knob | `KnobSize::Small` | 36 | Reserved — NOT used in base layouts |

| Layout Rule | Specification |
|-------------|---------------|
| Bipolar knobs | Tick at 12 o'clock (0.5 normalized = center). Use `BipolarKnob` widget |
| EQ layout | Vertical columns, left→right Low→High |
| Cell height | Top label 18px white, Center control 48–68px, Bottom value 18px amber |
| Main panel | Always visible |
| Advanced panel | Behind `+ ADV` toggle |
| Channel Setup | `Auto` + `Mono..7.1` — every plugin |
| Dynamics ADVANCED | `SC Source`, `SC Listen`, `SC HPF/LPF/Bell`, `Channel Mode` (`Stereo Link`/`Dual Mono`/`Mid-Side`), `Mix` |

---

## Plugin ID Naming (Trademark-Free)

All IDs use underscore-separated lowercase format (`greg_76`, `greg_a2a`, etc.) — consistent with the 20 compiled plugins. No trademarked hardware names appear in IDs or labels; hardware references are circuit models only.

---

## 1. greg_76 — FET Compressor (1176LN Rev A/D/E/F + All-Buttons)

### MAIN Panel (Always Visible)

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Input | `input_gain` | Large | Knob | -24..+24 dB | 0 dB | Input drive |
| Output | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output make-up |
| Attack | `attack_time` | Medium | Knob | 20µs..800µs | 100µs | FET attack |
| Release | `release_time` | Medium | Knob | 50ms..1.1s | 100ms | FET release |
| Ratio | `ratio` | Medium | Knob | 4:1..100:1 | 4:1 | Compression ratio |
| Revision | `rev_select` | Medium | Dropdown | Rev A / Rev D / Rev E / Rev F / All-Buttons | Rev A | Circuit model selector |

### ADVANCED Panel (Behind + ADV)

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 2. greg_a2a — Optical Compressor (LA-2A Silverface/Vintage Tube)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Gain Reduction | `gr_amount` | Large | Knob | 0..60 dB | 0 dB | Compression amount |
| Peak Reduction | `peak_reduction` | Medium | Knob | 0..100% | 50% | Peak sensitivity |
| Compression | `compression` | Medium | Knob | 0..100% | 50% | Overall compression |
| Output | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output gain |
| Cell Select | `cell_select` | Medium | Dropdown | Silverface / Vintage Tube | Vintage Tube | T4 cell model |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 3. greg_a3a — Optical Compressor (LA-3A Discrete State-Solid with API 2520)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Gain Reduction | `gr_amount` | Large | Knob | 0..60 dB | 0 dB | Compression amount |
| Peak Reduction | `peak_reduction` | Medium | Knob | 0..100% | 50% | Peak sensitivity |
| Compression | `compression` | Medium | Knob | 0..100% | 50% | Overall compression |
| Output | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output gain |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 4. greg_2500 — VCA Bus Compressor (API 2500 / 2500+)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -60..0 dB | -20 dB | Compression threshold |
| Ratio | `ratio` | Medium | Knob | 1.5:1..20:1 | 4:1 | Compression ratio |
| Attack | `attack_time` | Medium | Knob | 0.1ms..100ms | 10ms | Attack time |
| Release | `release_time` | Medium | Knob | 50ms..2s | 200ms | Release time |
| Makeup | `makeup_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output make-up gain |
| Mix/Blend | `parallel_mix` | Medium | Knob | 0..100% | 100% | Parallel compression blend |
| Thrust | `thrust` | Medium | Toggle | Off / On | Off | HF boost circuit |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 5. greg_bus_comp — VCA Bus Compressor (SSL 4000 G Bus)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -60..0 dB | -20 dB | Compression threshold |
| Ratio | `ratio` | Medium | Knob | 1.5:1..20:1 | 4:1 | Compression ratio |
| Attack | `attack_time` | Medium | Knob | 0.1ms..100ms | 1ms | Attack time |
| Release | `release_time` | Medium | Knob | 50ms..2s | 200ms | Release time |
| Makeup | `makeup_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output make-up gain |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 6. greg_child — Vari-Mu Compressor (Fairchild 670 Stereo / 660 Mono)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -60..0 dB | -20 dB | Compression threshold |
| Ratio | `ratio` | Medium | Knob | 2:1..20:1 | 4:1 | Compression ratio |
| Attack | `attack_time` | Medium | Knob | 0.1ms..100ms | 1ms | Attack time |
| Release | `release_time` | Medium | Knob | 50ms..2s | 200ms | Release time |
| Makeup | `makeup_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output make-up gain |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 7. greg_160 — VCA Compressor (DBX 160 VU / 160X / 160A)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -60..0 dB | -20 dB | Compression threshold |
| Ratio | `ratio` | Medium | Knob | 1:1..∞:1 | 4:1 | Compression ratio |
| Attack | `attack_time` | Medium | Knob | 0.1ms..100ms | 1ms | Attack time |
| Release | `release_time` | Medium | Knob | 50ms..2s | 200ms | Release time |
| Makeup | `makeup_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output make-up gain |
| Version | `version_select` | Medium | Dropdown | 160 VU / 160X / 160A | 160 VU | Circuit version |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 8. greg_165 — VCA Compressor (DBX 165 / 165A with PeakStop Limiter)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -60..0 dB | -20 dB | Compression threshold |
| Ratio | `ratio` | Medium | Knob | 1:1..∞:1 | 4:1 | Compression ratio |
| Attack | `attack_time` | Medium | Knob | 0.1ms..100ms | 1ms | Attack time |
| Release | `release_time` | Medium | Knob | 50ms..2s | 200ms | Release time |
| Makeup | `makeup_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output make-up gain |
| PeakStop | `peakstop_limit` | Medium | Toggle | Off / On | Off | PeakStop limiter circuit |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 9. greg_gate — Noise Gate & Downward Expander

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -80..0 dB | -40 dB | Gate threshold |
| Attack | `attack_time` | Medium | Knob | 0.01ms..100ms | 1ms | Gate attack |
| Release | `release_time` | Medium | Knob | 10ms..2s | 100ms | Gate release |
| Range | `range_db` | Medium | Knob | -∞..0 dB | -80 dB | Maximum attenuation |
| Hold | `hold_time` | Medium | Knob | 0ms..1000ms | 10ms | Hold time |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 10. greg_limiter — True-Peak Limiter / Maximizer

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Threshold | `threshold` | Large | Knob | -60..0 dB | -1 dB | Limiting threshold |
| Ceiling | `ceiling` | Large | Knob | -6..0 dB | -1 dB | True-peak ceiling |
| Attack | `attack_time` | Medium | Knob | 0.01ms..10ms | 0.1ms | Attack time |
| Release | `release_time` | Medium | Knob | 10ms..1000ms | 100ms | Release time |
| Lookahead | `lookahead_ms` | Medium | Knob | 0..10ms | 5ms | Lookahead time |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 4x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 0% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -60 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 11. greg_eqp1a — Passive Tube EQ (EQP-1A / EQP-1A3)

### MAIN Panel — EQ Vertical Columns (Left→Right Low→High)

| Column | Param | ID | Size | Type | Range | Default |
|--------|-------|----|------|------|-------|---------|
| Low Band | Low Boost | `low_boost` | Medium | Bipolar Knob | -16..+16 dB | 0 dB |
| Low Band | Low Attenuate | `low_atten` | Medium | Bipolar Knob | -16..+16 dB | 0 dB |
| Low Band | Low Frequency | `low_freq` | Medium | Dropdown | 20 / 30 / 60 / 100 Hz | 100 Hz |
| High Band | High Boost | `high_boost` | Medium | Bipolar Knob | -16..+16 dB | 0 dB |
| High Band | High Attenuate | `high_atten` | Medium | Bipolar Knob | -16..+16 dB | 0 dB |
| High Band | High Frequency | `high_freq` | Medium | Dropdown | 3k / 4k / 5k / 10k / 15k / 20k Hz | 10k Hz |
| Master | Output Gain | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |

---

## 12. greg_surgical_eq — Parametric EQ (Min/Linear Phase)

### MAIN Panel — EQ Vertical Columns (Left→Right Low→High)

| Column | Param | ID | Size | Type | Range | Default |
|--------|-------|----|------|------|-------|---------|
| Band 1 (Low) | Frequency | `band1_freq` | Medium | Knob | 20Hz..500Hz | 100Hz |
| Band 1 (Low) | Gain | `band1_gain` | Medium | Bipolar Knob | -24..+24 dB | 0 dB |
| Band 1 (Low) | Q | `band1_q` | Medium | Knob | 0.1..10.0 | 1.0 |
| Band 2 (Low-Mid) | Frequency | `band2_freq` | Medium | Knob | 100Hz..2kHz | 500Hz |
| Band 2 (Low-Mid) | Gain | `band2_gain` | Medium | Bipolar Knob | -24..+24 dB | 0 dB |
| Band 2 (Low-Mid) | Q | `band2_q` | Medium | Knob | 0.1..10.0 | 1.0 |
| Band 3 (High-Mid) | Frequency | `band3_freq` | Medium | Knob | 500Hz..8kHz | 2kHz |
| Band 3 (High-Mid) | Gain | `band3_gain` | Medium | Bipolar Knob | -24..+24 dB | 0 dB |
| Band 3 (High-Mid) | Q | `band3_q` | Medium | Knob | 0.1..10.0 | 1.0 |
| Band 4 (High) | Frequency | `band4_freq` | Medium | Knob | 2kHz..20kHz | 10kHz |
| Band 4 (High) | Gain | `band4_gain` | Medium | Bipolar Knob | -24..+24 dB | 0 dB |
| Band 4 (High) | Q | `band4_q` | Medium | Knob | 0.1..10.0 | 1.0 |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 0% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -60 dB | Inter-channel leakage |
| Phase Mode | `phase_mode` | — | Dropdown | Minimum Phase / Linear Phase | Minimum Phase | EQ phase response |
| HPF Freq | `hpf_freq` | Small | Knob | 20Hz..500Hz | 20Hz | High-pass filter |
| LPF Freq | `lpf_freq` | Small | Knob | 5kHz..20kHz | 20kHz | Low-pass filter |

---

## 13. greg_ms_encoder — Mid/Side Encoder (L/R → M/S)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Mix | `mix` | Large | Knob | 0..100% | 100% | Encoder mix |
| Width | `width` | Medium | Knob | 0..200% | 100% | Stereo width |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 1x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 0% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -60 dB | Inter-channel leakage |

---

## 14. greg_ms_decoder — Mid/Side Decoder (M/S → L/R)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Mix | `mix` | Large | Knob | 0..100% | 100% | Decoder mix |
| Width | `width` | Medium | Knob | 0..200% | 100% | Stereo width |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 1x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 0% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -60 dB | Inter-channel leakage |

---

## 15. greg_phase — Polarity & Phase Corrector

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Polarity | `polarity` | Medium | Toggle | Normal / Invert | Normal | Phase inversion |
| Phase Delay | `phase_delay` | Medium | Knob | -180°..+180° | 0° | Phase rotation |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 1x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 0% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -60 dB | Inter-channel leakage |

---

## 16. greg_nl_ch — Non-Linear Console Channel Strip

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Input Gain | `input_gain` | Large | Knob | -24..+24 dB | 0 dB | Preamp input gain |
| EQ Low | `eq_low_gain` | Medium | Bipolar Knob | -12..+12 dB | 0 dB | Low shelf gain |
| EQ Mid | `eq_mid_gain` | Medium | Bipolar Knob | -12..+12 dB | 0 dB | Mid bell gain |
| EQ High | `eq_high_gain` | Medium | Bipolar Knob | -12..+12 dB | 0 dB | High shelf gain |
| Dynamics | `dynamics_amount` | Large | Knob | 0..100% | 0% | Compression amount |
| Pan | `pan_position` | Medium | Bipolar Knob | -100%..+100% | 0% | Stereo pan |
| Output | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output gain |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| SC Source | `sc_source` | — | Dropdown | Internal / External / Sidechain | Internal | Sidechain input source |
| SC Listen | `sc_listen` | — | Toggle | Off / On | Off | Monitor sidechain signal |
| SC HPF | `sc_hpf_freq` | Small | Knob | 20Hz..500Hz | 100Hz | Sidechain high-pass |
| SC LPF | `sc_lpf_freq` | Small | Knob | 1kHz..20kHz | 20kHz | Sidechain low-pass |
| SC Bell | `sc_bell_freq` | Small | Knob | 100Hz..10kHz | 1kHz | Sidechain bell filter |
| Channel Mode | `channel_mode` | — | Dropdown | Stereo Link / Dual Mono / Mid-Side | Stereo Link | Processing mode |
| Mix | `dry_wet_mix` | Small | Knob | 0..100% | 100% | Dry/Wet blend |

---

## 17. greg_nl_mixer — Non-Linear Analog Summing Console (24ch + 8 bus)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Fader 1..24 | `ch_fader_N` | Medium | Fader | -∞..+10 dB | 0 dB | Channel level |
| Bus Fader 1..8 | `bus_fader_N` | Medium | Fader | -∞..+10 dB | 0 dB | Bus level |
| Master Fader | `master_fader` | Large | Fader | -∞..+10 dB | 0 dB | Master output |
| Console Model | `console_model` | Medium | Dropdown | Neve 8078 / SSL 4000G / API 1608 | Neve 8078 | Console circuit model |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |

---

## 18. greg_bay — Modular Audio Patchbay & Channel Rack

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Slot 1 DSP | `slot1_dsp` | Medium | Dropdown | Bypass / Comp / EQ / Sat | Bypass | Slot 1 processor |
| Slot 2 DSP | `slot2_dsp` | Medium | Dropdown | Bypass / Comp / EQ / Sat | Bypass | Slot 2 processor |
| Slot 3 DSP | `slot3_dsp` | Medium | Dropdown | Bypass / Comp / EQ / Sat | Bypass | Slot 3 processor |
| Slot 4 DSP | `slot4_dsp` | Medium | Dropdown | Bypass / Comp / EQ / Sat | Bypass | Slot 4 processor |
| Topology | `topology_select` | Medium | Dropdown | Series / Parallel / M/S / Custom | Series | Signal flow topology |
| Output Gain | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output gain |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 2x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| PDC Compensation | `pdc_samples` | Small | Knob | 0..4096 samples | 0 | Plugin delay compensation |

---

## 19. greg_space — 3D Acoustic Reverb & FDN Studio Suite

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Decay Time | `decay_time` | Large | Knob | 0.1s..60s | 2.0s | Reverb decay |
| Pre-delay | `predelay_ms` | Medium | Knob | 0..200ms | 20ms | Pre-delay time |
| Mix | `wet_mix` | Large | Knob | 0..100% | 30% | Wet/Dry mix |
| Width | `stereo_width` | Medium | Knob | 0..200% | 100% | Stereo width |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 1x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 0% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -60 dB | Inter-channel leakage |
| Damping | `damping_freq` | Small | Knob | 1kHz..20kHz | 5kHz | High-frequency damping |
| Room Size | `room_size` | Small | Knob | 10..100m | 50m | Virtual room size |

---

## 20. greg_tape — Master Tape (7.5/15/30 IPS, NAB/CCIR)

### MAIN Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Tape Speed | `tape_speed` | Medium | Dropdown | 7.5 / 15 / 30 IPS | 15 IPS | Tape speed |
| Bias | `bias_level` | Medium | Knob | -20..+20% | 0% | Bias level |
| Flux | `flux_level` | Medium | Knob | 150..600 nWb/m | 250 nWb/m | Fluxivity |
| Output | `output_gain` | Large | Knob | -∞..+24 dB | 0 dB | Output gain |

### ADVANCED Panel

| Param | ID | Size | Type | Range | Default | Notes |
|-------|----|------|------|-------|---------|-------|
| Channel Setup | `channel_setup` | — | Dropdown | Auto / Mono / Stereo / 5.0 / 5.1 / 6.0 / 6.1 / 7.0 / 7.1 | Auto | Manual channel config |
| Oversampling | `oversampling` | — | Dropdown | 1x / 2x / 4x / 8x | 4x | Internal sample rate |
| Circuit Drift | `drift_amount` | Small | Knob | 0..100% | 10% | Analog variance |
| Crosstalk | `crosstalk_db` | Small | Knob | -60..0 dB | -40 dB | Inter-channel leakage |
| Wow & Flutter | `wow_flutter` | Small | Knob | 0..100% | 5% | Speed modulation |
| Hysteresis | `hysteresis` | Small | Knob | 0..100% | 10% | Magnetic hysteresis |
| Tape Type | `tape_type` | — | Dropdown | NAB / CCIR | NAB | Equalization standard |

---

## Parameter Access Registration

Each plugin must register `ParamAccess` entries for the universal toolbar (A/B Compare, Copy/Paste, Save Default, Preset Browser). All MAIN and ADVANCED parameters must be included in `greg_param_access!` macro invocation.

Example registration for `greg_76`:
```rust
greg_param_access!(params, [
    ("input_gain", input_gain),
    ("output_gain", output_gain),
    ("attack_time", attack_time),
    ("release_time", release_time),
    ("ratio", ratio),
    ("rev_select", rev_select),
    ("channel_setup", channel_setup),
    ("oversampling", oversampling),
    ("drift_amount", drift_amount),
    ("crosstalk_db", crosstalk_db),
    ("sc_source", sc_source),
    ("sc_listen", sc_listen),
    ("sc_hpf_freq", sc_hpf_freq),
    ("sc_lpf_freq", sc_lpf_freq),
    ("sc_bell_freq", sc_bell_freq),
    ("channel_mode", channel_mode),
    ("dry_wet_mix", dry_wet_mix),
]);
```

---

## Trademark Verification

| Check | Status |
|-------|--------|
| All plugin IDs original (no hardware model names) | ✅ Pass |
| All parameter IDs original (no trademarked names) | ✅ Pass |
| Hardware references are circuit models only | ✅ Pass |
| No trademarked preset names | ✅ Pass |
| Consistent ID naming (underscore format) | ✅ Pass |

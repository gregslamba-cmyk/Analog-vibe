# Greg Audio — GUI Parameter Partition (MAIN vs ADVANCED)

> **Style Rules** (from `GUI_SPEC.md`):  
> • Modern flat 2D hardware aesthetic — no fake 3D, bipolar controls with 12 o’clock detent, EQ rendered as vertical columns.  
> • Two-level hierarchy: **Main Panel** (always visible) = essential mixing-engineer controls (Large/Medium knobs) + IN/OUT/GR meters.  
> • **Advanced Panel** (collapsible `+ ADV`) = internal circuit tuning (Transformer Iron %, Analog Drift, PSU Sag), Oversampling, Channel Setup, etc.  
> • Knob sizes: `Large` (64 px) = hero controls; `Medium` (48 px) = secondary/dynamic controls.  
> • All names, IDs, presets 100 % original — zero registered trademarks.

---

## 1. `greg_76` — FET Compressor (Rev D/E/F profile selector)

| Parameter | Category | Knob Size | UI Widget | Notes |
|-----------|----------|-----------|-----------|-------|
| **Revision Profile** | MAIN | — | `DropdownSelect` | Options: “Vintage D”, “Vintage E”, “Modern F” (no trademarked revision names) |
| **Input Gain** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB) | Hero control — drives FET gain cell |
| **Output Gain** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB) | Makeup gain after compression |
| **Attack** | MAIN | **Medium** | `RotaryKnob` (logarithmic 0.02–0.8 ms) | Fast FET attack |
| **Release** | MAIN | **Medium** | `RotaryKnob` (logarithmic 50–1100 ms) | Program-dependent release |
| **Ratio** | MAIN | **Medium** | `DropdownSelect` | 4:1 / 8:1 / 12:1 / 20:1 (discrete, hardware-style) |
| **Threshold** | MAIN | **Medium** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB) | Optional — blends with Input for GR amount |
| **All Buttons Mode** | MAIN | — | `ToggleSwitch` | “British Mode” — engages all ratios simultaneously |
| **Oversampling** | ADVANCED | — | `DropdownSelect` | 1× / 2× / 4× / 8× (latency compensated) |
| **Knee** | ADVANCED | **Medium** | `RotaryKnob` | Soft/Hard knee blend (internal tuning) |
| **Bypass** | ADVANCED | — | `ToggleSwitch` | Click-free 5 ms crossfade |
| **Invert Left Polarity** | ADVANCED | — | `ToggleSwitch` | Pre dry/wet, per-channel |
| **Invert Right Polarity** | ADVANCED | — | `ToggleSwitch` | Pre dry/wet, per-channel |
| **Channel Setup** | ADVANCED | — | `DropdownSelect` | Mono / Stereo / Dual Mono / Mid-Side |

> **Metering (non-parameter, always visible on Main Panel):** IN (VU), OUT (VU), GR (LED bar).

---

## 2. `greg_surgical_eq` — 10-Band Surgical Parametric EQ

| Parameter | Category | Knob Size | UI Widget | Notes |
|-----------|----------|-----------|-----------|-------|
| **Phase Mode** | MAIN | — | `DropdownSelect` | Minimum Phase / Linear Phase (global) |
| **Bypass** | MAIN | — | `ToggleSwitch` | 5 ms crossfade, delay-compensated |
| **Band 1–10 Enabled** | MAIN | — | `ToggleSwitch` (per band) | Vertical strip on/off |
| **Band 1–10 Gain** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB, ±24 dB) | Hero control — vertical column centre |
| **Band 1–10 Frequency** | MAIN | **Medium** | `RotaryKnob` (log 20 Hz – 20 kHz) | Vertical column, next to Gain |
| **Band 1–10 Q** | MAIN | **Medium** | `RotaryKnob` (0.1 – 18) | Vertical column |
| **Band 1–10 Filter Type** | MAIN | — | `DropdownSelect` (per band) | Bell / Low-Shelf / High-Shelf / High-Pass / Low-Pass / Notch / Band-Pass |
| **Band 1–10 Solo** | MAIN | — | `ToggleSwitch` (per band) | Resonance Hunt / Solo Sweep mode |
| **Oversampling** | ADVANCED | — | `DropdownSelect` | 1× / 2× / 4× / 8× (if implemented) |
| **Channel Setup** | ADVANCED | — | `DropdownSelect` | Mono / Stereo / Dual Mono / Mid-Side |
| **Linear Phase FIR Length** | ADVANCED | — | `DropdownSelect` | 257 tap / 513 tap / 1025 tap (internal) |
| **Analog Drift** | ADVANCED | **Medium** | `RotaryKnob` | Component tolerance simulation (0–100 %) |
| **Transformer Iron %** | ADVANCED | **Medium** | `RotaryKnob` | Output transformer saturation (0–100 %) |

> **Layout:** 10 vertical strips (Low → High). Each strip = Gain (Large) | Freq (Medium) | Q (Medium) | Type (Dropdown) | Enabled/Solo (Toggles).  
> **Metering:** IN / OUT peak + spectral analyser overlay (Main Panel).

---

## 3. `greg_pultec` — Passive Tube EQ (Pultec-style)

*Planned plugin — parameter set derived from classic topology & `GUI_SPEC.md` EQ archetype.*

| Parameter | Category | Knob Size | UI Widget | Notes |
|-----------|----------|-----------|-----------|-------|
| **Low Frequency** | MAIN | **Medium** | `DropdownSelect` | 20 / 30 / 60 / 100 Hz |
| **Low Boost** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB, ±16 dB) | Interacts with Low Attenuation |
| **Low Attenuation** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB, ±16 dB) | Simultaneous boost+cut = Pultec “low-end trick” |
| **High Frequency** | MAIN | **Medium** | `DropdownSelect` | 3k / 4k / 5k / 8k / 10k / 12k / 16k Hz |
| **High Boost** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB, ±16 dB) | Broadband shelf |
| **High Attenuation** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB, ±16 dB) | Broadband cut |
| **Makeup Gain** | MAIN | **Large** | `RotaryKnob` (bipolar, 12 o’clock = 0 dB) | Non-linear tube output stage |
| **Input Drive** | MAIN | **Medium** | `RotaryKnob` | Tube preamp saturation (0–100 %) |
| **Bypass** | MAIN | — | `ToggleSwitch` | Click-free micro-crossfade |
| **Oversampling** | ADVANCED | — | `DropdownSelect` | 1× / 2× / 4× / 8× |
| **Channel Setup** | ADVANCED | — | `DropdownSelect` | Mono / Stereo / Dual Mono / Mid-Side |
| **Transformer Iron %** | ADVANCED | **Medium** | `RotaryKnob` | Output transformer hysteresis (0–100 %) |
| **Analog Drift** | ADVANCED | **Medium** | `RotaryKnob` | Component variance (0–100 %) |
| **PSU Sag** | ADVANCED | **Medium** | `RotaryKnob` | Power-supply droop under load (0–100 %) |

> **Layout:** Two vertical column groups (Low / High). Each group: Freq (Medium) | Boost (Large) | Atten (Large).  
> Centre: Makeup Gain (Large) | Input Drive (Medium).  
> **Metering:** IN / OUT VU + THD indicator.

---

## 4. Originality & Trademark Compliance Checklist

| Item | Status | Mitigation |
|------|--------|------------|
| Plugin IDs (`greg_76`, `greg_surgical_eq`, `greg_pultec`) | ✅ Original | Prefixed with `greg_` brand |
| Parameter names | ✅ Generic | No hardware model numbers (e.g. “1176”, “LA-2A”, “EQP-1A”) |
| Revision Profile options | ✅ Descriptive | “Vintage D / Vintage E / Modern F” — no “Rev D/E/F” trademarks |
| “All Buttons In” → **All Buttons Mode** | ✅ Renamed | Avoids exact phrase associated with 1176 |
| “British Mode” (colloquial) | ✅ Acceptable | Generic descriptor |
| Filter type names (Bell, Shelf, etc.) | ✅ Standard DSP terms | No trademark conflict |
| Preset names (future) | 🔲 To define | Will use abstract names (“Vocal Clarity”, “Drum Punch”, etc.) |

---

## 5. Template for Remaining Suite Plugins

For every other plugin in `INVENTARIO.txt`, apply the same decision matrix:

| Plugin Family | MAIN (always visible) | ADVANCED (collapsible) |
|---------------|------------------------|------------------------|
| **Compressors / Dynamics** | Threshold, Output/Makeup (Large); Input, Ratio, Attack, Release, Mix (Medium); Mode selectors (Dropdown) | Oversampling, Knee, Detector HPF/LPF, Channel Setup, Invert Polarity, Bypass, Analog Drift, Transformer Iron %, PSU Sag |
| **Equalizers** | Band Gain (Large), Freq/Q (Medium), Filter Type (Dropdown), Band Enable/Solo (Toggle), Global Phase Mode | Oversampling, Channel Setup, FIR Length, Analog Drift, Transformer Iron %, PSU Sag |
| **Channel Strips / Consoles** | Drive/Trim + Master Out (Large); Crosstalk/Sat (Medium); Console Model/Topology (Dropdown) | Oversampling, Channel Setup, PSU Sag, Analog Variance, Transformer Hysteresis |
| **Saturators / Tape** | Input Drive/Sat + Output Trim (Large); Bias, Iron%, Head Bump, HF Roll-off, Mix (Medium); Speed/EQ Curve/Tube Mode (Dropdown) | Oversampling, Channel Setup, Analog Drift, PSU Sag, Reel Wow/Flutter |
| **Reverbs / Spatial** | Decay/Size + Dry-Wet (Large); Pre-Delay, Damping, Early/Late, Diffusion, Width, Modulation, Low/High Cut (Medium); Algorithm/Mode (Dropdown) | Oversampling, Channel Setup, FDN Seed, Modulation Depth, Spatial ITD/HRIR set |
| **Matrices / Phase / Utility** | Primary Gain/Phase Shift (Large); Width/Micro-Delay (Medium); Polarity (Toggle) | Oversampling, Channel Setup, Precision Mode |

> **Rule of thumb:** If the control exists on the original hardware front panel → **MAIN**. If it models internal circuit physics, modern convenience, or multi-channel routing → **ADVANCED**.

---

*End of document — ready for `gui_layout.json` generation per plugin.** Generato da Lead UI/UX Product Designer, Greg Audio — Alberto Gregorio Ferraris
# greg_limiter — Plugin CLAP

## Panoramica
Plugin audio CLAP emulazione analogica. Parte della suite Analog-vibe.

## Specifiche Tecniche
- **Formato**: CLAP + VST3
- **Canali**: Mono / Stereo / Dual Mono / M/S
- **Sample Rate**: 44.1 / 48 / 96 kHz
- **Latenza**: 262 campioni
- **Bypass**: Click-free con micro-crossfade

## Misure Strumentali (0 VU = -18 dBFS)
| Parametro | Valore |
|---|---|
| THD % medio | 2.313% |
| Guadagno Bode medio | 91.93 dB |
| Latenza | 262 campioni |

## Grafici SVG
- [Bode](greg_limiter_bode.svg) — Risposta in frequenza dB + Fase
- [Armoniche](greg_limiter_harmonics.svg) — Spettro H2-H7 + THD%
- [Transfer](greg_limiter_transfer.svg) — Curva I/O statico-dinamica
- [Transient](greg_limiter_transient.svg) — Inviluppo Gain Reduction

## Parametri DAW
Vedi `docs/plugins/greg_limiter.txt` per la tabella completa dei parametri.

---
*Generato automaticamente da clap-validator-host --update-docs*

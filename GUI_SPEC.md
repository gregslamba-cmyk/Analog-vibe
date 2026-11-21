# GUI_SPEC.md - ARCHITETTURA INTERFACCIA GRAFICA COMUNE (GREG ANALOG SUITE)

## 1. OBIETTIVO E STACK TECNOLOGICO
- **Framework:** `nih_plug_vizia` (basato su `baseview` + rendering vettoriale GPU/CPU **FemtoVG**).
- **Multipiattaforma:** Linux (X11/Wayland), Windows, macOS.
- **Zero impatto sul thread Audio:** Nessun lock o allocazione in `process()`. Telemetria tramite `AtomicU32` (meter) e ring-buffer lock-free pre-allocato in `initialize()` (spettrometro FFT).

---

## 2. LAYOUT GENERALE DELLA FINESTRA (DIMENSIONI FISSE, AREA CENTRALE PERSISTENTE)

```text
+---------------------------------------------------------------------------------------+

| [HEADER]  Nome Plugin | Preset | Routing (Mono/St/M-S/Dual/Multi) | OS | [ + ADV ]    |
+----------+-----------------------------------------------------------------+----------+

|          |                                                                 |          |
|   OUT    |                  MAIN PANEL (Sempre visibile)                   |    IN    |
|  METER   |           (Controlli originali Hardware: Knob/Btn/Slider)       |  METER   |
|  (SX)    |                                                                 |   (DX)   |
|  1..N ch +-----------------------------------------------------------------+  1..N ch |
|          |             ADVANCED PANEL (Espansione verso il basso)          |          |
|          |   (Sidechain EQ/Filtri | Saturazione/Drift | Spettrometro FFT)  |          |
+----------+-----------------------------------------------------------------+----------+

| [FOOTER]  Phase Correlation Meter (-1 .. 0 .. +1) | Gain Reduction (-dB)              |
+---------------------------------------------------------------------------------------+
```

### Dettaglio Specifiche:
1. **Comportamento Pannello `[ + ADV ]` (Area Centrale Persistente):**
   - Il **Main Panel** rimane **sempre visibile e accessibile** nella parte centrale superiore.
   - Premendo `[ + ADV ]` nell'header, la finestra si **espande verso il basso** mostrando il cassetto **Advanced Panel** sotto il Main Panel (mentre le colonne Meter SX/DX si estendono verticalmente per accompagnare l'intera altezza della finestra).
   - Ripremendo `[ - ADV ]`, la finestra si richiude alla dimensione compatta originale.

2. **Colonne Meter OUT (SX) e IN (DX) — Multicanale Dinamico (Tutti i Canali):**
   - I meter mostrano **simultaneamente tutte le barre dei canali attivi** negoziati con la DAW host (es. 1 barra in Mono, 2 in Stereo/M-S/Dual Mono, 6 in 5.1, 8 in 7.1, fino a `MAX_CHANNELS = 16`).
   - La larghezza delle singole barre si adatta automaticamente alla larghezza fissa della colonna meter (es. 16 px ciascuna in Stereo, 4–6 px affiancate in Multicanale 8ch/16ch).
   - Ogni canale ha barra istantanea (verde -> giallo -> rosso), tacca di **Peak Hold** (decadimento 1.5s) e lettura numerica del picco massimo + indicatore **Clip** (> 0.0 dBFS) resettabile al click.

3. **Footer Bar — Phase Correlation & Gain Reduction (Negativo):**
   - **Phase Correlation Meter:** Scala `-1.0` (rosso) <-> `0.0` (giallo) <-> `+1.0` (verde).
   - **Gain Reduction (GR) Meter (in negativo):** Lettura da `0.0 dB` verso `-30.0 dB` (colore ambra/rosso) + readout numerico negativo (es. `-4.2 dB`), visibile sui moduli di dinamica.

4. **Primitive Controlli Standardizzate (`crates/greg_gui_core`):**
   - `GregKnob`: Controllo rotativo vettoriale (continuo o a scatti), `Shift + Drag` per regolazione fine, doppio click per reset, valore numerico visibile.
   - `GregButton`: Interruttore toggle o selettore multi-posizione (es. Ratio 1176, mode switch).
   - `GregSlider`: Fader lineare orizzontale/verticale.
   - `GregSpectrum`: Spettrometro FFT real-time (2048/4096 punti su thread GUI) attivo solo nei plugin che lo richiedono e solo quando il pannello Advanced è aperto.


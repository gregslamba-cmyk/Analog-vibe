# GUI_SPEC.md — LINEE GUIDA DESIGN & ARCHITETTURA UI ANALOG-VIBE

## 1. Filosofia Estetica & Ergonomia Studio
- **Stile Hardware Autentico ma Leggibile:** Pannelli scuri antiriflesso (`#181A1F` / `#20232A`), accenti caldi ambra/ottone (`#E5A93B`), scale graduate nitide e feedback visivo immediato senza affaticamento in lunghe sessioni di mix.
- **Identità Visiva per Famiglia Hardware:**
  - *Vintage Tube / Opto (`greg-a2a`, `greg-child`, `greg-pultec`):* manopole stile Bakelite / Chickenhead, tonalità calde e VU meter ambrati.
  - *British Console / FET (`greg_76`, `greg-bus-comp`, `Neve/SSL`):* selettori a scatti colorati (stile Marconi/SSL) e indicatori LED ad alto contrasto.
  - *Surgical / Modern (`greg-surgical-eq`, `greg-limiter`, `greg_bay`):* grafica tecnica pulita, curve vettoriali nitide e lettura numerica diretta.
- **Gerarchia a Due Livelli:**
  - **Main Panel (sempre visibile):** solo i controlli primari dell'unità hardware reale + meter `IN` / `OUT` / `GR`.
  - **Pannello Avanzato a Scomparsa (`+ ADV`):** parametri fisici nascosti del circuito (*Transformer Iron %*, *Analog Drift*, *PSU Sag*, *Oversampling*), per mantenere il pannello principale pulito e fedelissimo all'originale.

## 2. Regole Tecniche Vizia & Real-Time Safety (`crates/greg_gui_core`)
- **Zero-Crash UI Thread:** Registra sempre il `Model` Vizia (`EditorData.build(cx)`) nel `Context` prima di istanziare widget/slider e avvolgi la closure `cx` in `std::panic::catch_unwind` affinché `window_thread` non possa mai abortire la DAW.
- **Telemetria Lock-Free (60 FPS):** Scritture dal thread audio `process()` esclusivamente tramite `AtomicU32` / `AtomicBool` con `Ordering::Relaxed`; sanitizzazione obbligatoria `if v.is_finite() { v } else { 0.0 }` prima del disegno.
- **Layout JSON Esterno (`gui_layout.json`):** Supporto completo a coordinate assolute (`GuiLayoutSpec`) con fallback automatico alla griglia procedurale se il JSON manca o è invalido.

## 3. Specifiche Studio GUI Designer Standalone (`target/gui_designer.html`)
- **Separato dai Report di Test:** Nessun grafico Bode/THD nella vista Designer; area di lavoro focalizzata su **un singolo plugin alla volta**.
- **Libreria Widget a Sinistra:** Anteprima reale trascinabile di manopole rotative, fader, switch, meter (`IN`, `OUT`, `GR`) e spettro.
- **Smart Guides Magnetiche & Riferimenti Fissi:** Allineamento automatico alla stessa altezza (`Y`), stesso centro (`X`) e bordi/sezioni della finestra.
- **Menu Contestuale Tasto Destro (`Right-Click`):** Assegnazione diretta del parametro CLAP (`param_id`), cambio stile widget, duplicazione ed eliminazione, più selettore colori e tasto **Salva `gui_layout.json`**.

---

## Standard Layout & Interazione GUI 2D (`greg_gui_core` v2)
- **3 Dimensioni Manopola (`KnobSize`):**
  - `KnobSize::Large` (`64px`): Controlli primari / *hero* (es. `Threshold`, `Output / Makeup`, `Drive / Trim`, Gain EQ principali).
  - `KnobSize::Medium` (`48px`): Controlli secondari / dinamici (`Input`, `Ratio` continuo, `Attack`, `Release`, `Mix`, Frequenze).
  - `KnobSize::Small` (`36px`): Predisposto nel motore grafico per rifiniture future, **non utilizzato** nei layout base attuali.
- **Cella Controllo Non Sovrapposta (`VStack` a 3 fasce centrate):**
  1. Top (`18px`): Label parametro bianca centrata (`hoverable(false)`).
  2. Center (`48px–68px`): Controllo grafico (`RotaryKnob`, `DropdownSelect`, `ToggleSwitch`).
  3. Bottom (`18px`): Valore formattato ambra centrato (`hoverable(false)`).
- **Menu a Tendina (`DropdownSelect`) & Interazione Mouse:**
  - Tutti i parametri discreti / enum (`Oversampling`, `Model / Revision`, `Topology`, `Filter Type`, `Knee`) usano `DropdownSelect` cliccabile con menu a tendina.
  - I parametri continui (incluso `Ratio` sui compressori) usano `RotaryKnob` con drag verticale relativo fluido (`200.0 px` per tutta la corsa `0.0..1.0`, `0.1x` con `Shift`) e `cx.needs_redraw()` emesso **solo** al cambio effettivo del valore.
  - Il footer mostra il valore dinamico reale della Gain Reduction in dB (`format!("{:.1} dB", gr_db)`), mai placeholder statici, e l'altezza della finestra si adatta alle righe effettive della griglia.
- **4 Archetipi di Layout Standardizzati:**
  1. **Compressori & Dinamica (10 plugin):** `Threshold` + `Output/Makeup` (`Large`); `Input`, `Ratio` (rotativo), `Attack`, `Release`, `Mix` (`Medium`); selettori modalità in `DropdownSelect`; `Stereo Link` / `Sidechain` in `ToggleSwitch`.
  2. **Equalizzatori (2 plugin):** Gain principali (`Large`), Frequenze/Q/Attenuazione (`Medium`), curve/filtri/fase in `DropdownSelect`.
  3. **Canali & Console (3 plugin):** `Drive/Trim` + `Master/Output` (`Large`), `Crosstalk/Sat` (`Medium`), `Console Model` / `Topology` in `DropdownSelect`.
  4. **Matrici & Fase (3 plugin):** Controllo primario (`Gain` / `Phase Shift` in `Large`), secondario (`Width` / `Micro-Delay` in `Medium`), polarità in `ToggleSwitch`.

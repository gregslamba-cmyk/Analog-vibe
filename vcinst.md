# Analog-Vibe — Regole Architetturali, Guida ai File & Metodo Operativo per l'AI

Questo documento costituisce la **guida normativa e operativa primaria** per qualsiasi agente AI o sviluppatore che opera sul repository **Analog-Vibe**. Stabilisce le regole architetturali inderogabili, la mappa completa di tutti i file di documentazione/ricerca e il ciclo di lavoro obbligatorio per garantire stabilità assoluta, zero bug e massima fedeltà fisica e acustica.

---

## 1. Mappa Completa dei Riferimenti Documentali

Prima di modificare o creare codice nel progetto, l'AI DEVE consultare i documenti pertinenti secondo questa mappa:

| Area / Documento | Percorso File | Scopo & Contenuto Obbligatorio da Consultare |
| :--- | :--- | :--- |
| **Fisica dei Componenti Elettronici** | [`docs/specs/Guida_Componenti_Audio_Fisica_Acustica.pdf`](file:///home/greg/Documenti/analog%20vibes/docs/specs/Guida_Componenti_Audio_Fisica_Acustica.pdf) | **LA GUIDA FISICA DEL PROGETTO:** Da leggere OBBLIGATORIAMENTE prima di programmare qualsiasi emulazione analogica. Contiene dati fisici, equazioni e misure reali su tubi termoionici (ECC83, 12AX7, 12AT7, EF86), transistor (JFET 2N5457, Neve 2N3055/BC108, API 2520), celle VCA (DBX 202), trasformatori (UTC O-12, UTC A-10, Marinair, Carnhill), saturazione magnetica, risonanza, slew-rate e deriva termica. **Vietato sostituire questi dati con supposizioni o stime aneddotiche.** |
| **Dossier Ricerca Hardware Outboard** | [`docs/specs/outboard_hardware_research.md`](file:///home/greg/Documenti/analog%20vibes/docs/specs/outboard_hardware_research.md) | **FONTE DI VERITÀ CIRCUITALE:** Schemi a blocchi reali, flussi di segnale storici (es. *Input Attenuator → Transformer → JFET Shunt → Output Transformer*), soglie fisse, costanti di tempo in µs/ms/s, All-Buttons mode, Pultec trick, selettori di revisione storica e mappatura esatta dei moduli `core-dsp`. |
| **Coda Operativa Prioritizzata** | [`TODO.md`](file:///home/greg/Documenti/analog%20vibes/TODO.md) | Coda attiva dei task ordinati per priorità decrescente (`P0`..`P4`) con checkbox `[ ]` e `[x]`. |
| **Catalogo Plugin & Roadmap** | [`docs/PLUGIN_CATALOG.md`](file:///home/greg/Documenti/analog%20vibes/docs/PLUGIN_CATALOG.md) | Catalogo dei 18 plugin della suite, specifiche di tutti i plugin futuri/pianificati, regole di accorpamento delle revisioni hardware e la roadmap strategica dei 7 progetti post-Analog-Vibe. |
| **Specifiche API Circuiti Specializzati** | [`docs/specs/greg_a2a_api.md`](file:///home/greg/Documenti/analog%20vibes/docs/specs/greg_a2a_api.md)<br>[`docs/specs/greg_a3a_api.md`](file:///home/greg/Documenti/analog%20vibes/docs/specs/greg_a3a_api.md)<br>[`docs/specs/greg_child_api.md`](file:///home/greg/Documenti/analog%20vibes/docs/specs/greg_child_api.md) | Specifiche circuitali e costanti di rilascio ottiche e Vari-Mu per LA-2A, LA-3A e Fairchild 670. |
| **Schede Tecniche Singoli Plugin** | [`docs/plugins/<plugin_id>.txt`](file:///home/greg/Documenti/analog%20vibes/docs/plugins/) e `.md` | Schede individuali per ciascun plugin (parametri esposti, topologia DSP, curve, latenza esatta, test e limiti). |
| **Schede Moduli Core DSP** | [`docs/modules/<module_id>.txt`](file:///home/greg/Documenti/analog%20vibes/docs/modules/) | Schede dei moduli DSP condivisi (`transformer_hysteresis`, `psu_sag`, `analog_variance`, `linear_phase_fir`, ecc.). |
| **Architettura di Sistema** | [`ARCHITECTURE.md`](file:///home/greg/Documenti/analog%20vibes/ARCHITECTURE.md) | Firme pubbliche delle API, tipi e garanzie dei 43 moduli di `core-dsp`, convenzioni multicanale Mono → 7.1. |
| **Stato Operativo Corrente** | [`STATE.md`](file:///home/greg/Documenti/analog%20vibes/STATE.md) | Cronologia delle fasi completate, verifiche di compilazione e stato dei singoli plugin rimodellati. |
| **Inventario Workspace** | [`docs/INVENTARIO.txt`](file:///home/greg/Documenti/analog%20vibes/docs/INVENTARIO.txt) | Inventario completo dei plugin e dei moduli implementati e pianificati. |
| **Guida Packaging Multi-Formato** | [`docs/PACKAGING.md`](file:///home/greg/Documenti/analog%20vibes/docs/PACKAGING.md) | Istruzioni per compilare e impacchettare bundle CLAP, VST3, LV2 (`.lv2`) e Audio Unit (`.component`) con `clap-wrapper`. |
| **Guida Strumenti di Test** | [`docs/testing/clap_signal_host.txt`](file:///home/greg/Documenti/analog%20vibes/docs/testing/clap_signal_host.txt) | Documentazione dell'host di test offline per stress-test di buffer irregolari e latenza. |
| **Guida DSP SDK `core-dsp`** | [`core-dsp/README.txt`](file:///home/greg/Documenti/analog%20vibes/core-dsp/README.txt) | Manuale completo per sviluppatori: precisione `f64`, 43 moduli per famiglia, esempi d'uso e test. |
| **Guida Host Bridge `plugin-support`** | [`plugin-support/README.txt`](file:///home/greg/Documenti/analog%20vibes/plugin-support/README.txt) | Manuale per layout I/O (`SURROUND_IO_LAYOUTS`, `DYNAMICS_IO_LAYOUTS`), sidechain, metri e macro `export_greg_plugin!`. |
| **Linee Guida Design & UI (`GUI_SPEC.md`)** | [`GUI_SPEC.md`](file:///home/greg/Documenti/analog%20vibes/GUI_SPEC.md) | Filosofia estetica studio, separazione Main vs `+ ADV`, telemetria 60 FPS e specifiche Studio GUI Designer. |
| **Guida Motore GUI `greg_gui_core`** | [`crates/greg_gui_core/README.txt`](file:///home/greg/Documenti/analog%20vibes/crates/greg_gui_core/README.txt) | Manuale GUI Vizia, telemetria lock-free DSP → UI, fallback `GREG_NO_GUI=1` e schema JSON `GuiLayoutSpec` (`gui_layout.json`). |
| **Guida Validatore `clap-validator-host`** | [`tools/clap-validator-host/README.txt`](file:///home/greg/Documenti/analog%20vibes/tools/clap-validator-host/README.txt) | Manuale del banco di collaudo universale CLAP (misure THD, armoniche H2–H7, report HTML/SVG e GUI Layout Designer). |
| **Panoramica Suite Analog-Vibe** | [`README.md`](file:///home/greg/Documenti/analog%20vibes/README.md) e [`README.txt`](file:///home/greg/Documenti/analog%20vibes/README.txt) | Presentazione generale, elenco dei 18 plugin, requisiti di sistema, build release e Doppia Licenza. |

---

## 2. Metodo Operativo Obbligatorio per l'AI

Ogni agente AI deve eseguire il lavoro in modo sistematico, senza saltare passaggi o procedere per tentativi:

```
┌────────────────────────────────────────────────────────────────────────┐
│                      FLUSSO OPERATIVO OBBLIGATORIO                     │
├────────────────────────────────────────────────────────────────────────┤
│ 1. Leggi TODO.md          → Identifica il task attivo a massima priorità│
│ 2. Consulta la Ricerca    → Leggi Guida Componenti PDF e Outboard MD   │
│ 3. Sviluppa & Testa       → Codice DSP f64, zero allocazioni, test unit│
│ 4. Spunta [x] in TODO.md  → Marca il task completato                   │
│ 5. Aggiorna STATE.md      → Riassumi l'avanzamento operativo           │
│ 6. Allinea Documentazione → Aggiorna schede, README.md e README.txt    │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Regole di Fedeltà Analogica & Accorpamento Hardware

1. **Accorpamento Obbligatorio delle Versioni di uno Stesso Outboard:**
   I plugin che simulano lo stesso dispositivo hardware attraverso diverse versioni, revisioni o annate **vanno obbligatoriamente accorpati in un unico plugin provvisto di apposito selettore di versione/profilo** nei parametri:
   - `greg_76`: accorpa 1176LN **Rev A**, **Rev D**, **Rev E**, **Rev F** e **All-Buttons-In** (commuta FET, bias, stadi Class A vs Class AB e trasformatori UTC).
   - `greg_160`: accorpa **160 VU**, **160X** e **160A** (selettore Hard Knee vs OverEasy, curve VCA Dbx 202, uscite bilanciate/sbilanciate, Infinity+).
   - `greg_165`: accorpa **165** e **165A** con selettore dedicato per il limitatore PeakStop.
   - `greg_2500`: accorpa **2500** e **2500+** con selettore Mix/Blend parallelo, THRUST e Feedback/Feed-Forward.
   - `greg_525`: accorpa **525 vintage** e **525A** (attacco fisso 15 µs vs tempi espansi 525A).
   - `greg_527`: accorpa **527** e **527A** con selettore di revisione.
   - `greg_sr22`: accorpa **SR22** (rack stereo) e **Select SV12** (modulo serie 500).
   - `greg_eqp1a`: accorpa **EQP-1A** ed **EQP-1A3** (interazione low boost/attenuation e trucco Pultec).
   - `greg_vari_mu_comp`: accorpa **Variable Mu Standard** e **Mastering Version**.
2. **Modelli Intrinsecamente Distinti:**
   Dispositivi con topologie, controlli e percorsi di segnale totalmente differenti restano plugin separati (es. Neve 1073 vs 1081 vs 1084 vs 88RS; API 525 FET vs 527 VCA vs 225L vs 2500 bus vs T25 tube; banco analogico Harrison 32C vs profilo software Mixbus).
3. **Flusso del Segnale Reale:**
   Il segnale deve attraversare gli stadi nell'ordine esatto dello schema elettrico reale documentato in `outboard_hardware_research.md` (es. *Input Transformer → Detector/Attenuatore → Makeup Amp → Output Transformer*). Non posizionare saturatori generici a valle.
4. **Calibrazione dei Livelli Nominale:**
   Tutti gli stadi analogici interni sono calibrati su `0 VU = +4 dBu = -18.0 dBFS` (headroom nominale di 18 dB prima del clipping a 0 dBFS).

---

## 4. Regole Architetturali Inderogabili del Codice

### 4.1 Sicurezza del Codice & Memory Safety
- **`#![forbid(unsafe_code)]` tassativo:** Vietato qualsiasi blocco `unsafe` nell'intero codebase (`core-dsp`, `plugin-support`, `greg_gui_core`, tutti i plugin e i tool di supporto).
- **Divieto di Panic nel Thread Audio:** Severamente vietato l'uso di `.unwrap()`, `.expect()`, `panic!()` o indicizzazioni di slice non protette all'interno del percorso audio `process()`.
- **Sanitizzazione Numerica Continua:** Tutti i valori `NaN` e `Inf` devono essere sanitizzati immediatamente a `0.0`. Ogni plugin deve includere il modulo `PanicProtection` per gestire degradazioni anomale senza crash della DAW.

### 4.2 Vincoli Real-Time nel Callback Audio (`process()`)
- **Zero Allocazioni Heap:** Nessun `Vec::new()`, `Box::new()`, `clone()` di strutture non-Copy, stringhe o ridimensionamenti di buffer in `process()`. Tutta la memoria deve essere pre-allocata in `initialize()` o `prepare()`.
- **Zero Lock o Mutex Bloccanti:** Nessun `std::sync::Mutex` bloccante nel thread audio. Lo scambio dati audio → UI avviene tramite primitive atomiche lock-free (`AtomicU64`, `AtomicU32`, `AtomicBool` con `Ordering::Relaxed`).
- **Zero Chiamate di Sistema o I/O:** Nessun accesso al filesystem, log sincroni su disco o chiamate di rete nel thread audio.
- **Gestione Blocchi Arbitrari:** Il processore deve gestire qualsiasi dimensione di blocco fornita dall'host (`1..max_block_size`), inclusi buffer irregolari o molto piccoli (1, 7, 64 campioni).

### 4.3 Precisione di Calcolo & Silenzio a Riposo
- **Elaborazione Interna a 64-bit (`f64`):** Tutti i calcoli DSP, i coefficienti dei filtri e i modelli non lineari operano internamente in doppia precisione (`f64`). La conversione da/verso `f32` avviene esclusivamente ai bordi del buffer host.
- **Silenzio Digitale Assoluto a Riposo:** Con ingresso nullo (`0.0`), l'uscita deve essere esattamente `0.0`. I generatori di rumore termico e ripple di rete devono essere **RIGOROSAMENTE SPENTI DI DEFAULT (`OFF` / `0.0`)**.

### 4.4 Routing Dinamico Multicanale (`Mono → 7.1`)
- **Supporto 1..8 Canali Indipendenti:** Ogni plugin deve supportare dinamicamente da 1 a 8 canali:
  Mono (1 ch), Stereo (2 ch), LCR (3 ch), Quad (4 ch), 5.0 (5 ch), 5.1 (6 ch), 7.0 (7 ch), 7.1 (8 ch).
- **Isolamento e Linking:** Lo stato DSP per-canale è isolato via `RoutingMatrix`. Nei plugin con `Stereo Link`, il massimo key dinamico viene calcolato su tutti i canali attivi `0..channel_count` e applicato uniformemente a tutti i canali linkati.

### 4.5 Catena Analogica Modellata & Trasformatori
- **Modellazione Fisica Obbligatoria:** Ogni plugin analogico DEVE integrare allo stadio di Input e di Output (e negli stadi intermedi previsti dalla topologia):
  - `TransformerHysteresis` (modello differenziale Jiles-Atherton con saturazione magnetica).
  - `AcCoupling` (disaccoppiamento DC e bias shift asimmetrico sui transienti).
  - `AnalogVariance` (deriva termica stocastica asincrona aggiornata a *block-rate*, zero `sin()` per campione).
  - Stadi non lineari appropriati (`TubeTriode`, `TubePentode`, `SolidStateClassA`, `SolidStateFet`, `DiscreteOpAmp`, `PsuSag`, `MillerRollOff`).
- **Controlli Utente (+ ADV):** Esporre nel pannello avanzato i controlli `Transformer / Iron` (0%..100%+) e `Analog Drift`.
- **Eccezione Utility Digitali:** I plugin digitali puri (`greg_phase`, `greg_ms_encoder`, `greg_ms_decoder`, `greg_surgical_eq`, `greg_leveler`, `greg_limiter`) restano trasparenti di default.

### 4.6 Latenza Esatta & Bypass Trasparente
- **Dichiarazione Esatta della Latenza:** Ogni plugin con oversampling, lookahead o convoluzione deve dichiarare all'host la latenza esatta in campioni tramite `latency_samples(&self) -> u32`.
- **Bypass Click-Free:** Il bypass interno esegue una dissolvenza incrociata morbida di 5 ms mantenendo l'allineamento di fase tra wet e dry.

### 4.7 Doppia Licenza & Esportazione Unificata
- **Doppia Licenza (GPLv3 + Commerciale):** Ogni crate dichiara `license = "GPL-3.0-or-later"`.
- **Macro Obbligatoria:** Ogni plugin esporta nativamente CLAP + VST3 tramite `export_greg_plugin!(PluginStruct, "nome-url")`.
- **Compatibilità LV2 e AUv2:** Generazione bundle tramite `scripts/package_all_formats.sh` basato su `clap-wrapper`.

### 4.8 Fallback GUI 2D Funzionale
- Ogni plugin include un'interfaccia 2D leggera creata tramite `greg_gui_core::create_greg_editor` (Main Panel, pannello a scomparsa `+ ADV`, meter In/Out e Gain Reduction).
- Se `GREG_NO_GUI=1` è impostato o su ambienti headless, il plugin esegue automaticamente il fallback all'interfaccia generica nativa della DAW senza crash.

---

## 5. Standard Obbligatorio di Commento del Codice

Ogni file sorgente `.rs` deve essere commentato secondo questo standard (con commenti scritti in **inglese** per leggibilità internazionale):

1. **Intestazione modulo (`//!`):** All'inizio di ogni file, descrivere lo scopo del modulo, la topologia circuitale/algoritmica emulata e i moduli `core-dsp` impiegati.
2. **Doc-comment pubblici (`///`):** Ogni `struct`, `enum`, campo di parametri (`Params`) e funzione pubblica deve riportare descrizione, unità di misura esatta (es. `dB`, `ms`, `Hz`, `samples`, `ratio`) e garanzie real-time (es. zero allocazioni heap).
3. **Commenti matematici DSP (`//`):** Ogni calcolo non banale in `initialize()` o `process()` (coefficienti filtri, costanti d'attacco/rilascio, curve non lineari, isteresi) deve spiegare la motivazione fisica, circuitale o acustica.
4. **Divieto di commenti ovvi:** Vietato commentare operazioni banali (es. `x += 1; // increment x`). Commentare solo il **perché fisico o matematico**.

---

## 4. Checklist di Conformità Obbligatoria per Ogni Plugin

Prima di considerare completato qualsiasi plugin della suite Analog-Vibe, verificare obbligatoriamente:
1. **Override Manuale Canali (`Channel Setup`):** Presente su **TUTTI** i plugin (`Auto`, `Mono`, `Stereo`, `LCR`, `Quad`, `5.0`, `5.1`, `7.0`, `7.1`) per correggere manualmente eventuali errori di autodetect dell'host DAW.
2. **Modern Extended Section su TUTTI i Processori di Dinamica (Compressori, Limiter, Gate/Expander):**
   - `SC Source` (`Internal` / `External` con fallback automatico) + `SC Listen`
   - Filtri Sidechain completi (`SC HPF`, `SC LPF`, `SC Bell` Freq/Q/Gain)
   - `Channel Mode` (`Stereo Link`, `Dual Mono`, `Mid/Side`) + controllo `Mix / Dry-Wet`.
3. **Composizione Modulare dei Channel Strip (`dsp_graph` + Riuso Moduli Singoli + Host CLAP/VST3):**
   - I motori DSP dei singoli moduli (**Preamp**, **EQ**, **Compressore/Dynamics**, già accorpati per revisioni storiche come da Sez. 3) devono essere esportati con `crate-type = ["cdylib", "rlib"]` affinché lo stesso codice funzioni sia come **plugin singolo** sia come **blocco DSP dentro i Channel Strip**.
   - Ogni **Channel Strip** (`greg_1073_strip`, `greg_4000_strip`, `greg_vision_strip`, ecc.) deve comporre tali moduli dentro `dsp_graph` (con ordine commutabile `Pre → EQ → Dyn` / `Pre → Dyn → EQ`), includere `aux_send` + `fader_console`, e integrare un **Sub-Host CLAP/VST3** per ospitare plugin esterni con PDC automatico al singolo campione.

## Layout Files
- **Physical storage**: Each plugin’s `gui_layout.json` must reside in the plugin’s root directory under `plugins/<name>/gui_layout.json`.  
  Examples:
  - `/home/greg/analog-vibes/plugins/greg_76/gui_layout.json`
  - `/home/greg/analog-vibes/plugins/greg_a2a/gui_layout.json`
  - `/home/greg/analog-vibes/plugins/greg_pultec/gui_layout.json`
  - … (all 18 plugins)
- **Rule**: Do NOT place any other layout‑related files outside these directories. All references in `lib.rs` and `create_greg_editor_with_params` must point to the same relative path (`main` & `advanced` sections).

*Added on 2026-11-23 by Direttore Tecnico.*

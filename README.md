# Analog-Vibe

Suite di plugin audio ad alta fedeltà in formato nativo **CLAP** e **VST3**, sviluppata interamente in Rust. L'intero ecosistema è progettato attorno alla libreria DSP condivisa **`core-dsp`**, che implementa l'elaborazione interna a 64-bit (`f64`), il divieto tassativo di codice non sicuro (`#![forbid(unsafe_code)]`), l'assenza totale di allocazioni heap o chiamate bloccanti nel thread audio in tempo reale (`process()`) e il routing dinamico universale da **Mono a 7.1 Surround (1–8 canali)**.

---

## 1. Architettura di Sistema

L'ecosistema di **Analog-Vibe** è strutturato su un'architettura modulare a livelli, dove ogni plugin riutilizza i componenti condivisi del workspace:

```
┌────────────────────────────────────────────────────────────────────────┐
│             SUITE DEI 18 PLUGIN ANALOG-VIBE (CLAP + VST3)              │
│ (greg_76, greg-a2a, greg-bus-comp, greg_nl_mixer, greg_bay, ...)      │
└──────────────┬─────────────────────────┬───────────────────────────────┘
               │                         │
               ▼                         ▼
┌───────────────────────────────┐ ┌──────────────────────────────────────┐
│        plugin-support         │ │            greg_gui_core             │
│ Layout I/O (Mono → 7.1),      │ │ Interfaccia vettoriale 2D (Vizia),   │
│ Sidechain aux, Bridge Metri,  │ │ Telemetria lock-free DSP → UI (60fps)│
│ Macro `export_greg_plugin!`   │ │ Fallback nativo host (`GREG_NO_GUI=1`)│
└──────────────┬────────────────┘ └──────────────────────────────────────┘
               │
               ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        core-dsp (Libreria Base)                        │
│ - Precisione interna a 64-bit (f64) & #![forbid(unsafe_code)]          │
│ - Zero allocazioni dinamiche o lock nel callback audio                 │
│ - Modellazione fisica componenti (Tubi, Trasformatori Jiles-Atherton,  │
│   Op-Amp discreti, VCA, PSU Sag, Deriva Termica deterministica)        │
│ - Motore Graph Audio `dsp_graph` con PDC automatico al campione        │
│ - Sommatore analogico Virtual Earth 32ch & Matrice di Diafonia         │
│ - Simulazione acustica di stanze 3D, Cabinet Klippel & Microfoni       │
│ - Audio Immersivo: Ambisonics (1°-3° ordine), HRIR Binaurale, VBAP     │
│ - Loudness Metering conforme a standard ITU-R BS.1770-4 / EBU R128     │
└────────────────────────────────────────────────────────────────────────┘
               │
               ▼
┌────────────────────────────────────────────────────────────────────────┐
│           STRUMENTI DI COLLAUDO & VALIDAZIONE OFFLINE                  │
│ clap-signal-host (stress-test blocchi) & clap-validator-host (THD/GUI) │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. I 18 Plugin della Suite Analog-Vibe

Tutti i plugin implementano la macro unificata `export_greg_plugin!`, esportando nativamente sia il punto d'ingresso **CLAP** (`clap_entry`) sia il factory **VST3** (`GetPluginFactory` / `ModuleEntry`) con ID deterministico e supporto completo al routing multicanale (1–8 canali) con sidechain ausiliaria:

### 2.1 Compressori e Processori di Dinamica
1. **`greg_76`**: Compressore FET vintage (riferimento 1176LN). Accorpa le revisioni storiche **Rev A**, **Rev D**, **Rev E**, **Rev F** e modalità **All-Buttons**, con trasformatori UTC O-12 / A-10 e circuiti di saturazione selezionabili.
2. **`greg-a2a`**: Compressore ottico valvolare (riferimento Teletronix LA-2A). Emula la cella elettroluminescente T4 (selettore Silverface vs Vintage Tube) con costante di tempo memoria a doppio inviluppo e doppio triodo 12AX7/12AT7.
3. **`greg-a3a`**: Compressore ottico a stato solido discreto (riferimento Teletronix LA-3A). Risposta rapida dei transistor discreti combinata con la cella fotocellulare T4B e stadio di guadagno API 2520.
4. **`greg_api_2500`**: Compressore stereo VCA da bus (riferimento API 2500). Accorpa le versioni **2500** e **2500+** con selettore Mix/Blend parallelo, circuito brevettato THRUST per bassi compatti e commutazione Feedback/Feed-forward.
5. **`greg-bus-comp`**: Compressore stereo VCA per mix bus (riferimento SSL 4000 G Bus Compressor). Stadio op-amp NE5534, VCA DBX e compressione d'incollaggio trasparente con saturazione progressiva.
6. **`greg-child`**: Compressore valvolare Vari-Mu leggendario (riferimento Fairchild 670 stereo / 660 mono). Modella il comportamento non lineare a transconduttanza variabile con 6 costanti di tempo storiche selezionabili e sag di alimentazione.
7. **`greg_dbx_160`**: Compressore VCA vintage (riferimento DBX 160). Accorpa le versioni **160 VU**, **160X** e **160A** con selettore OverEasy / Hard Knee, curve VCA Dbx 202, uscite bilanciate/sbilanciate e compressione Infinity+.
8. **`greg_dbx_165`**: Compressore VCA da studio (riferimento DBX 165). Accorpa le versioni **165** e **165A** con interruttore dedicato per il circuito limitatore PeakStop a diodi e attacco/rilascio automatici.
9. **`greg-gate`**: Noise Gate & Downward Expander analogico. Dotato di isteresi regolabile, rilevamento interno/esterno, filtri sidechain HPF/LPF dedicati e modalità Ducking ritmica.
10. **`greg-limiter`**: True-Peak Brickwall Limiter & Maximizer trasparente. Algoritmo lookahead ad alta precisione, oversampling anti-alias e ceiling calibrabile con distorsione inter-sample azzerata.

### 2.2 Equalizzatori
11. **`greg-pultec`**: Equalizzatore passivo valvolare di programma (riferimento Pultec EQP-1A / EQP-1A3). Modella l'interazione low boost/attenuation (trucco Pultec), make-up gain valvolare e risonanze a induttore.
12. **`greg-surgical-eq`**: Equalizzatore parametrico chirurgico a bande multiple. Modalità commutabile a Fase Minima o Fase Lineare (con latenza esatta dichiarata), filtri Bell, Shelf, High-Pass, Low-Pass, Notch e Band-Pass ad alta pendenza.

### 2.3 Matrici di Segnale e Utility Audio
13. **`greg-ms-encoder`**: Matrice ortonormale di codifica stereo Left/Right → Mid/Side con gestione trasparente del guadagno.
14. **`greg-ms-decoder`**: Matrice ortonormale di decodifica Mid/Side → Left/Right per elaborazioni stereo avanzate.
15. **`greg-phase`**: Correttore e allineatore di fase/polarità indipendente per ciascun canale con micro-ritardo temporale per riallineamento multi-microfonico.

### 2.4 Moduli Console & Routing Modulare
16. **`greg_nl_ch`**: Non-Linear Console Channel Strip. Emulazione completa del carattere di canale con modelli selezionabili tra **Neve 8078** (`CarnhillVtb9045` → `Neve2n3055_Bc108`), **SSL 4000G** (`Ne5534` → `Dbx202GoldCan`) e **API 1608** (`Api2520` → `Api2623`), controllo Drive, Channel ID deterministico e diafonia inter-canale.
17. **`greg_nl_mixer`**: Non-Linear Analog Summing Console a 32 canali (16 coppie stereo). Implementa il sommatore fisico `VirtualEarthSummer` con compressione dell'alimentatore condiviso (PSU sag), matrice di diafonia capacitiva `AdjacentCrosstalkMatrix` e trasformatori di uscita dedicati per ciascun banco analogico.
18. **`greg_bay`**: Modular Audio Patchbay & Channel Rack. Basato sull'engine `dsp_graph.rs` con Plugin Delay Compensation (PDC) automatico al singolo campione, consente di disporre liberamente fino a 4 slot DSP in 6 topologie di routing pre-compilate (Seriale, Split Parallelo, Mid/Side, Crossover a 2 bande, Doppio Parallelo, Invertito).

---

## 3. Requisiti di Sistema

- **Compilatore Rust & Cargo:** Rust 1.75 o superiore (consigliato canale stabile).
- **Sistemi Operativi:**
  - **Linux:** Kernel 5.x / 6.x. Formati generati: `.clap` e `.vst3`.
  - **macOS:** macOS 11+ (Intel & Apple Silicon). Formati: `.clap`, `.vst3`, e `.component` (AUv2) tramite `clap-wrapper`.
  - **Windows:** Windows 10/11 x64. Formati: `.clap` e `.vst3`.
- **Dipendenze X11/OpenGL (per la GUI Vizia su Linux):**
  ```sh
  sudo apt-get install -y libxcb-dri2-dev libxcb-icccm4-dev libxcb-image0-dev \
    libxcb-keysyms1-dev libxcb-randr0-dev libxcb-render-util0-dev libxcb-shape0-dev \
    libxcb-shm0-dev libxcb-xfixes0-dev libxcb-xinerama0-dev libgl1-mesa-dev
  ```

---

## 4. Compilazione, Test e Verifica

Tutte le operazioni possono essere eseguite tramite i comandi standard Cargo:

```sh
# 1. Verifica statica e conformità del codice
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 2. Esecuzione suite di test unitari completa (oltre 150 test deterministici)
cargo test --workspace

# 3. Compilazione ottimizzata in modalità Release
cargo build --workspace --release

# 4. Generazione documentazione Rustdoc dell'intero workspace
cargo doc --no-deps --workspace
```

Al termine della compilazione, i file binari condivisi (`libgreg_*.so`, `.dylib` o `.dll`) saranno disponibili in `target/release/`.

---

## 5. Collaudo e Validazione Offline

Analog-Vibe include due ambienti host dedicati per testare i plugin CLAP senza dipendere da una DAW commerciale:

### 5.1 `clap-signal-host` (Stress-Test DSP e Latenza)
Carica un plugin CLAP ed esegue test sintetici su blocchi irregolari (da 1 a 2048 campioni), verificando risposta impulsiva, latenza dichiarata, oversampling, crossfade di bypass e stabilità numerica:

```sh
cargo run --release -p clap-signal-host -- target/release/libgreg_76.so
cargo run --release -p clap-signal-host -- target/release/libgreg_nl_mixer.so
```

### 5.2 `clap-validator-host` (Misure THD, Spettro e GUI Designer)
Esegue un'analisi spettrale avanzata: misura THD e armoniche da H2 a H7 su 6 livelli di saturazione (-12 VU → +18 VU), genera diagrammi di Bode di ampiezza e fase, esporta i dati in CSV (`target/validation_data/`) e genera il report interattivo `target/validation_report.html` con il **GUI Layout Designer Drag & Drop** integrato per la personalizzazione visuale di `gui_layout.json`:

```sh
cargo run --release -p clap-validator-host -- target/release/libgreg_76.so
# Per aggiornare e validare tutti i plugin presenti in target/release/:
cargo run --release -p clap-validator-host -- --update-docs
```

---

## 6. Mappa della Documentazione

- [`TODO.md`](TODO.md): Coda operativa dei task prioritizzati (`[ ]` / `[x]`).
- [`vcinst.md`](vcinst.md): Regole architetturali inderogabili e metodo operativo obbligatorio per gli sviluppatori.
- [`docs/PLUGIN_CATALOG.md`](docs/PLUGIN_CATALOG.md): Catalogo completo dei plugin della suite e roadmap strategica dei 7 progetti futuri post-Analog-Vibe.
- [`ARCHITECTURE.md`](ARCHITECTURE.md): Specifiche dettagliate delle API, delle firme pubbliche e delle garanzie di `core-dsp`.
- [`STATE.md`](STATE.md): Registro cronologico dello stato di avanzamento operativo.
- [`core-dsp/README.txt`](core-dsp/README.txt): Guida tecnica approfondita all'SDK DSP `core-dsp`.
- [`plugin-support/README.txt`](plugin-support/README.txt): Guida all'esportazione multi-formato, layout I/O e bridging DAW.
- [`crates/greg_gui_core/README.txt`](crates/greg_gui_core/README.txt): Documentazione del motore grafico vettoriale 2D Vizia e schema `GuiLayoutSpec`.
- [`tools/clap-validator-host/README.txt`](tools/clap-validator-host/README.txt): Manuale completo del banco di validazione CLAP e del GUI Designer.
- [`docs/PACKAGING.md`](docs/PACKAGING.md): Istruzioni per il packaging multi-formato (CLAP, VST3, LV2, AUv2).

---

## 7. Licenza e Uso Commerciale (Dual Licensing)

Il progetto è regolato da un modello a **Doppia Licenza (Dual Licensing)**:

1. **Open-Source (GNU GPLv3 o successiva):**
   L'uso, lo studio, la modifica e la ridistribuzione del codice sono gratuiti e consentiti a chiunque, purché qualsiasi software derivato o incorporante questo codice rimanga anch'esso interamente open-source sotto licenza **GNU GPLv3** (o qualsiasi versione successiva). Consulta il file [`LICENSE`](LICENSE) per il testo completo della licenza.
2. **Licenza Commerciale Proprietaria:**
   Per aziende, studi di produzione o sviluppatori che desiderano integrare i moduli di `core-dsp`, le emulazioni hardware o i plugin completi all'interno di software proprietario a codice chiuso (senza essere soggetti ai vincoli copyleft della GPLv3), è disponibile una **licenza commerciale proprietaria a basso costo**.
   Per dettagli e richieste di licenza: [GitHub Repository / Issues](https://github.com/gregslamba-cmyk/Analog-vibe).

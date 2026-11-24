# Catalogo Plugin Analog-Vibe & Roadmap Futura

Questo documento raccoglie il catalogo completo dei plugin della suite **Analog-Vibe**, specificando lo stato di implementazione attuale, le regole di accorpamento delle revisioni hardware e la roadmap strategica per i progetti successivi.

---

## 1. Stato di Implementazione della Suite

### 1.1 I 18 Plugin della Suite Analog-Vibe
La suite Analog-Vibe comprende **18 plugin CLAP + VST3 nativi**, tutti con elaborazione interna a 64-bit (**f64**), `#![forbid(unsafe_code)]`, zero allocazioni o lock nel percorso audio e fallback automatico GUI:

| ID Plugin | Descrizione & Riferimento Hardware | Categoria | Stato |
| :--- | :--- | :--- | :--- |
| `greg_76` | Compressore FET vintage (1176LN). Accorpa le revisioni: Rev A, Rev D, Rev E, Rev F, All-Buttons. | Dinamica / FET | **Implementato** (Alpha) |
| `greg-a2a` | Compressore ottico valvolare (Teletronix LA-2A). Selettore cella T4 (Silverface vs Vintage Tube). | Dinamica / Opto | **Implementato** (Alpha) |
| `greg-a3a` | Compressore ottico a stato solido discreto (Teletronix LA-3A) con stadio API 2520. | Dinamica / Opto | **Implementato** (Alpha) |
| `greg_api_2500` | Compressore stereo VCA bus (API 2500). Accorpa le versioni 2500 e 2500+ (Mix/Blend, THRUST). | Dinamica / VCA | **Implementato** (Alpha) |
| `greg-bus-comp` | Compressore stereo VCA da mix bus (SSL 4000 G Bus Compressor). | Dinamica / VCA | **Implementato** (Alpha) |
| `greg-child` | Compressore Vari-Mu valvolare (Fairchild 670 stereo / 660 mono). | Dinamica / Vari-Mu | **Implementato** (Alpha) |
| `greg_dbx_160` | Compressore VCA (DBX 160). Accorpa le versioni 160 VU, 160X e 160A con selettore dedicato. | Dinamica / VCA | **Implementato** (Alpha) |
| `greg_dbx_165` | Compressore VCA (DBX 165). Accorpa 165 e 165A con selettore PeakStop Limiter. | Dinamica / VCA | **Implementato** (Alpha) |
| `greg-gate` | Noise Gate & Downward Expander analogico con isteresi, filtri sidechain e ducking. | Dinamica / Gate | **Implementato** (Alpha) |
| `greg-limiter` | True-Peak Limiter / Maximizer brickwall trasparente con oversampling e lookahead. | Dinamica / Limiter | **Implementato** (Alpha) |
| `greg-pultec` | EQ passivo valvolare (Pultec EQP-1A / EQP-1A3) con interazione boost/attenuation. | Equalizzatore | **Implementato** (Alpha) |
| `greg-surgical-eq` | EQ parametrico chirurgico a fase lineare e fase minima con filtri ad alta pendenza. | Equalizzatore | **Implementato** (Alpha) |
| `greg-ms-encoder` | Matrice ortonormale di codifica stereo Left/Right → Mid/Side. | Utility / Matrice | **Implementato** (Alpha) |
| `greg-ms-decoder` | Matrice ortonormale di decodifica stereo Mid/Side → Left/Right. | Utility / Matrice | **Implementato** (Alpha) |
| `greg-phase` | Correttore e allineatore di polarità/fase per-canale o linkato con delay di fase. | Utility / Fase | **Implementato** (Alpha) |
| `greg_nl_ch` | Non-Linear Console Channel Strip (Neve 8078 / SSL 4000G / API 1608 channel models). | Console / Canale | **Integrazione P2** |
| `greg_nl_mixer` | Non-Linear Analog Summing Console (16 Stereo Inputs / 32ch, Virtual Earth bus & crosstalk). | Console / Sommatore | **Integrazione P2** |
| `greg_bay` | Modular Audio Patchbay & Channel Rack (4 slot DSP, 6 topologie, PDC automatico). | Modulare / Routing | **Integrazione P2** |

---

## 2. Plugin Pianificati / Futuri da Implementare

### 2.1 Dinamiche, EQ e Preamplificatori

| ID Plugin | Obiettivo / Riferimento Hardware da Modellare | Note Architetturali |
| :--- | :--- | :--- |
| `greg_api_525` | Compressore feedback FET API 525 vintage e 525A | Accorpa 525 e 525A (attacco fisso vintage 15 µs vs tempi estesi 525A). |
| `greg_api_527` | Compressore VCA serie 500 API 527 e 527A | Accorpa 527 e 527A con selettore di revisione. |
| `greg_api_sr22` | Compressore VCA dual-channel circuito ATI Paragon | Accorpa versione rack SR22 e modulo serie 500 SV12. |
| `greg_api_225l` | Modulo compressore VCA serie 200 API 225L | Topologia feedback/feedforward selezionabile. |
| `greg_api_t25` | Compressore a due canali FET feedback valvolare Class A API T25 | Modello con trasformatore e doppio triodo. |
| `greg_api_529` | Compressore stereo bus serie 500 API 529 | Miniaturizzazione del circuito 2500 per serie 500. |
| `greg_ssl_eq` | EQ di canale SSL 4000 (Black 242 / Brown 02 / G-Series 292) | Selettore curve e risonanze di filtro analogico. |
| `greg_neve_eq` | EQ a induttori Neve 1073 (vintage / SPX-D con selettore) | Modello di saturazione a induttore e trasformatore Marinair. |
| `greg_neve_pre` | Preamplificatore discreto Neve 1073 | Modello con trasformatori Carnhill/Marinair e stadio BA283. |
| `greg_api_pre` | Preamplificatore discreto API 512c | Op-amp discreto API 2520 e trasformatore API 2623. |
| `greg_harrison_pre` | Preamplificatore da console Harrison 32C | Stadio discreto a bassissimo rumore e saturazione morbida. |
| `greg_harrison_comp` | Compressore VCA da canale Harrison 32C | Riconoscimento rapido dei transienti e smoothing. |
| `greg_harrison_eq` | EQ a 4 bande con filtri HPF/LPF Harrison 32C | Curve proporzionali Q e filtri risonanti unici. |
| `greg_manley_eq` | EQ passivo valvolare Massive Passive | Filtri passivi paralleli con make-up a valvole e trasformatori. |
| `greg_manley_comp` | Compressore valvolare Variable Mu | Accorpa le versioni Standard e Mastering con selettore di versione. |
| `greg_multiband_comp`| Compressore multibanda a 4 bande | Crossover Linkwitz-Riley a fase lineare/minima e detector indipendenti. |
| `greg_deesser` | De-Esser trasparente e musicale | Rilevamento dinamico delle sibilanti e filtro split-band / wideband. |
| `greg_depop` | Soppressore dinamico di pop e rumble microfonici | Rilevamento plosive ad alta velocità con attenuazione selettiva LF. |
| `greg_eq_match` | Equalizzatore a corrispondenza spettrale | Apprendimento curva target via sidechain ed estrapolazione FIR. |
| `greg_expander` | Downward Expander analogico | Espansione morbida del range dinamico con isteresi e filtri sidechain. |
| `greg_ducking` | Processore di ducking ritmico/vocale | Inviluppo con curve musicali, sidechain key e tempo sync. |
| `greg_lin_eq` | EQ a fase lineare chirurgico per mastering | Convoluzione partizionata, filtri simmetrici, zero sfasamento. |
| `greg_lin_comp` | Compressore multibanda a fase lineare | Crossover FIR a somma perfetta e dinamica trasparente. |

---

### 2.2 Channel Strip Completi di Console

Ogni channel strip modella fedelmente il percorso completo di canale (Preamp → Filtri → EQ → Dinamica → Routing / Pan → Output):

| ID Plugin | Modello di Riferimento | Caratteristiche Chiave |
| :--- | :--- | :--- |
| `greg_neve_1073_strip` | Neve 1073 Channel Strip | Preamp BA283, induttori EQ, trasformatori Marinair. |
| `greg_neve_1081_strip` | Neve 1081 Console Strip | EQ a 4 bande con medie parametriche e curve vintage. |
| `greg_neve_1084_strip` | Neve 1084 Console Strip | 1073 esteso con frequenze alte selezionabili e filtro passa-banda. |
| `greg_neve_88rs_strip` | Neve 88RS Modern Console Strip | Strip di console moderno con dinamica trasparente e EQ versatile. |
| `greg_harrison_32c_strip` | Harrison 32C Analog Console Strip | Strip completo del leggendario banco analogico 32C. |
| `greg_harrison_mixbus_strip` | Harrison Mixbus Software Profile | Emulazione del flusso di summing e saturazione stile Mixbus. |
| `greg_ssl_4000e_strip` | SSL 4000 E Channel Strip | Preamplificatore, filtri, VCA expander/gate, comp e Brown/Black EQ. |
| `greg_ssl_4000g_strip` | SSL 4000 G Channel Strip | Modello G-Series con G-EQ (292) e curve di compressione dedicate. |
| `greg_ssl_9000j_strip` | SSL 9000 J SuperAnalogue Strip | Banda ultra-larga, circuitazione accoppiata in DC, THD minimale. |
| `greg_api_vision_strip` | API Vision Console Channel Strip | Pre 212L, EQ 550L/560L, Comp 225L, Gate 235L. |
| `greg_api_legacy_plus_strip` | API Legacy Plus Console Strip | Architettura classica API con op-amp 2520 e saturazione calda. |
| `greg_trident_a_range_strip` | Trident A-Range Console Strip | EQ induttivo leggendario a 4 bande e preamplificatore discreto. |
| `greg_trident_80b_strip` | Trident Series 80B Strip | Carattere rock anni '80 con EQ aggressivo e preamplificatore IC/discreto. |
| `greg_amek_9098_strip` | AMEK 9098 (Rupert Neve Design) | Equalizzatore e dinamica con circuito TLA e curve ad alta precisione. |
| `greg_mci_jh500_strip` | MCI JH-500 Console Strip | Banco classico della disco/rock anni '70 con suono denso e caldo. |

---

### 2.3 Utility, Creativi, Spazio e Restauro

| ID Plugin | Categoria / Descrizione | Riferimento / Specifiche |
| :--- | :--- | :--- |
| `greg_leveler` | Vocal Rider / Automatic Leveler | Algoritmo basato su loudness K-weighted e ramping dB/s trasparente. |
| `greg_tape` | Emulatore di registratore a nastro analogico | Studer A800 Mk III / Ampex ATR-102 (bias, testine, wow & flutter, isteresi). |
| `greg_exciter` | Exciter armonico psicoacustico | Riferimento Aphex Aural Exciter Type C (sintesi di fase e brillantezza). |
| `greg_low_enhancer` | Subharmonic Synthesizer | dbx 120A (sintesi a -1 e -2 ottave tramite tracking dei passaggi per lo zero). |
| `greg_destroyer` | Multi-stage Creative Distortion | Saturazione, clipping sim/asim, wavefolding, rettificazione e bit reduction. |
| `greg_pitch_clean` | Pitch Shifter trasparente da studio | Algoritmo basato su signalsmith-stretch con preservazione dei transienti. |
| `greg_pitch_formant` | Pitch & Formant Shifter creativo | Controllo indipendente di pitch e formanti per voci e strumenti. |
| `greg_trans_mono` | Transient Shaper a banda singola | Rilevatore differenziale Fast vs Slow con controllo Attack/Sustain. |
| `greg_trans_multi` | Transient Shaper a 3 bande | Crossover Linkwitz-Riley e rimodellamento transienti multibanda. |
| `greg_space` | Stereo Imager & Spatial Enhancer | Decorrelazione di fase comb-free, allargamento stereo coerente. |
| `greg_pingpong` | Stereo Ping-Pong Tape/BBD Delay | Tempo sync (BPM, dotted, triplet), filtri feedback ZDF, modulazione LFO. |
| `greg_ir_reverb` | Zero-Latency Convolution Reverb | Convoluzione partizionata uniforme lock-free con caricamento IR wav. |
| `greg_verb` | Algorithmic Reverb vintage (Lexicon style) | FDN ad alta densità con matrici Householder/Hadamard e template Room/Hall/Plate. |
| `greg_plate` | Emulatore fisico di riverbero a piastra | Simulazione fisica EMT 140 con materiali, smorzamento e pickup regolabili. |
| `greg_bus_summer` | Sommatore analogico non lineare | Modelli fisici di mix bus (Neve, SSL, API) con VirtualEarthSummer e crosstalk. |
| `greg_denoiser` | Riduttore di rumore spettrale real-time | STFT overlap-add con spectral subtraction dinamica senza artefatti "musical noise". |
| `greg_declipper` | Ricostruttore di clipping digitale | Ricostruzione della forma d'onda clippata tramite interpolazione spline/polinomiale. |
| `greg_dehummer` | Soppressore di ronzio di rete (50/60 Hz) | Notch comb ad alto Q dinamici per fondamentale e armoniche superiori. |
| `greg_declicker` | Riparatore di click e pop spettrali | Rilevamento transienti impulsivi anomali e crossfade con sintesi autoregressiva. |

---

## 3. Regole di Accorpamento Hardware Obbligatorie

Per evitare la proliferazione di plugin ridondanti e ottimizzare l'usabilità per il produttore audio, **tutte le versioni o revisioni dello stesso dispositivo hardware devono essere accorpate in un unico plugin**, offrendo la scelta tramite un apposito selettore di versione/profilo nei parametri:

1. **`greg_76`**: accorpa le revisioni 1176LN **Rev A**, **Rev D**, **Rev E**, **Rev F** e modalità **All-Buttons** (scelta circuitazione, FET e trasformatori UTC O-12 / A-10).
2. **`greg_dbx_160`**: accorpa le versioni **160 VU**, **160X** e **160A** (commutazione tra Hard Knee e OverEasy, curve VCA Dbx 202, uscite sbilanciate vs bilanciate e compressione Infinity+).
3. **`greg_dbx_165`**: accorpa **165** e **165A** con interruttore dedicato per abilitare o escludere il circuito limitatore PeakStop.
4. **`greg_api_2500`**: accorpa **2500** e **2500+** con selettore del circuito di mixaggio parallelo integrato Blend/Mix.
5. **`greg_api_525`**: accorpa **525 vintage** e **525A** con commutazione dell'attacco fisso vintage a 15 µs vs i tempi espansi del 525A.
6. **`greg_api_527`**: accorpa **527** e **527A** con selettore di revisione e switch feed-forward/feedback.
7. **`greg_api_sr22`**: accorpa l'unità rack dual channel **SR22** e il modulo serie 500 **Select SV12**.
8. **`greg_pultec`**: accorpa **EQP-1A** ed **EQP-1A3** (gestione fedele dell'interazione low-frequency boost/attenuation e trucco Pultec).
9. **`greg_manley_comp`**: accorpa **Variable Mu Standard** e **Variable Mu Mastering Version** (scelta ratio e step T/C).

> **Nota di distinzione topologica:** Dispositivi con percorsi di segnale, topologie o banchi circuitali intrinsecamente differenti restano plugin separati (es. Neve 1073 vs 1081 vs 1084 vs 88RS; API 525 FET vs 527 VCA vs 225L vs 2500 VCA bus vs T25 tube; banco analogico Harrison 32C vs software Mixbus).

---

## 4. Roadmap: 7 Progetti Futuri Post-Analog-Vibe

Al completamento della suite Analog-Vibe, l'infrastruttura condivisa (`core-dsp`, `plugin-support`, `greg_gui_core`) costituirà la fondazione per 7 piattaforme autonome di livello broadcast e studio professionale:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   ROADMAP PROGETTI POST-ANALOG-VIBE                    │
├─────────────────────────────────────┬──────────────────────────────────┤
│ 1. Virtual Console & Summing        │ 2. Guitar & Bass Physical Rig    │
│ 3. Acoustic Spaces & Immersive Verb │ 4. Intelligent Restoration Suite │
│ 5. Modular Multi-FX Routing Grid    │ 6. Reel-to-Reel Master Tape      │
│ 7. Mastering & Immersive Processor  │                                  │
└─────────────────────────────────────┴──────────────────────────────────┘
```

### Progetto 1: Virtual Studio Console & Summing Environment
- **Obiettivo:** Piattaforma integrata di mixing console virtuale con emulazione multi-traccia di banchi leggendari (Neve 8078, SSL 4000/9000, API 1608, Trident A-Range).
- **Innovazione DSP:** Architettura distribuita canale/banco con diafonia inter-canale (crosstalk) capacitiva/induttiva calcolata dinamicamente, modellazione del bus di somma a terra virtuale con calo della tensione di alimentazione (PSU rail droop) e recall di scene istantaneo.

### Progetto 2: Guitar & Bass Physical Rig Studio
- **Obiettivo:** Suite completa per chitarra e basso elettrico basata esclusivamente su modellazione a componenti fisici discreti (White-Box Modeling) senza convoluzioni statiche grezze.
- **Innovazione DSP:** Modello non lineare dell'escursione del cono dell'altoparlante basato su parametri Klippel ($Bl(x)$ e $K_{ms}(x)$), breakup asimmetrico dinamico, compressione termica della bobina, interazione d'impedenza finale di potenza / mobile (cabinet resonance) e posizionamento continuo 3D dei microfoni.

### Progetto 3: Acoustic Spaces & Immersive Reverb Engine
- **Obiettivo:** Processore di riverbero spaziale per produzione musicale e mixing immersivo (Dolby Atmos, Ambisonics fino al 3° ordine, VBAP, Binaural 3D).
- **Innovazione DSP:** Ibrido Image-Source Method (riflessioni precoci 3D tracciate geometricamente secondo materiali assorbenti) + Feedback Delay Network ad alta densità (campo diffuso) con matrici ortogonali Householder e damping in frequenza ZDF.

### Progetto 4: Intelligent Restoration & Spectral Repair Suite
- **Obiettivo:** Pacchetto completo di restauro audio forense e da studio per eliminare artefatti senza alterare la naturalezza timbrica.
- **Innovazione DSP:** Algoritmi STFT overlap-add a zero fase, spectral subtraction con threshold di masking psicoacustico, riparazione click con predizione autoregressiva e de-clipping tramite ricostruzione polinomiale spline.

### Progetto 5: Modular Multi-FX & Signal Routing Grid
- **Obiettivo:** Rack virtuale modulare stile hardware patchbay a matrice libera per la creazione di catene di segnale complesse, splitting multibanda e routing parallelo.
- **Innovazione DSP:** Basato sul motore `dsp_graph.rs` con compilazione topologica real-time safe, Plugin Delay Compensation (PDC) automatico al singolo campione e matrice di modulazione incrociata.

### Progetto 6: Analog Tape Machine & Reel-to-Reel Master Collection
- **Obiettivo:** Emulazione fisica definitiva di registratori a nastro magnetico da 24 tracce da 2 pollici e master da 1/2 pollice (Studer A800, Ampex ATR-102, MCI JH-24).
- **Innovazione DSP:** Modello isteretico Jiles-Atherton delle particelle magnetiche, isteresi dinamica della testina, modulazione di velocità con wow & flutter stocastico, pre/de-enfasi CCIR/NAB calibrata e compressione da saturazione di flusso.

### Progetto 7: Mastering Suite & Immersive Audio Processor
- **Obiettivo:** Workstation di mastering all-in-one per formati stereo e immersivi fino a 7.1.4.
- **Innovazione DSP:** Metering integrale ITU-R BS.1770-4 / EBU R128 (Integrated LUFS, LRA, Short-Term, Momentary), True-Peak limiting inter-sample a fase lineare con lookahead a bassissima distorsione, correzione stereo ellittica e monitoraggio di correlazione di fase per la conformità broadcast.
- **Mastering Vibe:** Una variante focalizzata sull'emulazione caratteristica del calore analogico, con saturazione musicale controllata, risposta in frequenza "vintage" e imaging spaziale ottimizzato per il contenuto voci e strumenti acustici.

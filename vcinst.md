# Analog-vibe — istruzioni e catalogo unificati

Questo e' l'unico catalogo normativo della suite. E' volutamente compatto: non duplicare qui lo stato del progetto, le API di ogni plugin o le specifiche complete dei banchi.

## Memoria di progetto: cosa leggere

1. Leggi sempre questo file prima di modificare codice o requisiti.
2. Per il plugin/modulo interessato, leggi solo `docs/plugins/<id>.txt` e/o `docs/modules/<id>.txt`.
3. Per ricerca hardware, fedelta' analogica o confronto tra revisioni, consulta `docs/specs/outboard_hardware_research.md` e le fonti li' citate.
4. **Leggi** **`docs/specs/Guida_Componenti_Audio_Fisica_Acustica.pdf`** **per le caratteristiche dei componenti elettronici** prima di programmare un'emulazione hardware: questa guida contiene dati tecnici sui componenti fisici che informano il modello DSP. Non sostituire dati anecdotici o supposizioni con le misure documentate in questo file.
5. Leggi `STATE.md` solo per lo stato operativo corrente. Aggiornalo ai cambi di fase o a conclusione del task; non duplicarvi istruzioni, specifiche o cronologia tecnica.
6. Non aprire tutti i cataloghi/schede senza necessita'. Aggiorna la scheda specifica insieme al componente e questa pagina solo quando cambia un requisito comune, l'indice o una decisione globale.

La scheda del componente e' l'unica sede dei suoi parametri, API, test, latenza e stato dettagliato. Il dossier hardware e' l'unica sede delle specifiche documentate e delle fonti comuni. Se manca una fonte, marca il dato come non verificato: non colmare il vuoto per analogia.

## Principi di prodotto e fedelta'

- DSP originale, audio-first: alpha senza GUI, con parametri host/CLAP. Non copiare codice o asset di terzi.
- I nomi Greg/ID sono il branding dei plugin. Nomi commerciali e modelli dei costruttori si usano solo come identificatori di riferimento tecnico.
- **Accorpamento obbligatorio delle versioni di uno stesso outboard:** i plugin che simulano lo stesso dispositivo outboard hardware attraverso differenti versioni o revisioni vanno obbligatoriamente accorpati in un unico plugin, provvisto di un parametro/selettore per scegliere quale versione hardware utilizzare (es. 1176 Rev D/Rev E sotto `greg_76`; DBX 160 VU/160X/160A sotto `greg_dbx_160`; DBX 165/165A sotto `greg_dbx_165`; API 2500/2500+ sotto `greg_api_2500`; API 525/525A sotto `greg_api_525`; API 527/527A sotto `greg_api_527`; API SR22/SV12 sotto `greg_api_sr22`; Pultec EQP-1A/EQP-1A3 sotto `greg_pultec`; Manley Variable Mu Standard/Mastering sotto `greg_manley_comp`). Modelli hardware con topologia, controlli e percorsi di segnale intrinsecamente differenti restano plugin separati.
- Conserva controlli, topologia e limiti documentati dell'hardware. Sidechain, routing, range, controlli o modalita' aggiunti sono estensioni digitali e vanno identificati come tali. Non dichiarare fedelta' senza misure A/B ripetibili su una revisione identificata.
- Per le console: un plugin completo per modello di channel strip; i plugin preamp/EQ/comp gia' esistenti restano componenti separati e non sono sostituiti.
- `greg_harrison_mixbus_strip` e' un profilo software Harrison Mixbus, non una console analogica.
- Le specifiche di un modello moderno o di una revisione non vanno generalizzate all'hardware vintage senza verifica. Le differenze e le fonti sono nel dossier hardware e nella scheda relativa.

## Invarianti DSP e CLAP

- **Real time:** nessuna allocazione dinamica, lock bloccante, I/O o chiamata di sistema nel callback audio. Predisponi buffer/stato in inizializzazione; processa esattamente i frame forniti dall'host, anche con blocchi irregolari, senza assumere una dimensione fissa.
- **Sample rate e precisione:** deriva coefficienti/tempi dal sample rate effettivo in initialize/reset. Supporta la precisione e i formati realmente esposti dall'host/framework; non simulare supporto non verificato.
- **Layout e stato:** dichiara i layout host supportati e mantiene lo stato necessario per ciascun canale. Mono, stereo, dual mono e M/S sono modalita' distinte; N-canali solo quando il plugin lo dichiara e lo implementa correttamente.
- **Dinamiche stereo:** dove coerente con l'hardware, offri stereo-link oppure dual mono esplicito. Non confondere il detector/link DSP con il numero di canali host.
- **Sidechain:** i processori dinamici usano il detector interno e, se supportato dal layout host, quello esterno; fallback interno esplicito. Filtri HP/LP/Bell/Low-Shelf/High-Shelf e Sidechain Listen sono opzioni comuni, ma non devono falsare il circuito di riferimento.
- **Qualita' e latenza:** usa oversampling anti-alias solo dove serve alla non linearita'; dichiara e verifica latenza, fase, bypass e comportamento ai bordi dei buffer. Bypass click-free e allineamento dry/wet quando applicabili.
- **Robustezza:** usa le protezioni condivise contro panic se compatibili col callback. Non presentarle come protezione da abort, segfault o corruzione di memoria.
- **Metering:** riusa `core_dsp_modules::metering::Metering` per peak/RMS input-output e gain reduction quando pertinente; snapshot thread-safe e nessun lavoro bloccante nel callback.
- **Test:** test deterministici per DSP, formati/layout, blocchi variabili, valori finiti, latenza, bypass e parametri. Il test host sintetico non sostituisce ascolto o misure A/B hardware.

## Architettura e documentazione

- Workspace Rust/Cargo esistente: riusa `core-dsp` e i moduli condivisi, senza duplicare DSP gia' disponibile.
- Ogni plugin: `docs/plugins/<plugin_id>.txt`. Ogni modulo condiviso: `docs/modules/<module_id>.txt`. Aggiorna i documenti con API, parametri, stato, latenza, test e limiti.
- Riferimenti hardware e fonti: `docs/specs/outboard_hardware_research.md`. Inventario: `docs/INVENTARIO.txt`. Stato operativo: `STATE.md`; sintesi sessione: `AGGIORNAMENTO.md`.
- Procedura: Step 0 ricerca/specifica; poi parametri e layout; inizializzazione e allocazioni; DSP; integrazione non lineare; test e documentazione. Esegui il controllo/build/test piu' piccolo che copre la modifica, poi amplia se serve.
- Commenta solo formule o decisioni DSP non ovvie. Non fare commit automatici per singoli micro-step; salva commit solo quando richiesto o concordato.

## Catalogo completo

Gli ID sono canonici. La descrizione e' un obiettivo di prodotto, non prova che il plugin sia gia' implementato: controlla la scheda del plugin per lo stato.

### Dinamiche, EQ e preamp

| ID | Obiettivo / riferimento da mantenere |
| --- | --- |
| `greg_76` | Compressore FET 1176LN; ratio 4/8/12/20:1 e All Buttons. Accorpa le revisioni (Rev D, Rev E, All-Buttons) con selettore di versione. |
| `greg_a2a` | Compressore opto valvolare, riferimento LA-2A; memoria ottica/program-dependent. Con selettore revisione/cella T4 (Silverface vs Vintage Tube). |
| `greg_a3a` | Compressore opto a stato solido, riferimento LA-3A; distinto dall'A2A. |
| `greg_child` | Compressore valvolare Vari-Mu, riferimento Fairchild 670; ratio crescente e sei posizioni temporali documentate (con selettore modalità/versione 670 stereo / 660 mono). |
| `greg_bus_comp` | Compressore stereo VCA da bus, riferimento SSL 4000 G bus compressor; le estensioni digitali restano separate. |
| `greg_dbx_160` | Compressore VCA DBX 160; accorpa le versioni 160 VU, 160X e 160A con selettore di versione (OverEasy/Hard Knee, curve VCA, uscite single-ended vs bilanciate, Infinity+). |
| `greg_dbx_165` | Compressore VCA DBX 165; accorpa le versioni 165 e 165A con selettore di versione (inclusione/esclusione limiter PeakStop). |
| `greg_api_2500` | Compressore stereo VCA da bus API 2500; accorpa le versioni 2500 e 2500+ con selettore di versione (circuito Mix/Blend parallelo, THRUST, feedback/feed-forward). |
| `greg_api_525` | Compressore feedback FET API 525; accorpa le versioni 525 (vintage) e 525A con selettore di versione (attacco fisso vintage 15 µs vs tempi attacco/rilascio estesi 525A). |
| `greg_api_527` | Compressore VCA serie 500 API 527; accorpa le versioni 527 e 527A con selettore di versione. |
| `greg_api_sr22` | Compressore VCA dual-channel circuito ATI Paragon; accorpa la versione rack SR22 e modulo serie 500 SV12 con selettore di formato/versione. |
| `greg_api_225l` | Modulo compressore VCA serie 200 API 225L. |
| `greg_api_t25` | Compressore a due canali FET feedback con uscita valvolare Class A API T25. |
| `greg_api_529` | Compressore stereo bus serie 500 API 529. |
| `greg_pultec` | EQ passivo EQP-1A; accorpa le versioni EQP-1A e EQP-1A3 con selettore di versione (interazione low boost/attenuation e makeup valvolare). |
| `greg_ssl_eq` | EQ di canale SSL 4000; selettore curve/circuitazione E-Series (Black 242 / Brown 02) e G-Series (292). |
| `greg_neve_eq` | EQ a induttori, riferimento Neve 1073 (vintage / SPX-D con selettore); plugin di sezione, distinto dallo strip completo. |
| `greg_neve_pre` | Preamplificatore con trasformatore, riferimento Neve 1073; plugin di sezione. |
| `greg_api_pre` | Preamplificatore discreto, riferimento API 512c. |
| `greg_harrison_pre`, `greg_harrison_comp`, `greg_harrison_eq` | Componenti separati della channel strip Harrison 32C; mantenuti anche dopo l'aggiunta dello strip completo. |
| `greg_manley_eq` | EQ passivo valvolare Massive Passive. |
| `greg_manley_comp` | Compressore valvolare Variable Mu; accorpa le versioni Standard e Mastering con selettore di versione. |
| `greg_multiband_comp` | Compressore multibanda configurabile (almeno 4 bande); limiti, fase e latenza da specificare. |
| `greg_limiter` | Limiter/maximizer con Input/Drive e ceiling; documenta lookahead, oversampling e true-peak policy. |
| `greg_deesser` | De esser con varie tipologie di filtri e un range di lavoro molto alto, deve suonare morbido, senza rendere troppo taglienti le sibilanti |
| `greg_depop` | Un effetto dinamico con la capacita di andare ad comprimere abbassare le frequenze creato da un pop microfonico |
| `greg_surgical_eq` | EQ parametrico a bande configurabili, filtri Bell/Shelf/HP/LP/Notch/Band-Pass e modalita' minima o lineare con latenza dichiarata. |
| `greg_eq_match` | Eq in grado di replicare una curva di equalizzazione di un segnale esterno ricevuto tramite una sidechain |
| `greg_gate` | Gate con detector interno/esterno, threshold, range, attack/hold/release, isteresi e filtri sidechain. |
| greg\_expander | Expander con detector interno/esterno, threshold, range, attack/hold/release, isteresi e filtri sidechain. |
| greg\_ducking | Duck processor con detector interno/esterno, threshold, range, attack/hold/release, isteresi e filtri sidechain. |
| `greg_lin_eq`, `greg_lin_comp` | EQ e compressore multibanda a fase lineare; dichiarare latenza e pre-ringing. |

### Channel strip completi di console

Componenti preamp/EQ/dinamica esistenti non vengono rimossi. Un modello per ID; non fondere modelli distinti.

| ID | Modello |
| --- | --- |
| `greg_neve_1073_strip` | Neve 1073 |
| `greg_neve_1081_strip` | Neve 1081 |
| `greg_neve_1084_strip` | Neve 1084 |
| `greg_neve_88rs_strip` | Neve 88RS |
| `greg_harrison_32c_strip` | Harrison 32C analogico |
| `greg_harrison_mixbus_strip` | Harrison Mixbus software |
| `greg_ssl_4000e_strip` | SSL 4000 E |
| `greg_ssl_4000g_strip` | SSL 4000 G |
| `greg_ssl_9000j_strip` | SSL 9000 J |
| `greg_api_vision_strip` | API Vision, con configurazione/moduli espliciti |
| `greg_api_legacy_plus_strip` | API Legacy Plus |
| `greg_trident_a_range_strip` | Trident A-Range |
| `greg_trident_80b_strip` | Trident Series 80B |
| `greg_amek_9098_strip` | AMEK 9098 |
| `greg_mci_jh500_strip` | MCI JH-500 |

### Utility, creativi e restauro

| ID | Obiettivo |
| --- | --- |
| `greg_ms_encoder`, `greg_ms_decoder` | Codifica L/R→M/S e decodifica M/S→L/R, plugin distinti. |
| `greg_phase` | Inversione di polarita' per canale o linkata; non confondere con rotazione/ritardo di fase. |
| Greg\_\_leveler<br> | Plugin di automazione automatica per tenere la voce e altri suoni in un volume costante nel mix senza utilizzare automazion della daw |
| `greg_tape` | Nastro analogico; Studer A800 Mk III come riferimento, con calibrazione/allineamento dichiarati. |
| `greg_exciter` | Exciter armonico; Aphex Aural Exciter Type C come riferimento, non il prodotto Aphex moderno. |
| `greg_low_enhancer` | Sintesi subarmonica; dbx 120A come riferimento. Non sostituire la sintesi modellata con un semplice divide-per-due. |
| `greg_destroyer` | Distorsione selezionabile: saturazione, clipping sim/asim, wavefold, rettificazione, bit reduction; filtri e mix pre/post. |
| `greg_pitch_clean`, `greg_pitch_formant` | Pitch shifter trasparente e creativo con pitch/formanti indipendenti. |
| `greg_trans_mono`, `greg_trans_multi` | Transient shaper mono e multibanda; crossover e latenza documentati. |
| `greg_space` | Ampiezza e immagine stereo, con attenzione alla correlazione/fase. |
| `greg_pingpong` | Delay ping-pong sincronizzato alla DAW con routing/tap configurabili. |
| `greg_ir_reverb` | Riverbero a convoluzione; specificare formati supportati e latenza. |
| greg\_verb | Riverbero classico in stile lexicon con I classicii template (es: room, chamber, hall) dal suono pulito. Con parametri completi. |
| `greg_bus_summer` | Sommatore non lineare con modelli di mixer selezionabili; fase successiva e riferimenti da identificare. |
| greg\_plate | Simulazione di plate reverrb che possa simulare diversi magneti e diverse piastre metalliche, magneti posti a diverse distanze e con tutti I parametri |
| `greg_denoiser`, `greg_declipper`, `greg_dehummer`, `greg_declicker` | Restauro: riduzione rumore, ricostruzione clipping, rimozione ronzio e riparazione click senza danneggiare il programma. |

### Moduli DSP condivisi

Riusa i moduli disponibili in `core-dsp`; verifica la scheda prima di aggiungere logica duplicata: `routing_matrix`, `phase_inversion`, `filters`, `sidechain_processor`, `resampler_bitdepth`, `nonlinear_stages`, `transformer_hysteresis`, `psu_sag`, `analog_variance`, `realtime_controls`, `metering`, `surgical_eq`, `panic_protection`, `opto_compressor`, `vari_mu`, `delay_lines`, `fdn_reverb`, `partitioned_convolver`, `loudness_rider`, `spectral_analyzer`, `tape_physics`, `pitch_shifter`, `spectral_restoration`, `tempo_sync`, `mix_mastering_fx`, `modulation_engine`, `mpc_sampler_engine`, `component_models`, `cabinet_speaker`, `acoustic_room`.

### Nuovi Moduli DSP Condivisi Implementati

- **`cabinet_speaker`**: simulatore fisico di cabinet, coni e microfoni per chitarra e basso (Klippel, cone excursion breakup, thermal compression, speaker count config, enclosure types, mic positioner).
- **`acoustic_room`**: simulatore di room e early reflections 3D (image-source method, surface materials, diffusion, late field).
- **`component_models`**: modelli fisici di tubi, stato solido, trasformatori e non-linear elements (dipendenti da `f64`).

## Fedelta': riferimenti correnti e regole di accorpamento

- Il dossier `docs/specs/outboard_hardware_research.md` e' la fonte di verita' per stato delle fonti, dati pubblicati, lacune e revisioni. Le schede plugin riportano i dettagli pertinenti al singolo ID.
- **Regola di accorpamento obbligatorio per outboard:** i plugin che simulano le stesse outboard di versioni o revisioni differenti vanno accorpati in un unico plugin, offrendo all'utente la scelta esplicita di quale versione usare tramite selettore di versione/profilo:
    - `greg_76`: accorpa 1176LN Rev D e Rev E (e All-Buttons) con selettore di revisione.
    - `greg_dbx_160`: accorpa DBX 160 VU, DBX 160X e DBX 160A con selettore di versione (OverEasy/Hard Knee, curve VCA, uscite single-ended vs bilanciate, Infinity+).
    - `greg_dbx_165`: accorpa DBX 165 e DBX 165A con selettore di versione (inclusione/esclusione PeakStop limiter).
    - `greg_api_2500`: accorpa API 2500 e API 2500+ con selettore di versione (circuito Mix/Blend parallelo).
    - `greg_api_525`: accorpa API 525 e API 525A con selettore di versione (attacco fisso vintage vs tempi estesi 525A).
    - `greg_api_527`: accorpa API 527 e API 527A con selettore di versione.
    - `greg_api_sr22`: accorpa API SR22 (rack dual channel) e API Select SV12 (modulo serie 500) con selettore di versione/formato.
    - `greg_pultec`: accorpa EQP-1A e EQP-1A3 con selettore di versione.
    - `greg_manley_comp`: accorpa Variable Mu Standard e Variable Mu Mastering con selettore di versione.
- **Modelli intrinsecamente distinti:** restano plugin separati i dispositivi che presentano topologie hardware, controlli e percorsi di segnale totalmente differenti (es. Neve 1073 vs 1081 vs 1084 vs 88RS; API 525 FET vs 527 VCA vs 225L vs 2500 bus vs T25 tube; Harrison 32C banco analogico vs Mixbus strip software).
- Non usare caratteristiche di un channel strip moderno come se descrivessero tutte le revisioni vintage. Per console modulari/API Vision documenta la configurazione dei moduli.
- Le specifiche non bastano a provare una replica. Per ogni dichiarazione di fedelta' servono revisione identificata e misure ripetibili (risposta, livelli, transienti/recovery, curva I/O, spettro/THD, rumore, carico e setup) da effettuare con clap host.

\## STANDARD OBBLIGATORIO DI COMMENTO DEL CODICE (RUSTDOC \& DSP)

Ogni file \`.rs\` creato o modificato deve essere documentato chiaramente secondo queste regole tassative:

1\. \*\*Intestazione modulo (\`//!\`):\*\* All'inizio di ogni file, descrivere lo scopo del modulo, la topologia circuitale emulata e i riferimenti ai moduli \`core-dsp\` utilizzati.

2\. \*\*Doc-comment pubblici (\`///\`):\*\* Ogni \`struct\`, \`enum\`, campo di \`Params\` e funzione pubblica deve avere un commento \`///\` che spiega cosa fa, l'unità di misura (es. \`dB\`, \`ms\`, \`Hz\`) e i vincoli real-time.

3\. \*\*Commenti matematici DSP (\`//\`):\*\* Ogni blocco di calcolo in \`initialize()\` o \`process()\` (es. coefficienti filtri, costanti di tempo attack/release, curve di saturazione, hysteresis, crosstalk) deve spiegare la formula fisica/matematica applicata e il motivo acustico della scelta.

4\. \*\*Divieto di commenti ovvi:\*\* Non commentare operazioni banali (es. \`x += 1; // incrementa x\`), commenta sempre il \*\*perché fisico, circuitale o acustico\*\* (es. \`// Compensazione guadagno post-tanh per mantenere unity gain\`).

5\. \*\* il codice deve essere commentato in inglese anche quando necessario tra le righe del codice per facilitare la lettura umana di potenziali collaboratori

Obbligatorio per tutte le emulazioni analogiche

\- \*\*Catena Analogica \& Trasformatori Obbligatori (\`core-dsp\`):\*\* Ogni plugin che emula hardware analogico (compressori, EQ, channel strip, preamp, saturatori, nastro) DEVE integrare obbligatoriamente nello stadio di \*\*Input\*\* e nello stadio di \*\*Output\*\* (e negli stadi intermedi dove previsto dalla topologia originale, es. interstage/sidechain) il modello fisico \*\*\`TransformerHysteresis\`\*\* + \*\*\`AcCoupling\`\*\* + \*\*\`AnalogVariance\`\*\* di \`core-dsp\` (eseguiti dentro l'oversampler anti-alias), affiancati dagli stadi non lineari appropriati (\`TubeTriode\`/\`Pentode\`, \`SolidStateClassA\`, \`SolidStateFet\`, \`DiscreteOpAmp\`, \`PsuSag\`).

\- \*Controllo utente:\* Esporre nel pannello \`+ ADV\` il parametro \*\*\`Transformer / Iron\`\*\* (calibrato di default sul carattere nominalmente fedele all'unità originale, scalabile da \`0%\` bypass trasformatore fino a \`100%+\` saturazione spinta) e \*\*\`Analog Drift\`\*\* (con rumore/ripple rigorosamente \`OFF\` di default).

\- \*Eccezione:\* Le utility digitali pure (\`greg\_phase\`, \`greg\_ms\_encoder\`, \`greg\_ms\_decoder\`, \`greg\_surgical\_eq\`, \`greg\_leveler\`, \`greg\_limiter\`) restano trasparenti di default, con eventuale colorazione solo se attivata esplicitamente dall'utente.

\- \*\*Catena Analogica \& Trasformatori Obbligatori (\`core-dsp\`):\*\* Ogni plugin che emula hardware analogico (compressori, EQ, channel strip, preamp, saturatori, nastro) DEVE integrare obbligatoriamente nello stadio di \*\*Input\*\* e nello stadio di \*\*Output\*\* (e negli stadi intermedi dove previsto dalla topologia originale) il modello fisico \*\*\`TransformerHysteresis\`\*\* + \*\*\`AcCoupling\`\*\* + \*\*\`AnalogVariance\`\*\* di \`core-dsp\` (eseguiti dentro l'oversampler anti-alias), affiancati dagli stadi non lineari appropriati (\`TubeTriode\`/\`Pentode\`, \`SolidStateClassA\`, \`SolidStateFet\`, \`DiscreteOpAmp\`, \`PsuSag\`):

\- \*\*Effetto Miller (\`nonlinear\_stages\`):\*\* Roll-off dinamico delle altissime frequenze (filtro passa-basso a 1 polo che scende dolcemente proporzionalmente al drive) per arrotondare gli acuti come nei tubi a vuoto e nei transistor discreti.

\- \*\*Bias Shift Asimmetrico (\`AcCoupling\`):\*\* Spostamento temporaneo del punto di lavoro DC sotto transienti forti per generare compressione armonica naturale di 2ª armonica.

\- \*\*Deriva Termica Lenta (\`AnalogVariance\`):\*\* Aggiornata \*\*una sola volta per blocco audio\*\* (\*block-rate\*, zero \`sin()\` per campione) con due oscillatori asincroni L/R (\~17.3 s e \~23.7 s) inizializzati dal seed deterministico \`instance\_id\` e scalati da \`analog\_drift\`. Modula di ±0.2% bias DC e costanti di tempo senza alterare pitch, CPU o ripetibilità dei test (rumore/ripple rigorosamente \`OFF\` di default).

\- \*Controlli utente (\`+ ADV\`):\* Esporre \*\*\`Transformer / Iron\`\*\* (calibrato di default sull'unità originale, scalabile \`0% .. 100%+\`) e \*\*\`Analog Drift\`\*\*. Le utility digitali pure (\`greg\_phase\`, \`greg\_ms\_encoder\`, \`greg\_ms\_decoder\`, \`greg\_surgical\_eq\`, \`greg\_leveler\`, \`greg\_limiter\`) restano trasparenti di default.

\### Nuovi Moduli DSP Condivisi Pianificati (\`core-dsp\`)

\- \`delay\_lines\`: linee di ritardo pre-allocate con lettura frazionaria (interpolazione All-Pass / Cubica) e modulazione LFO senza perdita di alte frequenze (base per riverberi, chorus, echo).

\- \`fdn\_reverb\`: motore riverbero algoritmico con diffusori All-Pass, matrice di feedback ortogonale (Hadamard/Householder 8x8–16x16), damping in frequenza ZDF e modulazione multi-fase (per \`greg\_plate\_reverb\` EMT-style e \`greg\_algo\_reverb\` Lexicon/Valhalla-style).

\- \`partitioned\_convolver\`: convoluzione partizionata a blocchi (Overlap-Save via \`realfft\`, pre-allocata in \`initialize()\`) per risposte impulsive lunghe a latenza zero (\`greg\_ir\_reverb\`).

\- \`loudness\_rider\`: detector di loudness sillabica K-weighted, gate con isteresi per silenzio/respiri (\`Floor\`) e smoother a rampa in \`dB/sec\` con lookahead (\`greg\_leveler\`, \`greg\_deesser\`, \`greg\_depop\`).

\- \`spectral\_analyzer\`: calcolo FFT finestrato su ring-buffer lock-free con smoothing logaritmico a frazioni d'ottava per telemetria grafica (\`GregSpectrum\`).

\- \*\*GUI Temporanea Minimale 2D Obbligatoria (Fase Beta/Test Ergonomico):\*\* Ogni plugin del workspace DEVE includere obbligatoriamente una GUI 2D leggera e funzionale tramite \`greg\_gui\_core::create\_greg\_editor\` (controlli principali nel Main Panel + estensioni in \`+ ADV\`, meter IN/OUT, Phase e GR), pensata esclusivamente per testare l'ergonomia, i parametri e il suono in DAW senza rallentare lo sviluppo; la grafica definitiva ad alta fedeltà verrà realizzata in una fase successiva.

Obbligatorio:

\- \*\*GUI Temporanea Minimale 2D Obbligatoria (Fase Beta/Test Ergonomico):\*\* Ogni plugin del workspace DEVE includere obbligatoriamente una GUI 2D leggera e funzionale tramite \`greg\_gui\_core::create\_greg\_editor\` (controlli principali nel Main Panel + estensioni in \`+ ADV\`, meter IN/OUT, Phase e GR), pensata esclusivamente per testare l'ergonomia, i parametri e il suono in DAW senza rallentare lo sviluppo; la grafica definitiva ad alta fedeltà verrà realizzata in una fase successiva.

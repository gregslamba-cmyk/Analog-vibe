# Analog-vibe — istruzioni e catalogo unificati

Questo e' l'unico catalogo normativo della suite. E' volutamente compatto: non duplicare qui lo stato del progetto, le API di ogni plugin o le specifiche complete dei banchi.

## Memoria di progetto: cosa leggere

1. Leggi sempre questo file prima di modificare codice o requisiti.
2. Per il plugin/modulo interessato, leggi solo `docs/plugins/<id>.txt` e/o `docs/modules/<id>.txt`.
3. Per ricerca hardware, fedelta' analogica o confronto tra revisioni, consulta `docs/specs/outboard_hardware_research.md` e le fonti li' citate.
4. Leggi `STATE.md` solo per lo stato operativo corrente. Aggiornalo ai cambi di fase o a conclusione del task; non duplicarvi istruzioni, specifiche o cronologia tecnica.
5. Non aprire tutti i cataloghi/schede senza necessita'. Aggiorna la scheda specifica insieme al componente e questa pagina solo quando cambia un requisito comune, l'indice o una decisione globale.

La scheda del componente e' l'unica sede dei suoi parametri, API, test, latenza e stato dettagliato. Il dossier hardware e' l'unica sede delle specifiche documentate e delle fonti comuni. Se manca una fonte, marca il dato come non verificato: non colmare il vuoto per analogia.

## Principi di prodotto e fedelta'

- DSP originale, audio-first: alpha senza GUI, con parametri host/CLAP. Non copiare codice o asset di terzi.
- I nomi Greg/ID sono il branding dei plugin. Nomi commerciali e modelli dei costruttori si usano solo come identificatori di riferimento tecnico.
- Un plugin per modello hardware esatto. Le revisioni della stessa macchina/modello possono essere profili nello stesso ID; modelli diversi restano distinti. Non accorpare le differenze solo per appartenenza alla stessa famiglia.
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
|---|---|
| `greg_76` | Compressore FET 1176LN; ratio 4/8/12/20:1 e All Buttons. Rev D/E come profili dello stesso plugin. |
| `greg_a2a` | Compressore opto valvolare, riferimento LA-2A; memoria ottica/program-dependent. |
| `greg_a3a` | Compressore opto a stato solido, riferimento LA-3A; distinto dall'A2A. |
| `greg_child` | Compressore valvolare Vari-Mu, riferimento Fairchild 670; ratio crescente e sei posizioni temporali documentate. |
| `greg_bus_comp` | Compressore stereo VCA da bus, riferimento SSL 4000 G bus compressor; le estensioni digitali restano separate. |
| `greg_dbx_160a`, `greg_dbx_160x`, `greg_dbx_165a` | Tre compressori DBX distinti; non trasferire controlli/topologie tra modelli. Il 165A aggiunge PeakStop al 165. |
| `greg_api_2500`, `greg_api_2500_plus`, `greg_api_sr22`, `greg_api_225l`, `greg_api_t25`, `greg_api_525`, `greg_api_525a`, `greg_api_527`, `greg_api_527a`, `greg_api_529`, `greg_api_sv12` | Un plugin per ciascun modello compressore API standalone. Varianti/revisioni dello stesso modello restano profili; modelli diversi e controlli distintivi restano separati. |
| `greg_pultec` | EQ passivo EQP-1A con interazione low boost/attenuation e stadio makeup. |
| `greg_ssl_eq` | EQ di canale SSL 4000 E/G come componente legacy. I nuovi strip E e G sono plugin separati. |
| `greg_neve_eq` | EQ a induttori, riferimento Neve 1073; plugin di sezione, distinto dallo strip completo. |
| `greg_neve_pre` | Preamplificatore con trasformatore, riferimento Neve 1073; plugin di sezione. |
| `greg_api_pre` | Preamplificatore discreto, riferimento API 512c. |
| `greg_harrison_pre`, `greg_harrison_comp`, `greg_harrison_eq` | Componenti separati della channel strip Harrison 32C; mantenuti anche dopo l'aggiunta dello strip completo. |
| `greg_manley_eq`, `greg_manley_comp` | EQ Massive Passive e compressore Variable Mu, modelli distinti. |
| `greg_multiband_comp` | Compressore multibanda configurabile (almeno 4 bande); limiti, fase e latenza da specificare. |
| `greg_limiter` | Limiter/maximizer con Input/Drive e ceiling; documenta lookahead, oversampling e true-peak policy. |
| `greg_surgical_eq` | EQ parametrico a bande configurabili, filtri Bell/Shelf/HP/LP/Notch/Band-Pass e modalita' minima o lineare con latenza dichiarata. |
| `greg_gate` | Gate con detector interno/esterno, threshold, range, attack/hold/release, isteresi e filtri sidechain. |
| `greg_lin_eq`, `greg_lin_comp` | EQ e compressore multibanda a fase lineare; dichiarare latenza e pre-ringing. |

### Channel strip completi di console

Componenti preamp/EQ/dinamica esistenti non vengono rimossi. Un modello per ID; non fondere modelli distinti.

| ID | Modello |
|---|---|
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
|---|---|
| `greg_ms_encoder`, `greg_ms_decoder` | Codifica L/R→M/S e decodifica M/S→L/R, plugin distinti. |
| `greg_phase` | Inversione di polarita' per canale o linkata; non confondere con rotazione/ritardo di fase. |
| `greg_tape` | Nastro analogico; Studer A800 Mk III come riferimento, con calibrazione/allineamento dichiarati. |
| `greg_exciter` | Exciter armonico; Aphex Aural Exciter Type C come riferimento, non il prodotto Aphex moderno. |
| `greg_low_enhancer` | Sintesi subarmonica; dbx 120A come riferimento. Non sostituire la sintesi modellata con un semplice divide-per-due. |
| `greg_destroyer` | Distorsione selezionabile: saturazione, clipping sim/asim, wavefold, rettificazione, bit reduction; filtri e mix pre/post. |
| `greg_pitch_clean`, `greg_pitch_formant` | Pitch shifter trasparente e creativo con pitch/formanti indipendenti. |
| `greg_trans_mono`, `greg_trans_multi` | Transient shaper mono e multibanda; crossover e latenza documentati. |
| `greg_space` | Ampiezza e immagine stereo, con attenzione alla correlazione/fase. |
| `greg_pingpong` | Delay ping-pong sincronizzato alla DAW con routing/tap configurabili. |
| `greg_ir_reverb` | Riverbero a convoluzione; specificare formati supportati e latenza. |
| `greg_bus_summer` | Sommatore non lineare con modelli di mixer selezionabili; fase successiva e riferimenti da identificare. |
| `greg_denoiser`, `greg_declipper`, `greg_dehummer`, `greg_declicker` | Restauro: riduzione rumore, ricostruzione clipping, rimozione ronzio e riparazione click senza danneggiare il programma. |

### Moduli DSP condivisi

Riusa i moduli disponibili in `core-dsp`; verifica la scheda prima di aggiungere logica duplicata:
`routing_matrix`, `phase_inversion`, `filters`, `sidechain_processor`, `resampler_bitdepth`, `nonlinear_stages`, `transformer_hysteresis`, `psu_sag`, `analog_variance`, `realtime_controls`, `metering`, `surgical_eq`, `panic_protection`, `opto_compressor`, `vari_mu`.

## Fedelta': riferimenti correnti e regole di accorpamento

- Il dossier `docs/specs/outboard_hardware_research.md` e' la fonte di verita' per stato delle fonti, dati pubblicati, lacune e revisioni. Le schede plugin riportano i dettagli pertinenti al singolo ID.
- Accorpa solo revisioni dello stesso modello esatto. Esempi: 1176 Rev D/E sotto `greg_76`; 525 e 525A distinti; 2500 e 2500+ distinti; SSL 4000 E e G distinti; Neve 1073/1081/1084 distinti; Harrison 32C e Mixbus distinti.
- Non usare caratteristiche di un channel strip moderno come se descrivessero tutte le revisioni vintage. Per console modulari/API Vision documenta la configurazione dei moduli.
- Le specifiche non bastano a provare una replica. Per ogni dichiarazione di fedelta' servono revisione identificata e misure ripetibili (risposta, livelli, transienti/recovery, curva I/O, spettro/THD, rumore, carico e setup).

# Ricerca riferimenti hardware per le emulazioni

Ultimo controllo: 2026-11-19

## Scopo e metodo

Questo documento raccoglie i riferimenti hardware analogici per i plugin del catalogo, come base per lo Step 0 di ricerca. Riporta solo specifiche documentate o caratteristiche circuitali attribuibili al modello indicato; non trasforma descrizioni commerciali in misure DSP. Le specifiche possono cambiare con revisione, versione mono/stereo, alimentazione e ricondizionamento dell'unita'.

Ordine di priorita' per completare ciascuna emulazione:

1. Manuale d'uso e service manual della precisa revisione hardware.
2. Scheda tecnica e documentazione del costruttore.
3. Misure indipendenti riproducibili, separate dalle specifiche dichiarate.
4. Misure A/B su hardware identificato, con setup, livelli, carico, sample rate e incertezza annotati.

Le pagine prodotto citate sotto attestano il modello e le caratteristiche dichiarate; non bastano da sole per ricostruire risposte temporali, curve di trasferimento o spettro armonico. Non dichiarare una replica fedele finche' le misure non sono ripetibili.

## Mappa dei riferimenti

| Plugin | Riferimento | Stato del riferimento |
|---|---|---|
| `greg_76` | UREI / Universal Audio 1176LN FET Limiting Amplifier | Rev D/E restano profili di revisione dentro lo stesso plugin 1176LN. |
| `greg_a2a` | Teletronix / Universal Audio LA-2A Leveling Amplifier | Identificato; scegliere revisione/cella di riferimento. |
| `greg_a3a` | UREI / Universal Audio LA-3A Audio Leveler | Identificato; verificare tempi sulla revisione scelta. |
| `greg_child` | Fairchild 670 Stereo Limiter | Identificato; le costanti vanno confrontate con il manuale della versione scelta. |
| `greg_bus_comp` | SSL 4000 G-Series Stereo Bus Compressor | Famiglia identificata; sidechain esterna/filtrata del plugin e' un'estensione. |
| `greg_pultec` | Pulse Techniques EQP-1A Program Equalizer | Identificato; rete passiva e stadio makeup sono il riferimento. |
| `greg_ssl_eq` | SSL 4000 E e G Channel EQ | Plugin EQ storico mantenuto per compatibilita'; i nuovi strip completi E e G sono plugin separati. |
| `greg_neve_eq`, `greg_neve_pre` | Neve 1073 channel module | Plugin preamp/EQ esistenti mantenuti; aggiunto un channel strip completo 1073. |
| `greg_api_pre` | API 512c Mic Pre | Identificato; modello discreto con op-amp 2520 e trasformatore. |
| `greg_harrison_pre`, `greg_harrison_comp`, `greg_harrison_eq` | Harrison 32C channel strip | Plugin di sezione mantenuti; aggiunto lo strip completo della 32C. |
| `greg_manley_eq` | Manley Massive Passive | Identificato. |
| `greg_manley_comp` | Manley Variable Mu | Identificato; specifiche diverse tra versioni standard e mastering. |
| `greg_tape` | Studer A800 Mk III | Modello confermato; restano da fissare formulazione nastro, allineamento e velocita' di riferimento. |
| `greg_exciter` | Aphex Aural Exciter Type C | Modello confermato; revisione e parametri di processo da documentare. |
| `greg_low_enhancer` | dbx 120A Subharmonic Synthesizer | Modello confermato; tradurre le bande e i controlli dal manuale. |
| `greg_dbx_160a`, `greg_dbx_160x`, `greg_dbx_165a` | DBX 160A / 160X / 165A | Aggiunti al catalogo come plugin separati, su richiesta. |
| `greg_api_*` | API 2500, 2500+, SR22, 225L, T25, 525, 525A, 527, 527A, 529 e SV12 | Tutti i compressori standalone elencati nel catalogo ufficiale API sono mappati a plugin distinti. |

### Channel strip di console aggiunti

Ogni modello esatto ha un plugin di channel strip completo, distinto dai plugin di sezione gia' presenti (preamp, EQ e compressori standalone). Le revisioni della stessa sigla/modello sono documentate sotto un unico ID; modelli diversi, anche della stessa serie o dello stesso costruttore, restano plugin separati. La selezione di una revisione o configurazione e' esposta come profilo solo se documentata per quel modello.

| ID plugin | Modello/profilo | Natura e perimetro documentale |
|---|---|---|
| `greg_neve_1073_strip` | Neve 1073 | Channel module preamp/EQ; le specifiche moderne 1073SPX-D sono marcate come versione specifica, non generalizzate alle revisioni vintage. |
| `greg_neve_1081_strip` | Neve 1081 | Channel module preamp, EQ a quattro bande e filtri; i dati ufficiali correnti dichiarano il design originale 1972. |
| `greg_neve_1084_strip` | Neve 1084 | Modello distinto dal 1073; schemi/gamme da trascrivere dal manuale della revisione scelta. |
| `greg_neve_88rs_strip` | Neve 88RS | Channel strip della console 88RS, generazione distinta dai moduli Class A/AB vintage. |
| `greg_harrison_32c_strip` | Harrison 32C | Strip analogico completo della console, distinto dai tre plugin di sezione gia' presenti. |
| `greg_harrison_mixbus_strip` | Harrison Mixbus | Profilo di channel strip software della DAW Mixbus, non un banco analogico; versioni software da accorpare solo dopo confronto documentale. |
| `greg_ssl_4000e_strip` | SSL 4000 E | Channel strip della console E; non unificare con G. |
| `greg_ssl_4000g_strip` | SSL 4000 G | Channel strip della console G; non unificare con E. |
| `greg_ssl_9000j_strip` | SSL 9000 J | Modello distinto della famiglia SSL 9000. |
| `greg_api_vision_strip` | API Vision | Strip configurabile della console Vision; mantenere esplicite le opzioni di EQ/dinamica installate. |
| `greg_api_legacy_plus_strip` | API Legacy Plus | Channel strip della console Legacy Plus, distinta da Vision. |
| `greg_trident_a_range_strip` | Trident A-Range | Strip della console vintage; identificazione di revisione/modulo da confermare sul manuale. |
| `greg_trident_80b_strip` | Trident Series 80B | Modello separato dall'A-Range; documentare la revisione del modulo. |
| `greg_amek_9098_strip` | AMEK 9098 | Strip della console 9098; specifiche da legare alla revisione e al modulo. |
| `greg_mci_jh500_strip` | MCI JH-500 | Strip della console JH-500; specifiche da legare alla revisione e al modulo. |

Le fonti primarie raccolte confermano la natura e alcune caratteristiche di 1073, 1081 e API Vision. Per Harrison, SSL e i modelli in shortlist senza manuale primario accessibile, i documenti individuali indicano le lacune e vietano di inventare controlli, gamme o topologie. La shortlist Trident/AMEK/MCI e' un insieme iniziale di banchi storici da documentare, non una dichiarazione che i manuali siano gia' stati verificati.

Non hanno per ora un outboard analogico univoco nel catalogo: `greg_multiband_comp`, `greg_limiter`, `greg_destroyer`, `greg_pitch_clean`, `greg_pitch_formant`, `greg_space`, `greg_pingpong`, `greg_ir_reverb`, `greg_trans_mono`, `greg_trans_multi`, `greg_gate`, `greg_bus_summer`, `greg_denoiser`, `greg_declipper`, `greg_dehummer`, `greg_declicker`, `greg_lin_eq`, `greg_lin_comp`, `greg_surgical_eq`, `greg_phase`, `greg_ms_encoder` e `greg_ms_decoder`. Non assegnare loro specifiche hardware per analogia estetica.

## Specifiche per modello identificato

### `greg_76` — 1176LN

- **Topologia da riprodurre:** compressore FET con elemento di controllo nel circuito di gain reduction, amplificatore a stato solido e percorso a trasformatori nelle revisioni classiche con trasformatore. Il rapporto e' selezionato con pulsanti distinti; la combinazione simultanea ("all buttons") non e' un quinto rapporto lineare e produce una curva/risposta non standard.
- **Controlli originali da conservare:** Input, Output, Attack, Release, Ratio 4:1/8:1/12:1/20:1 e selezione stereo-link dove presente. Attacco e rilascio sono regolazioni continue, con comportamento inverso rispetto alla convenzione di molti compressori: senso orario piu' veloce.
- **Dati del manuale hardware UA 1176LN:** le versioni D/E di riferimento dichiarano attack 20–800 microsecondi, release 50 ms–1.1 s, guadagno 50 dB e risposta ±1 dB da 20 Hz a 20 kHz; la pagina specifica un trasformatore d'uscita custom e interconnessione stereo. Questo identifica le specifiche di famiglia, non risolve quale tra D ed E calibrare.
- **Punti di fedelta':** rilevatore FET e rete temporale dipendenti dal livello, curve ratio-specifiche, interazione Input/threshold/output stage, modalita' all-buttons distinta; le revisioni differiscono anche per trasformatori e percorso d'uscita.
- **In attesa:** indicare una revisione (es. Rev D o Rev E) e una sorgente di calibrazione. La specifica presente non giustifica il valore software attuale di knee o la sua mappatura dei controlli.
- **Stato implementazione:** selettore Rev D/Rev E presente nel plugin, con range attack/release e ratio hardware implementati. Finche' revisione e differenze non sono misurate, i due profili condividono il DSP e non dichiarano caratteristiche circuitali distinte.
- **Fonti primarie:** [manuale hardware 1176LN e specifiche di Universal Audio](https://help.uaudio.com/hc/en-us/articles/206007816-1176LN-Classic-Limiting-Amplifier); [pagina prodotto 1176LN](https://www.uaudio.com/hardware/compressors/1176ln.html).

### `greg_a2a` — LA-2A

- **Topologia:** attenuatore elettro-ottico T4 (sorgente luminosa e fotoresistore) nel loop di riduzione, amplificatore valvolare e trasformatore in uscita. La memoria del fotoresistore rende release e recupero dipendenti dalla storia del programma, non descrivibili da un solo esponenziale.
- **Controlli originali:** Peak Reduction e Gain; selettore Compress/Limit sui modelli che lo prevedono. Non esiste un controllo utente Attack/Release indipendente.
- **Punti di fedelta':** curva opto dipendente dal livello e dal programma; recupero multi-temporale; non linearita' e risposta del circuito valvolare/trasformatore. I valori variano con tipo e stato della cella T4 e con revisione.
- **Differenza dal codice alpha:** i parametri temporali nominali gia' presenti nel modello sono valori di partenza fenomenologici, non dati calibrati del T4. Non esporli come costanti hardware universali.
- **In attesa:** identificare revisione LA-2A e specifica cella optica; acquisire tracce di attacco/release a piu' livelli e intervalli di silenzio.
- **Fonti primarie:** [manuali analogici UA, incluso LA-2A](https://help.uaudio.com/hc/en-us/articles/12264250115476-Analog-Hardware-Manuals); [articolo hardware LA-2A](https://help.uaudio.com/hc/en-us/search?query=LA-2A%20hardware%20manual).

### `greg_a3a` — LA-3A

- **Topologia:** compressore elettro-ottico a stato solido; la cella ottica resta program-dependent, ma il circuito di amplificazione differisce dall'LA-2A e ha una risposta transitoria piu' pronta. Non e' un LA-2A con lo stadio valvolare semplicemente rimosso.
- **Controlli originali:** Peak Reduction, Gain e selettore Compress/Limit; niente Attack/Release autonomi.
- **Punti di fedelta':** caratterizzare separatamente attacco, rilascio/memoria ottica, rapporto e knee, distorsione del percorso a stato solido, filtri e risposta in frequenza.
- **Differenza dal codice alpha:** l'attacco software di circa 3.5 ms e la coda parametrizzata descrivono l'implementazione attuale, non costituiscono una specifica verificata del LA-3A.
- **In attesa:** consultare il manuale per la revisione presa a riferimento e misurare la cella reale; non concludere che un numero pubblicato per un altro esemplare sia universale.
- **Fonti primarie:** [manuale LA-3A nella documentazione Universal Audio](https://help.uaudio.com/hc/en-us/articles/206371653-LA-3A); [manuali analogici UA](https://help.uaudio.com/hc/en-us/articles/12264250115476-Analog-Hardware-Manuals).

### `greg_child` — Fairchild 670

- **Topologia:** limiter stereo a valvole con controllo di guadagno Vari-Mu a pendenza variabile; canali e modalita' stereo/dual-channel sono legati al routing originale. Le sei posizioni Time Constant selezionano reti temporali composte; i modi piu' lenti includono recuperi a piu' stadi.
- **Punti di fedelta':** non tradurre le sei posizioni in sei singoli release esponenziali; riprodurre per ciascuna posizione il profilo completo di attack, recupero iniziale e coda, nonche' il legame tra amount of limiting e rapporto. Le configurazioni mono/stereo e la combinazione dei canali sono parte del comportamento.
- **Differenza dal codice alpha:** le costanti I–VI gia' annotate in `greg_child` sono valori nominali semplificati; la logica a emivita non rappresenta la rete RC originale. Mantenere esplicita questa limitazione.
- **In attesa:** trascrivere la tabella completa dei tempi dal manuale della revisione scelta; determinare la curva input/output e il comportamento sidechain con segnali ripetibili.
- **Fonti reperite:** [manuale Fairchild Tube Limiter Collection di Universal Audio](https://help.uaudio.com/hc/en-us/search?query=Fairchild%20670%20manual). E' documentazione della collezione software ispirata ai 660/670, non il service manual dell'unita' storica; quest'ultimo resta da reperire.

### `greg_bus_comp` — SSL 4000 G-Series Stereo Bus Compressor

- **Topologia:** compressore VCA stereo sul bus di mix. La modalita' storica non equivale a un compressore generico: controllo stereo del detector, soft-knee/curve e auto-release fanno parte del carattere.
- **Controlli hardware da documentare:** Threshold, Make-up, Ratio, Attack, Release e Auto Release. Annotare gamme e posizioni dalla revisione di manuale scelta.
- **Fuori modello originale:** detector sidechain esterno, filtri sidechain avanzati, dual mono e oversampling devono essere dichiarati estensioni, non caratteristiche del classico G Bus Compressor.
- **In attesa:** definire se si emula una scheda console 4000G originale o un outboard/rack moderno e la sua revisione; recuperare manuale, soglie, tempi e schema di link.
- **Fonti primarie:** [download e documentazione Solid State Logic](https://solidstatelogic.com/support/downloads); [archivio ufficiale SSL](https://solidstatelogic.com/).

### `greg_pultec` — Pulse Techniques EQP-1A

- **Topologia:** filtro passivo con stadio di makeup a valvole e trasformatori custom. Il Low Boost e il Low Atten interagiscono elettricamente sulla rete; non sono due shelf digitali indipendenti. La rete high-peak e l'attenuatore high shelf sono sezioni distinte.
- **Controlli e frequenze dichiarati dal costruttore:** low boost/attenuation a 20, 30, 60 e 100 CPS; high peak a 3, 4, 5, 8, 10, 12 e 16 KCS; high attenuation shelf a 5, 10 e 20 KCS. La famiglia EQP-1A3 riutilizza il circuito in chassis diverso.
- **Punti di fedelta':** curve misurate delle reti, sovrapposizione boost/attenuation, impedenze e loading, risposta/fase, saturazione e rumore dello stadio makeup. Non usare filtri bell/shelf indipendenti come sostituto dell'interazione low.
- **In attesa:** confrontare curve/fase alle impostazioni discrete e documentare la revisione EQP-1A vs EQP-1A3 scelta.
- **Fonte primaria:** [Pulse Techniques — EQP-1A e principio del Pultec trick](https://pulsetechniques.com/).

### `greg_ssl_eq` — SSL 4000 E / G Channel EQ

- **Riferimento:** il catalogo prevede selezione tra circuitazioni E e G; sono due caratteri/curve, non una singola EQ neutra con un colore aggiunto.
- **Caratteristiche da documentare dal manuale console:** quattro bande, curve di shelving/bell, Q proporzionale, gamme e frequenze selezionabili; filtri HPF/LPF e differenze E/G. Non fissare frequency ranges finche' non e' scelta la console/revisione precisa.
- **Punti di fedelta':** acquisire curve di ampiezza e fase per ciascuna banda e posizione, inclusi i cambi di pendenza/Q con il gain. Il requisito plugin di HPF/LPF addizionali non deve modificare la risposta hardware.
- **In attesa:** bloccare revisione E/G e reperire manuale di servizio della console sorgente.
- **Fonti primarie:** [documentazione e manuali SSL](https://solidstatelogic.com/support/downloads); [archivio SSL](https://solidstatelogic.com/).

### `greg_neve_eq` e `greg_neve_pre` — Neve 1073

- **Nota di modello:** preamp ed EQ sono funzioni dello stesso modulo channel 1073. I due plugin dovrebbero poter condividere il modello/stato di preamp ed EQ; i due controlli non rappresentano due dispositivi diversi.
- **Preamplificatore:** circuito discreto Class A con trasformatori Marinair in ingresso/uscita, piu' stadi di guadagno e commutazione a passi di 5 dB. Il clipping e la saturazione dipendono da gain staging e output trim; non sono un semplice soft clipper in uscita.
- **EQ documentato dalla 1073SPX-D (implementazione attuale del 1073):** HF shelving 12 kHz, ±16 dB; LF shelving 35/60/110/220 Hz, ±16 dB; MF peaking a 360/700/1600/3200/4800/7200 Hz, ±18 dB, Q fisso; HPF 50/80/160/300 Hz con pendenza 18 dB/ottava.
- **Punti di fedelta':** modellare la revisione EQ scelta, l'interazione di stadi/trasformatori e il percorso di gain; i valori pubblicati dalla versione moderna 1073SPX-D non garantiscono identita' con ogni modulo vintage.
- **Fonte primaria:** [AMS Neve 1073SPX-D — specifiche ufficiali](https://www.ams-neve.com/outboard/1073-range/1073spx-d/); [AMS Neve 1073DPD — topologia discreta, Class A e trasformatori](https://www.ams-neve.com/outboard/1073-range/1073dpd/).

### `greg_api_pre` — API 512c

- **Topologia dichiarata dal costruttore:** preamp microfonico discreto con op-amp API 2520 e trasformatore d'uscita bilanciato; ingressi mic e Hi-Z/line, selettore, pad -20 dB e inversione di polarita'.
- **Specifiche 512c pubblicate:** guadagno mic 34–65 dB (14–45 dB con pad); ingresso Hi-Z 14–45 dB; impedenza mic 1.5 kΩ, Hi-Z 400 kΩ; risposta 30 Hz–20 kHz (+0/-0.3 dB); clipping +30 dBu; THD <0.05% a +4 dBu.
- **Punti di fedelta':** variare il drive degli stadi 2520/trasformatore in funzione del livello interno; la distorsione non e' solo una funzione statica di tanh.
- **Fonte primaria:** [API 512c — descrizione e specifiche ufficiali](https://apiaudio.com/product/512c-mic-pre/).

### `greg_manley_eq` — Manley Massive Passive

- **Topologia dichiarata:** EQ passivo valvolare con induttori avvolti a mano e stadi valvolari di makeup; quattro bande sovrapposte, selezione shelf/bell e filtri HP/LP passivi. E' un EQ stereo di mastering, non una ricostruzione dell'EQP-1A.
- **Specifiche pubblicate:** bande distribuite nel range 22 Hz–27 kHz; bandwidth 10 Hz–70 kHz; THD <0.01%, S/N 116 dB, massimo output +37 dBu (circa +26 dBu nella LF section).
- **Punti di fedelta':** curve e fattori Q a ogni frequenza/boost-cut, interazione tra bande e filtri, stadio valvolare/makeup. Ottenere dal manuale le posizioni discrete prima di fissare la GUI/API.
- **Fonte primaria:** [Manley Massive Passive — pagina ufficiale](https://www.manley.com/products/pro-audio/eqs/massive-passive-eq).

### `greg_manley_comp` — Manley Variable Mu

- **Topologia dichiarata:** compressore/limiter stereo all-valve, transformer-coupled, push/pull o remote-cutoff tube gain control; percorso completamente differenziale.
- **Specifiche pubblicate per la versione Standard corrente:** Compress 1.5:1 soft knee; Limit 4:1–20:1 knee piu' deciso; risposta 20 Hz–25 kHz ±0.1 dB; max output +30 dBu; noise floor -85 dB. La versione Mastering e le revisioni precedenti hanno differenze: non trasferire automaticamente questi valori.
- **Punti di fedelta':** misurare curva Vari-Mu in entrambe le modalita', costanti temporali, detector stereo, filtri sidechain e armoniche a diversi livelli.
- **Fonte primaria:** [Manley Variable Mu — pagina ufficiale e specifiche](https://www.manley.com/products/pro-audio/dynamics/variable-mu).

### `greg_harrison_pre`, `greg_harrison_comp`, `greg_harrison_eq` — Harrison 32C

- **Riferimento:** channel strip della console Harrison 32C, selezionata come modello per i tre plugin. Non trasferire dati dalla 32Classic moderna o dalla 32C MS senza verificare che corrispondano alla revisione hardware presa a riferimento.
- **Perimetro hardware:** preamplificatore, compressore ed EQ sono funzioni della stessa channel strip; i plugin separati sono una scelta di catalogo e non indicano tre macchine distinte.
- **Specifiche non ancora trascritte:** revisione del modulo, guadagno/impedenze del preamp, controlli e curve dell'EQ, topologia/range e tempi del compressore, livelli operativi e risposta/distorsione.
- **Fonti:** non e' ancora stato reperito un manuale di servizio o una scheda tecnica primaria della specifica 32C. La documentazione commerciale Harrison attualmente reperibile su [Harrison Audio](https://harrisonaudio.com/) non e' sufficiente a confermare i dati elettrici della console storica.
- **Vincolo di fedelta':** mantenere non confermati i valori software e le curve finche' non sono legati a un manuale/revisione o a misure A/B.

### Channel strip di console: criteri comuni

- Ogni strip mantiene il percorso e i controlli documentabili del modello scelto (ingressi e gain staging, filtri, EQ, dinamica, routing e stadi di uscita). Moduli opzionali di console sono configurazioni selezionabili solo se documentate per quello specifico banco.
- I plugin di preamp/EQ/compressore gia' esistenti restano disponibili come processori di sezione; i nuovi strip sono entita' aggiuntive e non sostituzioni.
- Un plugin rappresenta un singolo modello esatto. Le revisioni dello stesso modello possono diventare profili selezionabili; non accorpare modelli distinti (per esempio Neve 1073/1081, SSL 4000 E/G, API Vision/Legacy Plus o Trident A-Range/80B).
- `greg_harrison_mixbus_strip` rappresenta una strip software Harrison Mixbus e non va presentata come hardware analogico. Le versioni della DAW richiedono un confronto specifico, non una selezione automatica di ogni release.

### Neve 1073 e 1081

- Il modulo 1073 comprende preamplificatore ed EQ; il riferimento Neve moderno 1073SPX-D dichiara circuito 1073 con trasformatori Marinair su ingresso/uscita, gain mic a passi di 5 dB, EQ HF shelving 12 kHz, LF 35/60/110/220 Hz, MF peaking a sei frequenze e HPF 50/80/160/300 Hz. Questi numeri sono specifici della documentazione SPX-D e non sono automaticamente attribuiti a ogni revisione vintage.
- Il 1081 e' un modello distinto: Neve lo descrive come design Class A/B del 1972, con trasformatori Marinair, EQ a quattro bande piu' filtri HP/LP selezionabili a 18 dB/ottava, preamp mic fino a +80 dB e impedenza selezionabile 300/1200 ohm. Frequenze e curve complete vanno prese dal manuale.
- Fonti primarie: [Neve 1073 Range](https://www.ams-neve.com/outboard/1073-range/), [specifiche 1073SPX-D](https://www.ams-neve.com/outboard/1073-range/1073spx-d/), [Neve 1081 Classic](https://www.ams-neve.com/outboard/1081-range/1081-classic/).

### `greg_tape` — Studer A800 Mk III

- **Riferimento:** registratore multitraccia da 2 pollici, modello Mk III scelto per l'emulazione.
- **Variabili hardware da fissare:** velocita' 15/30 IPS, formulazione del nastro, flusso di riferimento, equalizzazione NAB/IEC, bias, livelli di registrazione/riproduzione, test tape e allineamento. Queste scelte cambiano risposta in frequenza, head bump, saturazione e distorsione.
- **Specifiche non ancora trascritte:** non e' stato reperito un manuale primario specifico Mk III; non fissare curve o valori di allineamento sulla sola base della famiglia A800.
- **Fonti:** il dominio Studer attuale reindirizza alla pagina [Studer di Evertz](https://evertz.com/solutions/studer/), dedicata alle soluzioni broadcast correnti e non documenta l'A800 Mk III. Serve il manuale di revisione corretto.

### `greg_exciter` — Aphex Aural Exciter Type C

- **Riferimento:** Aural Exciter Type C, modello scelto per l'emulazione.
- **Specifiche non ancora trascritte:** revisione, circuito di generazione armonica, frequenza di sintonia, livello/mix e risposta elettrica devono essere ricavati dal manuale Type C; non dedurli dal nome generico "Aural Exciter".
- **Nota sulle fonti:** la [pagina ufficiale Aphex dell'Exciter attuale](https://aphex.com/products/exciter) documenta un prodotto moderno Aural Exciter/Optical Big Bottom e non prova le specifiche del Type C storico. Non trasferire al Type C le specifiche o i controlli del prodotto attuale.

### `greg_low_enhancer` — dbx 120A Subharmonic Synthesizer

- **Topologia dichiarata dal costruttore:** sintesi subarmonica proprietaria che usa la forma d'onda del basso sorgente e genera una nota modellata un'ottava sotto; due bande di sintesi e un circuito LF Boost separato.
- **Controlli e bande:** Subharmonics master; livelli indipendenti 26–36 Hz e 36–56 Hz (i controlli frontali sono etichettati 24–36 e 36–56 Hz); LF Boost; crossover selezionabile 80/120 Hz, bypass sintesi e livello uscita subwoofer. Il manuale dichiara range di sintesi 26–56 Hz da ingresso 54–110 Hz.
- **Crossover:** HPF 12 dB/ottava (-3 dB a 80 o 120 Hz) e LPF derivato 6 dB/ottava, dichiarato phase-coherent/unity-sum.
- **Specifiche pubblicate:** risposta 20 Hz–20 kHz ±0.5 dB senza sintesi in full-range mode; THD 0.05% senza sintesi; rumore uscita -88 dBu (controlli al massimo, non pesato 22 Hz–22 kHz); dynamic range 112 dB; ingresso max +22 dBu su jack e +8.5 dBV RCA; uscita max +22 dBu su 600 ohm.
- **Punti di fedelta':** il processore non e' un semplice sub-oscillatore divide-per-due: serve preservare dipendenza dal materiale sorgente, le due bande, livelli e crossover separati. La sintesi musicale va caratterizzata con segnali di basso reali e test controllati.
- **Fonte primaria:** [manuale ufficiale dbx 120A](https://dbxpro.com/en/product_documents/120a-manual-18-2217-cpdf) e [pagina prodotto dbx](https://dbxpro.com/en/products/120a).

## Famiglie compressori aggiunte al catalogo

### DBX 160A

- **Topologia e controlli:** compressore/limiter VCA con detector RMS; curve OverEasy o Hard Knee, Threshold, Compression Ratio e Output Gain. Il controllo Infinity+ consente compressione inversa oltre la soglia; ingresso detector esterno e collegamento stereo fra due 160A sono documentati.
- **Manuale 160A:** threshold -40..+20 dBu; ratio variabile da 1:1 a infinito e oltre fino alla curva -1:1; attacco program-dependent (tipicamente 15 ms per 10 dB, 5 ms per 20 dB, 3 ms per 30 dB); rilascio program-dependent (8 ms per 1 dB, 80 ms per 10 dB, 400 ms per 50 dB); risposta 20 Hz–20 kHz +0/-0.5 dB; uscita massima +24 dBm bilanciata.
- **Punto di fedelta':** il tracking dinamico della curva OverEasy e l'andamento attack/release dipendente dal programma sono fondamentali. Non sostituirli con un soft-knee statico più un envelope follower a costanti fisse.
- **Fonte primaria:** [DBX 160A — pagina e manuale del produttore](https://dbxpro.com/en/products/160a).

### DBX 160X

- **Topologia e controlli (manuale d'archivio):** detector true RMS, VCA Blackmer e circuito feed-forward; Over Easy o hard knee, ratio continuo da 1:1 a infinito e oltre fino a -1:1 (Infinity+), threshold, output gain, bypass hard-wire e detector accessibile. Stereo link tra due unita' tramite detector RMS; attacco e rilascio sono program-dependent.
- **Range e tempi dichiarati:** threshold -40..+20 dBm; output gain -20..+20 dB; attack 15 ms per incremento di 10 dB, 5 ms/20 dB, 3 ms/30 dB; rilascio automatico program-dependent, indicato come 0..500 ms. Compressione massima dichiarata >60 dB.
- **Dati elettrici della copia consultata:** ingresso max +24 dBm; uscita max +24 dBm su 600 ohm o superiore; uscita single-ended 22 ohm nella configurazione standard; risposta +0/-1 dB, 20 Hz..20 kHz; dynamic range >113 dB; rumore equivalente ingresso < -89 dBm, 20 Hz..20 kHz.
- **Revisione/limite fonte:** dati estratti da una scansione del manuale 160X ospitata da terzi, non dal produttore. La stessa documentazione descrive opzioni di uscita bilanciata attiva o a trasformatore, quindi verificare la configurazione della macchina fisica prima di usare i valori d'uscita come riferimento.
- **Punto di fedelta':** non trasferire al 160X i dati del 160A. La copia del manuale 160X documenta l'uscita standard single-ended, mentre il 160A ha uscita elettronicamente bilanciata.
- **Fonti:** [DBX 160A e storia della famiglia](https://dbxpro.com/en/products/160a); [manuale 160X d'archivio, copia non ospitata dal produttore](https://tcfurlong.com/wp-content/uploads/dbx-160X-Rental-Manual.pdf).

### DBX 165A

- **Differenza fra 165 e 165A:** il memo DBX allegato al manuale dichiara che il 165A e' identico al 165 salvo PeakStop. Percio' i dati seguenti provengono dal manuale del 165 e dal memo 165A: non sono specifiche generiche della famiglia 160.
- **Topologia e controlli:** compressore mono Over Easy con detector RMS true e VCA in feed-forward; soglia, rapporto 1:1..infinito:1, attacco e rilascio manuali o auto, output gain, meter input/output/gain change, bypass hard-wire e accoppiamento stereo di due unita'. Il detector e' accessibile separatamente per EQ/filtri o altre elaborazioni esterne.
- **Tempi dichiarati:** in auto, attack tipico 15 ms per variazione di 10 dB, 5 ms/20 dB e 3 ms/30 dB; release 120 dB/s. In manuale, attack 1..400 dB/ms e release 10..4000 dB/s. Non convertire questi slew rate direttamente in un singolo tempo RC.
- **Dati elettrici del manuale 165:** threshold -40..+10 dB riferiti a 0.775 V RMS; ingresso max +24 dB (12.3 V RMS); uscita max +23 dBm su 600 ohm o superiore; risposta +0/-1 dB, 20 Hz..20 kHz; rumore equivalente ingresso < -90 dBm, 20 Hz..20 kHz. Distorsione tipica a compressione infinita e 1 kHz/0 dBm: 0.05% seconda armonica, 0.2% terza con tempi centrati.
- **PeakStop del 165A:** limiter/clipper a valle del compressore e dell'output gain, istantaneo, basato su clipper controllato in tensione con arrotondamento dei picchi. Il memo specifica range -2..+24 dBm; ammette brevi overshoot per l'arrotondamento e raccomanda di impostare il limite 1..2 dB sotto il ceiling assoluto.
- **Punto di fedelta':** Over Easy, risposta temporale auto/manuale, detector separato, stereo-link e PeakStop sono comportamenti distinti; non riutilizzare i valori del 160A/160X. Il PeakStop non e' un limiter lookahead trasparente.
- **Fonti primarie:** [DBX 165A — manuale ufficiale combinato 165/165A](https://dbxpro.com/en/products/165a).

### Compressori API standalone

La pagina ufficiale [API Compressors](https://apiaudio.com/compressors/) elenca i modelli sotto. TCS-II e' un channel strip integrato, non aggiunto come un secondo plugin compressore: la sua sezione compressore e' gia' coperta dal relativo riferimento 527A.

| Plugin proposto | Modello | Specifica utile alla fedelta' (pagina costruttore) | Fonte |
|---|---|---|---|
| `greg_api_2500` | API 2500 (fuori produzione) | Dual-channel; threshold -20..+10 dBu; THRUST con filtro prima del detector RMS; feedback oppure feed-forward; knee soft/medium/hard; auto/manual makeup; sidechain bilanciata; output clipping dichiarato a +28 dBu. I tempi e le posizioni di ratio/attack/release sono da completare dal manuale esatto. | [API 2500](https://apiaudio.com/product/2500-stereo-compressor/) |
| `greg_api_2500_plus` | API 2500+ | Dual-channel stereo o due unità mono; vecchio feedback e nuovo feed-forward; THRUST; auto makeup; Mix/Blend OFF, crossfade o parallel. Impedenza in 60 kΩ, uscita bilanciata a trasformatore 75 Ω, max +32 dBu, 20 Hz–50 kHz ±0.5 dB, S/N 120 dB. | [API 2500+](https://apiaudio.com/product/2500-stereo-bus-compressor/) |
| `greg_api_sr22` | API SR22 | Circuito nato nella console ATI Paragon; due canali, link stereo con somma RMS, THRUST, knee hard/soft; threshold -40..+15 dBu, ratio 1:1..10:1, attack adattivo 10–40 ms, release 30–400 ms, makeup 0..+20 dB. | [API SR22](https://apiaudio.com/product/sr22-compressor/) |
| `greg_api_225l` | API 225L | Modulo VCA della serie 200; feedback/feed-forward, hard/soft, auto makeup, sidechain e link stereo; threshold +10..-20 dBu, ratio 1:1..infinito, attack 1–25 ms, release 0.3–3 s. | [API 225L](https://apiaudio.com/product/225l-compressor/) |
| `greg_api_t25` | API T25 | Due canali, FET feedback, uscita Class A a valvole e trasformatore; ratio 2/4/8/20:1, attack 20 µs..0.8 ms, release 50 ms..1.2 s, 11 posizioni per ciascun tempo, THRUST e D-S. | [API T25](https://apiaudio.com/product/t25-tube-compressor/) |
| `greg_api_525` | API 525 | Peak-detecting feedback; attacco fisso 15 µs, ratio 2:1 oppure limit 20:1, quattro release mode, Ceiling accoppiato fra quantità di compressione e output, de-ess dedicato, collegamento di piu' unita'. | [API 525](https://apiaudio.com/product/525-compressor/) |
| `greg_api_525a` | API 525A | Evoluzione discreta del 525 con attack 15 µs..15 ms, sette release 0.05..2 s, ratio Compress 2:1 / Limit 20:1, Ceiling, 2510/2520 e trasformatore d'uscita. | [API 525A](https://apiaudio.com/product/525a-fet-compressor/) |
| `greg_api_527` | API 527 (fuori produzione) | VCA con threshold +10..-20 dBu, ratio 1:1..infinito, attack 1–25 ms, release 0.3–3 s, feed-forward/feedback, hard/soft, THRUST e link multi-unit. | [API 527](https://apiaudio.com/product/527-compressor-limiter/) |
| `greg_api_527a` | API 527A | VCA discreto; feed-forward/feedback, hard/soft, THRUST e bypass; attack 1–25 ms, release 0.3–3 s, ratio 1:1..infinito, output fader -infinito..+10 dB, link stereo. | [API 527A](https://apiaudio.com/product/527a-compressor-limiter/) |
| `greg_api_529` | API 529 | Compressore stereo dual-slot 500 Series derivato dal 2500; old/new, THRUST, tre knee, auto/variable makeup; ratio 1.5:1..infinito, attack <1..50 ms, release 0.05..4 s, threshold +10..-20 dBu. | [API 529](https://apiaudio.com/product/529-stereo-compressor/) |
| `greg_api_sv12` | API Select SV12 | Circuito ATI Paragon in modulo 500; due canali e stereo-link con somma RMS, THRUST, hard/soft; threshold -40..+15 dBu, ratio 1:1..10:1, attack adattivo 10–40 ms, release 30–400 ms. | [API SV12](https://apiaudio.com/product/api-select-sv12-compressor/) |

**Regola di modellazione:** non condividere una singola curva API generica. 525/525A sono FET peak-detecting con caratteristiche temporali e Ceiling differenti; 527/527A/225L sono varianti VCA con controlli e detector specifici; 2500/2500+ e 529 hanno routing e opzioni stereo; SR22/SV12 sono basati su un circuito ATI Paragon ma differiscono nel formato e nelle specifiche pubblicate; T25 aggiunge un percorso FET/valvolare a due canali. Riutilizzare codice non deve cancellare queste differenze.

## Lacune di specifiche e riferimenti non assegnati

| Plugin | Riferimento selezionato | Informazioni mancanti prima di scrivere un modello circuitale |
|---|---|---|
| `greg_harrison_pre`, `greg_harrison_comp`, `greg_harrison_eq` | Harrison 32C | Reperire manuale/revisione; documentare guadagni, curve EQ, topologia e tempi del compressore. |
| `greg_tape` | Studer A800 Mk III, 2-inch multitrack | Velocita' (15/30 IPS), formulazione nastro, flusso, EQ NAB/IEC, bias e allineamento. |
| `greg_exciter` | Aphex Aural Exciter Type C | Manuale Type C, revisione, generazione armoniche, frequenze e livelli. |
| `greg_low_enhancer` | dbx 120A Subharmonic Synthesizer | Specifiche di manuale trascritte; restano da caratterizzare algoritmo di sintesi e comportamento ai segnali reali. |
| `greg_bus_summer` | Nessun modello indicato | Scegliere mixer/console e percorso di somma (trasformatori, amplificatori, headroom, numero canali). |
| `greg_gate` | Nessun modello indicato | Scegliere unità/revisione e documentare detector, attack/hold/release, range, hysteresis e key filtering. |

Le macchine confermate hanno ora un'identita' hardware di progetto; i parametri mancanti non vanno comunque indovinati. Il gate e il sommatore restano senza outboard assegnato.

## Gap di documentazione e prossimo passo

1. Rappresentare 1176 Rev D/E come profili del medesimo plugin; bloccare le revisioni LA-2A, LA-3A, Fairchild 670, SSL bus comp e gli esatti moduli SSL E/G.
2. Recuperare e archiviare i manuali esatti con identificativo/revisione per Harrison 32C, Studer A800 Mk III, Aphex Type C e i compressori ottici; trascrivere le tabelle temporali del Fairchild.
3. Per ogni unità scelta creare un set di misure A/B: risposta a piccoli segnali, sweep a piu' livelli, transiente e recupero dopo burst, curva input/output, THD/spettro, rumore e condizioni di carico.
4. Aggiornare le schede `docs/plugins/` con i dati verificati e separare i controlli estesi dagli originali hardware.

Fino a questi passaggi, le implementazioni alpha restano modelli ispirati alla topologia, non emulazioni hardware validate.

# Aggiornamento del progetto

## Unificazione del catalogo
- Consolidati istruzioni, requisiti e indice completo dei plugin in `vcinst.md`, mantenendolo compatto e rinviando specifiche e stato alle schede dedicate e al dossier hardware.
- Rimosso `vcinstext.md` e aggiornati i riferimenti in README, inventario e stato operativo. Verificata la presenza delle schede per tutti i 69 ID del catalogo.

## Ricerca riferimenti outboard per la fedelta' (2026-11-19)
- Ripreso lo Step 0 partendo dal catalogo `vcinst.md` e dalle schede plugin.
- Creato `docs/specs/outboard_hardware_research.md`: mappa dei riferimenti analogici, descrizione dei modelli confermati, specifiche pubblicate da costruttori, estensioni digitali e scelte ancora aperte.
- Confermati dall'utente i riferimenti 1176 D/E (documentarli entrambi), SSL 4000 E/G, Harrison 32C, Studer A800 Mk III, Aphex Aural Exciter Type C, dbx 120A e SSL G-Series bus compressor.
- Aggiunti plugin distinti per DBX 160A, 160X e 165A e per gli 11 compressori API standalone nel catalogo ufficiale: 2500, 2500+, SR22, 225L, T25, 525, 525A, 527, 527A, 529 e SV12.
- Create 14 nuove schede `docs/plugins/` e aggiornati il catalogo, `docs/INVENTARIO.txt` e il dossier hardware.
- Documentate topologia e controlli dai manuali/pagine dei produttori; le schede distinguono i dati pubblicati dai parametri ancora da trascrivere e dalle estensioni digitali.
- Estratte dal manuale ufficiale combinato 165/165A le specifiche principali del 165 e il supplemento PeakStop del 165A; i valori storici sono marcati come tali. Il 160X e' stato trascritto da una scansione d'archivio terza, da verificare contro revisione e configurazione. Il manuale A800 Mk III, Harrison 32C e Aphex Type C restano da reperire.
- Trascritto il manuale ufficiale del dbx 120A: bande e range di sintesi, crossover, controlli, risposta, rumore, THD e livelli elettrici sono ora riportati nel dossier e nella scheda plugin.
- Aggiunti 15 channel strip distinti per modello: Neve 1073/1081/1084/88RS, Harrison 32C e Mixbus, SSL 4000 E/G e 9000 J, API Vision/Legacy Plus e shortlist Trident A-Range/80B, AMEK 9098 e MCI JH-500. I plugin pre/EQ/comp esistenti restano disponibili. Mixbus e' separato dall'hardware; per ogni modello le revisioni confluiranno in profili dello stesso ID, senza unire modelli diversi.
- Fonti Neve 1073/1081 e API Vision consultate; per gli altri strip le schede indicano le lacune di manuali primari e non assumono specifiche mancanti.
- Nessuna modifica DSP e nessuna calibrazione A/B eseguita. Le alpha restano hardware-inspired, non misurate.
- Prossimo: validare le fonti manuali mancanti, poi costruire la matrice di misure A/B prima di implementare i nuovi DSP.

## Nuovo obiettivo: `greg_child` Vari-Mu (2026-11-19)
- Requisiti: compressione con ratio crescente in funzione della gain reduction, sei costanti di tempo selezionabili e detector stereo-link oppure dual mono.
- Implementati modulo DSP `vari_mu` e crate CLAP `greg-child`; `Stereo Link` condivide il detector L/R, disattivandolo si hanno inviluppi indipendenti dual mono.
- Host CLAP: testati i sei tempi (latenza zero a 1x), differenza misurata fra stereo-link e dual mono e Sidechain Listen esterna.
- Verifiche finali: 48 test workspace, Clippy strict sulle librerie, build release e host CLAP di regressione per Child, FET, A2A, A3A, limiter, EQ, fase e M/S passati.
- Fedelta': le sei costanti nominali sono I 0.3 s, II 0.8 s, III 2 s, IV 5 s, V 2+10 s, VI 0.3+25 s; i passaggi di coda, la ratio e la saturazione sono semplificazioni ancora da calibrare A/B su hardware/manuale.
- Opzioni digitali aggiunte (sidechain, oversampling, inversione e output drive) documentate separatamente dalle caratteristiche circuitali.
- Prossimo plugin in ordine di catalogo: `greg_bus_comp`, compressore VCA da bus.

## Completamento alpha `greg_a3a` (2026-11-19)
- Verificato il crate CLAP `greg-a3a`, compressore opto a stato solido con curva e risposta temporale distinte da `greg-a2a`, modo Compress/Limit e stadio SoftClip oversampled.
- Esteso l'host CLAP offline con verifiche di riduzione a 1x/2x/4x/8x, sidechain esterna in modo Limit, latenza 0/15/22/25 campioni, bypass compensato e inversione L.
- Host: RMS compresso 0.0237 (1x..8x), sidechain esterna/Limit 0.0300, bypass RMS 0.1768; tutti i test host passano.
- Aggiornati scheda plugin/API, README, inventario e documentazione del test host.
- Prossimo plugin in ordine di catalogo: `greg_child`, compressore Vari-Mu.

## Modulo condiviso di metering (2026-11-19)
- Aggiunto `core_dsp_modules::metering::Metering` con misure input/output peak e RMS in dBFS, gain reduction massima per blocco e snapshot atomici leggibili in concorrenza.
- Integrato nei plugin CLAP presenti nel workspace; le utility non dinamiche riportano GR pari a zero.
- Aggiornati `vcinst.md`, inventario e scheda DSP.
- Limite alpha: il motore raccoglie e pubblica i valori ma non esiste ancora GUI né esposizione a parametri meter host-visibili.

## Nuovo obiettivo: limiter alpha `greg_limiter` (2026-11-19)
- Selezionato `greg_limiter` come prossimo plugin da creare: il crate non era ancora presente nel workspace.
- Obiettivo: controlli Input/Drive e Ceiling/Limit, true-peak conservativo, lookahead dichiarato, oversampling, release, bypass crossfade, sidechain opzionale e protezione da panic.
- Implementazione alpha avviata nel crate `plugins/greg-limiter`: parametri CLAP, layout mono/stereo, lookahead, oversampling, sidechain filtrato, bypass, dry delay e protezione panic.
- Verifica host: 1x/2x/4x/8x riportano latenza 240/255/262/265 campioni inclusi 5 ms di lookahead; bypass latenza-compensato e inversione passano.
- Verifiche: limiter crate compila con Clippy strict, 2 test unitari passano, suite workspace 43 test passano e build release workspace passa.

Ultimo aggiornamento: 2026-11-19
Stato complessivo: IN CORSO

## Prima emulazione analogica `greg_a2a` (2026-11-19)
- Aggiunto il crate CLAP `greg-a2a` (ID `com.greg.audio.greg-a2a`), mono/stereo, con Peak Reduction, Gain, Compress/Limit, Tube Drive, oversampling 1x/2x/4x/8x, bypass crossfade e inversione di polarita' L/R.
- Aggiunto `core_dsp_modules::opto_compressor`: detector soft-knee a ratio progressiva e memoria ottica con attacco circa 10 ms, prima fase di rilascio circa 60 ms e coda 0.5..5 s dipendente dall'esposizione del programma; coefficienti derivati dal sample rate.
- Aggiunta `tube_saturate`, curva asimmetrica normalizzata vicino allo zero per introdurre una componente armonica pari nello stadio d'uscita; il modello e' processato con oversampling e dichiara 0/15/22/25 campioni di latenza.
- Integrati `PanicProtection` e pass-through latched gia' condivisi; aggiornate scheda, specifica API, inventario, README e host CLAP offline.
- Limite di fedelta': modello fenomenologico alpha, non modello elettrico completo o replica calibrata. Sono necessarie misure A/B su una macchina reale per accordare costanti di tempo, curva di compressione, risposta in frequenza e spettro armonico.
- Verifiche passate: `cargo fmt --all -- --check`, `cargo test --workspace` (37 test), Clippy con `-D warnings`, build release e host CLAP per `greg_a2a`.
- Host: gain reduction e latenze a 1x/2x/4x/8x (0/15/22/25 campioni); a 4x il segnale di test misura RMS 0.069. Bypass compensato RMS 0.177 e inversione L corretta. Test host regressivi FET, EQ, encoder/decoder M/S e fase tutti passati.

## Obiettivo
Realizzare una suite di plugin audio CLAP in Rust con DSP prioritario, senza GUI nella fase alpha, elaborazione real-time e latenza dichiarata correttamente alla DAW.

## Inventario e documentazione individuale (2026-11-19)
- Richiesto un documento `.txt` dedicato per ogni plugin e per ciascun modulo DSP condiviso.
- Audit completato: aggiunti al catalogo il compressore multibanda configurabile (4 bande iniziali + espansione), limiter/maximizer e utility di inversione di polarità.
- Scomposta la suite Harrison in tre plugin; separate le schede per encoder/decoder M/S, EQ/comp Manley, pitch shifter e transient shaper mono/multibanda.
- Creati 40 documenti `docs/plugins/<id>.txt` e 12 documenti `docs/modules/<id>.txt`; verifica automatica: 40/40 ID plugin e 11/11 moduli dell'inventario hanno un documento dedicato.
- I TXT attuali sono specifiche iniziali, non manuali finali: parametri/API, test, latenza e limiti vanno completati con l'implementazione.

## Stato dei moduli
- IN CORSO: compressore FET `greg-fet`; scheletro CLAP e DSP alpha implementati.
- ALPHA: coppia di utility `greg_ms_encoder` e `greg_ms_decoder`, implementate in crate separati; mapping, bypass e latenza zero verificati nell'host CLAP offline.
- ALPHA: utility `greg_phase` per inversione mono/stereo della polarita', con bypass e latenza zero.
- DA FARE: compressore opto valvolare.
- DA FARE: compressore opto a stato solido.
- DA FARE: compressore Vari-Mu.
- DA FARE: compressore VCA bus.
- DA FARE: EQ passivo con interazione boost/attenuation sulle basse.
- DA FARE: EQ parametrico a quattro bande con filtri passa-alto e passa-basso.
- DA FARE: EQ a induttori.
- DA FARE: preamplificatore discreto.
- DA FARE: preamplificatore con trasformatore.
- DA FARE: compressore multibanda con quattro bande iniziali ed espansione configurabile.
- DA FARE: limiter/maximizer con input e limite di uscita.
- ALPHA: EQ chirurgico a dieci bande configurabili con filtri selezionabili e modalità a fase minima/lineare.
- DA FARE: exciter armonico con controllo per bande e livello di picco.
- DA FARE: low enhancer con generazione di sub-armoniche e controllo dinamico.
- DA FARE: distorsore multi-tecnologia con algoritmi e filtri pre/post selezionabili.

## Aggiornamento documentazione (2026-11-19)
- Aggiunti i quattro moduli sopra al catalogo breve e/o esteso; dettagliato il comportamento DSP di exciter, low enhancer e distorsore già presenti nel catalogo esteso.
- L'EQ chirurgico include filtri Bell, shelf, passa-alto/basso, notch e passa-banda, nonché modalità a fase minima e lineare con latenza da dichiarare.
- Verifica automatica non eseguita: modifica esclusivamente documentale.

## Avvio implementazione `greg-fet` (2026-11-19)
- Aggiunti i controlli Threshold, Knee e All Buttons In; il modo All Buttons In seleziona ratio 20:1 e knee minimo di 12 dB.
- Reso indipendente il detector envelope per ciascun canale e aggiornati i coefficienti quando cambiano Attack/Release.
- Il selettore Ratio è stato convertito a opzioni discrete 4:1, 8:1, 12:1 e 20:1.
- Verifiche passate: `cargo fmt --all -- --check`, `cargo test -p greg-fet` (7 test) e `cargo build -p greg-fet --release`.
- Prossimo: completare la rifinitura DSP del FET, quindi iniziare il modulo successivo.

## Libreria DSP condivisa (2026-11-19)
- Creato il crate workspace `core-dsp`, con API in `core_dsp_modules.rs` e moduli per routing/fase, filtri biquad, sidechain, FIR 2x, dithering TPDF, stadi non lineari, sag alimentazione, isteresi del trasformatore, variazioni analogiche deterministiche per seed, envelope follower, smoothing e bypass crossfade.
- Collegate a `greg-fet` le implementazioni condivise di saturazione ed envelope follower.
- `cargo test --workspace` passa (19 test); `cargo clippy --workspace --all-targets -- -D warnings` passa.
- `cargo build --workspace --release` passa.
- Aggiunti filtri notch e band-pass alle primitive biquad; test, lint e build release ripetuti e superati.
- Da completare: integrare i moduli comuni negli altri processori; persistenza dell'instance ID; routing M/S multi-canale completo; oversampling selezionabile 1x/2x/4x/8x con filtri definiti e latenza misurata; documentazione per modulo e ulteriori test numerici.
- Limite attuale: `FirResampler` è un mattone FIR a 2x con coefficienti forniti dal chiamante, non una soluzione pronta/automatica di anti-aliasing universale.

## Oversampling FIR (2026-11-19)
- Implementati fattori 1x/2x/4x/8x con tre stadi FIR half-band Blackman da 31 tap, creati con stato statico e senza allocazioni nel callback audio.
- Il FET applica l'oversampling solo allo stadio di saturazione; il parametro aggiorna la latenza host in initialize e quando cambia.
- La misura del ritardo di gruppo impulsivo rileva 0/14.5/21.75/25.375 campioni; il plugin comunica all'host la latenza intera più vicina: 0/15/22/25 campioni.
- Un test spettrale misura la componente alias a 0.1 Fs prodotta dalla terza armonica di un tono a 0.3 Fs: 8x deve ridurla almeno del 50% rispetto a 1x.
- Verifiche passate: formattazione, `cargo test --workspace` (21 test), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --release`.

## Integrazione bypass e polarita nel FET (2026-11-19)
- Aggiunti bypass con crossfade lineare di 5 ms e inversione di polarita indipendente per Left/Right.
- Il ramo dry e' compensato con delay a memoria fissa pari alla latenza intera dichiarata dall'oversampler; le istanze per canale vengono create fuori dal callback.
- Aggiornate le schede `docs/plugins/greg_76.txt` e `docs/modules/realtime_controls.txt`.
- Stato DSP interno (delay, oversampler, envelope e crossfade) azzerato in initialize/reset; l'automazione della polarita viene letta per campione.
- Verifiche passate: formattazione, `cargo test --workspace` (23 test), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --release`.

## Risultati verificati
- `cargo fmt --all -- --check` passa.
- `cargo test -p greg-fet` passa con 7 test.
- `cargo build -p greg-fet --release` passa.
- `target/release/libgreg_fet.so` e' una libreria ELF e esporta `clap_entry`.
- Il test host offline carica il plugin e verifica latenza dichiarata 0/15/22/25 campioni a 1x/2x/4x/8x, processamento delle sinusoidi, bypass e inversione indipendente L.
- Il controllo NIH-plug delle allocazioni e' abilitato nei build debug.

## EQ chirurgico `greg-surgical-eq` (2026-11-19)
- Aggiunto il crate CLAP con dieci bande attivabili e filtri Bell, Low/High Shelf, High/Low Pass, Notch e Band Pass.
- Il modulo `core-dsp::surgical_eq` riutilizza i biquad condivisi per la modalita' minimum-phase e implementa la modalita' linear-phase con FIR simmetrico a 257 tap.
- Latenza dichiarata all'host: 0 campioni minimum-phase, 128 campioni linear-phase; bypass crossfade e percorso dry allineato alla latenza attiva.
- Test DSP: boost sinusoidale min/linear-phase, simmetria della risposta impulsiva, latenza del FIR, risposta identita' e rifiuto di configurazioni non valide.
- Host CLAP carica il plugin e verifica toni identita' in entrambe le modalita', correlazione e latenza.
- Verifiche passate: formattazione, `cargo test --workspace` (29 test), Clippy con `-D warnings` e build release.
- Limiti alpha: FIR diretto a 257 tap; risposta spettrale linear-phase approssimata; riconfigurazione parametri ai confini dei blocchi. Restano da misurare prestazioni e risposta in DAW.

## Utility Mid/Side `greg_ms_encoder` e `greg_ms_decoder` (2026-11-19)
- Creati due plugin CLAP distinti che riusano il modulo `routing_matrix`.
- Convenzione ortonormale: M=(L+R)/sqrt(2), S=(L-R)/sqrt(2); il decoder ricostruisce L=(M+S)/sqrt(2), R=(M-S)/sqrt(2).
- Entrambi espongono trim Mid/Side (-24..+24 dB), balance stereo (-1..+1), inversione di polarita' indipendente e bypass con crossfade di 5 ms.
- Layout mono: pass-through con inversione L opzionale; trim e balance non applicati. Stereo: processamento in-place senza allocazioni; latenza dichiarata zero.
- Schede plugin aggiornate con parametri, convenzioni, layout, test e limiti alpha; aggiornati README, inventario, scheda routing matrix e documentazione dell'host.
- Verifiche passate: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace` (29 test), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --release`.
- Host CLAP offline: encoder L=R produce M attivo e S nullo; decoder M con S nullo produce L=R; bypass passa; latenza zero per entrambi.
- Prossimo: testare i due plugin in DAW reale; continuare con i moduli rimanenti.

## Utility di polarita' `greg_phase` (2026-11-19)
- Aggiunto un plugin CLAP standalone mono/stereo; riusa `StereoFrame::invert_phase` da `core-dsp`.
- Controlli: inversione indipendente L/R, comando linkato "Invert Both" e bypass crossfade di 5 ms.
- Il plugin applica solo una moltiplicazione per -1: nessuna rotazione dipendente dalla frequenza o ritardo; latenza CLAP zero.
- Scheda del plugin e modulo, inventario, README e test host aggiornati.
- Verifiche passate: formattazione, `cargo check --workspace`, `cargo test --workspace` (30 test), Clippy con `-D warnings` e build release.
- Host CLAP: inversione L, R e linkata verificata con correlazione; bypass confermato; latenza zero.
- E' un'utility digitale, non una simulazione hardware; il vincolo di fedelta' alle macchine reali si applica ai plugin di emulazione.

## Compressore VCA da bus `greg-bus-comp` (2026-11-20)

- Aggiunto il crate CLAP `greg-bus-comp`, compressore stereo VCA da bus con riferimento SSL 4000 G.
- Controlli: Input, Threshold, Ratio (1.5/2/4/10/∞:1), Attack, Release, Makeup, Stereo Link, Oversampling (1x/2x/4x/8x), VCA Drive, Bypass, inversione L/R, sidechain esterna opzionale con filtri (HP/LP/Bell/LowShelf/HighShelf) e Sidechain Listen.
- VCA: riduzione in dB calcolata dal livello rispetto alla soglia con rapporto selezionato; soft knee di 2 dB attorno alla soglia.
- Stereo Link condivide il detector L/R; disattivato si hanno detector e gain reduction indipendenti dual mono.
- Latenza dichiarata: 0/15/22/25 campioni a 1x/2x/4x/8x; bypass compensato con crossfade 5 ms e percorso dry allineato.
- Riutilizza `EnvelopeFollower`, `SidechainProcessor`, `Metering`, `Oversampler`, `BypassCrossfade`, `LatencyDelay`, `PanicProtection` da `core-dsp`.
- Test DSP: target reduction sotto soglia zero, scalamento con ratio, tracking attack/release dell'envelope follower, valori ratio positivi.
- Verifiche passate: `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --release`.
- Fedeltà: modello alpha hardware-inspired; la curva VCA, i tempi e il comportamento del bus compressor devono essere confrontati con manuale di servizio e misure A/B su unità originale prima di dichiarare replica fedele.
- Prossimo: validare in host CLAP offline e in DAW reale.

## Limiti e verifiche aperte
- Il plugin e' stato caricato dall'host CLAP offline; resta da validare in una DAW reale e la latenza totale di monitoraggio non e' misurata.
- NIH-plug fornisce al plugin buffer `f32`; il core DSP generico puo' supportare `f32` e `f64`, ma il wrapper non offre elaborazione host-native `f64`.
- La protezione condivisa intercetta panic Rust solo con `panic=unwind`; `panic=abort`, segfault e corruzione della memoria non sono recuperabili in-process e richiedono isolamento a livello host/processo.

## Protezione condivisa dai panic (2026-11-19)
- Aggiunto `core_dsp_modules::panic_protection::PanicProtection`, riusato dai cinque plugin CLAP attualmente implementati.
- Il panic nel callback DSP viene intercettato: il blocco audio corrente viene azzerato e l'istanza resta latched in pass-through, senza eseguire altro DSP nei callback successivi.
- Test del modulo condiviso: il panic porta allo stato `Tripped`, l'esecuzione successiva torna `Disabled` senza richiamare la closure e l'elaborazione normale resta attiva in assenza di fault.
- La cattura richiede la strategia Rust `panic=unwind`; non protegge da `panic=abort`, abort espliciti, segfault, corruzione della memoria o guasti del processo host.
- La verifica in una DAW reale e l'isolamento di un plugin guasto a livello processo restano fuori dalla protezione in-process.

## Prossimi passi
- Eseguire i test in una DAW reale.
- Validare `greg_surgical_eq` in una DAW reale, misurando risposta, carico CPU e automazioni.
- Proseguire con i moduli rimanenti, aggiornando `STATE.md` per i micro-step e questo file per il riepilogo cumulativo.

## Rapporto finale
Da completare al termine dell'intero progetto con gli artefatti, le istruzioni di build/installazione, le verifiche e le limitazioni residue.
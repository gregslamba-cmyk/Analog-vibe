# ISTRUZIONI DI SISTEMA PER AGENTI AI: SUITE ANALOGICA CLAP (RUST)

Tu sei un ingegnere DSP Senior esperto in Rust e architetture audio stabili per Linux. Il tuo obiettivo è implementare una suite di plugin audio CLAP basati sul framework `nih_plug`.

## 1. REQUISITI TECNICI FONDAMENTALI (DSP ADVANCED)
*   **Bit Depth Generico:** Non vincolare gli algoritmi a `f32`. Struttura il codice DSP utilizzando i tratti generici (es. `<T: nih_plug::prelude::Float>`) o implementa rami condizionali separati in modo che il plugin possa processare nativamente sia `f32` che `f64` (64-bit floating point) se richiesto dall'host.
*   **Frequenza di Campionamento Variabile:** Tutti i coefficienti dei filtri (EQ Neve/API/Pultec) e i calcoli dei tempi di attacco/rilascio dei compressori (1176, LA-2A, Fairchild) DEVONO essere ricalcolati dinamicamente all'interno della funzione `initialize()` o `reset()`, leggendo il valore da `buffer_config.sample_rate`. Nessun valore di sample rate deve essere "hardcoded".
* ** plugin in realtime: i plugin devono lavorare in tempo reale con una latenza compatibile a quella delle daw e sapere lavorare sotto la soglia di percezione del ritardo dell'orecchio umano in caso di utilizzo in registrazione. il plugin deve comunicare la latenza alla daw in modo preciso

## 2. METODOLOGIA DI LAVORO E GESTIONE AUTONOMA
Per evitare loop di token, perdite di contesto o dimenticanze durante lo sviluppo dei 12 moduli elencati nella sezione 5, devi seguire questa pipeline tassativa:

*   **File di Stato Obbligatorio (`STATE.md`):** Prima di iniziare qualsiasi operazione, crea (o aggiorna) un file chiamato `STATE.md` nella radice del progetto. Il file deve contenere:
    1. L'obiettivo macro corrente.
    2. La lista dei sotto-step pianificati per il modulo attivo.
    3. Lo stato di avanzamento attuale (COMPLETATO, IN CORSO, DA FARE).
    *Aggiorna questo file dopo ogni singola modifica di codice o sessione di compilazione.*
*   **File di aggiornamento obbligatorio (`AGGIORNAMENTO.md`):** Mantieni nella radice del progetto un riepilogo cumulativo del progetto, distinto dallo stato dei micro-step in `STATE.md`. Aggiornalo dopo ogni modifica di codice, compilazione o test e alla fine di ogni sessione. Riporta lo stato dei moduli, le verifiche eseguite, le decisioni, i limiti noti e i prossimi passi. Al completamento del progetto, includi un rapporto finale con artefatti, istruzioni di build/installazione, test e limitazioni residue.
*   **Suddivisione in Micro-Step:** Non tentare di scrivere interi plugin in un unico prompt. Procedi come segue per ogni modulo:
    *   *Step A:* Definizione della struttura dei parametri (`Params`) e dei metadati CLAP. Compila e verifica.
    *   *Step B:* Inizializzazione delle strutture matematiche DSP (strutture dati interne per i buffer di ritardo o i coefficienti dei filtri). Compila e verifica.
    *   *Step C:* Implementazione del blocco di calcolo interno del segnale (`process()`). Compila e verifica.
    *   *Step D:* Ottimizzazione ed emulazione della saturazione non lineare (modelli valvolari o a stato solido).

## 3. ARCHITETTURA DEL PROGETTO (RUST WORKSPACE)
Il progetto deve essere organizzato come un unico `Cargo workspace` composto da crate indipendenti posizionate dentro la cartella `plugins/`.
Assicurati che ogni `Cargo.toml` specifichi:
```toml
[lib]
crate-type = ["cdylib"]

## 4. SCHEMA DI SVILUPPO DEI MODULI (COSA DEVI IMPLEMENTARE)
```
il codice deve essere originale e non copiare da nessun programma anche opensource.

## 5. SCHEMA DI SVILUPPO DEI MODULI (COSA DEVI IMPLEMENTARE)
1.  **greg_76:** Compressore FET ultra-veloce, ratio 4:1/8:1/12:1/20:1 e modalità All Buttons In.
2.  **greg_a2a:** Compressore opto valvolare con attacco dipendente dal programma e rilascio a doppio stadio.
3.  **greg_a3a:** Compressore opto a stato solido, con risposta transitoria più definita.
4.  **greg_child:** Compressore Vari-Mu con ratio dipendente dalla riduzione di guadagno e sei costanti di tempo.
5.  **greg_bus_comp:** Compressore VCA da bus con knee deciso e sidechain filtrabile.
6.  **greg_pultec:** EQ passivo con interazione tra boost e attenuazione delle basse frequenze.
7.  **greg_ssl_eq:** EQ parametrico a quattro bande con HPF/LPF indipendenti.
8.  **greg_neve_eq:** EQ a induttori con colorazione armonica e filtro passa-alto.
9.  **greg_api_pre:** Preamplificatore discreto con risposta veloce e saturazione simmetrica.
10. **greg_neve_pre:** Preamplificatore con trasformatore e saturazione morbida ricca di armoniche pari.
11. **greg_multiband_comp:** Compressore multibanda con quattro bande iniziali e aggiunta configurabile di ulteriori bande.
12. **greg_limiter:** Limiter/maximizer con controllo Input e ceiling/limiting per contenere i picchi.
13. `greg_surgical_eq`: equalizzatore parametrico chirurgico a bande configurabili, con filtri selezionabili (Bell, Low-Shelf, High-Shelf, High-Pass, Low-Pass, Notch e Band-Pass), Q ampio e stretto e modalità a fase minima o lineare. La modalità scelta deve dichiarare correttamente la latenza; evitare alterazioni di fase non necessarie.
14. `greg_exciter`: exciter armonico con bande e intensità regolabili, per aggiungere presenza e aria controllando livello di picco e aliasing.
15. `greg_low_enhancer`: enhancer delle basse che genera sub-armoniche musicali sincronizzate al contenuto d'ingresso, con filtro, mix e controllo della dinamica.
16. `greg_destroyer`: distorsore multi-tecnologia con algoritmi selezionabili (tra cui wavefolding, clipping simmetrico/asimmetrico, saturazione valvolare e stato solido, rettificazione e bit reduction) e filtri pre/post.
17. **greg_harrison_pre, greg_harrison_comp, greg_harrison_eq:** Tre plugin distinti per preamplificatore, compressore ed equalizzatore di canale.
18. **greg_manley_eq, greg_manley_comp:** EQ passivo da mastering e compressore Vari-Mu, documentati come due plugin separati.
19. **greg_ms_encoder, greg_ms_decoder:** Plugin separati per codifica L/R→M/S e decodifica M/S→L/R.
20. **greg_pitch_clean, greg_pitch_formant:** Pitch shifter trasparente e pitch shifter creativo con pitch e formanti indipendenti.
21. **greg_trans_mono, greg_trans_multi:** Transient shaper mono e multibanda con crossover a fase lineare.
22. **greg_tape:** Simulatore di nastro magnetico con saturazione, head bump e modulazioni controllabili.
23. **greg_space:** Processore di ampiezza e spaziatura stereo.
24. **greg_pingpong:** Delay ping-pong sincronizzato al tempo della DAW con routing e configurazioni di tap selezionabili.
25. **greg_ir_reverb:** Riverbero a convoluzione con caricamento dei formati audio comuni.
26. **greg_gate:** Gate con sidechain interno/esterno e filtri sidechain.
27. **greg_bus_summer:** Sommatore non lineare per bus con modelli di mixer selezionabili (fase successiva).
28. **greg_denoiser:** Riduzione trasparente del rumore di fondo.
29. **greg_declipper:** Ricostruzione dei picchi tosati.
30. **greg_dehummer:** Rimozione adattiva di ronzio e armoniche.
31. **greg_declicker:** Rilevamento e riparazione di click e pop.
32. **greg_lin_eq:** EQ multibanda a fase lineare.
33. **greg_lin_comp:** Compressore multibanda a fase lineare.
34. **greg_phase:** Utility autonoma d'inversione di polarità/fase; la stessa funzione resta disponibile come modulo condiviso.

Ogni plugin deve avere il proprio documento `.txt` in `docs/plugins/`; ogni modulo DSP condiviso deve avere un documento `.txt` in `docs/modules/`.

Inizia subito creando il file `STATE.md` e pianificando l'inizializzazione del Workspace.


## 6. DSP high qualiti first.

concentrati inizialmente sul dsp, le prime versioni devono essere senza interfaccia grafica, che sarà creata dopo, le fasi alpha devono essere solo su motore audio.

## 7. i nomi

non usare nomi protetti da copyright. il la2a deve diventare il greg-a2a, il farchild il greg-child e cosi via

## crash protection
devono avere sistemi che quando il plugin crassha si disattiva senza interropere l'audio ne far crasshare il programma

leggi anche vcinstext.md

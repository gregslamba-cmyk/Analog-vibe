# Stato del progetto

## Ultimo modulo: utility Mid/Side
- [COMPLETATO] Implementare `greg_ms_encoder` e `greg_ms_decoder` come plugin CLAP distinti.
- [COMPLETATO] Riutilizzare la matrice M/S condivisa, documentare la scala ortonormale e il comportamento mono.
- [COMPLETATO] Aggiungere bypass crossfade di 5 ms, trim M/S, bilanciamento L/R e inversione di polarita'.
- [COMPLETATO] Verificare build, latenza zero e mapping M/S nell'host CLAP offline.
- [COMPLETATO] Aggiornare schede TXT, inventario, README e documentazione del test host.

## Obiettivo
Creare una suite di plugin audio CLAP in Rust, con DSP senza interfaccia grafica nella fase iniziale e requisiti real-time. Allineare il catalogo breve e quello esteso.

## Passi pianificati
- [COMPLETATO] Integrare nel FET bypass con crossfade, inversione di polarità L/R e dry delay allineato alla latenza dichiarata.
- [COMPLETATO] Aggiornare le schede TXT del plugin e dei controlli real-time condivisi.
- [COMPLETATO] Verificare formattazione, test workspace (23), Clippy e build release.
- [COMPLETATO] Confrontare l'inventario richiesto con i due cataloghi e aggiungere i moduli/plugin mancanti.
- [COMPLETATO] Creare un documento TXT per ogni plugin e modulo DSP condiviso, con stato e specifica iniziale.
- [COMPLETATO] Verificare che ogni ID plugin/modulo dell'inventario abbia il relativo TXT.
- [COMPLETATO] Creare un host CLAP offline riutilizzabile per caricare il plugin e inviare segnali audio sintetici.
- [COMPLETATO] Verificare in host CLAP i fattori oversampling, latenza dichiarata, bypass e inversione L/R del FET.
- [COMPLETATO] Implementare `greg_surgical_eq` alpha con dieci bande, filtri selezionabili e modalita' min/linear phase.
- [COMPLETATO] Implementare `greg_ms_encoder` e `greg_ms_decoder` CLAP con trim, balance, polarita', bypass e latenza zero.
- [COMPLETATO] Aggiungere alla libreria DSP condivisa FIR linear-phase, latenza fissa e test numerici.
- [COMPLETATO] Caricare FET ed EQ nell'host CLAP offline e verificare processing e latenze dichiarate.
- [DA FARE] Ampliare le schede da specifiche iniziali a manuali/API completi durante l'implementazione di ciascun componente.
- [COMPLETATO] Implementare oversampling FIR selezionabile 1x/2x/4x/8x nella libreria condivisa.
- [COMPLETATO] Integrare oversampling nello stadio di saturazione FET e aggiornare la latenza all'host.
- [COMPLETATO] Misurare il ritardo di gruppo impulsivo arrotondato (0/15/22/25 campioni) e la riduzione delle armoniche ripiegate nel test DSP.
- [COMPLETATO] Verificare formattazione, test workspace (21), Clippy e build release.
- [COMPLETATO] Creare il crate condiviso `core-dsp` con primitive audio real-time riusabili.
- [COMPLETATO] Collegare la saturazione di `greg-fet` alla libreria condivisa.
- [COMPLETATO] Integrare nella libreria i controlli real-time condivisi (envelope, smoothing e bypass crossfade).
- [COMPLETATO] Riutilizzare in `greg-fet` envelope follower e saturazione condivisi.
- [COMPLETATO] Aggiungere notch e band-pass al modulo filtri; test (19), Clippy e build release passati.
- [DA FARE] Integrare bypass, routing, sidechain, modelli analogici e oversampling nei singoli plugin quando implementati.
- [DA FARE] Completare in iterazioni successive bypass crossfade, smoothing e ulteriori moduli elencati in `vcinstext.md`.
- [COMPLETATO] Aggiungere threshold, knee e modalità All Buttons In al compressore FET.
- [COMPLETATO] Separare lo stato dell'envelope follower per canale e aggiornare attack/release durante il processamento.
- [COMPLETATO] Limitare il selettore alle ratio classiche 4:1, 8:1, 12:1 e 20:1.
- [COMPLETATO] Verificare formattazione e test del crate `greg-fet` (7 test).
- [COMPLETATO] Compilare la libreria `greg-fet` in release.
- [DA FARE] Proseguire la rifinitura DSP del FET e poi iniziare il successivo modulo della suite.
- [COMPLETATO] Confrontare `vcinst.md` e `vcinstext.md` e rilevare i requisiti non allineati.
- [COMPLETATO] Aggiungere al catalogo EQ chirurgico, exciter, low enhancer sub-armonico e distorsore multi-tecnologia.
- [DA FARE] Allineare i requisiti mancanti e le incongruenze residue tra i due documenti.
- [COMPLETATO] Inizializzare il workspace Cargo.
- [COMPLETATO] Creare la struttura `plugins/`.
- [COMPLETATO] Creare il crate `greg-fet` nel workspace.
- [COMPLETATO] Definire parametri e metadati CLAP; compilare e verificare.
- [COMPLETATO] Inizializzare il DSP in funzione del sample rate e supportare `f32`/`f64` nel core.
- [COMPLETATO] Implementare il processamento audio soft-knee.
- [COMPLETATO] Aggiungere saturazione e validare il comportamento numerico.
- [SUPERATO] Dichiarare 0 campioni di latenza algoritmica (baseline prima dell'oversampling; ora 0/15/22/25 a seconda del fattore).
- [COMPLETATO] Ricostruire il binario release con il reporting di latenza aggiornato.
- [COMPLETATO] Verificare formato ELF e simbolo `clap_entry` del binario.
- [COMPLETATO] Caricare il plugin in un host CLAP offline e verificare elaborazione, bypass, polarità e latenza dichiarata.
- [DA FARE] Validare il plugin in una DAW reale e misurare la latenza totale del sistema audio.
- [DA FARE] Validare `greg_surgical_eq` in una DAW reale; misurare risposta, carico CPU e automazioni.

## Avanzamento
Il FET usa bypass crossfade di 5 ms, inversione di polarità indipendente L/R e dry delay pari alla latenza dichiarata dell'oversampling. Stato DSP ripristinato in initialize/reset. `cargo fmt --all -- --check`, `cargo test --workspace` (23 test), Clippy e build release passano.
Oversampling FIR 1x/2x/4x/8x integrato nello stadio di saturazione FET; il ritardo di gruppo misurato è 0/14.5/21.75/25.375 campioni e il plugin comunica all'host il valore intero più vicino (0/15/22/25). Test sinusoidale verifica almeno il 50% di riduzione di una componente alias a 8x rispetto a 1x. `cargo fmt --all -- --check`, `cargo test --workspace` (21 test), Clippy e build release passano.
`cargo fmt --all -- --check` e `cargo test -p greg-fet` passano (7 test). Il percorso audio applica input drive, envelope follower, compressione soft-knee, output level e saturazione morbida. NIH-plug ha `assert_process_allocs` abilitato nei build debug.
Host offline: `clap-signal-host` carica realmente `target/release/libgreg_fet.so`, processa sinusoidi a 440 Hz per 1x/2x/4x/8x e una sinusoide a 1 kHz con bypass/inversione L. Output finito, livelli, correlazione bypass e latenza CLAP 0/15/22/25 campioni verificati. La latenza totale percepita dipende anche da DAW, driver, interfaccia audio e dimensione del buffer; non è misurata da questo host. Il build release è un ELF condiviso che esporta `clap_entry`.
`greg-surgical-eq` aggiunto al workspace: 10 bande con sette tipi di filtro, DSP minimum-phase biquad e linear-phase FIR simmetrico a 257 tap; latenza CLAP 0/128 campioni. Test core verificano boost min/linear-phase, simmetria FIR, identita' con ritardo dichiarato e input invalidi. Host CLAP conferma caricamento, sinusoidi d'identita', correlazione e latenza in entrambi i modi.

Utility M/S: `greg_ms_encoder` e `greg_ms_decoder` sono crate CLAP separati e riusano `StereoFrame::route(MidSide)` e `from_mid_side()` con scala ortonormale. Entrambi espongono trim Mid/Side (-24..+24 dB), Balance L/R (-1..+1), inversione selettiva, bypass crossfade 5 ms e latenza zero. Layout mono: pass-through con inversione L opzionale. `cargo test --workspace` (29 test), Clippy e build release passano; host offline verifica M attivo/S nullo nell'encoder, L=R nel decoder, bypass e output finito.

## Vincoli
- `nih_plug::Buffer` espone campioni `f32`: un core DSP generico può supportare `f32` e `f64`, ma il wrapper NIH-plug non fornisce processing host-native `f64`.
- Il plugin sanitizza i campioni non finiti e verifica le allocazioni in debug; l'isolamento da panic o segfault non può essere garantito dal plugin stesso e dipende dall'host audio.
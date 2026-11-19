# Aggiornamento del progetto

Ultimo aggiornamento: 2026-11-19
Stato complessivo: IN CORSO

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

## Limiti e verifiche aperte
- Il plugin e' stato caricato dall'host CLAP offline; resta da validare in una DAW reale e la latenza totale di monitoraggio non e' misurata.
- NIH-plug fornisce al plugin buffer `f32`; il core DSP generico puo' supportare `f32` e `f64`, ma il wrapper non offre elaborazione host-native `f64`.
- La sanitizzazione dei campioni non finiti non sostituisce l'isolamento da panic o segfault, che dipende dall'host audio.

## Prossimi passi
- Eseguire i test in una DAW reale.
- Validare `greg_surgical_eq` in una DAW reale, misurando risposta, carico CPU e automazioni.
- Proseguire con i moduli rimanenti, aggiornando `STATE.md` per i micro-step e questo file per il riepilogo cumulativo.

## Rapporto finale
Da completare al termine dell'intero progetto con gli artefatti, le istruzioni di build/installazione, le verifiche e le limitazioni residue.
================================================================================
PLUGIN-SUPPORT — BRIDGE MULTI-FORMATO & HOST INTEGRATION (CLAP + VST3)
================================================================================

plugin-support e' il crate di supporto condiviso dell'ecosistema Analog-Vibe.
Funge da ponte architetturale tra i motori audio puri di core-dsp e il framework
di hosting dei plugin (nih-plug), gestendo la negoziazione dei layout I/O
multicanale, la sidechain ausiliaria, il bridging telemetrico verso la DAW
e l'esportazione unificata in formato nativo CLAP e VST3.


--------------------------------------------------------------------------------
1. ARCHITETTURA DEI LAYOUT I/O & ROUTING MULTICANALE (Mono -> 7.1)
--------------------------------------------------------------------------------

plugin-support standardizza la dichiarazione dei canali supportati da ciascun
plugin, consentendo all'host DAW di instanziare il processore in qualsiasi
formato compreso tra 1 e 8 canali:

- Costante MAX_AUDIO_CHANNELS: 8 canali simultanei.

- Vettore Ordinato dei Canali (CHANNELS = [2, 1, 3, 4, 5, 6, 7, 8]):
  Dichiara la precedenza di negoziazione con l'host:
  * Posizione 0: Stereo (2 ch) - default preferito dalla maggioranza delle DAW.
  * Posizione 1: Mono (1 ch).
  * Posizione 2: LCR (3 ch - Left, Center, Right).
  * Posizione 3: Quad / 4.0 (4 ch - L, R, Ls, Rs).
  * Posizione 4: 5.0 Surround (5 ch - L, R, C, Ls, Rs).
  * Posizione 5: 5.1 Surround (6 ch - L, R, C, LFE, Ls, Rs).
  * Posizione 6: 7.0 Surround (7 ch - L, R, C, Ls, Rs, Lrs, Rrs).
  * Posizione 7: 7.1 Surround (8 ch - L, R, C, LFE, Ls, Rs, Lrs, Rrs).

- Matrici Predefinite di Layout:
  1. SURROUND_IO_LAYOUTS (8 elementi):
     Utilizzata dai plugin di utilita', EQ e saturazione privi di ingresso
     sidechain ausiliario (es. greg-surgical-eq, greg-pultec, greg-phase,
     greg-ms-encoder, greg-ms-decoder).
  2. DYNAMICS_IO_LAYOUTS (21 elementi):
     Utilizzata da tutti i plugin di dinamica (compressori, limiter, gate).
     Espone per ogni formato audio principale una porta ausiliaria "Sidechain"
     configurabile come MONO o STEREO (es. Stereo + Mono Sidechain, Stereo +
     Stereo Sidechain, 5.1 + Stereo Sidechain, ecc.).


--------------------------------------------------------------------------------
2. LA MACRO CONDIVISA export_greg_plugin!
--------------------------------------------------------------------------------

Per garantire la massima consistenza e azzerare il boilerplate, ogni plugin
esporta nativamente entrambi i formati industriali tramite una singola macro:

```rust
core_dsp::export_greg_plugin!(
    NomePluginStruct,
    "nome-plugin-url"
);
```

Cosa genera automaticamente la macro:
1. Export Nativo CLAP:
   Espone il simbolo C clap_entry conforme alle specifiche CLAP 1.1+, rendendo
   il binario immediatamente caricabile da DAW native CLAP (Bitwig, Reaper).
2. Export Nativo VST3:
   Espone i simboli C GetPluginFactory e ModuleEntry per compatibilita' nativa
   con host VST3 (Cubase, Nuendo, Studio One, Ableton Live, FL Studio).
3. Calcolo Deterministico del VST3 Class ID:
   Deriva un UUID a 128 bit deterministico (RFC 4122 v4) partendo dall'URL
   univoco del plugin, assicurando che lo stesso plugin mantenga identico ID
   su qualsiasi piattaforma e compilazione successiva.
4. Metadati Standardizzati Greg Audio:
   - Vendor: "Greg Audio"
   - URL: "https://github.com/gregslamba-cmyk/Analog-vibe"
   - Email: "support@example.invalid"
   - Categorie ufficiali CLAP e VST3.


--------------------------------------------------------------------------------
3. METERING & TELEMETRY BRIDGE
--------------------------------------------------------------------------------

Il modulo unifica la telemetria tra il thread di calcolo audio in tempo reale
e la visualizzazione grafica / reporting DAW:

- Picchi Input/Output e Clip:
  Tracciati per-canale su array atomici [AtomicU32; MAX_GUI_CHANNELS] mediante
  f32::to_bits() con semantica Ordering::Relaxed. Zero mutex, zero contesa di memoria.

- Gain Reduction Bridge:
  La riduzione di guadagno e' memorizzata in dB negativi (es. -3.5 dB) e resa
  disponibile sia alla GUI vettoriale sia all'host DAW tramite l'apposita
  estensione CLAP / VST3 per i meter di Gain Reduction sul canale mixer.

- Correlazione di Fase Stereo:
  Calcolata costantemente nell'intervallo [-1.0 .. +1.0] per il monitoraggio
  della compatibilita' mono e delle cancellazioni di fase.

- Latenza Dichiarata all'Host:
  Integrazione diretta con il metodo latency_samples(&self) -> u32 per informare
  l'host DAW dell'esatto ritardo introdotto (ad es. dall'oversampling lineare o
  dalla convoluzione), consentendo il Plugin Delay Compensation (PDC) automatico.


--------------------------------------------------------------------------------
4. ESEMPIO DI DEFINIZIONE PLUGIN CON PLUGIN-SUPPORT
--------------------------------------------------------------------------------

```rust
use nih_plug::prelude::*;
use plugin_support::DYNAMICS_IO_LAYOUTS;

pub struct MyCompressor {
    params: Arc<MyCompressorParams>,
}

impl Plugin for MyCompressor {
    const NAME: &'static str = "Greg My Compressor";
    const VENDOR: &'static str = "Greg Audio";
    const URL: &'static str = "https://github.com/gregslamba-cmyk/Analog-vibe";
    const EMAIL: &'static str = "support@example.invalid";
    const VERSION: &'static str = "0.1.0";

    // Esposizione di tutti i layout Surround + Sidechain con una sola riga
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &DYNAMICS_IO_LAYOUTS;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // Inizializzazione, reset e callback process() ...
}

// Esportazione nativa CLAP + VST3
core_dsp::export_greg_plugin!(MyCompressor, "greg-my-compressor");
```
================================================================================

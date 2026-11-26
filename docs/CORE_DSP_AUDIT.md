# Core-DSP Technical Audit (`core-dsp/src/core_dsp_modules/`)

Audit tecnico in sola lettura della qualità del codice, della correttezza matematica/algoritmica, della sicurezza real-time e dell'aderenza agli standard industriali DSP per `core-dsp` (44 moduli, ~16.5k LOC; focus sui 18 moduli principali suddivisi in 4 gruppi logici, con verifica delle dipendenze dirette `filters.rs` e `delay_lines.rs`).

> **Contesto Build (`core-dsp/Cargo.toml`):** Rust Edition 2024; dipendenze principali `num-traits 0.2`, `realfft 3.5`, `rustfft 6.4`, `rubato 5.1`, `nih_plug` con feature `assert_process_allocs` abilitata. Alcuni moduli testati (`pitch_shifter.rs`, `dsp_graph.rs`, `spatial_3d.rs`, `loudness_metering.rs`) eseguono allocazioni heap nel path di elaborazione audio: in configurazione debug/test con `assert_process_allocs` attivo nei plugin `nih_plug`, tali chiamate provocano un panic immediato.

---

## Sintesi Esecutiva

| Gruppo | Stato Complessivo | Criticità P0 (Bloccanti) | Criticità P1 (Funzionali / Standard) | Opportunità P2 (CPU / SIMD) |
| :--- | :--- | :---: | :---: | :---: |
| **1. Filtri & Dinamica** | Parzialmente critico | 2 | 4 | 3 |
| **2. Modellazione Analogica & Non-Linearità** | Critico su ADAA / Factory / Preamp | 4 | 5 | 4 |
| **3. Resampling, Spazialità & Tempo** | Critico su Pitch / FDN / Room / Spatial | 7 | 4 | 3 |
| **4. Metering, Psicoacustica & Grafo** | Critico su BS.1770-4 / Weighting / Graph | 5 | 4 | 3 |

---

## Gruppo 1: Filtri & Dinamica
Moduli analizzati: `zdf_filters.rs`, `opto_compressor.rs`, `vari_mu.rs`, `sidechain_processor.rs` (con verifica incrociata su `filters.rs`).

### 1.1 Correttezza Matematica & Stabilità Numerica
- **`zdf_filters.rs` — Regressione TPT/ZDF in `ZdfSvf::process` (P0):**
  - A differenza di `SvfFilter::process` in `filters.rs:163-165` (che implementa correttamente Simper TPT: `v3 = input - ic2eq`, `v2 = ic2eq + a2 * ic1eq + a3 * v3`), `ZdfSvf::process` (`zdf_filters.rs:170, 187`) calcola `v3 = input - self.ic2eq - self.k * self.ic1eq` e `v2 = ic2eq + self.a3 * self.ic1eq + self.a3 * v3`. Sottraendo `k * ic1eq` da `v3` quando `a1` già include `k` e usando `a3` invece di `a2` su `ic1eq`, si alterano Q, pendenza e risposta in fase/ampiezza.
  - **Newton-Raphson errato (`zdf_filters.rs:172-184`):** Il loop risolve `(v3 * nl).tanh() / nl = input - ic2eq - k * ic1eq` invece dell'equazione implicita TPT con saturazione nello stato integratore/feedback (es. `v1` o `v2`). Di fatto cerca di invertire `tanh` (`atanh`), divergendone la derivata `d_sat -> 0` non appena `|input - ic2eq - k*ic1eq| >= 1/nl`. Inoltre, in modalità `Bell` (`zdf_filters.rs:128`), `update_coefficients` usa `k_bell = 1 / (q * bell_a)` per `a1..a3` ma `process` continua a usare `self.k` sulla riga 170.
- **`zdf_filters.rs` — `CombFilter::process` non usa il ritardo configurato (P0):**
  - `CombFilter::configure` (`zdf_filters.rs:390-393`) calcola `delay_samples` e lo valida, ma **non lo salva** nella struct. `CombFilter::process` (`zdf_filters.rs:421-427`) calcola `read_idx = DELAY - self.write_idx`, leggendo il buffer circolarmente **all'indietro** invece che a `write_idx - delay_samples`.
- **`zdf_filters.rs` — `IirCascade` & `NaturalPhaseEq` (P1):**
  - `IirCascade::configure_butterworth_low_pass` (`zdf_filters.rs:308`) imposta `q = 0.51 + (i * 0.02).min(0.95)`, che non corrisponde ai poli di Butterworth ($Q_i = 1 / (2 \cos(\frac{(2i+1)\pi}{4N}))$), e `process` itera su tutti gli `STAGES` anche se `stages < STAGES`, lasciando attivi stadi non configurati.
  - `NaturalPhaseEq::configure` (`zdf_filters.rs:460-489`) ignora `_shelf_freq`, `_shelf_gain_db`, `pivot_hz` e configura `ZdfSvf` a `sample_rate` (1x) pur eseguendolo dentro `Oversampler` a 2x (`zdf_filters.rs:503`), dimezzando la frequenza centrale effettiva del filtro Bell.
- **`opto_compressor.rs` — Transizione Attack -> Release su `OptoCell` (P1):**
  - Durante la fase di attacco (`opto_compressor.rs:146`), `self.release_start_db = target`. Se un transiente breve porta `target = 20 dB` mentre `reduction_db` sale solo a `2 dB` e poi `target` scende a `0 dB`, la condizione `self.reduction_db <= self.release_start_db * 0.5` (`opto_compressor.rs:186`) è già vera (`2 <= 10`), saltando completamente lo stadio di `fast_release`. `release_start_db` va catturato come `self.reduction_db` all'ingresso della fase di rilascio.
  - Assenza di `flush_denormal` su `reduction_db` e `program_memory` (con code di rilascio fino a 5 s, l'inviluppo decade inevitabilmente in area subnormale).
- **`vari_mu.rs` — Rilevamento inviluppo e soft-knee (P1):**
  - `VariMuCell::process` (`vari_mu.rs:143-151`) applica un attacco a 0.2 ms su `self.envelope` ma rilascio istantaneo (`self.envelope = detector_level`), per poi applicare un secondo polo di attacco a 0.2 ms su `gain_reduction_db` (`vari_mu.rs:163-165`). Su segnale audio grezzo rettificato `|x|`, ogni zero-crossing azzera `self.envelope`.
  - In `vari_mu_target_reduction_db` (`vari_mu.rs:221-233`), `over = over_threshold.max(0.0)` azzera `reduction` in tutto l'intervallo `[-knee, 0.0]`: la metà inferiore del ginocchio (`[-2, 0] dB`) risulta sempre `0.0`, generando una discontinuità di pendenza a `0 dB` invece di un raccordo parabolico su `[-knee, +knee]`.
  - Assenza di `flush_denormal` su `envelope` e `gain_reduction_db`.
- **`sidechain_processor.rs`:**
  - `SidechainProcessor::process_key` (`sidechain_processor.rs:140-164`) sanitizza `external` se `use_external`, ma non verifica `internal.is_finite()` quando `filter_enabled == false`, rischiando di propagare `NaN`/`Inf` dal canale interno.

### 1.2 Sicurezza Real-Time & Performance CPU
- Zero allocazioni heap in tutti e 4 i moduli del Gruppo 1.
- **Hotspot trascendentali per-sample (`P1`/`P2`):**
  - `OptoCell::process_with_character` (`opto_compressor.rs:139, 159, 215`) calcola `(-1.0 / (t * sr)).exp()` fino a **2 volte per campione** per parametri costanti (`memory_seconds`, `OPTO_*_FAST_RELEASE_SECONDS`).
  - `VariMuCell::process` (`vari_mu.rs:146, 155, 164, 187, 225`) esegue fino a **3 `exp()` + 1 `log10()` per campione**, nonostante `attack` e le 6 costanti `TimeConstant` dipendano solo da `sample_rate` e dalla posizione del selettore.
  - `ZdfSvf::process` (`zdf_filters.rs:175, 177`) calcola `(v3 * nl).tanh()` due volte per iterazione Newton (4 `tanh` per campione).
- **SIMD (`f64x4`) & Cache:** In `IirCascade<STAGES>`, la processazione per-sample seriale limita il parallelismo intra-canale, ma su bus multicanale (4/8 canali indipendenti) `SvfFilter`, `ZdfSvf`, `OptoCell` e `VariMuCell` possono essere raggruppati in SoA (`[f64; 4]`) per eseguire 4 canali in parallelo tramite SIMD `f64x4` senza dipendenze tra stati.

### 1.3 Aderenza agli Standard Industriali Audio
- `SvfFilter` (`filters.rs`) segue fedelmente la topologia TPT/ZDF di Zavalishin / Simper; `ZdfSvf` (`zdf_filters.rs`) invece se ne discosta per i bug citati su `v3`, `v2` e Newton-Raphson (per non-linearità TPT standard è necessario risolvere $f(v_1) = 0$ o $f(v_2) = 0$ con singolo `tanh` per iterazione).
- `SidechainProcessor` usa `Biquad` Direct Form II Transposed (`filters.rs:325-334`), adeguato per filtri statici, ma meno resiliente di `SvfFilter` TPT in caso di modulazione rapida della frequenza di sidechain.

---

## Gruppo 2: Modellazione Analogica & Non-Linearità
Moduli analizzati: `nonlinear_stages.rs`, `transformer_hysteresis.rs`, `component_models.rs`, `psu_sag.rs`, `cabinet_speaker.rs`.

### 2.1 Correttezza Matematica & Stabilità Numerica
- **`nonlinear_stages.rs` — Antiderivate ADAA1/ADAA2 matematicamente errate (P0):**
  - `adaa_antiderivative` (`nonlinear_stages.rs:362-404`):
    1. Per `SoftClip` e varianti `tanh`, implementa `x.abs().cosh().ln_1p() * x.signum()`. L'integrale esatto di $\tanh(x)$ è $\ln(\cosh(x))$ (funzione **pari**, mai moltiplicata per `signum(x)`!) e **non** $\ln(1 + \cosh(x))$ (`ln_1p`). Con `ln_1p * signum`, a $x=0$ la primitiva salta da $-\ln 2$ a $+\ln 2$ introducendo impulsi spuri ad ogni passaggio per lo zero, dimezza il guadagno per piccoli segnali ($\ln(1+\cosh x) \approx \ln 2 + x^2/4$) e inverte il segno sui campioni negativi. Inoltre `cosh(x)` va in overflow (`inf - inf = NaN`) per $|x| > 710$ (forma numericamente stabile: $|x| + \ln(1 + e^{-2|x|}) - \ln 2$).
    2. `Asymmetric`, `TubeTriode` e `TubePentode` integrano $\tanh(x+\text{bias})$ e il termine polinomiale separatamente anziché $\tanh(x+\text{bias}+c x^k)$, ignorando anche `- bias_tanh` e `norm` usati in `process_nonlinear`.
    3. `WaveFold` (`nonlinear_stages.rs:381-385`) restituisce lo waveshaper diretto anziché la sua antiderivata.
  - `adaa_antiderivative2` & `process_adaa2` (`nonlinear_stages.rs:409-453`):
    1. `0.5 * x * adaa_antiderivative(x, mode)` non è la seconda antiderivata $G(x) = \int_0^x F(u)\,du$.
    2. `(g_n - 2.0 * g_prev + g_prev2) / (d1 * d2 * drive)` usa la differenza finita a passo uniforme su nodi non uniformi $x_n, x_{n-1}, x_{n-2}$, violando la formula di Parker et al. (DAFx 2016): $\frac{2}{x_n - x_{n-2}}\left(\frac{G(x_n)-G(x_{n-1})}{x_n-x_{n-1}} - \frac{G(x_{n-1})-G(x_{n-2})}{x_{n-1}-x_{n-2}}\right)$.
- **`nonlinear_stages.rs` — `DynamicMirrorPreEmphasis` & `drive_with_unity_compensation` (P1):**
  - `de_emphasis` (`nonlinear_stages.rs:273-279`) applica un filtro FIR `x - (x - z1) * 0.5 * e` invece del polo IIR inverso $1 / ((1 + 0.5e) - 0.5e z^{-1})$: a `emphasis = 1.0` la cascata `pre -> de` azzera completamente Nyquist ($2 \cdot 0 = 0$).
  - `drive_with_unity_compensation` (`nonlinear_stages.rs:41-42`) divide per `drive.ln_1p()` quando `process_nonlinear` ha già diviso per `drive`, amplificando enormemente i piccoli drive (es. a `drive = 0.01`, guadagno $\approx 100\times$).
- **`transformer_hysteresis.rs` — Annullamento del feedback di isteresi (P1):**
  - Alle righe `99-103`: `self.flux = low * bass_drive + self.flux * hysteresis; core = ...; self.flux = core * (1.0 - hysteresis);`. Il guadagno di feedback effettivo tra un campione e il successivo è $h(1-h) \le 0.25$: per `hysteresis -> 0.985` il contributo di memoria magnetica collassa a `~0.0147` anziché aumentare. Inoltre manca `flush_denormal` su `flux`, `low_state` e `dc_state`.
- **`component_models.rs` — Bug su filtro Miller, Slew Rate e `DualStageProgramDependentRelease` (P0/P1):**
  - **Bug Miller in `TubeStage` e `SolidStateStage` (`component_models.rs:155-162, 279-292`) (P0):** Il filtro passa-basso `m_alpha` (8.5–20 kHz, pensato per l'effetto Miller sulle alte frequenze) viene applicato a `low` (la banda `< 200 Hz` uscita dal crossover `c_alpha`), mentre `high` (`sat - low`) viene sommato **senza alcun roll-off**!
  - **Slew limiting inattivo (`component_models.rs:201-216, 284-286`) (P1):** `slew_per_sec` è espresso in $V/s$ reali ($2\cdot 10^6 \dots 2.5\cdot 10^7$), producendo `max_delta = 41..520` per campione a 48 kHz su segnali normalizzati in `[-1, 1]`: il limiter non interviene mai.
  - **`DualStageProgramDependentRelease` non implementato (`component_models.rs:496-576`) (P0):** `state_fast`, `state_slow`, `c_fast`, `c_slow` e `sample_rate` sono campi morti; `process` non integra nel tempo ma restituisce `envelope / (1 + tc * over)`.
- **`psu_sag.rs` — Ripple non scalato per sample rate (P1):**
  - `self.ripple_phase += 0.01` (`psu_sag.rs:127`) è un incremento fisso in rad/campione (76.4 Hz a 48 kHz, 305.6 Hz a 192 kHz invece di 100 Hz); `pseudo_noise` (`psu_sag.rs:137`) è deterministico sull'inviluppo (diventa offset DC su toni stazionari). Manca `flush_denormal` su `envelope`.
- **`cabinet_speaker.rs` — Risonanza/HeadBump errati e `mic_preamp` stateless (P0/P1):**
  - **`CabinetSpeaker::mic_preamp` (`cabinet_speaker.rs:347-367`) (P0):** Istanzia `TubeStage::new(...)` e `create_transformer(...)` **ad ogni singolo campione**, azzerando ad ogni campione tutti gli stati di filtro/isteresi/DC e ricalcolando `.tan()` e `.exp()`.
  - **`CabinetResonance` (`cabinet_speaker.rs:72-109`) (P1):** Implementa un passa-basso a 1 polo con taglio `freq_hz / q` (`120 / 2.5 = 48 Hz`), non un risonatore passa-banda/Helmholtz a 2 poli.
  - **`HeadBumpShelf` (`cabinet_speaker.rs:131-145`) (P1):** Moltiplica `alpha` per `g` (instabile per `gain_db < 0` perché `alpha < 0`) e somma `self.alpha * hp` (boost sulle **alte frequenze**, non un low-shelf a 90 Hz!).

### 2.2 Sicurezza Real-Time & Performance CPU
- Zero allocazioni heap nel path audio (eccetto `render_impulse_response` offline in `cabinet_speaker.rs:309`).
- **Calcoli pesanti per campione:** `10.0_f64.powf(drive_db / 20.0)` è ricalcolato per campione in `TubeStage`, `SolidStateStage`, `NonlinearElementStage` e `SpeakerDriver`; `NonlinearElementStage` esegue due `.powf()` aggiuntivi; `TubeStage` esegue `abs().powf(p.p - 1.0)`; `MillerRollOff` esegue `.tan()` per campione.
- **Opportunità SIMD:** Le non-linearità polinomiali/razionali e i banchi di filtraggio per-canale in `MillerRollOff`/`AcCoupling` (già dimensionati a `[f64; 8]`) sono candidati diretti a due registri `f64x4`.

### 2.3 Aderenza agli Standard Industriali Audio
- Le saturazioni in `TransformerHysteresis`, `TubeStage`, `SolidStateStage` e `cone_breakup_soft_clip` sono prive di oversampling o ADAA funzionante, generando aliasing sulle armoniche alte per drive elevati.
- L'isteresi in `TransformerHysteresis` è una rete a singolo stato non lineare e non un modello Jiles-Atherton o Preisach dipendente dalla derivata del flusso $dH/dt$.

---

## Gruppo 3: Resampling, Spazialità & Tempo
Moduli analizzati: `resampler_bitdepth.rs`, `pitch_shifter.rs`, `fdn_reverb.rs`, `acoustic_room.rs`, `spatial_3d.rs` (con verifica di `delay_lines.rs`).

### 3.1 Correttezza Matematica & Stabilità Numerica
- **`pitch_shifter.rs` — Phase Vocoder non funzionante e Formant NaN (P0):**
  - `spectral_modify` (`pitch_shifter.rs:225-256`) modifica solo la fase scrivendo nello **stesso bin `i`**, senza effettuare né spostamento/interpolazione dei bin a `i * speed_ratio` né resampling temporale: non realizza pitch shifting reale.
  - `new_phase = prev + dphase * speed_ratio` usa la fase di analisi precedente `prev` invece della fase di sintesi accumulata `synth_phase[i]`, e ignora `_expected` (`pitch_shifter.rs:236`).
  - **Bug Formant (`pitch_shifter.rs:249-251`):** `envelope[i]` è calcolato sul log-spettro `ln(mags[i] + 1e-12)` (valori negativi quando `mag < 1`). Elevare `env < 0` a `formant_ratio` (`env.powf(blend)`) produce **`NaN`**, silenziando l'uscita (`L324`) ogni volta che `formant_ratio < 0.999`.
  - **Violazione COLA (`pitch_shifter.rs:28, 319`):** Finestra di analisi Hann + finestra di sintesi Hann con overlap al 50% (`DEFAULT_HOP = 1024`, `DEFAULT_WINDOW = 2048`): $\sin^4 + \cos^4$ oscilla tra `0.5` e `1.0` causando tremolo periodico di 6 dB (Hann² richiede 75% overlap, oppure analisi Hann + sintesi rettangolare al 50%).
  - `PolyphonicStudio` (`pitch_shifter.rs:259-271`) è di fatto un'identità che non implementa peak-picking né phase-locking verticale di Laroche-Dolson.
- **`fdn_reverb.rs` — Instabilità esponenziale della matrice Hadamard e taglio coda (P0):**
  - **Instabilità critica (`fdn_reverb.rs:106, 113-124`):** Il butterfly di Hadamard calcola $H_N x$ non normalizzato (guadagno in ampiezza $\sqrt{N}$, cioè `2.83x` per 8 canali e `4x` per 16 canali) e lo salva in `mix_buffer`. Al campione successivo, `feedback_gain * damped` (default `0.8`) viene reiniettato **senza divisione per $\sqrt{N}$**: guadagno d'anello = `0.8 * sqrt(8) = 2.263 > 1.0`, portando a **divergenza esponenziale** non appena trascorre la latenza delle linee di ritardo (~50 ms; i test passano solo perché si fermano a `2048` campioni = 42.6 ms).
  - **Taglio coda su silenzio (`fdn_reverb.rs:77-79`):** Se dopo un impulso `input == 0.0`, `mix_buffer` e `damping_states` sono ancora `< 1e-20` prima che il suono esca dalle delay line (50 ms), il fast-path restituisce `(0.0, 0.0)` troncando l'intera coda.
  - **LFO e Sample Rate (`fdn_reverb.rs:61-70, 87, 219-221`):** `AsymmetricLfo::advance` hardcoda `48.0 kHz`, moltiplica per `TAU` sia in `advance` che in `process_sample` (frequenza $2\pi\times$ più alta del nominale), e `set_sample_rate` non ricalcola `damping_coeff`. Inoltre i tempi di ritardo (`L104`) non sono coprimi in campioni (rischio risonanze metalliche).
- **`acoustic_room.rs` — Corruzione buffer `DelayLine`, troncamento coda e All-Pass errati (P0/P1):**
  - `process_mono` (`L304-305`) e `process_stereo` (`L336-337`) chiamano `self.early_delay_lines[i].process_sample(...)` **due volte per campione** sulla stessa `DelayLine`, avanzando il puntatore di scrittura 2 volte per campione (dimezzando i ritardi e sovrascrivendo L su R).
  - `if x == 0.0 { return (0.0, 0.0); }` (`L294, L326`) tronca istantaneamente tutte le riflessioni e la coda quando l'ingresso torna a zero.
  - `DiffusionAllPass::process` (`L71`) calcola `-coeff * x + delayed + coeff * delayed` (filtro comb feedforward con picco `1 + 2*coeff` invece di all-pass IIR); analogamente `DiffusionNetwork::process` (`L121-122`) usa `fb_delay` in feedforward senza feedback, risultando privo di coda riverberante tardiva.
  - `build_image_sources` (`L247-260`) include auto-riflessioni doppie sullo stesso piano (es. `x=0` poi `x=0`) e `images_l == images_r` (nessun ITD/ILD binaurale).
- **`spatial_3d.rs` — Distorsione ad onda quadra da auto-normalizzazione e bug HRTF/VBAP (P0):**
  - `Spatial3d::process_sample` (`L165`), `process_stereo` (`L174`), `process_ambisonic` (`L182, L185`) e `AmbisonicCoefficients::to_stereo` (`L68-69`) normalizzano ogni campione per `max(|L|, |R|)`. Questo trasforma qualsiasi segnale mono/binaurale in ampiezza piena `±1.0` (distorsione rettangolare estrema / +∞ dB su segnali deboli). Lo stesso difetto colpisce `AudioObjectPanner::process` (`L283-292`) quando `|inp| * sum_g > 1.0`.
  - `hrtf_for_direction` (`L37-45`): `az = azimuth.rem_euclid(2*PI)` è sempre $\ge 0$, quindi `if az < 0.0` (`L44`) è sempre falso; `itd_sec` è calcolato ma mai applicato (nessuna delay line); `ild` azzera il canale destro per `az >= 90°`.
  - `VbapState` usa una funzione peso `cos(az_diff).max(0.0)` con somma L1 anziché l'inversione matriciale VBAP per coppia/tripletta adiacente con legge a potenza costante ($\sqrt{g_1^2 + g_2^2} = 1$).
- **`resampler_bitdepth.rs`:**
  - La risposta in ampiezza del filtro half-band Blackman a 31 tap (~74 dB di attenuazione in banda oscura) è adeguata per 16-bit ma inferiore ai >96 dB richiesti in ambito mastering 24-bit. `TpdfDither` non supporta noise shaping.

### 3.2 Sicurezza Real-Time & Performance CPU
- **Violazioni Real-Time critiche (allocazioni heap in audio thread) (`P0`):**
  - `pitch_shifter.rs:195, 202, 210`: alloca `mags`, `logmag`, `envelope` (`Vec<f64>`) ad ogni frame FFT dentro `process_sample`.
  - `spatial_3d.rs:105-119, 163`: `VbapState::compute_weights` e `process_sample` allocano `Vec` ad ogni campione quando `binaural == false`.
- **Inefficienza CPU (`P2`):**
  - `FirResampler` (`resampler_bitdepth.rs:206-230`) esegue convoluzione piena a 31 tap anche sugli zeri di zero-stuffing e sui coefficienti pari nulli del filtro half-band, computando ~4× MAC superflui rispetto a una decomposizione polifase vera.
  - `AcousticRoom` alloca 30 `DelayLine` distinte anziché una singola linea di ritardo multi-tap (30× spreco di memoria e cache miss).

### 3.3 Aderenza agli Standard Industriali Audio
- `Oversampler` implementa correttamente il ritardo di gruppo dichiarato (15, 22, 25 campioni) verificato dai test, ma richiede upgrade a polifase SIMD e filtro Kaiser/Parks-McClellan per mastering.
- `PitchShifter`, `FdnReverb`, `AcousticRoom` e `Spatial3d` richiedono correzioni strutturali per aderire rispettivamente a Phase Vocoder Laroche-Dolson, FDN ortogonali normalizzate (Jot/Puckette), ISM 2° ordine con all-pass Schroeder reali e VBAP/HRTF con ITD frazionario.

---

## Gruppo 4: Metering, Psicoacustica & Grafo
Moduli analizzati: `loudness_metering.rs`, `psychoacoustics.rs`, `dsp_graph.rs`, `routing_matrix.rs`.

### 4.1 Correttezza Matematica & Stabilità Numerica
- **`loudness_metering.rs` — Non conformità ITU-R BS.1770-4 / EBU R128 (P0):**
  1. **Stato K-Weighting condiviso tra L e R (`L212, L273-274`):** `LoudnessEngine` possiede una sola istanza `k_weight: KWeightFilter` e la invoca alternativamente su `l` e `r`, corrompendo i registri `x1, x2, y1, y2` tra i due canali.
  2. **Somma canali errata (`L275-276`):** Calcola `((kl + kr) * 0.5)^2` invece di `kl^2 + kr^2`. Segnali stereo in controfase (`L = -R`) si annullano (`-70 LUFS`), il mono coerente (`L = R`) risulta `-3 dB` sotto specifica e lo stereo incorrelato `-6 dB` sotto specifica.
  3. **Finestre Momentary (400 ms) e Short-Term (3 s) non scorrevoli (`L280-281, L299-308`):** `ms_sum_sq` e `st_sum_sq` accumulano indefinitamente senza buffer ad anello/sottoblocchi da 100 ms, e `add_block` non viene mai invocato da `process_sample`.
  4. **True-Peak fittizio (`L198-202`):** Non esegue oversampling 4× (BS.1770-4 Annex 2) ma moltiplica semplicemente il picco campione per `1.25` (+1.94 dB fisso, errato sia su DC/quadre che su picchi inter-sample > +2 dBTP).
  5. **Integrated & LRA (`L241, L310-368`):** `int_blocks` è limitato a 120 blocchi (48 s, sovrascrive la storia invece di usare istogramma o capacità configurabile) e `lra()` ordina blocchi da 400 ms senza il gate relativo a `-20 LU` (EBU Tech 3342 richiede blocchi da 3 s e gate `-70 LUFS` / `-20 LU`).
- **`psychoacoustics.rs` — Normalizzazione `WeightingFilter` errata e curve non standard (P0/P1):**
  - **Bug normalizzazione & risposta in frequenza (`L233-245, L257-266`) (P0):** Sia `magnitude_db` che `compute_coefficients` valutano $H(e^{j\omega})$ sostituendo $z^{-1} \to \cos(\omega)$ anziché $e^{-j\omega}$ (perdendo tutta la componente immaginaria $\sin(\omega)$), e poi applicano `inv = 1 / gain1k` a **entrambi** gli stadi `stage1` e `stage2`, applicando una correzione doppia (`1 / gain1k^2`).
  - **Curve A/B/C/D/ITU-R 468 non conformi (`L273-277`) (P1):** Non implementano i poli/zeri IEC 61672-1 o ITU-R BS.468-4 (es. la curva A usa un singolo HPF 20 Hz + peak +1.3 dB @ 2.5 kHz, dando ~`-0.3 dB` a 100 Hz invece dei `-19.1 dB` normativi).
  - **ISO 226:2003 (`L96-102, L133`) (P1):** `sound_pressure_level_db` omette il termine `+ 94.0` della norma (innocuo in `gain_db` differenziale, errato se usato come SPL assoluto) e `interpolate_iso226` interpola linearmente in Hz invece che su scala logaritmica $\log(f)$.
- **`dsp_graph.rs` — Routing archi scollegato, cicli e smoothing errati (P0):**
  - **Archi mai scritti (`L370-399`):** `CompiledAudioGraph::process` non scrive mai nei `self.edge_buffers`: ogni nodo scrive direttamente su `outputs` e legge da `in_staging` popolato solo sul canale 0 (`L384`) da `edge_buffers` sempre nulli; inoltre `_inputs` della funzione `process` è ignorato.
  - **Cicli con `FeedbackDelayZ1Node` rifiutati (`L314-318`):** L'ordinamento topologico di Kahn non esclude gli archi uscenti da `FeedbackDelayZ1Node`, rigettando qualsiasi feedback loop con `CycleDetected`.
  - **PDC incompleto (`L325`):** `graph_latency` considera solo i nodi terminali senza propagare le latenze lungo i rami e senza compensazione sui rami paralleli.
  - **Overshoot dello smoothing (`L75-85, L161-172`):** `GainPanNode` e `ParallelDryWetNode` incrementano `cur_gain`/`cur_mix` all'interno del loop sui canali (raddoppiando la rampa in stereo) senza clamp/target-stop al termine di `smooth_samples`, provocando overshoot lineare quando `num_samples > smooth_samples`.
- **`routing_matrix.rs`:**
  - Matrice M/S ortonormale (`1/sqrt(2)`) corretta (`L291-310`) e topologia LFE-isolata ben testata (`L95-111`). Unica limitazione minore (`P2`): `RoutingMatrix::encode_input`/`decode_output` gestiscono `MidSide` solo per esattamente 2 canali (a differenza di `route_channels` che usa `as_chunks_mut::<2>()`).

### 4.2 Sicurezza Real-Time & Performance CPU
- **Violazioni Real-Time (`P0`):**
  - `dsp_graph.rs:391`: `Vec::with_capacity` allocato dentro `CompiledAudioGraph::process()` per ogni nodo eseguito.
  - `loudness_metering.rs:260, 263, 314, 355`: `reset()`, `integrated_lufs()` e `lra()` allocano `Vec`; `GoniometerRing::snapshot` alloca 2 `Vec` (accettabile solo se chiamato da UI thread).
- **Denormali & SIMD (`P2`):** `KWeightHighShelf`, `KWeightHighPass` (`loudness_metering.rs`) e `BiquadState` (`psychoacoustics.rs`) non eseguono `flush_denormal`. Le operazioni per-blocco su `AudioEdgeBuffer` e `GainPanNode`/`SummerMixerNode` in `dsp_graph.rs` sono ottimi candidati per vettorizzazione SIMD `f64x4` su frame allineati.

---

## Piano di Miglioramento Prioritizzato

### Priorità P0 — Critico (Bug Matematici Bloccanti & Violazioni Real-Time)

1. **Eliminare tutte le allocazioni heap nei path audio (`assert_process_allocs`):**
   - `pitch_shifter.rs`: pre-allocare `mags`, `logmag`, `envelope` (e `synth_phase`) dentro `PitchShifter` in `new()`/`initialize()`.
   - `spatial_3d.rs`: sostituire `Vec` temporanei in `VbapState::compute_weights`/`process_sample` con buffer pre-allocati (`self.weights`, `self.accumulated`) e API a slice (`&[f64]`).
   - `dsp_graph.rs`: pre-allocare `in_slices` (o usare array/buffer di indici pre-allocato `Vec<usize>` fuori dal thread audio) in `CompiledAudioGraph`.
   - `loudness_metering.rs`: sostituire `int_blocks` dinamico/allocazioni in `integrated_lufs()`/`lra()`/`reset()` con ring buffer/istogramma pre-allocato (risoluzione 0.1 LU per BS.1770-4 senza `Vec::sort` in audio thread).
2. **Riparare i core matematici di filtraggio e saturazione:**
   - `zdf_filters.rs`: allineare `ZdfSvf::process` alle equazioni TPT esatte di `SvfFilter` (`v3 = input - ic2eq`, `v2 = ic2eq + a2 * ic1eq + a3 * v3`) e riformulare Newton-Raphson sull'equazione implicita di feedback (`v1` o `v2` saturato), usando `k_bell` in modalità Bell; memorizzare `delay_samples` in `CombFilter` e leggere `(write_idx + DELAY - delay_samples) % DELAY`.
   - `nonlinear_stages.rs`: correggere `adaa_antiderivative` ($\ln(\cosh x) = |x| + \ln(1 + e^{-2|x|}) - \ln 2$, senza `* signum` e senza `ln_1p`), implementare $G(x)$ analitica o per-tratti e la differenza divisa non-uniforme di Parker et al. in `process_adaa2`.
   - `component_models.rs`: applicare `m_alpha` (Miller) ad `high` (o all'uscita totale `sat`) in `TubeStage` e `SolidStateStage`; completare `DualStageProgramDependentRelease` con reale integrazione RC dual-stage su `state_fast`/`state_slow` (o deprecarlo).
   - `cabinet_speaker.rs`: rendere persistenti `TubeStage` e `TransformerHysteresis` come campi di `CabinetSpeaker` anziché ricrearli per campione in `mic_preamp`.
3. **Riparare motori temporali/spaziali, metering e grafo:**
   - `fdn_reverb.rs`: normalizzare il feedback Hadamard per `1.0 / (mix_n as f64).sqrt()`, rimuovere il fast-path di silenzio che taglia le code in transito nelle delay line, correggere la scala tempo/LFO su `self.sample_rate`.
   - `acoustic_room.rs`: condividere una singola `DelayLine` multi-tap tramite `read_sample` per i riflessi precoci, rimuovere il taglio `if x == 0.0`, correggere `DiffusionAllPass` in vero all-pass IIR (`y[n] = -g*x[n] + x[n-d] + g*y[n-d]`) e chiudere il feedback di `DiffusionNetwork`.
   - `pitch_shifter.rs`: implementare reale traslazione/interpolazione spettrale (o resampling + phase vocoder con `synth_phase` accumulato), sostituire `env.powf(blend)` su log-magnitudini negative con differenza/esponenziale lineare in dB (`exp(log_env)`), passare a overlap 75% (`hop = window / 4`) per finestre Hann².
   - `spatial_3d.rs`: rimuovere la normalizzazione per-sample `left / max_out` (che genera onda quadra), sistemare il segno `az` in `hrtf_for_direction` (`[-PI, PI]`), applicare davvero `itd_sec` tramite delay frazionario.
   - `loudness_metering.rs`: usare due istanze indipendenti di `KWeightFilter` (`k_weight_l`, `k_weight_r`), sommare le potenze `kl*kl + kr*kr`, implementare finestre scorrevoli a sottoblocchi da 100 ms (400 ms / 3 s) e vero True-Peak 4× FIR BS.1770-4.
   - `psychoacoustics.rs`: valutare $|H(e^{j\omega})|$ complesso completo in `compute_coefficients` e `magnitude_db`, e scalare solo `stage1` (non entrambi gli stadi) per `1 / gain1k`.
   - `dsp_graph.rs`: collegare scrittura/lettura multi-canale su `edge_buffers` e `_inputs`, escludere gli archi da `FeedbackDelayZ1Node` dal rilevamento cicli in `compile()`, sistemare la rampa `GainPanNode`/`ParallelDryWetNode` fuori dal loop canali con clamp a fine rampa.

### Priorità P1 — Funzionale & Aderenza agli Standard

1. **Precisione filtri e curve standard:**
   - `zdf_filters.rs`: implementare i veri fattori Q di Butterworth in `IirCascade` ed eseguire solo `stages` attivi; configurare `NaturalPhaseEq` a `sample_rate * 2.0`.
   - `psychoacoustics.rs`: sostituire le approssimazioni a 2 biquad delle curve `A`, `B`, `C`, `D`, `ItuR468` con cascate conformi a IEC 61672-1 / ITU-R BS.468-4; usare interpolazione log-frequenza in `interpolate_iso226` e aggiungere `+ 94.0` in `sound_pressure_level_db`.
   - `loudness_metering.rs`: implementare LRA secondo EBU Tech 3342 su blocchi da 3 s con doppio gate (`-70 LUFS` assoluto, `-20 LU` relativo).
2. **Modelli fisici e dinamica:**
   - `opto_compressor.rs`: catturare `release_start_db = self.reduction_db` alla transizione Attack->Release e aggiungere `flush_denormal`.
   - `vari_mu.rs`: correggere il soft-knee su `[-knee, +knee]` (`(over_threshold + knee)^2 / (4 * knee)`), separare o chiarire il detector RMS/peak rispetto al polo di griglia da 0.2 ms, aggiungere `flush_denormal`.
   - `transformer_hysteresis.rs` & `psu_sag.rs`: correggere il feedback dell'isteresi (`1 - h` annulla il feedback ad alto `h`), scalare `ripple_phase` per `sample_rate` (100 Hz costanti), aggiungere `flush_denormal`.
   - `cabinet_speaker.rs`: sostituire `CabinetResonance` e `HeadBumpShelf` con un vero filtro passa-banda 2-poli e un low-shelf TPT/biquad corretto.
   - `nonlinear_stages.rs`: correggere `DynamicMirrorPreEmphasis::de_emphasis` con il polo IIR inverso e sistemare la compensazione di guadagno in `drive_with_unity_compensation`.
   - `dsp_graph.rs`: implementare calcolo PDC su cammino critico + linee di ritardo di compensazione sui rami paralleli.

### Priorità P2 — Ottimizzazione CPU, Cache-Locality & SIMD (`f64x4`)

1. **Precalcolo coefficienti fuori dal loop per-sample:**
   - `OptoCell` e `VariMuCell`: precalcolare `attack_coeff`, `memory_coeff` e i coefficienti delle `TimeConstant` statiche in `initialize()` (eliminando fino a 4 chiamate `exp()`/`log10()` per campione).
   - `TubeStage`, `SolidStateStage`, `NonlinearElementStage`, `SpeakerDriver`: passare `drive_lin` pre-convertito (o aggiornare solo al cambio di `drive_db` con smoothing) evitando `10.0_f64.powf(drive_db / 20.0)` ad ogni campione.
2. **Decomposizione polifase & SIMD `f64x4` su `Oversampler` (`resampler_bitdepth.rs`):**
   - Sostituire la convoluzione diretta a 31 tap (`FirResampler::push` su zeri) con filtro polifase half-band a soli tap dispari + ramo passa-tutto centrale (`0.5 * z^{-7}`): riduzione teorica ~4× delle operazioni MAC, vettorizzabile su 4/8 canali in parallelo (`f64x4`).
   - Valutare coefficienti Kaiser/Equiripple ($\ge 96$ dB stopband) per il profilo mastering 24-bit.
3. **Vettorizzazione SoA e cache-locality:**
   - Riorganizzare i banchi multi-canale (`SvfFilter`, `ZdfSvf`, `MillerRollOff`, `AcCoupling`, `WeightingFilter`) in layout SoA (`[f64; 4]` / `[f64; 8]`) per consentire auto-vettorizzazione AVX2/NEON o SIMD esplicito su tracce stereo/surround/7.1.
   - In `acoustic_room.rs`, sostituire `Vec<DelayLine>` (30 buffer separati) con un unico ring buffer contiguo + array fisso `[f64; 30]` di offset per massimizzare la località in L1 cache.
   - In `pitch_shifter.rs`, sostituire il boxcar $O(N^2)$ dell'inviluppo con media mobile a prefix-sum $O(N)$ (o FFT/IFFT cepstrale reale sfruttando i piani `realfft` già allocati).

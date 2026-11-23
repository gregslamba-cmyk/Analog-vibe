# Riferimenti Hardware e Specifiche di Rimodellazione Circuitale

Ultimo controllo: 2026-11-23

## 1. Regole Ingegneristiche di Rimodellazione (`core-dsp`)

1. **Calibrazione Livelli (`0 VU = -18 dBFS`):** Tutti gli stadi analogici interni devono essere riferiti a `0 VU = +4 dBu = -18.0 dBFS` (headroom nominale `18 dB` prima dello `0 dBFS`).
2. **Ordine Fisico degli Stadi:** Il segnale audio deve attraversare gli stadi nell'ordine esatto dello schema elettrico reale (es. *Input Transformer → Attenuator/Gain Element → Active Amplification Stage → Output Transformer*), mai con un saturatore generico messo a valle.
3. **Colorazione a Riposo (Unity / Zero-Action):** Se l'hardware originale non ha un vero *hard-wire bypass* e l'audio attraversa trasformatori e stadi attivi anche con controllo `Gain Reduction = 0 dB` (es. LA-2A, Pultec, Neve 1073), la colorazione passiva/attiva deve restare attiva (disattivabile solo dal pulsante `Bypass` del plugin).
4. **Separazione Estensioni Moderne:** Controlli come `Dry/Wet Mix`, `External Sidechain`, `Sidechain HPF/LPF`, `Stereo Link` su macchine mono e `Oversampling` sono estensioni del plugin e non devono alterare la risposta quando impostati a riposo.
5. **Revisioni Hardware Distinte:** Ogni revisione selezionabile nel plugin deve commutare realmente i modelli di componenti (`component_models.rs`), le costanti temporali e le curve di trasferimento.

---

## 2. Compressori & Limiter — FET e Elettro-Ottici

### `greg_76` — UREI / Universal Audio 1176 (Rev A / D / E / F)
* **Flusso del Segnale Reale:** `Input Attenuator` → `Input Transformer` → `JFET Shunt Gain Reduction (verso massa)` → `Preamp Stage` → `Line Amp Output Stage` → `Output Transformer`. (Sidechain feedback prelevata dopo lo stadio d'uscita).
* **Meccanica di Controllo & Curve:**
  * **Soglia Fissa & Ratio-Dependent Threshold:** Non esiste manopola `Threshold`. L'attenuatore `Input` spinge il segnale contro la soglia fissa del rilevatore FET. Cambiando `Ratio` (`4:1`, `8:1`, `12:1`, `20:1`), la tensione di bias del gate del JFET cambia, **alzando progressivamente la soglia effettiva** (a `4:1` inizia a comprimere molto prima che a `20:1`; il ginocchio diventa via via più netto).
  * **Tempi Inversi (Senso Orario = Più Veloce):**
    * `Attack`: continuo da `800 µs` (tutto antiorario) a `20 µs` (tutto orario).
    * `Release`: continuo da `1.1 s` (tutto antiorario) a `50 ms` (tutto orario), con recupero leggermente dipendente dalla durata del picco.
  * **Modalità "All-Buttons In" (British Mode):** Premendo tutti e 4 i pulsanti ratio contemporaneamente si cortocircuitano le resistenze del partitore di bias: il rapporto sale oltre `20:1`, il ginocchio diventa brutale, l'attacco mostra una caratteristica esitazione iniziale (effetto *suck/pump* sui transienti) e la distorsione armonica aumenta sensibilmente.
* **Mappatura Componenti `core-dsp` per Revisione:**
  * **Rev A ("Bluestripe", 1967):** Trasformatore d'ingresso `UtcO12`, JFET `Fet2n5457` con bias più caldo e maggiore 2ª/3ª armonica, stadio finale Class A (`UtcA10`).
  * **Rev D / E (LN - Low Noise, 1970–73):** Circuito LN a basso rumore con JFET `Fet2n5457` selezionato, trasformatore d'uscita `UtcA10`, suono più pulito e controllato rispetto alla Rev A.
  * **Rev F (1976):** Stadio d'uscita push-pull **Class AB** (sostituisce il Class A della Rev D/E), maggiore headroom (+24 dBm), slew-rate più veloce e minor calore sulle mediobasse.

### `greg_a2a` — Teletronix LA-2A Leveling Amplifier
* **Flusso del Segnale Reale:** `Input Transformer` → `T4B Photoresistor Shunt Attenuator` → `12AX7 Voltage Amp` → `12AT7 White Cathode Follower Output Stage` → `Output Transformer`. (Sidechain alimentata da un secondo stadio `12AX7` + `6AQ5` pentodo che pilota il pannello elettroluminescente della cella `T4B`).
* **Meccanica di Controllo & Curve:**
  * **Controlli Originali:** Solo `Peak Reduction` (guadagno sidechain) e `Gain` (makeup post-attenuatore), più selettore `Compress` (~3:1 soft-knee) vs `Limit` (~10:1+ graduale). Nessun controllo manuale Attack/Release.
  * **Curva Opto `TeletronixT4b` (Memoria Fotoresistiva):**
    * *Attack:* ~`10 ms` medi, leggermente dipendente dalla frequenza e dal livello.
    * *Release Bifasico:* Il primo ~`50%` della gain reduction recupera rapidamente in `40–80 ms`; il restante `50%` decresce lentamente da `0.5 s` fino a `5–15 s` in funzione della durata e dell'intensità della luminanza accumulata dal pannello EL (*effetto memoria*).
* **Mappatura Componenti `core-dsp`:**
  * Trasformatore d'ingresso `UtcHa100x` (revisione Teletronix originale) / `UtcA10` (uscita), valvole `Ecc83_12ax7` + `Ecc81_12at7` con Miller roll-off, cella opto `TeletronixT4b`.

### `greg_a3a` — UREI LA-3A Audio Leveler
* **Flusso del Segnale Reale:** `Input Attenuator` → `T4B Photoresistor Shunt Attenuator` → `Solid-State Class A Discrete Amplifier` → `Output Transformer`.
* **Differenze Circuitali Critiche rispetto all'LA-2A:** Non è un LA-2A senza valvole. L'amplificatore a stato solido ha una risposta transitoria più secca e brillante sulle medio-alte (`2–5 kHz`), minore headroom prima del clipping rispetto alle valvole ad alta tensione dell'LA-2A e un attacco percettivo più rapido (`~1.5–3.5 ms`), pur mantenendo il rilascio bifasico della cella `T4B`.
* **Mappatura Componenti `core-dsp`:** Stadio discreto `Neve2n3055_Bc108` / Classe A, cella opto `TeletronixT4b`, trasformatore d'uscita `UtcA10`.

## 4. Equalizzatori e Preamplificatori Standalone

### `greg_pultec` — Pulse Techniques EQP-1A / MEQ-5
* **Flusso del Segnale Reale:** `Input Transformer (UtcHa100x)` → `Passive RLC Attenuation/Boost Network (PermalloyCore inductors)` → `Push-Pull Tube Makeup Amp (Ecc83_12ax7 + Ecc82_12au7)` → `Output Transformer (UtcA10)`.
* **Comportamento Circuitale Critico:**
  * **Interazione "Pultec Trick":** `Low Boost` e `Low Atten` (`20, 30, 60, 100 Hz`) agiscono sulla stessa rete passiva con frequenze d'angolo leggermente sfalsate; alzandoli simultaneamente non si annullano, ma creano un boost profondo sulle sub-basse seguito da un dip stretto intorno a `150–300 Hz`.
  * **High Boost & Bandwidth:** `Bandwidth` (`Sharp ↔ Broad`) cambia contemporaneamente sia il fattore `Q` sia l'ampiezza effettiva del picco (`3, 4, 5, 8, 10, 12, 16 kHz`), mentre `High Atten` (`5, 10, 20 kHz`) è uno shelf passivo separato a valle.
  * **Colorazione Passiva a Riposo:** Anche con boost e attenuazione a `0`, il segnale attraversa trasformatori, induttori e valvole.

### `greg_neve_pre` & `greg_neve_eq` — Neve 1073
* **Flusso del Segnale Reale:** `Input Transformer (MarinairT1444)` → `Class A Discrete Gain Stages (Neve2n3055_Bc108)` → `Inductor/Cap Passive-Active Feedback EQ` → `Single-Ended Class A 2N3055 Output Stage (DC-biased)` → `Output Transformer (MarinairLo1166)`.
* **Curve & Frequenze Esatte (1073):**
  * `HPF`: `18 dB/oct` (`50, 80, 160, 300 Hz`).
  * `LF Shelf`: `35, 60, 110, 220 Hz` (`±16 dB`), pendenza morbida con leggero *overshoot* analogico.
  * `MF Bell (Induttore)`: `0.36, 0.7, 1.6, 3.2, 4.8, 7.2 kHz` (`±18 dB`), Q fisso musicale che si stringe leggermente agli estremi di guadagno.
  * `HF Shelf`: fisso a `12 kHz` (`±16 dB`).

### `greg_api_pre` — API 512c
* **Flusso del Segnale Reale:** `Input Transformer (RE-115K)` → `Pad -20 dB` → `Op-Amp Discreto Api2520` → `Output Transformer (Api2623)`.
* **Comportamento:** Risposta transitoria ultra-veloce, basso stretto e autoritario, saturazione armonica progressiva dell'op-amp `2520` e del nucleo `2623` al crescere del guadagno mic (`34–65 dB`) o Hi-Z (`14–45 dB`).

### `greg_manley_eq` — Manley Massive Passive
* **Flusso del Segnale Reale:** `Input Buffer` → `4-Band Overlapping Passive LC Network (PermalloyCore inductors)` → `All-Tube Differential Makeup Amp (Ecc88_6dj8 / 5751)` → `Output Transformer`.
* **Comportamento Circuitale:** Le 4 bande (`22 Hz .. 27 kHz`) sono in parallelo/interazione passiva; in modalità `Shelf`, gli induttori generano un caratteristico *overshoot/dip* opposto prima della frequenza di taglio; i filtri `HPF` e `LPF` passivi a `18 dB/oct` hanno una leggera risonanza al ginocchio.

---

## 5. Channel Strip di Console (15 Modelli Distinti)

Ogni modello mantiene architettura, componenti attivi e curve EQ proprie (non condividere mai la stessa curva EQ tra serie diverse):

### Famiglia Neve
1. **`greg_neve_1073_strip` (Neve 1073):** Preamp Class A (`Neve2n3055_Bc108` + `MarinairLo1166`), 3 bande EQ (HF fisso `12 kHz`, MF a induttore, LF shelf) + HPF `18 dB/oct`.
2. **`greg_neve_1081_strip` (Neve 1081, 1972):** Architettura **Class A/B** + `Marinair`, EQ **4 bande** (2 bande medie parametriche con pulsante `Hi-Q`, LF e HF commutabili `Shelf / Bell`) + filtri **HPF e LPF** `18 dB/oct`.
3. **`greg_neve_1084_strip` (Neve 1084):** Evoluzione Class A del 1073: HF selezionabile a **3 frequenze (`10, 12, 16 kHz`)**, pulsante `Hi-Q` sulla banda media e aggiunta del filtro **LPF** (`10, 14, 18 kHz` a `18 dB/oct`).
4. **`greg_neve_88rs_strip` (Neve 88RS):** Console moderna ad altissimo headroom (`+26 dBu`), filtri HPF/LPF continui a pendenza variabile, EQ 4 bande completamente parametrico + sezione dinamica VCA (`That2180`) con Gate/Expander e Compressore/Limiter integrati.

### Famiglia SSL (Solid State Logic)
5. **`greg_ssl_4000e_strip` (SSL 4000 E):** Op-amp `Ne5534`, VCA `Dbx202BlackCan`, EQ selezionabile tra **rev. `02` ("Brown Knob")** (curva musicale più morbida) e **rev. `242` ("Black Knob")** (Q costante e chirurgia più decisa, filtri HPF `18 dB/oct` e LPF `12 dB/oct`) + dinamica VCA E-Series.
6. **`greg_ssl_4000g_strip` (SSL 4000 G):** Op-amp `Ne5534`, VCA `Dbx202GoldCan`, EQ **rev. `292` ("Pink Knob")** con **Proportional Q** (la campana si stringe al crescere del boost/cut) e curve shelf con leggero *undershoot/overshoot* opposto + dinamica VCA G-Series.
7. **`greg_ssl_9000j_strip` (SSL 9000 J):** Architettura **SuperAnalogue** (accoppiamento in DC privo di condensatori elettrolitici sul percorso audio, banda estesa `5 Hz – 100 kHz`, distorsione quasi nulla, transienti cristallini e sub-basse profondissime) + switch E/G sulla curva EQ.

### Famiglia API
8. **`greg_api_vision_strip` (API Vision):** Catena discreta completa: Preamp `212L` (`Api2520` + `Api2623`), filtri `215L`, compressore `225L` + gate/expander `235L`, EQ selezionabile tra `550L` (4 bande a **Proportional Q** con frequenze discrete) e `560L` (EQ grafico 10 bande a Q proporzionale).
9. **`greg_api_legacy_plus_strip` (API Legacy Plus):** Console discreta `Api2520` + `Api2623` con EQ `550L` Proportional Q e bus routing analogico classico.

### Famiglia Harrison, Trident, AMEK & MCI
10. **`greg_harrison_32c_strip` (Harrison 32C):** EQ 4 bande a Q proporzionale con curve morbide e musicali + celebri filtri **HPF (`25 Hz – 3.1 kHz`) e LPF (`160 Hz – 20 kHz`) continui a `12 dB/oct` con caratteristica risonanza al ginocchio**.
11. **`greg_harrison_mixbus_strip` (Harrison Mixbus):** Profilo DSP console virtuale (saturation drive progressivo, EQ 3 bande + HPF, compressore/leveler integrato).
12. **`greg_trident_a_range_strip` (Trident A-Range):** Console discreta vintage a induttori (`PermalloyCore`), EQ 4 bande con interazione musicale tra le bande, filtri passa-alto/passa-basso a scatti e saturazione ricca dei trasformatori.
13. **`greg_trident_80b_strip` (Trident Series 80B):** Progetto senza induttori basato su op-amp (`Tl072` / `Ne5534`), EQ 4 bande con medie swept e alte fisse a `12 kHz`, risposta aperta e brillante.
14. **`greg_amek_9098_strip` (AMEK 9098):** Design Rupert Neve anni '90 ad altissima linearità con controlli **Glow** (generazione controllata di armoniche pari sulle basse) e **Sheen** (enfasi ariosa sulle alte) + EQ parametrico ad alta precisione.
15. **`greg_mci_jh500_strip` (MCI JH-500):** Console americana anni '70: op-amp discreti/integrati con trasformatore d'uscita, EQ 3/4 bande con medie incisive e punch rock.

## 4. Equalizzatori e Preamplificatori Standalone

### `greg_pultec` — Pulse Techniques EQP-1A / MEQ-5
* **Flusso del Segnale Reale:** `Input Transformer (UtcHa100x)` → `Passive RLC Attenuation/Boost Network (PermalloyCore inductors)` → `Push-Pull Tube Makeup Amp (Ecc83_12ax7 + Ecc82_12au7)` → `Output Transformer (UtcA10)`.
* **Comportamento Circuitale Critico:**
  * **Interazione "Pultec Trick":** `Low Boost` e `Low Atten` (`20, 30, 60, 100 Hz`) agiscono sulla stessa rete passiva con frequenze d'angolo leggermente sfalsate; alzandoli simultaneamente non si annullano, ma creano un boost profondo sulle sub-basse seguito da un dip stretto intorno a `150–300 Hz`.
  * **High Boost & Bandwidth:** `Bandwidth` (`Sharp ↔ Broad`) cambia contemporaneamente sia il fattore `Q` sia l'ampiezza effettiva del picco (`3, 4, 5, 8, 10, 12, 16 kHz`), mentre `High Atten` (`5, 10, 20 kHz`) è uno shelf passivo separato a valle.
  * **Colorazione Passiva a Riposo:** Anche con boost e attenuazione a `0`, il segnale attraversa trasformatori, induttori e valvole.

### `greg_neve_pre` & `greg_neve_eq` — Neve 1073
* **Flusso del Segnale Reale:** `Input Transformer (MarinairT1444)` → `Class A Discrete Gain Stages (Neve2n3055_Bc108)` → `Inductor/Cap Passive-Active Feedback EQ` → `Single-Ended Class A 2N3055 Output Stage (DC-biased)` → `Output Transformer (MarinairLo1166)`.
* **Curve & Frequenze Esatte (1073):**
  * `HPF`: `18 dB/oct` (`50, 80, 160, 300 Hz`).
  * `LF Shelf`: `35, 60, 110, 220 Hz` (`±16 dB`), pendenza morbida con leggero *overshoot* analogico.
  * `MF Bell (Induttore)`: `0.36, 0.7, 1.6, 3.2, 4.8, 7.2 kHz` (`±18 dB`), Q fisso musicale che si stringe leggermente agli estremi di guadagno.
  * `HF Shelf`: fisso a `12 kHz` (`±16 dB`).

### `greg_api_pre` — API 512c
* **Flusso del Segnale Reale:** `Input Transformer (RE-115K)` → `Pad -20 dB` → `Op-Amp Discreto Api2520` → `Output Transformer (Api2623)`.
* **Comportamento:** Risposta transitoria ultra-veloce, basso stretto e autoritario, saturazione armonica progressiva dell'op-amp `2520` e del nucleo `2623` al crescere del guadagno mic (`34–65 dB`) o Hi-Z (`14–45 dB`).

### `greg_manley_eq` — Manley Massive Passive
* **Flusso del Segnale Reale:** `Input Buffer` → `4-Band Overlapping Passive LC Network (PermalloyCore inductors)` → `All-Tube Differential Makeup Amp (Ecc88_6dj8 / 5751)` → `Output Transformer`.
* **Comportamento Circuitale:** Le 4 bande (`22 Hz .. 27 kHz`) sono in parallelo/interazione passiva; in modalità `Shelf`, gli induttori generano un caratteristico *overshoot/dip* opposto prima della frequenza di taglio; i filtri `HPF` e `LPF` passivi a `18 dB/oct` hanno una leggera risonanza al ginocchio.

---

## 5. Channel Strip di Console (15 Modelli Distinti)

Ogni modello mantiene architettura, componenti attivi e curve EQ proprie (non condividere mai la stessa curva EQ tra serie diverse):

### Famiglia Neve
1. **`greg_neve_1073_strip` (Neve 1073):** Preamp Class A (`Neve2n3055_Bc108` + `MarinairLo1166`), 3 bande EQ (HF fisso `12 kHz`, MF a induttore, LF shelf) + HPF `18 dB/oct`.
2. **`greg_neve_1081_strip` (Neve 1081, 1972):** Architettura **Class A/B** + `Marinair`, EQ **4 bande** (2 bande medie parametriche con pulsante `Hi-Q`, LF e HF commutabili `Shelf / Bell`) + filtri **HPF e LPF** `18 dB/oct`.
3. **`greg_neve_1084_strip` (Neve 1084):** Evoluzione Class A del 1073: HF selezionabile a **3 frequenze (`10, 12, 16 kHz`)**, pulsante `Hi-Q` sulla banda media e aggiunta del filtro **LPF** (`10, 14, 18 kHz` a `18 dB/oct`).
4. **`greg_neve_88rs_strip` (Neve 88RS):** Console moderna ad altissimo headroom (`+26 dBu`), filtri HPF/LPF continui a pendenza variabile, EQ 4 bande completamente parametrico + sezione dinamica VCA (`That2180`) con Gate/Expander e Compressore/Limiter integrati.

### Famiglia SSL (Solid State Logic)
5. **`greg_ssl_4000e_strip` (SSL 4000 E):** Op-amp `Ne5534`, VCA `Dbx202BlackCan`, EQ selezionabile tra **rev. `02` ("Brown Knob")** (curva musicale più morbida) e **rev. `242` ("Black Knob")** (Q costante e chirurgia più decisa, filtri HPF `18 dB/oct` e LPF `12 dB/oct`) + dinamica VCA E-Series.
6. **`greg_ssl_4000g_strip` (SSL 4000 G):** Op-amp `Ne5534`, VCA `Dbx202GoldCan`, EQ **rev. `292` ("Pink Knob")** con **Proportional Q** (la campana si stringe al crescere del boost/cut) e curve shelf con leggero *undershoot/overshoot* opposto + dinamica VCA G-Series.
7. **`greg_ssl_9000j_strip` (SSL 9000 J):** Architettura **SuperAnalogue** (accoppiamento in DC privo di condensatori elettrolitici sul percorso audio, banda estesa `5 Hz – 100 kHz`, distorsione quasi nulla, transienti cristallini e sub-basse profondissime) + switch E/G sulla curva EQ.

### Famiglia API
8. **`greg_api_vision_strip` (API Vision):** Catena discreta completa: Preamp `212L` (`Api2520` + `Api2623`), filtri `215L`, compressore `225L` + gate/expander `235L`, EQ selezionabile tra `550L` (4 bande a **Proportional Q** con frequenze discrete) e `560L` (EQ grafico 10 bande a Q proporzionale).
9. **`greg_api_legacy_plus_strip` (API Legacy Plus):** Console discreta `Api2520` + `Api2623` con EQ `550L` Proportional Q e bus routing analogico classico.

### Famiglia Harrison, Trident, AMEK & MCI
10. **`greg_harrison_32c_strip` (Harrison 32C):** EQ 4 bande a Q proporzionale con curve morbide e musicali + celebri filtri **HPF (`25 Hz – 3.1 kHz`) e LPF (`160 Hz – 20 kHz`) continui a `12 dB/oct` con caratteristica risonanza al ginocchio**.
11. **`greg_harrison_mixbus_strip` (Harrison Mixbus):** Profilo DSP console virtuale (saturation drive progressivo, EQ 3 bande + HPF, compressore/leveler integrato).
12. **`greg_trident_a_range_strip` (Trident A-Range):** Console discreta vintage a induttori (`PermalloyCore`), EQ 4 bande con interazione musicale tra le bande, filtri passa-alto/passa-basso a scatti e saturazione ricca dei trasformatori.
13. **`greg_trident_80b_strip` (Trident Series 80B):** Progetto senza induttori basato su op-amp (`Tl072` / `Ne5534`), EQ 4 bande con medie swept e alte fisse a `12 kHz`, risposta aperta e brillante.
14. **`greg_amek_9098_strip` (AMEK 9098):** Design Rupert Neve anni '90 ad altissima linearità con controlli **Glow** (generazione controllata di armoniche pari sulle basse) e **Sheen** (enfasi ariosa sulle alte) + EQ parametrico ad alta precisione.
15. **`greg_mci_jh500_strip` (MCI JH-500):** Console americana anni '70: op-amp discreti/integrati con trasformatore d'uscita, EQ 3/4 bande con medie incisive e punch rock.

## 6. Nastro Magnetico, Enhancer & Processori Speciali

### `greg_tape` — Studer A800 Mk III (2-Inch Multitrack)
* **Flusso del Segnale Reale:** `Input Electronics (Discrete/Op-Amp)` → `Record Pre-Emphasis EQ (NAB / CCIR)` → `Record Head + Bias Oscillator` → `Magnetic Tape Hysteresis (Jiles-Atherton)` → `Repro Head Gap Loss & Head Bump` → `Repro De-Emphasis EQ`.
* **Parametri Fisici Obbligatori:**
  * **Velocità Nastro:** `15 IPS` (head bump pronunciato tra `50–80 Hz`, roll-off dolce sopra `15 kHz`, compressione naturale sui transienti) vs `30 IPS` (head bump spostato a `100–130 Hz`, banda estesa `30 Hz – 22 kHz`, distorsione inferiore e transienti più aperti).
  * **Formulazioni Nastro (`TapeFormulaModel`):** `Ampex456`, `Ampex499` (+3 dB headroom), `BasfSm900`, `EmtecSm911`, `Scotch250`.
  * **Curva EQ & Bias:** Commutazione **NAB** (standard americano, più basse) vs **CCIR / IEC1** (standard europeo, minore enfasi sulle basse e rumore inferiore) + calibrazione **Bias** (`Under-Bias` brillante/granuloso vs `Over-Bias` scuro/compresso).

### `greg_exciter` — Aphex Aural Exciter Type C
* **Topologia Circuitale:** Processore parallelo additivo. Il segnale viene sdoppiato: il ramo dry passa inalterato, mentre il ramo sidechain attraversa un **filtro passa-alto accordabile (`Tune` `1 .. 10 kHz`)**, un **generatore di armoniche pari/dispari asimmetrico con leggero sfasamento dinamico** e viene risommato in fase (`Mix`).
* **Comportamento Chiave:** Aumenta la presenza, l'intelligibilità e l'aria sulle alte frequenze senza innalzare il livello di picco (`dBFS`) quanto farebbe un equalizzatore tradizionale.

### `greg_low_enhancer` — dbx 120A Subharmonic Synthesizer
* **Topologia Circuitale:** Crossover fase-coerente a `80 Hz` o `120 Hz` (`HPF 12 dB/oct` + `LPF 6 dB/oct` unity-sum) abbinato a due sintetizzatori di sub-armoniche a ottava inferiore (`f / 2`) guidati dall'inviluppo e dai passaggi per lo zero del segnale di ingresso (`54–110 Hz`).
* **Controlli:** `Subharmonics Master`, Banda 1 (`24–36 Hz`), Banda 2 (`36–56 Hz`), circuito `LF Boost` indipendente e `Subwoofer Out`.

---

## 7. Trasduttori, Microfoni, Cabinet e Spazi Acustici (`core-dsp`)

Per i processori di re-amping, microfonazione virtuale e ambientazione acustica, i parametri fisici sono governati dai moduli dedicati di `core-dsp`:

* **`microphone_models.rs` (Capsula + Circuito Interno):**
  * **Capsule:** Condensatore LDC (`M7`, `K47`, `K67`, `CK12`, `C800G`), SDC (`KM84`, `MK4`), Dinamiche (`SM57`, `SM7B`, `MD421`, `RE20`, `D112`), Nastro (`Royer R121`, `Coles 4038`) e Piezo/Carbon.
  * **Fisica Acustica & Circuito:** Effetto prossimità legato alla figura polare (`Cardioid`, `Figure8`, `Omni`), risposta fuori asse (`0°..180°`), compressione meccanica del diaframma ad alti SPL e accoppiamento diretto al circuito interno (`Octal6sn7` + `HaufeT14`, `Fet2n5457`, `Jensen990`) con override `.with_custom_circuit(...)`.
* **`pickup_transducers.rs` (Pickup & Trasduttori):**
  * Circuito risonante passivo **RLC** (`SingleCoilStrat`, `TeleBridge`, `P90`, `HumbuckerPaf`, `Emg81`, `Precision/Jazz Bass`, `Piezo`, `RhodesTine`, `VinylCartridgeMm`) modulato da capacità del cavo (`100..1500 pF`), impedenza di carico (`10 kΩ .. 1 MΩ`), posizione sul body (*comb aperture*) e saturazione magnetica dei poli.
* **`cabinet_speaker.rs` (Coni, Casse & Microfonazione):**
  * Driver chitarra/basso (`Greenback G12M`, `Vintage 30`, `Alnico Blue`, `Jensen P12N`, `EVM12L`, `Ampeg 10"`, `Hartke 10"`, `JBL 15"`), configurazione (`1x`, `2x`, `4x`, `8x`), cassa (`OpenBack`, `ClosedBack`, `BassReflexPorted` con accordo Helmholtz) e posizionamento continuo del microfono (`cap_to_edge`, `distance_cm`, `off_axis_deg`).
* **`acoustic_room.rs` (Stanza 3D & Prime Riflessioni):**
  * Motore **Image-Source** 3D fino al 2° ordine pre-allocato in `initialize()`, dimensioni stanza (`width_m`, `length_m`, `height_m`), materiali distinti per pareti, pavimento e soffitto, assorbimento dell'aria e diffusione morbida.

---

## 8. Processori Puramente Digitali e Utility (Nessun Outboard Analogico)

I seguenti plugin della suite sono processori digitali originali ad altissima precisione (`f64`, zero colorazione analogica non richiesta, latenza esatta dichiarata all'host ove presente):

* **Dinamica & Livellamento:**
  * `greg_multiband_comp` (Compressore multibanda con crossover Linkwitz-Riley LR4/LR8 a fase coerente).
  * `greg_limiter` (True-Peak Mastering Limiter 4x oversampled ITU-R BS.1770-4 con lookahead e rilascio adattivo).
  * `greg_gate` (Noise Gate / Expander con isteresi, hold e sidechain filter).
  * `greg_leveler` (Vocal / Level Auto-Fader sillabico K-weighted con lookahead, silence floor e music sidechain).
  * `greg_trans_mono` & `greg_trans_multi` (Transient Shaper differenziale a banda singola e multibanda).
* **Equalizzazione & Spettro:**
  * `greg_surgical_eq` (EQ parametrico chirurgico ZDF a 10 bande con analizzatore di spettro).
  * `greg_lin_eq` & `greg_lin_comp` (Equalizzatore FIR e compressore a fase lineare).
  * `greg_eq_match` (Spectrum Matcher LTAS a fase lineare o minima con smoothing psicoacustico e persistenza della curva).
* **Spazio, Modulazione & Tempo:**
  * `greg_space` (Riverbero algoritmico FDN a 8/16 linee con diffusione e modulazione).
  * `greg_pingpong` (Stereo / Multi-tap Delay sincronizzato al BPM con filtraggio nel loop).
  * `greg_ir_reverb` (Convoluzione partizionata a latenza zero con caricamento WAV 16..64-bit).
* **Pitch, Restauro, Lo-Fi & Routing:**
  * `greg_pitch_clean` & `greg_pitch_formant` (Pitch shifter fase-coerente e preservazione delle formanti).
  * `greg_denoiser`, `greg_declipper`, `greg_dehummer`, `greg_declicker` (Suite di restauro audio spettrale e nel dominio del tempo).
  * `greg_destroyer` (Decimator, Bitcrusher e simulatore campionatori vintage Akai MPC60 / MPC3000 / SP-1200).
  * `greg_bus_summer`, `greg_phase`, `greg_ms_encoder`, `greg_ms_decoder` (Utility di somma, rotazione continua di fase e matrice Mid/Side).


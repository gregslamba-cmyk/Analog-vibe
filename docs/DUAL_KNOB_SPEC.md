# DUAL_KNOB_SPEC.md — Controllo Rotativo Concentrico Doppio (Dual Concentric Knob)

**Progetto:** Analog-Vibe — Greg Audio (proprietà di Alberto Gregorio Ferraris)  
**File:** `docs/DUAL_KNOB_SPEC.md`  
**Modulo di riferimento:** `crates/greg_gui_core` (widget Vizia 2D)  
**Standard UI di riferimento:** `GUI_SPEC.md` (colori, font, tacca a ore 12, zero marchi registrati)

---

## 1. Scopo (Purpose)

Il widget `dual_knob` è un controllo rotativo concentrico doppio che espone **due parametri distinti** (`primary_param` e `secondary_param`) occupando lo stesso spazio schermo di una singola manopola. È pensato per coppie di controlli strettamente correlate nel workflow del fonico, ad esempio:

- `Input Gain` + `Drive`
- `Threshold` + `Ratio`
- `Frequency` + `Q`

L'**anello esterno grande** controlla il parametro primario; il **disco interno piccolo** controlla il parametro secondario. Questo consente di risparmiare spazio nel pannello principale (`Main Panel`) mantenendo immediato l'accesso ai controlli essenziali, in linea con la gerarchia a due livelli di `GUI_SPEC.md` (Main Panel sempre visibile; tarature interne e parametri avanzati nel pannello `+ ADV`).

---

## 2. Standard Visivi e Palette (da GUI_SPEC.md)

Il widget segue lo standard hardware moderno **piatto 2D** (niente finto 3D, nessuna texture Bakelite/chickenhead, nessun gradiente che simuli volume).

| Elemento | Valore | Note |
|---|---|---|
| Sfondo pannello | `#181A1F` / `#20232A` | Dark antiriflesso, da GUI_SPEC.md |
| Primary accent (anello esterno) | `#E5A93B` | Ambra/ottone, da GUI_SPEC.md |
| Secondary accent (disco interno) | `#6FA8DC` | Blu acciaio, distinto e leggibile su sfondo scuro |
| Testo etichette parametri | `#FFFFFF` | Bianco, centrato, non hoverabile |
| Testo valore formattato | `#E5A93B` | Ambra, centrato, non hoverabile |
| Tick mark default (ore 12) | Colore dell'anello/disco corrispondente | 12-o'clock sul valore di default |
| Spessore traccia anello esterno | 3px | Più spesso |
| Spessore traccia disco interno | 2px | Più sottile |

Nessun effetto di rilievo, ombra portata o gradiente 3D: il controllo è piatto 2D con tacche ruotabili.

---

## 3. JSON Schema (gui_layout.json)

Lo schema va inserito nel file `gui_layout.json` (sezione widget / layout) come elemento di tipo `"dual_knob"`:

```json
{
  "type": "dual_knob",
  "primary_param": "string",
  "secondary_param": "string",
  "large_radius": 36.0,
  "small_radius": 18.0,
  "min_value": -48.0,
  "max_value": 24.0,
  "step": 0.1,
  "tooltip": "Primary: Input Gain | Secondary: Drive"
}
```

### Descrizione campi

| Campo | Tipo | Obbligatorio | Descrizione |
|---|---|---|---|
| `type` | `"dual_knob"` | Sì | Deve essere esattamente `dual_knob` |
| `primary_param` | string | Sì | ID del parametro CLAP controllato dall'anello esterno |
| `secondary_param` | string | Sì | ID del parametro CLAP controllato dal disco interno |
| `large_radius` | f32 | Sì | Raggio dell'anello esterno in px (es. 36.0 → diametro 72px, compatibile con l'area `KnobSize::Large` 64px + padding) |
| `small_radius` | f32 | Sì | Raggio del disco interno in px (es. 18.0 → diametro 36px, compatibile con l'area `KnobSize::Medium` 48px) |
| `min_value` | f32 | Sì | Valore minimo (vedi §3.1: condiviso o per-parametro) |
| `max_value` | f32 | Sì | Valore massimo (vedi §3.1: condiviso o per-parametro) |
| `step` | f32 | Sì | Granulatura di incremento (es. 0.1) |
| `tooltip` | string | No | Tooltip mostrata al hover con nomi e valori di entrambi i parametri |

### 3.1 Range valori: condiviso vs per-parametro

Il schema supporta due modalità:

1. **Range condiviso (default):** `min_value` e `max_value` si applicano sia a `primary_param` che a `secondary_param`. Utile quando i due parametri condividono l'unità di misura (es. entrambi in dB).
2. **Range per-parametro:** se il secondario ha unità diversa (es. `Ratio` da 1.0 a ∞, `Q` da 0.1 a 40.0), il widget accetta i campi opzionali `primary_min`/`primary_max` e `secondary_min`/`secondary_max`. In loro assenza valgono `min_value`/`max_value` per entrambi.

Tutti i calcoli interni avvengono in **`f64`**; la conversione in `f32` avviene solo al momento del rendering (§8).

---

## 4. Modello di Interazione (Interaction Model)

| Azione | Target | Effetto |
|---|---|---|
| Drag mouse (tasto sinistro trascinato) | Anello esterno (`small_radius` < radius ≤ `large_radius`) | Modifica `primary_param` |
| Drag mouse | Disco interno (radius ≤ `small_radius`) | Modifica `secondary_param` |
| Scroll rotella (nessun modificatore) | Qualunque parte del widget | Modifica `primary_param` a passo `step` (coarse) |
| Scroll rotella + `Shift` | Qualunque parte del widget | Modifica `secondary_param` a passo `step` (fine) |
| Doppio click | Anello esterno | Resetta `primary_param` al valore di default |
| Doppio click | Disco interno | Resetta `secondary_param` al valore di default |
| Hover | Qualunque parte | Mostra tooltip con entrambi i nomi e i valori correnti |

### 4.1 Direzione e mappatura drag

- Il drag **verticale** (su/giù) mappa sul valore: trascinare verso l'alto aumenta, verso il basso diminuisce.
- La corsa di riferimento per l'escursione completa `min_value` → `max_value` è di **200.0 px** verticali, coerente con lo standard `RotaryKnob` di `GUI_SPEC.md`.
- Con `Shift` premuto durante il drag, la risoluzione è ridotta di fattore **0.1x** (fine) per entrambi i parametri.

### 4.2 Valore di default e tacca ore 12

Ogni parametro ha un valore di default (neutral point) definito nello schema del plugin. Il widget visualizza la **tacca a ore 12** quando il rispettivo parametro è al default (§6).

### 4.3 Tooltip

Al passaggio del mouse (`hover`), il widget mostra un tooltip della forma:

```
Primary: Input Gain — +12.0 dB
Secondary: Drive — 0.35
```

Il testo è fornito dal campo `tooltip` quando presente; altrimenti viene generato automaticamente dai nomi e valori correnti dei due parametri.

---

## 5. Hit-Testing (Regioni di colpito)

Il widget definisce due regioni di hit-test centrate sul punto medio del widget:

1. **Inner disk (disco interno):**  
   `distance(center, mouse) <= small_radius` → agisce su `secondary_param`.

2. **Outer ring (anello esterno):**  
   `small_radius < distance(center, mouse) <= large_radius` → agisce su `primary_param`.

3. **Area esterna:** `distance > large_radius` → nessun controllo attivo (pass-through ai widget sottostanti).

Se il widget viene scalato non uniformemente, le regioni diventano ellittiche usando i semi-assi `large_radius_x`, `large_radius_y`, `small_radius_x`, `small_radius_y` derivati da `large_radius` e `small_radius` moltiplicati per i fattori di scala X e Y.

---

## 6. Visual Design

- Entrambi gli anelli mostrano una **tacca di riferimento a ore 12** allineata con il valore di default del rispettivo parametro.
- L'anello esterno ha uno **spessore di traccia 3px**; il disco interno ha uno **spessore di traccia 2px**.
- L'anello esterno usa il **primary accent** (`#E5A93B`); il disco interno usa il **secondary accent** (`#6FA8DC`).
- Stile piatto 2D: nessun finto rilievo, nessuna ombra, nessun gradiente 3D.
- Layout secondo la cella a 3 fasce di `GUI_SPEC.md`:
  1. **Top (18px):** etichetta parametri (es. "Input Gain / Drive") bianca centrata.
  2. **Center:** widget `dual_knob` (diametro = 2 × `large_radius` + padding).
  3. **Bottom (18px):** valore formattato (entrambi i valori, separati da `|` o su due righe) in ambra centrato.

---

## 7. Accessibilità (ARIA)

Ogni anello/disco è esposto come controllo indipendente per le tecnologie assistive:

| Attributo | Anello esterno | Disco interno |
|---|---|---|
| `role` | `slider` | `slider` |
| `aria-label` | Nome del `primary_param` (es. "Input Gain") | Nome del `secondary_param` (es. "Drive") |
| `aria-valuemin` | `min_value` | `min_value` (o `secondary_min` se presente) |
| `aria-valuemax` | `max_value` | `max_value` (o `secondary_max` se presente) |
| `aria-valuenow` | Valore corrente di `primary_param` | Valore corrente di `secondary_param` |

Una **live region** (`aria-live="polite"` o equivalente Vizia/CLAP) annuncia gli aggiornamenti di valore quando il parametro cambia via drag, scroll o automazione host, senza rubare il focus.

---

## 8. Integrazione con i Parametri (`param_access!` macro)

- La macro `param_access!` deve generare **getter e setter** per entrambi gli ID parametro (`primary_param` e `secondary_param`).
- Il widget si sottoscrive alle variazioni di entrambi i parametri per la **sincronizzazione bidirezionale**:
  - Variazione host / automazione → aggiornamento visuale del widget.
  - Variazione utente sul widget → scrittura sul parametro host.
- I nomi degli ID parametro devono essere originali e privi di marchi registrati (§9).

---

## 9. Real-Time Safety (Sicurezza in tempo reale)

- **Zero allocazioni heap** nell'aggiornamento del widget (niente `Vec::push`, `Box::new`, `String`, `format!` nel path di rendering/UI update).
- Calcoli interni esclusivamente in **`f64`**; conversione in `f32` solo per il rendering Vizia.
- Sanitizzazione obbligatoria: qualsiasi `NaN` o `Inf` viene convertito in `0.0` prima del disegno e prima di essere inviato al thread audio.
- Nessun lock, mutex o I/O nel thread di aggiornamento UI.
- Le scritture dal thread audio verso la UI usano `AtomicU32` / `AtomicBool` con `Ordering::Relaxed`, come da `GUI_SPEC.md` §2.

---

## 10. Esempi di utilizzo in `gui_layout.json`

### Esempio A — Input Gain + Drive

```json
{
  "type": "dual_knob",
  "primary_param": "input_gain",
  "secondary_param": "drive",
  "large_radius": 36.0,
  "small_radius": 18.0,
  "min_value": -48.0,
  "max_value": 24.0,
  "step": 0.1,
  "tooltip": "Primary: Input Gain | Secondary: Drive"
}
```

### Esempio B — Threshold + Ratio

```json
{
  "type": "dual_knob",
  "primary_param": "threshold",
  "secondary_param": "ratio",
  "large_radius": 36.0,
  "small_radius": 18.0,
  "primary_min": -60.0,
  "primary_max": 0.0,
  "secondary_min": 1.0,
  "secondary_max": 100.0,
  "step": 0.1,
  "tooltip": "Primary: Threshold | Secondary: Ratio"
}
```

### Esempio C — Frequency + Q

```json
{
  "type": "dual_knob",
  "primary_param": "band_frequency",
  "secondary_param": "band_q",
  "large_radius": 36.0,
  "small_radius": 18.0,
  "primary_min": 20.0,
  "primary_max": 20000.0,
  "secondary_min": 0.1,
  "secondary_max": 40.0,
  "step": 0.1,
  "tooltip": "Primary: Frequency | Secondary: Q"
}
```

---

## 11. Conformità Marchi Registrati (Trademark / IP)

Tutti i nomi di file, ID parametro, preset, etichette UI, commenti e documentazione devono essere **100% originali e privi di marchi registrati**. È assolutamente vietato l'uso di nomi commerciali quali Neve, SSL, API, Pultec, Fairchild, Urei, 1176, LA-2A, Genelec, NS10, Avantone, Adam, Marshall, Fender, Moog, Lexicon, Studer, Dolby e simili. Usare esclusivamente nomenclature tecniche e descrittive (es. `input_gain`, `drive`, `threshold`, `ratio`, `band_frequency`, `band_q`).

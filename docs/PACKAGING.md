# Guida al Packaging Multi-Formato: CLAP, VST3, LV2, Audio Unit

Questa guida descrive l'architettura di esportazione e distribuzione dei plugin della suite **Analog-vibe**, garantendo compatibilità universale con tutte le Digital Audio Workstation (DAW) su Linux, macOS e Windows.

---

## 1. I 4 Formati Supportati

La suite supporta nativamente o tramite wrapping trasparente quattro formati standard:

| Formato | Estensione | Metodo di Generazione | Ambito Principale & DAW Compatibili |
| :--- | :--- | :--- | :--- |
| **CLAP** | `.clap` | Nativo (`nih_plug`) | Standard aperto moderno, Reaper, Bitwig Studio, ecc. |
| **VST3** | `.vst3` | Nativo (`nih_plug`) | Cubase/Nuendo, Studio One, FL Studio, Ableton Live, Reaper |
| **LV2** | `.lv2` | Bundle & `clap-wrapper` | Linux open-source, Ardour, Harrison Mixbus, Carla |
| **Audio Unit (AUv2)** | `.component` | Bundle & `clap-wrapper` | macOS nativo, Apple Logic Pro, GarageBand |

> [!NOTE]
> **VST2 Deprecato:** Lo standard VST2 è stato ufficialmente dismesso da Steinberg, che non rilascia più accordi di licenza sviluppatore. Per questo motivo, il supporto VST2 è stato escluso per garantire piena legalità e conformità normativa.

---

## 2. Esportazione Nativa CLAP + VST3 (`export_greg_plugin!`)

Grazie al modulo condiviso [`core-dsp/src/core_dsp_modules/plugin_export.rs`](file:///home/greg/Documenti/analog%20vibes/core-dsp/src/core_dsp_modules/plugin_export.rs), ogni plugin esporta nativamente **sia l'entry-point CLAP (`clap_entry`) sia l'entry-point VST3 (`GetPluginFactory` / `ModuleEntry`)** dallo stesso binario compilato.

Nei sorgenti del plugin in `plugins/<nome-plugin>/src/lib.rs` è sufficiente invocare la macro con 3 righe di codice:

```rust
core_dsp::export_greg_plugin!(
    GregFet,
    "com.greg.audio.greg-fet",
    "FET-style dynamics processor",
    compressor
);
```

La macro esegue automaticamente:
1. `impl ClapPlugin for GregFet` con metadati ufficiali e categorie audio.
2. `impl Vst3Plugin for GregFet` con generazione deterministica del Class ID VST3 (UUID RFC 4122 v4) calcolato tramite hashing `const fn` dell'ID del plugin.
3. Invocazione congiunta di `nih_export_clap!(GregFet)` e `nih_export_vst3!(GregFet)`.

---

## 3. Conversione in LV2 e Audio Unit (`clap-wrapper`)

Per coprire le DAW che non supportano nativamente CLAP (o ambienti Linux/macOS specifici):
- Viene utilizzato il framework open-source **`clap-wrapper`** (licenza permissiva MIT, sviluppato dall'organizzazione Free Audio su GitHub: `https://github.com/free-audio/clap-wrapper`).
- `clap-wrapper` funge da host CLAP ultra-leggero e a bassissimo overhead incorporato all'interno del wrapper target (LV2 o AUv2).

### Struttura Bundle LV2 (`.lv2`)
Una directory per plugin contenente:
- `manifest.ttl`: definisce l'URI del plugin, il file binario e il file descrittore `plugin.ttl`.
- `<plugin>.ttl`: specifica la categorizzazione LV2 (`lv2:Plugin`, `lv2:DynamicsPlugin`), il supporto `hardRTCapable`, e le porte audio (In L/R, Out L/R).
- `<plugin>.so`: il binario compilato.

### Struttura Bundle Audio Unit (`.component`)
Un bundle standard per macOS contenente:
- `Contents/Info.plist`: registra l'AudioComponent in macOS CoreAudio (`aufx`, subtype a 4 caratteri univoco, manufacturer `Greg`).
- `Contents/MacOS/<plugin>`: l'eseguibile binario.
- `Contents/PkgInfo`: tipo bundle `BNDL????`.

---

## 4. Script Automatico di Packaging (`scripts/package_all_formats.sh`)

Lo script bash [`scripts/package_all_formats.sh`](file:///home/greg/Documenti/analog%20vibes/scripts/package_all_formats.sh) compila e impacchetta automaticamente tutti i formati nella cartella `dist/`:

### Utilizzo

```bash
# Impacchetta tutti i plugin in modalità release
./scripts/package_all_formats.sh --all

# Impacchetta un singolo plugin
./scripts/package_all_formats.sh -p greg-fet

# Impacchetta senza ricompilare i sorgenti già compilati
./scripts/package_all_formats.sh --skip-build
```

### Struttura Output Generata in `dist/`

```text
dist/
├── clap/
│   ├── greg-fet.clap
│   ├── greg-a2a.clap
│   └── ...
├── vst3/
│   ├── greg-fet.vst3/
│   │   └── Contents/
│   │       └── x86_64-linux/
│   │           └── greg-fet.so
│   └── ...
├── lv2/
│   ├── greg-fet.lv2/
│   │   ├── manifest.ttl
│   │   ├── greg-fet.ttl
│   │   └── greg-fet.so
│   └── ...
└── au/
    ├── greg-fet.component/
    │   └── Contents/
    │       ├── Info.plist
    │       ├── PkgInfo
    │       └── MacOS/
    │           └── greg-fet
    └── ...
```

---

## 5. Percorsi di Installazione Utente per le DAW

Per rendere disponibili i plugin alla propria DAW sul computer locale:

### Linux
```bash
# CLAP
mkdir -p ~/.clap && cp dist/clap/*.clap ~/.clap/

# VST3
mkdir -p ~/.vst3 && cp -r dist/vst3/*.vst3 ~/.vst3/

# LV2 (Ardour, Mixbus, Reaper, Carla)
mkdir -p ~/.lv2 && cp -r dist/lv2/*.lv2 ~/.lv2/
```

### macOS
```bash
# CLAP
mkdir -p ~/Library/Audio/Plug-Ins/CLAP
cp dist/clap/*.clap ~/Library/Audio/Plug-Ins/CLAP/

# VST3
mkdir -p ~/Library/Audio/Plug-Ins/VST3
cp -R dist/vst3/*.vst3 ~/Library/Audio/Plug-Ins/VST3/

# Audio Unit (Logic Pro, GarageBand)
mkdir -p ~/Library/Audio/Plug-Ins/Components
cp -R dist/au/*.component ~/Library/Audio/Plug-Ins/Components/
```

---

## 6. Regime di Licenza & Compatibilità

- **Dual Licensing:** Il codice sorgente DSP e i plugin di Analog-vibe sono rilasciati sotto licenza **GNU GPLv3 o successiva** (uso gratuito e libero purché derivati rimangano open-source) oppure sotto **Licenza Commerciale Proprietaria** a basso costo su richiesta all'autore.
- **clap-wrapper (MIT):** La libreria di wrapping `clap-wrapper` è rilasciata sotto licenza MIT, compatibile sia con distribuzioni GPLv3 sia con distribuzioni commerciali.

# AGENTS.md — Protocollo Operativo Analog-Vibe

## 1. Ruoli e gerarchia

- **Studio Director (agente principale)**: analizza intenti, coordina i subagenti, produce task atomici, verifica output, aggiorna documentazione di stato (`STATE.md`, `TODO.md`, `ROADMAP.md`, `doit.txt`).
- **Specialisti interni (MCP)**: `DSP & Audio Architect`, `DSP Math & Physics Validator`, `Web Research & Hardware Scout`, `Studio UI/UX Designer`, `Real-Time QA Critic`, `Rust Code Reviewer`, `Studio Secretary`, `cline task dispacer`, `Senior Engineer`, `GUI Designer`.
- **Cline**: **unico autorizzato a scrivere codice**. Legge `doit.txt` e i file di riferimento; esegue implementazione atomica. Non effettua analisi architetturali né decisioni di design.
- **Regola d'oro**: **il codice lo scrive Cline**. L'utente (Alberto Gregorio Ferraris) può autorizzare raramente l'agente Direttore a scrivere codice; in tal caso la richiesta deve essere esplicita e motivata.

> **Nota**: Nei casi rari in cui il Direttore scrive codice, esso viene inserito direttamente in `doit.txt` come task per Cline, oppure Cline lo riceve da un input umano autorizzato.

## 2. Consultazione preventiva obbligatoria

Prima di qualsiasi azione, decisione o scrittura di codice, **è obbligatorio leggere** i seguenti file (anche se solo bozza/riassunto) per restare allineati allo stato attuale del progetto:

- **`STATE.md`** — stato corrente validato, test, build.
- **`TODO.md`** — task pendenti e stati plugin.
- **`ROADMAP.md`** — milestone e roadmap.
- **`doit.txt`** — task atomici in esecuzione/pronti.
- **`ARCHITECTURE.md`** — architettura core-dsp, API moduli.
- **`GUI_SPEC.md`** — linee guida UI/UX quando si toccano plugin GUI.
- **`vcinst.md`** — vincoli real-time, zero-alloc, sicurezza DSP.
- **`.clinerules`** — regole operative per Cline.
- **`docs/PLUGIN_CATALOG.md`** — stato plugin, riferimento hardware, roadmap futura.
- **`docs/specs/`** — research e specifiche tecniche.
- **`docs/modules/`** — descrizione moduli DSP.
- **`plugins/<nome_plugin>/src/lib.rs`** (solo per conferma stato o revocare/modificare decisioni).

> **Disciplina**: se il file non è stato consultato nell'ultima ora, l'agente deve fermarsi e rileggerlo prima di procedere.

## 3. Flusso operativo standard

```
1. Studio Director riceve obiettivo alto-level (es. "implementa greg_tube_drive")
2. Studio Director legge file di stato (STATE.md, TODO.md, ROADMAP.md, doit.txt)
3. Studio Director consulta specialist interni in parallelo/sequenza:
   - Architettura e topologia → DSP & Audio Architect
   - Equazioni, stabilità, gain → DSP Math & Physics Validator
   - Layout manopole/parametri → Studio UI/UX Designer
   - Vincoli RT, zero-allocation → Real-Time QA Critic
   - Pattern Rust idiomatici, clippy → Rust Code Reviewer
4. Studio Director sintetizza task atomici in `doit.txt` (un task per riga, con ID, file target, criteri accettazione)
5. Studio Secretary aggiorna TODO.md e ROADMAP.md per riflettere la nuova suddivisione
6. Cline legge `doit.txt` ed esegue l'implementazione (scrive .rs, .toml, .clap, .md se specificato)
7. Al termine: build `cargo build --release -p <plugin>`, test `cargo test -p <plugin>`, `clap-signal-host` → 20/20 PASS
8. Se passano: `cargo clippy -D warnings -p <plugin>` → 0 warning
9. Studio Director verifica outcome, aggiorna STATE.md e docs, esegue commit/push con subtree appropriati
```

## 4. Vincoli per specialisti interni

- **Non scrivere codice .rs** (a meno che non sia esplicitamente autorizzato dal Direttore in casi di emergenza o refactoring massivo). Produrre solo: analisi, report, equazioni, sketch architetturali, task per Cline.
- **Leggere obbligatoriamente** i file citati in sezione 2 prima di produrre qualsivoglia deliverable.
- **Convenzione nomi task**: `task <id>-<descrizione>` (es. `task P1-2`), descritti in `doit.txt`.

## 5. Vincoli per Cline

- **Leggere `doit.txt` all'inizio di ogni sessione**; se manca o è vuoto, chiedere direttive al Direttore.
- **Non proporre cambiamenti architetturali**; limitarsi all'implementazione di task specificati.
- **Non modificare documentazione di design** (`ARCHITECTURE.md`, `GUI_SPEC.md`, `vcinst.md`) se non espressamente richiesto dal Direttore dopo verifica.
- **Usare solo gli editor/strumenti previsti**; non fare `git` direttamente se non autorizzato (il Direttore gestisce commit/push).

## 6. Allineamento continuo

- Ogni completamento di task deve essere riflesso **immediatamente** in:
  - `doit.txt` (spuntato o rimosso)
  - `TODO.md` (aggiornamento stato milestone/plugin)
  - `STATE.md` (eventuale nuovo "PASS" o "FAIL")
  - `ROADMAP.md` (se milestone raggiunta)
- Se uno o più file non sono aggiornati, **nessun nuovo task** deve essere avviato finché lo stato non è sincronizzato.

## 7. Casi rari: Direttore autorizzato a scrivere codice

Quando l'utente scrive esplicitamente `"autorizza agente a scrivere codice"` o in contesti di urgenza critica di refactoring, il Direttore può scrivere direttamente snippet `.rs` o `.toml`, ma deve:

1. Inserirli in `doit.txt` come task per Cline con annotazione `AUTHORIZED_DIRECTOR_CODE`.
2. Marcarli con commento `// [Direttore Autorizzato — non modificare senza revisione Cline]`.
3. Subito dopo, eseguire `cargo clippy -D warnings` e `cargo test` per validare.
4. Aggiornare `STATE.md` e `doit.txt` a completamento.

---
*Ultimo aggiornamento: 2026-11-27*
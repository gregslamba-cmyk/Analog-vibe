#!/usr/bin/env bash
set -e
cd /home/greg/analog-vibes

OUT="FULL_CONTEXT.md"
{
  echo "# FULL CONTEXT — GREG AUDIO ECOSYSTEM (ANALOG-VIBE + DIGITAL-VIBE + MASTERING-VIBE)"
  echo "Generato automaticamente: $(date '+%Y-%m-%d %H:%M')"
  echo ""
  for f in \
    vcinst.md \
    STATE.md \
    ROADMAP.md \
    doit.txt \
    GUI_SPEC.md \
    ARCHITECTURE.md \
    docs/INVENTARIO.txt \
    docs/PLUGIN_CATALOG.md \
    docs/CORE_DSP_AUDIT.md \
    TODO.md \
    core-dsp/TODO.md \
    crates/greg_gui_core/TODO.md \
    tools/clap-validator-host/TODO.md \
    "/home/greg/Documenti/digital-vibes/ARCHITECTURE.md" \
    "/home/greg/Documenti/digital-vibes/TODO.md" \
    "/home/greg/Documenti/mastering-vibes/ARCHITECTURE.md" \
    "/home/greg/Documenti/mastering-vibes/TODO.md" \
    "/home/greg/Documenti/progettazione futuri audio pro.txt"
  do
    if [ -f "$f" ]; then
      echo -e "\n================================================================================"
      echo "## FILE: $f"
      echo -e "================================================================================\n"
      cat "$f"
    fi
  done
} > "$OUT"

echo "OK: Creato $OUT ($(du -h "$OUT" | cut -f1) — $(wc -l < "$OUT") righe)"

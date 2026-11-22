#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

# Pulizia eventuali file/cartelle spurie
rm -rf "./'" './"'

# Controllo veloce che ci siano modifiche da salvare
if [ -z "$(git status --porcelain)" ]; then
  echo "Nessuna modifica da committare."
  exit 0
fi

# Messaggio personalizzabile passato da terminale, oppure default automatico con data e ora
MSG="${1:-[Analog-Vibe] Aggiornamento suite DSP/GUI $(date '+%Y-%m-%d %H:%M')}"

echo "=== File modificati ==="
git status -s

echo "=== Aggiunta e Commit ==="
git add -A
git commit -m "$MSG"

echo "=== Push su GitHub ==="
git push

echo "✅ Commit e push completati con successo!"


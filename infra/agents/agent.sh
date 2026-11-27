#!/usr/bin/env bash
# agent.sh — Attiva un agente Docker di Greg Audio su richiesta (on-demand).
# Uso: ./agent.sh <agente> <comando> [argomenti...]
# Esempi:
#   ./agent.sh agent-web-research python -m playwright --version
#   ./agent.sh agent-rust-review cargo clippy --workspace --all-targets -- -D warnings
#   ./agent.sh agent-dsp-architect gcc --version
#   ./agent.sh agent-sandbox sh -c "ls -la /workspace"
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
AGENT="${1:-}"
shift || true

if [[ -z "$AGENT" ]]; then
  echo "Uso: $0 <agente> <comando> [arg...]"
  echo ""
  echo "Agenti disponibili:"
  echo "  agent-web-research    Web Research & Hardware Scout"
  echo "  agent-rust-review     Rust Code Reviewer"
  echo "  agent-dsp-architect   DSP & Audio Architect"
  echo "  agent-sandbox         Sandbox (operazioni ad alto rischio)"
  exit 1
fi

# Verifica che il servizio esista nel compose
if ! docker compose -f "$COMPOSE_FILE" config --services | grep -qx "$AGENT"; then
  echo "ERRORE: servizio '$AGENT' non trovato in $COMPOSE_FILE"
  exit 1
fi

echo ">> Avvio agente on-demand: $AGENT"
docker compose -f "$COMPOSE_FILE" run --rm "$AGENT" "$@"

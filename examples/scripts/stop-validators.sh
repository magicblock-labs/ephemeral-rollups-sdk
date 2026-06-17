#!/usr/bin/env bash
# Tear down the local MagicBlock stack started by start-validators.sh.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

for svc in qfs er base; do
  pidfile="${ER_RUN_DIR}/${svc}.pid"
  if [ -f "$pidfile" ]; then
    pid="$(cat "$pidfile")"
    if kill -0 "$pid" 2>/dev/null; then
      log "stopping ${svc} (pid ${pid})"
      kill "$pid" 2>/dev/null || true
      # give it a moment, then force
      for _ in 1 2 3 4 5 6; do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
      kill -9 "$pid" 2>/dev/null || true
    fi
    rm -f "$pidfile"
  fi
done
log "stack stopped"

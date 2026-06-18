#!/usr/bin/env bash
# Tear down the local MagicBlock stack started by start-validators.sh.
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

# Kill a process and its whole process group (each daemon is its own session).
kill_tree() {
  local pid="$1" sig="$2"
  kill "-${sig}" "-${pid}" 2>/dev/null || kill "-${sig}" "${pid}" 2>/dev/null || true
}

for svc in qfs er base vrf; do
  pidfile="${ER_RUN_DIR}/${svc}.pid"
  [ -f "$pidfile" ] || continue
  pid="$(cat "$pidfile")"
  if kill -0 "$pid" 2>/dev/null; then
    log "stopping ${svc} (pid ${pid})"
    kill_tree "$pid" TERM
    for _ in 1 2 3 4 5 6; do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
    kill_tree "$pid" KILL
  fi
  rm -f "$pidfile"
done
clean_stack_ports
log "stack stopped"

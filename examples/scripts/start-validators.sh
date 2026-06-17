#!/usr/bin/env bash
# Boot the local MagicBlock stack: base + ephemeral validator + query-filtering-service.
#
# Requires the binaries from `yarn global add @magicblock-labs/ephemeral-validator@latest`
# (mb-test-validator, ephemeral-validator, query-filtering-service) on PATH.
#
# Optional first argument: a directory of extra SBF programs to load onto the base
# layer. Any `<PROGRAM_ID>.so` paired with a `<PROGRAM_ID>-keypair.json` (or a plain
# `*.so` whose name is the program id) is added via --bpf-program. Anchor's
# `target/deploy` layout works directly.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

on_exit() {
  local code=$?
  if [ "$code" -ne 0 ]; then
    print_validator_logs
  fi
}
trap on_exit EXIT

EXTRA_PROGRAMS_DIR="${1:-}"

mkdir -p "${ER_RUN_DIR}"
rm -f "${ER_RUN_DIR}"/*.pid

# --- Build the list of extra programs to preload on the base layer ----------------
base_extra_args=()
if [ -n "${EXTRA_PROGRAMS_DIR}" ] && [ -d "${EXTRA_PROGRAMS_DIR}" ]; then
  shopt -s nullglob
  for so in "${EXTRA_PROGRAMS_DIR}"/*.so; do
    name="$(basename "$so" .so)"
    kp="${EXTRA_PROGRAMS_DIR}/${name}-keypair.json"
    if [ -f "$kp" ]; then
      pid="$(solana-keygen pubkey "$kp")"
    else
      pid="$name"
    fi
    log "preloading program ${pid} from ${so}"
    base_extra_args+=(--bpf-program "$pid" "$so")
  done
  shopt -u nullglob
fi

# Launch a daemon in the background and record its PID. The daemons are children of
# the shell that sources/runs this script, so run start + tests within a single
# shell (a single CI step) and tear down with stop-validators.sh.
# Each daemon must run in its own session so it (a) is not disturbed by the
# launching shell's job control / controlling terminal (the ephemeral-validator
# exits otherwise) and (b) survives this script returning, so a later step in the
# same job can talk to it. `setsid` on Linux/CI; a POSIX `setsid()` Perl shim on
# macOS. The launcher itself becomes the daemon via exec, so $! is its real PID and
# the session leader (== process group), which stop-validators.sh group-kills.
SETSID_BIN="$(command -v setsid || true)"

spawn() {
  local name="$1" logfile="$2"; shift 2
  if [ -n "$SETSID_BIN" ]; then
    "$SETSID_BIN" "$@" > "$logfile" 2>&1 < /dev/null &
  else
    # macOS lacks setsid(1); detach from the launching shell's job control.
    nohup perl -e 'use POSIX qw(setsid); setsid(); open(STDIN, "</dev/null"); exec @ARGV or die $!;' \
      "$@" > "$logfile" 2>&1 < /dev/null &
    disown >/dev/null 2>&1 || true
  fi
  echo "$!" > "${ER_RUN_DIR}/${name}.pid"
}

# --- base layer (mb-test-validator) -----------------------------------------------
log "starting base validator (mb-test-validator) on ${BASE_RPC_URL}"
spawn base "${ER_RUN_DIR}/base.log" \
  mb-test-validator \
  --reset \
  --rpc-port "${BASE_RPC_PORT}" \
  --ledger "${ER_RUN_DIR}/base-ledger" \
  ${base_extra_args[@]+"${base_extra_args[@]}"}
wait_for_rpc "${BASE_RPC_URL}" "base validator" 90

# Give the base layer a moment to settle before the ER connects. Without this,
# the first ephemeral-validator attempt often hangs until the retry loop's sleep.
ER_BASE_SETTLE_SECS="${ER_BASE_SETTLE_SECS:-3}"
if [ "$ER_BASE_SETTLE_SECS" -gt 0 ]; then
  log "waiting ${ER_BASE_SETTLE_SECS}s for base validator to settle"
  sleep "$ER_BASE_SETTLE_SECS"
fi

# --- ephemeral rollup validator ---------------------------------------------------
# The ephemeral-validator occasionally comes up "half-dead" (bound to its port but
# not serving RPC), so retry a few times: kill the attempt, free the ports, respawn.
kill_er_attempt() {
  local pid; pid="$(cat "${ER_RUN_DIR}/er.pid" 2>/dev/null || true)"
  if [ -n "$pid" ]; then
    kill -KILL "-${pid}" 2>/dev/null || kill -KILL "${pid}" 2>/dev/null || true
  fi
  clean_er_ports
  rm -rf "${ER_RUN_DIR}/er-storage"
}

clean_er_ports

er_ready=false
for attempt in 1 2 3; do
  log "starting ephemeral-validator on ${ER_RPC_URL} (remotes -> ${BASE_RPC_URL}) [attempt ${attempt}]"
  er_log="${ER_RUN_DIR}/er-attempt-${attempt}.log"
  spawn er "$er_log" \
    ephemeral-validator \
    --remotes "${BASE_RPC_URL}" \
    --listen "127.0.0.1:${ER_RPC_PORT}" \
    --lifecycle ephemeral \
    --no-tui \
    --reset \
    --storage "${ER_RUN_DIR}/er-storage"
  ln -sf "$(basename "$er_log")" "${ER_RUN_DIR}/er.log"
  er_pid="$(cat "${ER_RUN_DIR}/er.pid")"
  if wait_for_er "$er_log" "$er_pid" 90; then
    er_ready=true
    break
  fi
  err "ephemeral validator did not come up (attempt ${attempt}); retrying"
  kill_er_attempt
  sleep 3
done
if [ "$er_ready" != true ]; then
  err "ephemeral validator failed to start after 3 attempts"
  exit 1
fi

# --- query-filtering-service (router/proxy in front of the ER) --------------------
log "starting query-filtering-service on ${ROUTER_RPC_URL} -> ${ER_RPC_URL}"
spawn qfs "${ER_RUN_DIR}/qfs.log" \
  query-filtering-service \
  --listen-addr "127.0.0.1:${ROUTER_RPC_PORT}" \
  --listen-addr-ws "127.0.0.1:${ROUTER_WS_PORT}" \
  --ephemeral-url "${ER_RPC_URL}" \
  --ephemeral-url-ws "ws://127.0.0.1:${ER_WS_PORT}" \
  --add-cors-headers
wait_for_rpc "${ROUTER_RPC_URL}" "query-filtering-service" 60

# --- VRF oracle (optional; the vrf example needs it) ------------------------------
# Set START_VRF_ORACLE=1 to run a vrf-oracle subscribed to the base layer, which
# fulfils randomness requests by calling back the requesting program.
if [ "${START_VRF_ORACLE:-0}" = "1" ]; then
  log "starting vrf-oracle (subscribed to ${BASE_RPC_URL})"
  export VRF_ORACLE_SKIP_PREFLIGHT=true
  export RPC_URL="${BASE_RPC_URL}"
  export WEBSOCKET_URL="ws://127.0.0.1:${BASE_WS_PORT}"
  spawn vrf "${ER_RUN_DIR}/vrf.log" vrf-oracle
  sleep 3
  if ! kill -0 "$(cat "${ER_RUN_DIR}/vrf.pid")" 2>/dev/null; then
    err "vrf-oracle failed to start"
    exit 1
  fi
  log "vrf-oracle running"
fi

log "stack ready:"
log "  BASE_RPC_URL=${BASE_RPC_URL}"
log "  ROUTER_RPC_URL=${ROUTER_RPC_URL}   (rollup endpoint)"
log "  logs in ${ER_RUN_DIR}/{base,er,qfs}.log"

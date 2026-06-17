#!/usr/bin/env bash
# Boot the local MagicBlock stack: base + ephemeral validator + query-filtering-service.
#
# Requires the binaries from `npm install -g @magicblock-labs/ephemeral-validator@latest`
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

# --- base layer (mb-test-validator) -----------------------------------------------
log "starting base validator (mb-test-validator) on ${BASE_RPC_URL}"
mb-test-validator \
  --reset \
  --rpc-port "${BASE_RPC_PORT}" \
  --ledger "${ER_RUN_DIR}/base-ledger" \
  "${base_extra_args[@]}" \
  > "${ER_RUN_DIR}/base.log" 2>&1 &
echo $! > "${ER_RUN_DIR}/base.pid"
wait_for_rpc "${BASE_RPC_URL}" "base validator" 90

# --- ephemeral rollup validator ---------------------------------------------------
log "starting ephemeral-validator on ${ER_RPC_URL} (remotes -> ${BASE_RPC_URL})"
ephemeral-validator \
  --remotes "${BASE_RPC_URL}" \
  --listen "127.0.0.1:${ER_RPC_PORT}" \
  --lifecycle ephemeral \
  --no-tui \
  --reset \
  --storage "${ER_RUN_DIR}/er-storage" \
  > "${ER_RUN_DIR}/er.log" 2>&1 &
echo $! > "${ER_RUN_DIR}/er.pid"
wait_for_rpc "${ER_RPC_URL}" "ephemeral validator" 90

# --- query-filtering-service (router/proxy in front of the ER) --------------------
log "starting query-filtering-service on ${ROUTER_RPC_URL} -> ${ER_RPC_URL}"
query-filtering-service \
  --listen-addr "127.0.0.1:${ROUTER_RPC_PORT}" \
  --listen-addr-ws "127.0.0.1:${ROUTER_WS_PORT}" \
  --ephemeral-url "${ER_RPC_URL}" \
  --ephemeral-url-ws "ws://127.0.0.1:${ER_WS_PORT}" \
  --add-cors-headers \
  > "${ER_RUN_DIR}/qfs.log" 2>&1 &
echo $! > "${ER_RUN_DIR}/qfs.pid"
wait_for_rpc "${ROUTER_RPC_URL}" "query-filtering-service" 60

log "stack ready:"
log "  BASE_RPC_URL=${BASE_RPC_URL}"
log "  ROUTER_RPC_URL=${ROUTER_RPC_URL}   (rollup endpoint)"
log "  logs in ${ER_RUN_DIR}/{base,er,qfs}.log"

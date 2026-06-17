#!/usr/bin/env bash
# Shared helpers for the examples' local validator stack.
#
# Topology (validated):
#   base  mb-test-validator        RPC 8899 / WS 8900   (dlp, magic, vrf, acl, spl preloaded)
#   ER    ephemeral-validator      RPC 7799 / WS 7800   (--remotes -> base)
#   QFS   query-filtering-service  RPC 2999 / WS 3000   (client's rollup endpoint -> ER)
#
# Clients talk to the base directly (8899) and to the rollup through the
# query-filtering-service (2999).

set -euo pipefail

# Ports (override via env if needed).
export BASE_RPC_PORT="${BASE_RPC_PORT:-8899}"
export BASE_WS_PORT="${BASE_WS_PORT:-8900}"
export ER_RPC_PORT="${ER_RPC_PORT:-7799}"
export ER_WS_PORT="${ER_WS_PORT:-7800}"
export ROUTER_RPC_PORT="${ROUTER_RPC_PORT:-2999}"
export ROUTER_WS_PORT="${ROUTER_WS_PORT:-3000}"

export BASE_RPC_URL="http://127.0.0.1:${BASE_RPC_PORT}"
export ER_RPC_URL="http://127.0.0.1:${ER_RPC_PORT}"
export ROUTER_RPC_URL="http://127.0.0.1:${ROUTER_RPC_PORT}"
export ROUTER_WS_URL="ws://127.0.0.1:${ROUTER_WS_PORT}"

# Where to keep ledgers, logs and pidfiles. One dir per run.
export ER_RUN_DIR="${ER_RUN_DIR:-/tmp/er-examples-stack}"

log() { printf '\033[36m[stack]\033[0m %s\n' "$*" >&2; }
err() { printf '\033[31m[stack]\033[0m %s\n' "$*" >&2; }

# Poll a Solana RPC endpoint until it answers getVersion (or time out).
wait_for_rpc() {
  local url="$1" name="$2" timeout="${3:-60}" i=0
  log "waiting for ${name} at ${url} ..."
  until curl -s --connect-timeout 2 --max-time 2 -X POST "$url" \
      -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"getVersion"}' 2>/dev/null | grep -q '"result"'; do
    i=$((i + 1))
    if [ "$i" -ge "$((timeout * 2))" ]; then
      err "${name} did not become ready within ${timeout}s"
      return 1
    fi
    sleep 0.5
  done
  log "${name} is up"
}

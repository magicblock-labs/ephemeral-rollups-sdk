#!/usr/bin/env bash
# Run example tests the same way CI does (.github/workflows/examples.yml).
#
# Usage:
#   examples/scripts/run-ci-tests.sh                  # all CI examples
#   examples/scripts/run-ci-tests.sh counter-anchor   # one or more examples
#   examples/scripts/run-ci-tests.sh --skip-sdk-build # skip ts/web3js + ts/kit build
#
# Prerequisites (same as CI):
#   - Node.js, Rust 1.93.1, Solana CLI v3.1.10
#   - yarn global add @magicblock-labs/ephemeral-validator@latest

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=lib.sh
source "${SCRIPT_DIR}/lib.sh"

# Keep in sync with .github/workflows/examples.yml
RUST_VERSION="${RUST_VERSION:-1.93.1}"
SOLANA_VERSION="${SOLANA_VERSION:-v3.1.10}"

CI_EXAMPLES=(
  counter-anchor
  counter-pinocchio
  access-control
  vrf-anchor
  intent-bundle-pinocchio
  spl
  ephemeral-accounts-anchor
)

SKIP_SDK_BUILD=0
SELECTED=()

usage() {
  cat <<EOF
Usage: $(basename "$0") [--skip-sdk-build] [EXAMPLE ...]

Run example end-to-end tests using the same steps as CI.

With no EXAMPLE arguments, runs every example in the CI matrix:
  ${CI_EXAMPLES[*]}

Options:
  --skip-sdk-build   Skip building ts/web3js and ts/kit (use when already built)
  -h, --help         Show this help
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --skip-sdk-build) SKIP_SDK_BUILD=1; shift ;;
    -h|--help) usage; exit 0 ;;
    -*) err "unknown option: $1"; usage >&2; exit 1 ;;
    *) SELECTED+=("$1"); shift ;;
  esac
done

if [ "${#SELECTED[@]}" -eq 0 ]; then
  SELECTED=("${CI_EXAMPLES[@]}")
fi

step=0
run_step() {
  step=$((step + 1))
  printf '\033[34;01m\n────────────────────────────────────────────────────────────\n==> [%02d] %s\n────────────────────────────────────────────────────────────\033[0m\n\n' \
    "$step" "$1" >&2
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    err "missing required command: $1"
    exit 1
  fi
}

check_prerequisites() {
  run_step "checking prerequisites"
  require_cmd node
  require_cmd yarn
  require_cmd cargo
  require_cmd solana
  require_cmd mb-test-validator
  require_cmd ephemeral-validator
  require_cmd query-filtering-service

  log "node $(node --version)"
  log "rustc $(rustc --version 2>/dev/null || echo 'not on PATH')"
  log "solana $(solana --version | head -1)"

  if ! solana config get keypair >/dev/null 2>&1; then
    log "no default Solana keypair; creating one (same as CI)"
    solana-keygen new --silent --no-bip39-passphrase
  fi
}

build_ts_sdks() {
  if [ "$SKIP_SDK_BUILD" = 1 ]; then
    log "skipping TypeScript SDK build (--skip-sdk-build)"
    return
  fi

  run_step "building local TypeScript SDKs (ts/web3js, ts/kit)"
  for pkg in ts/web3js ts/kit; do
    log "building ${pkg}"
    (cd "${REPO_ROOT}/${pkg}" && yarn install --frozen-lockfile && yarn build)
  done
}

needs_vrf_oracle() {
  case "$1" in
    vrf*) return 0 ;;
    *) return 1 ;;
  esac
}

run_example() {
  local example="$1"
  local example_dir="${REPO_ROOT}/examples/${example}"
  local test_status=0
  local stack_started=0

  cleanup_stack() {
    if [ "$stack_started" = 1 ]; then
      log "stopping validator stack"
      "${SCRIPT_DIR}/stop-validators.sh" || true
    fi
  }
  trap cleanup_stack RETURN

  if [ ! -d "$example_dir" ]; then
    err "unknown example: ${example}"
    exit 1
  fi

  run_step "example: ${example}"

  if [ -f "${example_dir}/Cargo.toml" ]; then
    log "building SBF program"
    if ! (
      cd "$example_dir"
      RUSTUP_TOOLCHAIN="${RUST_VERSION}" "${SCRIPT_DIR}/build-program.sh" .
    ); then
      err "${example} build failed"
      return 1
    fi
  else
    log "no Cargo.toml; skipping program build"
  fi

  local deploy_dir="${example_dir}/target/deploy"
  local start_vrf=0
  if needs_vrf_oracle "$example"; then
    start_vrf=1
  fi

  log "starting validator stack (START_VRF_ORACLE=${start_vrf})"
  if ! START_VRF_ORACLE="$start_vrf" "${SCRIPT_DIR}/start-validators.sh" "$deploy_dir"; then
    err "${example} validator stack failed to start"
    return 1
  fi

  log "running yarn test in ${example_dir}"
  if ! (cd "$example_dir" && yarn install --frozen-lockfile && yarn test); then
    test_status=1
    err "${example} tests failed"
    err "validator logs: ${ER_RUN_DIR}/{base,er,qfs,vrf}.log"
  fi

  return "$test_status"
}

main() {
  local failed=()
  local example

  check_prerequisites
  build_ts_sdks

  for example in "${SELECTED[@]}"; do
    if ! run_example "$example"; then
      failed+=("$example")
    fi
  done

  if [ "${#failed[@]}" -gt 0 ]; then
    err "failed examples: ${failed[*]}"
    exit 1
  fi

  log "all selected examples passed"
}

main

#!/usr/bin/env bash
# Run prettier lint in every example (same as: cd examples/<name> && yarn lint).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

for example_dir in "${EXAMPLES_DIR}"/*/; do
  [ -f "${example_dir}/package.json" ] || continue
  example="$(basename "${example_dir}")"
  printf '\n==> lint %s\n' "${example}"
  (cd "${example_dir}" && yarn lint)
done

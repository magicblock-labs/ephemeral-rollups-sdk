#!/usr/bin/env bash
# Run prettier lint for example(s).
#
# Usage:
#   examples/scripts/lint.sh                  # all examples
#   examples/scripts/lint.sh counter-anchor   # one or more
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
SELECTED=("$@")

lint_example() {
  local example_dir="$1"
  local example
  example="$(basename "${example_dir}")"
  printf '\n==> lint %s\n' "${example}"
  (cd "${example_dir}" && yarn install --frozen-lockfile && yarn lint)
}

if [ "${#SELECTED[@]}" -eq 0 ]; then
  for example_dir in "${EXAMPLES_DIR}"/*/; do
    [ -f "${example_dir}/package.json" ] || continue
    lint_example "${example_dir}"
  done
else
  for example in "${SELECTED[@]}"; do
    example_dir="${EXAMPLES_DIR}/${example}"
    if [ ! -f "${example_dir}/package.json" ]; then
      printf 'unknown example: %s\n' "${example}" >&2
      exit 1
    fi
    lint_example "${example_dir}"
  done
fi

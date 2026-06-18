#!/usr/bin/env bash
# Build an example SBF program with the committed program-keypair.json so the
# deployed program id matches declare_id! / the TypeScript tests.
#
# Usage: examples/scripts/build-program.sh examples/<example>
set -euo pipefail

if [ $# -ne 1 ]; then
  printf 'usage: %s examples/<example>\n' "$(basename "$0")" >&2
  exit 1
fi

example_dir="$(cd "$1" && pwd)"
cd "$example_dir"

if [ ! -f Cargo.toml ]; then
  printf 'no Cargo.toml in %s\n' "$example_dir" >&2
  exit 1
fi

if [ ! -f program-keypair.json ]; then
  printf 'missing program-keypair.json in %s\n' "$example_dir" >&2
  exit 1
fi

lib_name="$(
  awk '/^\[lib\]/{found=1} found && /^name *=/{
    gsub(/[" ]/, "", $3); print $3; exit
  }' Cargo.toml
)"
if [ -z "$lib_name" ]; then
  printf 'could not read [lib].name from %s/Cargo.toml\n' "$example_dir" >&2
  exit 1
fi

mkdir -p target/deploy
cp program-keypair.json "target/deploy/${lib_name}-keypair.json"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
cargo build-sbf

#!/bin/bash

set -euo pipefail

color=true
build_count=0

if [[ "${1:-}" == "--no-color" ]]; then
    color=false
    shift
fi

if [[ $# -gt 0 ]]; then
    echo "usage: $0 [--no-color]" >&2
    exit 1
fi

print_command() {
    local command="$1"
    local line="────────────────────────────────────────────────────────────"

    build_count=$((build_count + 1))

    echo
    if [[ "$color" == true ]]; then
        printf '\033[34;01m%s\n==> [%02d] %s\n%s\033[0m\n' "$line" "$build_count" "$command" "$line"
    else
        printf '%s\n==> [%02d] %s\n%s\n' "$line" "$build_count" "$command" "$line"
    fi
    echo
}

build() {
    local features="${1:-}"

    if [[ -z "$features" ]]; then
        print_command "cargo build"
        cargo build
    else
        print_command "cargo build --features $features"
        cargo build --features "$features"
    fi
}

build_no_default() {
    local features="${1:-}"

    if [[ -z "$features" ]]; then
        print_command "cargo build --no-default-features"
        cargo build --no-default-features
    else
        print_command "cargo build --no-default-features --features $features"
        cargo build --no-default-features --features "$features"
    fi
}

build
build_no_default "solana-system-interface"
build_no_default "solana-system-interface,instruction"

build "vrf"

build "modular-sdk"
build "modular-sdk,access-control"
build "modular-sdk,spl"
build "modular-sdk,spl,access-control"

build "anchor"
build "anchor,vrf"
build "anchor,anchor-debug"
build "anchor,access-control"
build "anchor,access-control,anchor-debug"

build "access-control"

build "instruction"

build "encryption"

build "spl"
build "spl,access-control"

build "anchor,spl"
build "anchor,spl,anchor-debug"
build "anchor,spl,access-control"
build "anchor,spl,access-control,anchor-debug"

#!/bin/bash

set -euo pipefail

color=true

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

    echo
    if [[ "$color" == true ]]; then
        printf '\033[32m==> %s\033[0m\n' "$command"
    else
        printf '==> %s\n' "$command"
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

build "backward-compat"

build "modular-sdk"
build "modular-sdk,backward-compat"
build "modular-sdk,access-control"
build "modular-sdk,access-control,backward-compat"
build "modular-sdk,spl"
build "modular-sdk,spl,access-control"
build "modular-sdk,spl,access-control,backward-compat"

build "anchor"
build "anchor,anchor-debug"
build "anchor,access-control"
build "anchor,access-control,anchor-debug"

build "anchor-compat"
build "anchor-compat,access-control"

## anchor-lang-compat/anchor-debug currently fails inside anchor-lang 0.32 
## because of a bug in 0.32 itself, specially at this ##  line:
##
##    solana_program::msg!(...)
##
## But that is an stale/incorrect path, as it should have been:
##
##    crate::solana_program::msg!(...)
##
## Because solana_program is a modulde (not a crate) defined by anchor-lang 
## itself in lib.rs.
#
# build "anchor-compat,anchor-compat-debug"
# build "anchor-compat,access-control,anchor-compat-debug"

build "access-control"
build "access-control,backward-compat"

build "encryption"
build "encryption,backward-compat"

build "spl"
build "spl,backward-compat"
build "spl,access-control"
build "spl,access-control,backward-compat"

build "anchor,spl"
build "anchor,spl,anchor-debug"
build "anchor,spl,access-control"
build "anchor,spl,access-control,anchor-debug"
build "anchor-compat,spl"
build "anchor-compat,spl,access-control"

##
## anchor-compat,anchor-compat-debug wont work. see the elaborated comment above in this file.
##
# build "anchor-compat,spl,anchor-compat-debug"
# build "anchor-compat,spl,access-control,anchor-compat-debug"

# Unsupported by design: anchor targets current Anchor/Solana and must not be
# combined with backward-compat. Use anchor-compat for older Anchor support.
# build "anchor,backward-compat"
# build "anchor,access-control,backward-compat"
# build "anchor,spl,backward-compat"
# build "anchor,spl,access-control,backward-compat"

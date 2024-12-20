#!/bin/bash

set -e

# Step 1: Read the version from Cargo.toml
version=$(grep '^version = ' rust/Cargo.toml | head -n 1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$version" ]; then
    echo "Version not found in Cargo.toml"
    exit 1
fi

echo "Aligning for version: $version"

# GNU/BSD compat
sedi=(-i'')
case "$(uname)" in
  # For macOS, use two parameters
  Darwin*) sedi=(-i '')
esac

# Update the version for all crates in the Cargo.toml workspace.dependencies section
sed "${sedi[@]}" -e '/\[workspace.dependencies\]/,/## External crates/s/version = ".*"/version = "='$version'"/' rust/Cargo.toml

# Update the version in clients/bolt-sdk/package.json
jq --arg version "$version" '.version = $version' ts/package.json > temp.json && mv temp.json ts/package.json

# Potential for collisions in Cargo.lock, use cargo update to update it
cargo update --workspace --manifest-path rust/Cargo.toml

# Check if any changes have been made to the specified files, if running with --check
if [[ "$1" == "--check" ]]; then
    files_to_check=(
        "ts/package.json"
        "rust/Cargo.toml"
    )

    for file in "${files_to_check[@]}"; do
        # Check if the file has changed from the previous commit
        if git diff --name-only | grep -q "$file"; then
            echo "Error: version not aligned for $file. Align the version, commit and try again."
            exit 1
        fi
    done
    exit 0
fi

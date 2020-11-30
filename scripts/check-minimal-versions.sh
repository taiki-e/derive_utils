#!/bin/bash

# Check all public crates with minimal version dependencies.
#
# Usage:
#    bash scripts/check-minimal-versions.sh [+toolchain] [check|test] [options]
#
# Note:
# - This script modifies Cargo.toml and Cargo.lock while running
# - This script exits with 1 if there are any unstaged changes
# - This script requires nightly toolchain and cargo-hack
#
# Refs: https://github.com/rust-lang/cargo/issues/5657

set -euo pipefail
IFS=$'\n\t'

cd "$(cd "$(dirname "${0}")" && pwd)"/..

# Decide Rust toolchain.
# Nightly is used by default if the `CI` environment variable is unset.
if [[ "${1:-}" == "+"* ]]; then
  toolchain="${1}"
  shift
elif [[ -z "${CI:-}" ]]; then
  toolchain="+nightly"
fi
# Make sure toolchain is installed.
cargo ${toolchain:-} -V >/dev/null
if [[ "${toolchain:-+nightly}" != "+nightly"* ]] || ! cargo hack -V &>/dev/null; then
  echo "error: this script requires nightly toolchain and cargo-hack"
  exit 1
fi

# Decide subcommand.
subcmd="check"
if [[ "${1:-}" =~ ^(check|test)$ ]]; then
  subcmd="${1}"
  shift
fi

# This script modifies Cargo.toml and Cargo.lock, so make sure there are no
# unstaged changes.
git diff --exit-code
# Restore original Cargo.toml and Cargo.lock on exit.
trap 'git checkout .' EXIT

if [[ "${subcmd}" == "check" ]]; then
  # Remove dev-dependencies from Cargo.toml to prevent the next `cargo update`
  # from determining minimal versions based on dev-dependencies.
  cargo hack --remove-dev-deps --workspace
fi

# Update Cargo.lock to minimal version dependencies.
cargo ${toolchain:-} update -Z minimal-versions
# Run check for all public members of the workspace.
cargo ${toolchain:-} hack "${subcmd}" \
  --workspace --all-features --ignore-private -Z features=all \
  "$@"

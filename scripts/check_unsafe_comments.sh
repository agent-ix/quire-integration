#!/usr/bin/env bash
# Enforce the repository's zero-unsafe policy for every Rust target.
set -euo pipefail

export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }-Funsafe-code"
"${CARGO:-cargo}" check --all-targets --locked

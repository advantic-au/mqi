#!/usr/bin/env bash
set -euo pipefail
set -x

cd "$(dirname "${BASH_SOURCE[0]}")/.."
export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }-Dwarnings"

run_clippy() {
    cargo clippy "$@"
}

run_clippy --tests --examples --all-features
run_clippy --tests --examples
run_clippy --tests --examples --no-default-features --features mock,exits,link
run_clippy --tests --examples --no-default-features --features mock,mqai,link
run_clippy --tests --examples --no-default-features --features mock,mqai,exits,dlopen2
run_clippy --tests --examples --no-default-features --features mock,link

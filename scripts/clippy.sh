#!/usr/bin/env bash
set -euo pipefail
set -x

cd "$(dirname "${BASH_SOURCE[0]}")/.."
export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }-Dwarnings"

# Keep these API-documentation lints visible in the report below while cleaning them up gradually.
run_clippy() {
    cargo clippy "$@" -- -A clippy::missing_errors_doc -A clippy::missing_panics_doc
}

run_clippy --tests --examples --all-features
run_clippy --tests --examples
run_clippy --tests --examples --no-default-features --features mock,exits,link
run_clippy --tests --examples --no-default-features --features mock,mqai,link
run_clippy --tests --examples --no-default-features --features mock,mqai,exits,dlopen2
run_clippy --tests --examples --no-default-features --features mock,link

if [[ "${REPORT_DOC_LINTS:-true}" == "true" ]]; then
    env -u RUSTFLAGS cargo clippy --lib --all-features -- -W clippy::missing_errors_doc -W clippy::missing_panics_doc
fi

# Contributing

Thanks for your interest in improving `mqi`. Contributions, bug reports and constructive
feedback are welcome. Please follow the project's [Code of Conduct](CODE_OF_CONDUCT.md).

## Before you start

- Read [DESIGN.md](DESIGN.md) for the principles that shape the public API.
- Check the existing [issues](https://github.com/advantic-au/mqi/issues) before starting
  substantial work. For a large API change, open an issue first to discuss the design.
- This crate is the safe Rust API. Raw MQ FFI declarations and generated bindings belong in
  [`libmqm-sys`](https://github.com/advantic-au/libmqm-sys).

## Prerequisites

Building and testing requires an IBM MQ client or server installation:

- Set `MQ_HOME` to its install directory (defaults to `/opt/mqm` on Linux and
  `C:\Program Files\IBM\MQ` on Windows).
- Make the MQ shared library available to the system loader. On Linux, for example, set
  `LD_LIBRARY_PATH=$MQ_HOME/lib64`.
- Install Rust 1.95 or newer. The project will not raise its MSRV to a stable Rust release until
  that release has been available for at least six months; see the
  [MSRV policy](README.md#version-compatibility).
- `cargo-nextest` is used by CI for unit and integration tests.

A local queue manager can be run with the IBM MQ container:

```sh
docker run -d --publish 1414:1414 -e LICENSE=accept -e MQ_APP_PASSWORD=app \
  icr.io/ibm-messaging/mq:10.0.1.0-r1
```

The IBM MQ license must be accepted. See the
[container usage guide](https://github.com/ibm-messaging/mq-container/blob/master/docs/usage.md).

## Build, test and lint

The default `link` feature needs the MQ client library at build time. CI's
[workflow](.github/workflows/ci.yml) is the authoritative list of checks and feature combinations.
Common local checks include:

```sh
cargo check --all-features
cargo nextest run --profile ci --all-features
cargo test --all-features --doc
cargo fmt -- --check
cargo clippy --tests --examples --all-features --
cargo docs-rs
```

Run checks with the relevant feature combinations when changing feature-gated code. CI treats
warnings as errors. Tests that connect to a real queue manager require the MQ connection settings
described in the workflow; do not use production credentials or services for local tests.

## Pull requests

- Target the `develop` branch.
- Describe the motivation, user-visible behavior, and relevant API or feature changes.
- Add or update tests and documentation when behavior changes.
- Run the relevant formatting, lint, build and test checks, and report which checks you ran.
- Avoid unrelated changes. Public API changes should follow the design principles and be discussed
  early if they are substantial.
- Use conventional commit messages. `release-plz` uses them to prepare versions and changelog
  entries; do not edit versions or `CHANGELOG.md` by hand.

Pull requests are reviewed on a best-effort basis. Please keep discussion respectful and
constructive.

## Reporting problems

- Usage questions: open an issue using the **Question** form and include a minimal example and
  relevant MQ client, Rust, target, and feature information.
- Bugs and feature requests: use the corresponding issue forms.
- Security vulnerabilities: report them privately as described in [SECURITY.md](SECURITY.md).
- Conduct concerns: contact the maintainer as described in [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

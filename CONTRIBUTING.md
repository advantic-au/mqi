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

An IBM MQ client installation is required to build or test `mqi`: the build needs the MQ headers,
and the default `link` feature needs the MQ shared library. An MQ server installation can also be
used as a client install.

- Set `MQ_HOME` to the install directory (defaults to `/opt/mqm` on Linux and
  `C:\Program Files\IBM\MQ` on Windows).
- Make the MQ shared library available to the system loader. On Linux, for example, run
  `export LD_LIBRARY_PATH="$MQ_HOME/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"`.
- Install Rust 1.95 or newer. The project will not raise its MSRV to a stable Rust release until
  that release has been available for at least six months; see the
  [MSRV policy](README.md#version-compatibility).
- Add the Rust formatting and lint components, and install the test runner used by CI:

  ```sh
  rustup component add rustfmt clippy
  cargo install cargo-nextest --locked
  ```

A queue manager is **not** needed to compile, run unit tests, or run doctests. It is only needed
for tests that connect to a real MQ server. For those tests, start a local developer queue manager
with the IBM MQ container:

```sh
docker run --detach --rm --name QM1 --publish 1414:1414 \
  --env LICENSE=accept --env MQ_APP_PASSWORD=app \
  icr.io/ibm-messaging/mq:10.0.1.0-r1
```

Then configure the local test connection:

```sh
export MQSERVER='DEV.APP.SVRCONN/TCP/localhost(1414)'
export MQ_USER=app
export MQ_PASSWORD=app
```

The IBM MQ license must be accepted. See the
[container usage guide](https://github.com/ibm-messaging/mq-container/blob/master/docs/usage.md)
and [default developer configuration](https://github.com/ibm-messaging/mq-container/blob/master/docs/developer-config.md).

## Build, test and lint

For a first contribution, read [DESIGN.md](DESIGN.md), make a focused change, then run the checks
that cover it. These common checks require the MQ client install above, but not a queue manager:

```sh
cargo check --all-features
cargo nextest run --profile ci --features mock
cargo test --doc
cargo fmt -- --check
bash scripts/clippy.sh
cargo +nightly docs-rs
```

`cargo docs-rs` requires nightly Rust and the `cargo-docs-rs` tool. Install them with:

```sh
rustup toolchain install nightly
cargo install cargo-docs-rs --locked
```

Run checks with the relevant feature combinations when changing feature-gated code. CI's
[workflow](.github/workflows/ci.yml) is the authoritative list of checks and combinations; it treats
warnings as errors, except missing `# Errors` and `# Panics` sections, which are reported as
warnings during their gradual cleanup.

### Coverage and mutation testing

Coverage and mutation testing supplement the regular test suite; neither is a substitute for
reviewing test behavior. CI uploads coverage to Codecov, and PRs also get an advisory,
diff-scoped mutation run. The mutation run does not currently fail CI.

To reproduce the coverage report locally, install `cargo-llvm-cov` and use the same test and
doctest feature set as CI:

```sh
cargo install cargo-llvm-cov --locked
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report nextest --profile ci --features mqai,dlopen2,mqc_latest,mock
cargo llvm-cov --no-report test --doc --features mqai,dlopen2,mqc_latest,mock
cargo llvm-cov report --doctests --ignore-filename-regex "build/.+"
```

Coverage helps locate code and branches that tests do not exercise. Prioritize meaningful behavior,
especially error handling and boundary cases, rather than aiming for a percentage or adding tests
that only execute lines.

For a local mutation run on the changes relative to `develop`, fetch the branch if needed, then:

```sh
cargo install cargo-mutants --locked
git diff origin/develop > git.diff
cargo mutants --no-shuffle -vV --in-diff git.diff --all-features
```

Review surviving mutants to decide whether they reveal a missing assertion or represent equivalent,
unreachable, or otherwise irrelevant behavior. CI preserves the mutation output as an artifact.

### Adding an MQ client API version

When adding support for a client version, keep the feature chain and supported-version claims in
sync:

- Add the `mqc_*` feature to the chain in [Cargo.toml](Cargo.toml), forwarding it to the relevant
  `libmqm-*` dependencies and updating `mqc_latest` when this is the newest supported version.
- Add or update a matching client-version/feature entry in the `client_check` matrix in
  [ci.yml](.github/workflows/ci.yml).
- Update the supported version range, default, and examples in the
  [README compatibility section](README.md#version-compatibility).
- Run the client check with an installed client of that version, and check feature combinations
  that use the new APIs.

To run the MQ server integration test, use the local connection settings above and the same feature
set as CI:

```sh
cargo nextest run --profile ci --no-default-features --features link,mqai,exits,mqc_latest
```

Do not use production credentials or services for local tests.

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

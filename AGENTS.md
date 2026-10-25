# AGENTS.md

Guidance for agents working in this repository.

## What this repository is

`mqi` is a **safe, idiomatic Rust wrapper** over the IBM MQ C APIs (MQI and MQAI). It is the
companion crate to `libmqm-sys`, which provides the raw `unsafe` FFI bindings, structures,
constants and the `Mqi` / `Mqai` / `Exits` traits.

The split matters: **all `unsafe` FFI lives in `libmqm-sys`, not here.** This crate's job is to
turn those calls into ergonomic Rust - lifetimes, newtypes, `Result`s, and type inference. If you
find yourself wanting to add raw FFI here, the change probably belongs upstream instead.

Almost every design decision follows from four principles recorded in `DESIGN.md`:

| Id | Principle | Means |
|---|---|---|
| D1 | Minimal prelude | `mqi::prelude::*` re-exports only the traits you cannot call the API without. Do not grow it casually. |
| D2 | Newtype pattern | MQ "types" are newtypes over the C integer (e.g. `MQCO`, `MQOO`, `MQRC`), never bare `MQLONG`. |
| D3 | Type inference for return types | `connect_as`, `open_as`, `get_as`, `*_with` let the caller's binding choose the return type via an `*Value` / `*Attr` trait. |
| D4 | Traits as parameters | Options are passed as `&impl SomeOption` where `SomeOption` is implemented for tuples of sub-options (see below). |

## Prerequisites

An IBM MQ client (or server) install is **required** to build or test anything, because the default
`link` feature links the MQ shared library at compile time.

- `MQ_HOME` must point at the install (default `/opt/mqm`, or `c:/Program Files/IBM/MQ`).
- Put the MQ shared library on the loader path at runtime, e.g.
  `LD_LIBRARY_PATH=$MQ_HOME/lib64` on Linux (`libmqm.so`). Required for `dlopen2`-loaded and
  link-time-linked tests alike.
- Rust 1.95 minimum (see `rust-version` in `Cargo.toml`; edition 2024).
- `cargo-nextest` is the test runner CI uses; `cargo-mutants`, `cargo-llvm-cov` and
  `cargo-semver-checks` are used by specific workflows.

A quick local queue manager for experiments:
`docker run -d --publish 1414:1414 icr.io/ibm-messaging/mq:latest`

## Commands

There is no Makefile; everything is cargo. `.github/workflows/ci.yml` is the authoritative source
for the exact invocations that must pass.

```sh
# check (CI does this across client versions 9.3 / 9.4)
cargo check --features mqc_9_3_5_0,exits,mqai,dlopen2
cargo check --features mqc_9_4_3_0,exits,mqai,dlopen2

# test - feature combinations are NOT optional, each of these is a separate CI job
cargo nextest run --profile ci --all-features
cargo test --all-features --doc
cargo nextest run --profile ci --features mock          # default features + mock
cargo nextest run --profile ci --no-default-features --features mock

# integration test against a real queue manager (the test_server CI job)
MQSERVER=DEV.APP.SVRCONN/TCP/localhost(1414) MQ_USER=app MQ_PASSWORD=app \
  cargo nextest run --profile ci --no-default-features --features link,mqai,exits,mqc_latest

# format / lint
cargo fmt -- --check
cargo clippy --tests --examples --all-features --

# docs (must be warning-free; CI sets RUSTDOCFLAGS=-Dwarnings on nightly)
cargo docs-rs

# coverage (CI runs nextest AND doctests under llvm-cov; both reports use --doctests)
cargo llvm-cov --no-report nextest --profile ci --features mqai,dlopen2,mqc_latest,mock
cargo llvm-cov --no-report test --doc --features mqai,dlopen2,mqc_latest,mock
cargo llvm-cov report --doctests --ignore-filename-regex "build/.+" --codecov --output-path coverage/codecov.json
cargo llvm-cov report --doctests --ignore-filename-regex "build/.+"

# mutation testing (PR incremental diff in ci.yml; full manual run in mutants.yml)
cargo mutants --no-shuffle -vV --in-diff git.diff --all-features
cargo mutants -vV --in-place --all-features
```

The `ci` nextest profile is in `.config/nextest.toml` (note: `.config/`, not `.cargo/`) and only
sets `fail-fast = false`.

Examples require specific features, declared in `Cargo.toml` - e.g.
`cargo run --example readme --features link`, `cargo run --example dlopen --features dlopen2`.
`examples/args/mod.rs` is the shared clap `ConnectionArgs` parser used by the examples; extend it
rather than duplicating connection CLI flags.

## Architecture and data flow

Four layers, each in its own place:

1. **`libmqm-sys`** (external crate). Raw `unsafe` FFI: `Mqi` / `Mqai` / `Exits` traits,
   C structures, `LinkedMq` (compile-time link), `MqWrapper` (runtime load), `MockMq`.
2. **`src/verb/`** - `mqi.rs`, `mqai.rs`, `exits.rs`. Defines `impl<L: Library<MQ: Mqi>> MqFunctions<L>`
   and the MQAI equivalent. Each method calls one raw MQ verb, records the result in an
   `MqiOutcome<T>` (`verb` name + `cc` + `rc` + value), optionally traces it, and converts it with
   `.into()` into a `ResultComp<T>`. There is exactly one wrapper method per MQ verb, and these are
   named after the C verb (`mqconnx`, `mqopen`, `mqget`, `mq_setmp`, ...). This layer holds the FFI
   calls; a few other modules also contain small `unsafe` blocks (`src/bag/bag_item.rs`,
   `src/properties/param.rs`), so `unsafe` is not exclusively confined here.
3. **Public API modules**. One module per MQ concept: `connection`, `open`, `get`, `put`,
   `properties`, `subscription`, `attribute`, `stat`, `bag`, `execute`, plus the container types
   `Object`, `Syncpoint`, `EventCallback`, `Connection`. These own the user-facing ergonomics and
   encapsulate the raw handles behind `Drop`.
4. **`src/support/`** - `conversion`, `encoding`, `macros`, `outcome`: shared internals
   (`MqiOutcome`, conversion traits, encoding/CCSID machinery, tuple-impl macros). Not public API
   except `outcome` re-exported pieces.

`PROGRESS.md` tracks which IBM MQ verbs and MQAI functions are wrapped and at what API status;
update its table when you add or change a verb wrapper (it is linked from the README).

Data flow for one call, e.g. `Object::put_message`:

```
user options (tuple of impl PutOption)
  -> Option::apply_param(&mut PutParam)     // src/put/option.rs, users never impl this directly
  -> MqFunctions::mqput(...)                // src/verb/mqi.rs, unsafe FFI
  -> MqiOutcome<()> -> ResultComp<()>       // cc/rc -> Completion or Error
  -> user gets Completion<T> (warnings) or Error
```

### The option / param / function triple

Nearly every module repeats the same three-file shape. Follow it for any new API surface:

- **`param.rs`** - the `*Param` struct holding the MQ structures (from `structs.rs`) plus parsed
  values, and the `*Option` / `*Value` / `*Attr` traits with their `apply_param` methods.
- **`option.rs`** - the user-facing option types (usually newtypes over MQ constants or a struct)
  that implement the trait from `param.rs`. Contains doctests illustrating custom implementations.
- **`function.rs`** - the methods on `Object` / `Connection` / `Subscription` / `Bag`.

Option traits are implemented for `Option<T>`, `()`, and tuples of arity 2-8 by the
`all_option_tuples!` / `impl_option_tuple!` macros in `src/support/macros.rs`. That is why callers
write `&(QueueName(...), MQOO_INPUT_SHARED | MQOO_OUTPUT)` and why adding a 9th option type would
require extending `all_multi_tuples!` there. `#[diagnostic::do_not_recommend]` is applied on
generated impls to keep compiler errors readable.

### Error and warning handling - a core convention

This is the most important non-obvious API rule:

- `Completion<T>(T, Option<(MQRC, &'static str)>)` wraps every successful result. MQ treats
  "worked, but with a warning" as `MQCC_WARNING`, not `MQCC_OK`; that warning is preserved here.
- `Error(MQCC, &'static str /* verb */, MQRC)` is the error type. The verb string is how users see
  which MQ call failed.
- Type aliases in `src/result.rs`: `ResultComp<T>` (= `Result<Completion<T>, Error>`),
  `ResultCompErr<T, E>` (custom error), `ResultErr<T>`.
- `ResultCompExt::warn_as_error()` turns a warning into `Err`; `discard_warning()` throws it away;
  `warning()` inspects it. `ResultCompErrExt::already_connected_ref()` is special-cased to avoid
  closing a connection on `MQRC_ALREADY_CONNECTED`.
- **Public API functions return `ResultComp<...>` and deliberately do not choose a warning policy
  for the caller.** Preserve this: the caller decides via the `*Ext` traits.

### Containers, handles and `Drop`

- `src/handle.rs` defines `Handle<raw::Kind>` newtypes (`ConnectionHandle`, `ObjectHandle`,
  `MessageHandle`, `BagHandle`) wrapping the C handles, with `is_closeable()` and `mut_raw_handle()`.
  The `raw` module is where the `*HandleType` mapping to `MQHCONN` / `MQHOBJ` / etc. lives.
- `Connection`, `Object`, `Subscription`, `Bag`, `Properties`, `Syncpoint` and `EventCallback` all
  clean up in `Drop` (disconnect / close / delete bag / delete message handle / back out the unit of
  work / deregister the handler).
  Closing explicitly consumes `self` (`pub fn close(self)`), relying on `Drop` only as the fallback.
  `Object::close_options(&mut self, MQCO)` lets the user change the options used at drop time.
- `Connection<L, H>` is generic over the library `L` and the threading marker `H`
  (`ThreadNone` = `!Send + !Sync`, `ThreadNoBlock` = `Send + !Sync`, `ThreadBlock` = `Send + Sync`).
  The marker selects `MQCNO_HANDLE_SHARE_*`. `ConnectionRef` is a non-owning borrowed handle;
  `ConnectionEither` holds either an owned or borrowed connection (used by the
  `MQRC_ALREADY_CONNECTED` path); `Connection::leak` promotes to a `'static` ref.
- `src/structs.rs` defines type aliases over the `libmqm_sys` C structures with a `'ptr` lifetime so
  pointers into user data can be attached; `src/struct_attach.rs` holds the `attach_*` methods. The
  `libmqm_default` crate supplies the `*_DEFAULT` initialisers (referenced as `default::*` in the
  option/param modules). Structures carry a `Version`; code asserts the version does not exceed
  `libmqm_sys::<X>_CURRENT_VERSION` before the call. When you add an option that needs a newer
  structure version, call `set_min_version(...)` (`src/struct_attach.rs` and the option modules).

### Strings and CCSID

`MqStr<N>` (from `mqstr!`, backed by the `MqChar<N>` array alias) is the compile-time,
fixed-size MQ string newtype. `StringCcsid<T>` in `src/string/str_ccsid.rs` carries an explicit
CCSID alongside the bytes, with the aliases `StrCcsid<'a>` / `StrCcsidOwned` / `StrCcsidCow<'a>`
for the borrowed/owned/`Cow` cases; `EncodedString` is the trait the API takes by
`&(impl EncodedString + ?Sized)`. `CCSID` is the newtype over the C codepage number.
`src/support/encoding.rs` holds the CCSID encoding/decoding engine (with the only proptest in the
repo). `mqstr!` expands to `const { MqStr::def_from_str($val) }`, so `mqstr!("...")` works in
`const` position, as used for the `QueueName` constants in tests.

## Feature flags

`default = ["link", "mqc_9_3_0_0"]`. The flags are non-trivial and combinatorial:

- **`link`** (default) vs **`dlopen2`**. `link` resolves the MQ library at compile time and is what
  `mqi::connect` / `connect_as` / `connect_with` use (they are `link`-only). `dlopen2` loads at
  runtime; use `connect_lib` / `connect_lib_as` / `connect_lib_with` with a loaded container.
- **`mqc_*`** version features form a **chain**: each implies all the lower ones
  (`mqc_9_4_2_0` -> `mqc_9_4_1_0` -> ... -> `mqc_9_2_0_0`). `mqc_latest` currently resolves to
  `mqc_9_4_5_0`. Enabling a feature above the installed client version makes the upstream
  `libmqm-sys` build script fail with *"MQC version ... does not meet the minimum requirement for
  feature ..."*.
- **`mqai`**, **`exits`**, **`tracing`**, **`mock`** gate whole modules and are all off by default
  except where implied. `Bag`, `execute` and the `mqai` verb module only exist with `mqai`.
- **`docsrs`** is a bundle of everything, used by `cargo docs-rs` and docs.rs
  (`[package.metadata.docs.rs]` disables default features and enables `docsrs`).

Feature documentation is generated: `document_features::document_features!()` in `src/lib.rs`
reads the `##` doc comments in `Cargo.toml`. To document a new feature, add a `##` comment above it
there - do not add a hand-written table to the README.

## Testing

- Two families of integration tests live in `tests/`:
  - **Mock-based** - `connection.rs`, `properties.rs` and `subscribe.rs` are whole-file gated
    `#![cfg(feature = "mock")]`. `cb.rs`, `object.rs` and `put_get_bag.rs` mix per-test
    `#[cfg(feature = "mock")]` / `#[cfg(not(feature = "mock"))]` arms in one file; `object.rs` is
    additionally gated on `link`/`dlopen2` and `put_get_bag.rs` on `mqai`. `admin.rs` requires
    `mqai`, `mock` and `link`/`dlopen2`. Mock arms need no queue manager.
  - **Server-based** - the `#[cfg(not(feature = "mock"))]` arms connect to a real queue manager
    using `MQSERVER` / `MQ_USER` / `MQ_PASSWORD` (see `src/test/credentials.rs`). CI runs these in
    the `test_server` job against the `icr.io/ibm-messaging/mq` container.
- `tests/dlopen2.rs` is gated on the `dlopen2` feature; `tests/exits.rs` requires both `mock` and
  `exits` (`#![cfg(all(feature = "mock", feature = "exits"))]`).
- Unit tests are inline `#[cfg(test)] mod test(s)` blocks in `src/`, normally tagged
  `#[cfg_attr(coverage_nightly, coverage(off))]`. Keep that attribute on any new test module.
- The **`mqi::test`** module (`src/test/`) is the mock toolkit. It is `#[doc(hidden)]` but compiled
  unconditionally - it is public API for downstream tests and examples, not `#[cfg(test)]`-only:
  - `mock::connect_ok(|mock| { ... })` builds a `Connection<Rc<MockMq>, ThreadNone>` with `MQCONNX` /
    `MQDISC` already stubbed.
  - `mock::open_ok`, `mock::get_ok`, `mock::get_error`, `mock::subscribe_managed_ok`,
    `mock::properties_ok`, `mock::mqai::*` stub individual verbs and register the matching close
    calls. They take a `mockall::Sequence`, so ordering is asserted.
  - `mock::mqi_outcome*` set `CompCode` / `Reason` on a raw call.
  - `test::mq_library()` returns the real linked library (or a `DebugContainer` wrapping a
    `dlopen2` `MqmContainer` when only `dlopen2` is enabled).
  - `src/test/mock/callback.rs` contains a fairly involved `MQCB` mock that actually invokes the
    registered C callback - reuse it rather than re-deriving it when testing event handlers.
- `proptest-regressions/mq/encoding.txt` holds committed regression seeds for the proptest in
  `src/support/encoding.rs`. Do not delete it; it is meant to be committed.
- `benches/all.rs` is a stub (`divan::main()` is commented out, `divan` is not a dependency). Do not
  assume benchmarks run.

## Code style and conventions

- `rustfmt.toml` sets `max_width = 130`, Unix newlines, edition 2024, `reorder_imports`,
  `use_field_init_shorthand`. Run `cargo fmt`.
- Lints are strict: `clippy::all` is **deny**, `nursery` + `pedantic` are warn,
  `rust::unsafe_op_in_unsafe_fn` is warn, and CI compiles clippy with `RUSTFLAGS=-Dwarnings`.
  So every clippy variant (six different feature combinations in the `clippy` job) must be warning-free.
- Lint suppression: prefer **`#[expect(lint, reason = "...")]`** for targeted cases. Existing code is
  mixed - many `#[expect(...)]` sites omit `reason` (e.g. `src/open/param.rs`, `src/string/ccsid.rs`).
  A bare `#[allow]` trips the `clippy::allow_attributes` warning (which `clippy::all = deny` makes an
  error), so when `#[allow]` is genuinely required (generated code, `non_snake_case`, false
  positives) it is used inline with the warning allowed, e.g.
  `#[allow(clippy::allow_attributes, clippy::missing_const_for_fn)]` in `src/mq_type.rs:138`, or
  under a file-level `#![expect(clippy::allow_attributes, ...)]` prologue in `src/connection/param.rs`,
  `src/get/param.rs`, `src/properties/param.rs` and `src/test/mock/mod.rs`. Add a `reason` to new
  suppressions.
- `missing_errors_doc` and `missing_panics_doc` are explicitly allowed in `Cargo.toml` with a TODO -
  do not "fix" that without removing the TODO.
- Doc comments are heavy and load-bearing: many doctests demonstrate how to implement the
  `*Value` / `*Option` traits yourself (`src/get/option.rs` is a good example). Doctests are run
  (`cargo test --doc`) and must compile under `--no-default-features` too.
- Public items need docs; `cargo docs-rs` runs with `RUSTDOCFLAGS=-Dwarnings`.
- `#[must_use]` on constructors and container types is used consistently. Match it.

## Gotchas

- **`dlopen2` and `mockall` must stay in lockstep with the versions `libmqm-sys` resolves.** The
  generated `MqWrapper` and `MockMq` types only work when both crates share one crate instance, so
  bumping either here ahead of a `libmqm-sys` release breaks the build in ways that look unrelated:
  `E0277` (`MqWrapper: WrapperApi`, `Container<MqWrapper>: Library` / `MqaiLibrary`), and
  `E0308` / `E0277` on `mockall::Sequence` / `TimesRange`. `cargo tree -i dlopen2` (or `mockall`)
  reports an *ambiguous specification* when two versions are in the tree. Bump these two only
  alongside a `libmqm-sys` release that carries the same bump - `derive_more`, `proptest`, `clap`
  and `ctrlc` have no such coupling and can move independently.
- **`derive_more::Constructor` generates `field: field` and trips `clippy::redundant_field_names`
  (which `clippy::all = deny` turns into an error).** It proxy-passes `#[allow]`/`#[expect]`
  attributes from the struct onto the generated `impl`, but `#[expect]` there is reported as
  *unfulfilled* (it is duplicated onto the struct, where the lint never fires), so the working
  form is `#[allow(clippy::redundant_field_names, reason = ...)]` - which then needs the file-level
  `#![expect(clippy::allow_attributes, ...)]` prologue.
- **`Cargo.lock` is gitignored and not committed.** A stale local `Cargo.lock` can also pin a
  dependency mix that no longer matches `Cargo.toml`; if dependency errors look impossible,
  regenerate it. The CI `minimum` job intentionally regenerates it with `-Z minimal-versions` and
  builds on Rust 1.95.
- **`mock` is a first-class feature, not only a dev-dependency toggle.** It forwards to
  `libmqm-sys/mock` and is combined with `link`, `dlopen2`, `mqai`, `exits` in CI in six different
  clippy permutations. Renaming or regating anything under `src/test/` must keep all of them working.
- **`--no-default-features` disables `link`**, so `mqi::connect*` disappears and tests must use
  `connect_lib` with `test::mq_library()` instead. Keep both arms compiling.
- **MQ handles are `Copy` newtypes but not `Send`/`Sync`**; thread safety is a type parameter on
  `Connection`, not an attribute on handles. Do not add `unsafe impl Send` to a handle.
- Option ordering in tests matters: `mock::*_ok` helpers take a `mockall::Sequence`, so inserting an
  extra MQ call in the implementation breaks tests that only look like they set up mocks.
- `#[cfg(coverage)]` / `#[cfg(coverage_nightly)]` are declared in the `check-cfg` list in
  `Cargo.toml`. If you add a new cfg name, add it there or the build warns (and CI denies warnings).

## CI, releases and repo workflow

- PRs target **`develop`**, which is also where pushes trigger `release-plz` and the CI `push` /
  `pull_request` triggers. Do not assume `main` is the integration branch.
- Workflows: `ci.yml` (check / test / test_server / miri / mutants / rustfmt / clippy / minimum /
  docs), `coverage.yml` (llvm-cov to Codecov), `mutants.yml` (manual, full), `release-plz.yml`,
  `semver-check.yml` (posts a `cargo-semver-checks` comment on PRs),
  `static-analysis.yml` (zizmor on workflow files - third-party actions must stay pinned to SHAs,
  policy in `.github/zizmor.yml`).
- `miri` runs with `-Zmiri-strict-provenance`, `PROPTEST_CASES=64` and failure persistence disabled,
  on `x86_64-unknown-linux-gnu` and `s390x-unknown-linux-gnu`. Big-endian matters: do not assume
  little-endian or native `usize` width in encoding/conversion code.
- **release-plz** drives versioning and `CHANGELOG.md`, triggered on push to `develop`. Do not
  hand-edit the version or the changelog - write conventional commits and let the release PR do it.
  (Commits like `feat:`, `fix:`, `chore:`, `build(deps):` are already the norm; dependabot adds
  `build(deps)` itself.)
- Dependabot runs weekly for cargo and GitHub Actions with a cooldown.

## Local environment overrides

Machine-specific setup (MQ client install paths, the sibling `libmqm-sys` checkout, and local
Jujutsu/Git workflow notes) lives in `AGENTS.local.md`. That file is gitignored and must not be
committed. If it exists in the working tree, read it before building or testing; its local paths
take precedence over the generic defaults above.

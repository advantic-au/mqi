# Security policy

## Supported versions

Maintainers intend to address security vulnerabilities in the latest published release of `mqi`
and on the `develop` branch. Older releases are not patched; upgrade to the latest release to
receive fixes.

Security maintenance is best-effort. There is no guaranteed response or fix timeline, and this
policy does not promise general support or a release schedule.

## Reporting a vulnerability

Please report vulnerabilities privately through the repository's **Security** tab using
**Report a vulnerability**. This creates a private security advisory visible only to the
reporter and repository maintainers. Do not report security issues in public issues or pull
requests.

Include, when possible:

- The `mqi` version or git revision, enabled Cargo features, target triple, and Rust version.
- The IBM MQ client version and relevant queue manager configuration.
- The security impact and a minimal reproduction.
- Whether the issue appears to involve `mqi`, `libmqm-sys`, IBM MQ, or a dependency.

Do not include real credentials, private keys, production message contents, or other sensitive
data in a report. Maintainers will investigate on a best-effort basis and coordinate disclosure
with the reporter.

## Scope

In scope:

- The safe API and its guarantees, including memory safety, resource ownership, and validation of
  data passed between Rust and the MQ C APIs.
- Hand-written code in this repository and its integration with `libmqm-sys`, including issues
  that could cause a safe caller to trigger undefined behavior or expose protected data.
- Project automation in `.github/workflows` where a vulnerability could affect this repository
  or its users.

Out of scope:

- Vulnerabilities in IBM MQ or the IBM MQ client libraries. Report these to IBM.
- Vulnerabilities in `libmqm-sys` itself. Report them privately through that repository's
  [security reporting page](https://github.com/advantic-au/libmqm-sys/security/policy).
- Vulnerabilities in third-party dependencies unrelated to how `mqi` uses them. Report these to
  the dependency maintainers; if the issue creates a specific risk to `mqi` users, include that
  context in a private report here as well.
- Incorrect use of the API that violates its documented requirements.

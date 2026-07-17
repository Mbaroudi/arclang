# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately via
[GitHub Security Advisories](https://github.com/Mbaroudi/arclang/security/advisories/new)
rather than opening a public issue. You should receive an acknowledgement
within 7 days.

## Supply chain

- Dependencies are audited in CI on every push (`cargo audit` against the
  RustSec advisory database).
- Release binaries ship with a `SHA256SUMS` file and a GitHub build
  provenance attestation; verify both before deploying in a qualified
  environment (see `docs/qualification/TOOL_QUALIFICATION.md` §7).
- The compiler and gate run fully offline: no network access, no
  telemetry.

## Scope notes

The HTML explorer output embeds model content in a self-contained page;
treat generated HTML from untrusted models as untrusted content.

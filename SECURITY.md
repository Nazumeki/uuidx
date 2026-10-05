# Security Policy

## Supported versions

`uuidx` is pre-1.0. Security fixes are applied to the latest published release
and to the `main` branch. Older releases are not maintained.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |
| < 0.1   | No        |

## Reporting a vulnerability

Report suspected vulnerabilities privately through GitHub Security Advisories:

- Open a draft advisory at
  <https://github.com/Nazumeki/uuidx/security/advisories/new>

Do not open a public issue for a security vulnerability, and do not include
exploit details in public discussions.

Include, where possible:

- the affected version (`uuidx --version`) or crate version;
- a description of the vulnerability and its impact;
- a minimal reproduction, with secrets and sensitive identifiers removed;
- any suggested fix or mitigation.

## Disclosure process and timelines

We follow coordinated vulnerability disclosure:

1. We acknowledge a report within 3 business days.
2. We provide an initial assessment within 10 days.
3. We aim to publish a fix or mitigation within 90 days of the initial report,
   and sooner for high-severity vulnerabilities.
4. We publish a GitHub Security Advisory once a fix is available, crediting the
   reporter unless anonymity is requested.

If a vulnerability is being actively exploited, say so in the report so we can
prioritise an immediate response.

## Scope

In scope:

- `uuidx-core`, `uuidx-cli`, `uuidx-wasm`, and `uuidx-ffi`;
- incorrect UUID generation, parsing, validation, or inspection that could lead
  to security-relevant data exposure.

Out of scope:

- vulnerabilities in third-party dependencies; report those upstream. Dependabot
  tracks them for this repository;
- denial of service caused only by unrealistic resource limits set by the
  caller.

Thank you for helping keep `uuidx` and its users safe.

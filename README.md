# cnvy — Envoy

## Scope

Follow the public community trust feed under Assurance (`csrn`). Own bounded
refresh, announcement polling, retries, cancellation and native HTTPS. Charter
(`cchr`) verifies and retains every original signed byte. Envoy never decides
admission, stores member state, sends identifiers or implements cryptography.

The state machine and original native/wasm/TLS tests were extracted from
csrn commit 99d8ac24b642ca8f750acc48a41fe1ad47b95c16. The LGPL leaf has no FSL
runtime or development dependency. Its test driver only wires real leaf APIs.

## Reuse and candidates

Use reqwest/rustls for native TLS and serde for bounded announcements. Reuse
Charter's validated canonical origin and cchr/csgn/cshm for authenticated publications.
Generic retry middleware cannot fence response generations or enforce signed
Charter freshness; the extracted explicit state machine owns those transitions.
No new cryptographic, network or storage primitives are introduced.

`feed::Follower` returns bounded `Request` operations; the host supplies time
and completes or cancels I/O. Optional `http::Http` authenticates the configured
HTTPS origin with redirect refusal and bounded collection. The original real
TLS, native and wasm vectors remain required. See [CONTRACT.md](docs/CONTRACT.md)
and [COVERAGE.md](docs/COVERAGE.md). Green compile checks are not coverage claims.

First-party dependencies follow main with one resolved revision per crate.
Dependabot covers Cargo and Actions; metadata-only auto-merge requires successful
substantive CI on the exact current head and respects branch protections.
There are no npm manifests or production-code coverage exclusions.

LGPL-3.0-only WITH LGPL-3.0-linking-exception. No registry publication.

Publishing authority is supplied through per-community deployment configuration.
HTTPS returns bytes only; Charter verifies the configured root, original signed
ring continuity and revision floors. No feed response can authorize its own ring.

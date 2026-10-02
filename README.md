# cnvy — Envoy

## Scope

### Purpose

cnvy follows the public community trust feed from cvld, fetches updates, hands them to cchr, and refreshes on a new revision or a newer policy epoch.

### Owns

- The I/O state machine for following the feed: start, periodic refresh that recovers lost announcements, long-poll for changes, bounded retry with backoff and jitter, one in-flight request at a time, and fencing of late replies.
- Immediate refresh when a member presents a newer policy epoch.
- Requests that carry no member data, use the configured origin exactly, refuse redirects, cookies, and proxies, and use a generic client string; time and randomness are injected.
- Reuse on member devices with their own authenticated origin context. The counterpart cbcn assembles and serves the signed snapshots; cnvy only follows them.

### Never

- Sends member IDs, sessions, wallet IDs, or join events in feed requests.
- Treats an announcement as verified material, extends expired material because a refresh failed, or hides key-bootstrap errors.
- Uses endpoints other than the two existing trust feed endpoints.
- Publishes or assembles snapshots; that belongs to cbcn.
- Verifies signatures itself; verification belongs to cchr with csgn.

### States

Stopped to Fetching to Following. Network failure leads to Backoff. Expired material leads to Unavailable until a verified refresh completes. An announced revision returns to Fetching.

### Test obligations

- Policy, schema, and revocation changes trigger fetch and install, while ordinary member activity leaves feed bytes unchanged.
- Lost announcements recover through bounded refresh; out-of-order responses cannot roll back; retry storms stay bounded; stale trust closes forum admission.
- Captured requests carry no member data or cookies; redirects and oversized responses are refused.
- Clock regression fails closed; stop fences late completions.

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

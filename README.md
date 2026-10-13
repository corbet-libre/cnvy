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

## Balance material follows the existing feed

Envoy transports the extra Balance snapshot without inspecting, transforming or
verifying it. Charter verifies the manifest-bound kind and retains its floor.
A stripped or invalid Balance document follows the ordinary failed-refresh and
readiness rules; no second endpoint or member identifier is introduced. Shared
native/Wasm vectors exercise original signed bytes, a required-kind downgrade,
and recovery through the same fetch route.

## Browser transport

Optional `browser::Browser::new(&cchr::Configuration)` uses the original exact
HTTPS origin and `execute(feed::Request, now)` returns `Received::bytes()` for
Follower/Charter. It preserves raw JSON, including duplicate keys, for the
original strict parser. No request-selected URL, parsed trust DTO, cookies,
bearer or automatic redirect/retry is accepted.

The adapter reuses maintained web-sys Fetch bindings, wasm-streams0.7 and
gloo-timers0.4. A simple whole-body Fetch helper would allocate before enforcing
the feed limit; this adapter consumes the maintained stream incrementally and
aborts on size/deadline/drop. Native reqwest transport remains available under
`http`. API sources: https://docs.rs/wasm-streams/latest/wasm_streams/ and
https://docs.rs/gloo-timers/latest/gloo_timers/ . No transport/crypto is rewritten.

The source targets browser windows and isolated iframes. Real browser Request,
AbortController and stream/refusal vectors are written but unexecuted. Full
HTTPS/CORS, redirect, slow-stream cancellation and original signed-feed install
must still run against a real fixture with authenticated TLS on remote CI;
native HTTPS results do not establish this browser transport's acceptance.

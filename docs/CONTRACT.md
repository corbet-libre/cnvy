# Envoy contract as implemented

Envoy owns the extracted feed state machine and HTTPS adapter. `csrn` wires
`cchr::Charter` and `cnvy::feed::Follower`. The leaf has no facade dependency.

## API and ports

`Follower::new(Config)`, `start(now)`, `next(current_revision, now)`,
`feed(id, bytes, &mut Charter, now)`, `announcement(id, bytes, now)`,
`on_change(revision, current_revision, now)`, `on_policy_epoch(epoch, current_epoch, now)`, `refresh(now)`, `failed(id, now, jitter)`, `permits_current(revision, epoch, now)`, `stop()`.
Config contains explicit refresh, minimum poll, request, backoff and maximum
staleness durations, plus per-instance `retry_entropy` supplied by the runtime. All times are injected Unix seconds. No timer, background
thread, singleton storage or network task is started. No unused storage port is
invented for this volatile machine; a server restart starts unbootstrapped.

The injected network port is an owned `Request` output and a matching completion
input. `on_change` must come from the configured public feed; an epoch hint is bounded to one epoch beyond the caller-supplied current verified epoch. It requests refresh without installing an arbitrary floor or granting authority. Request exposes only operation ID, Fetch or Poll(public revision), exact
existing cvld path/body and exclusive deadline. The enclosing runtime must cancel
transport at that deadline or on stop/change; dropping a native execute future
cancels its I/O. Follower's operation ID also rejects any late completion. It
never creates another live operation before completing, fencing or timing out the
current one. The runtime must obey cancellation to preserve the I/O bound.

`http::Http` (optional native `http` feature) accepts only an exact HTTPS origin,
uses reqwest with redirect refusal, no ambient credentials, cookies, proxy or
referrer, explicit reqwest retry refusal, a generic User-Agent, ten-second connect timeout
and remaining request deadline as total timeout. Only configured origin plus the
two fixed paths can be requested. Body collection is checked chunk by chunk, not
merely against Content-Length: at most cchr's 32 MiB for a feed and 1024 bytes for
an announcement. All non-success statuses and missing/non-JSON Content-Type values fail. JSON media-type parameters are accepted. Explicit operator CA roots may
supplement the public store; certificate and hostname checks stay enabled.
Origin validation and canonicalization use Charter's existing constructor and
configured-origin accessor; invalid input retains the same Charter error.

`Http::new(&Configuration)` and `with_roots(&Configuration, roots)` use
Charter's validation of deployment configuration. `Received::bytes()` exposes
bounded transport bytes only. HTTPS never turns a response key ring into
publishing authority. The configured initial ring and minimum revisions live in
Charter; every later ring requires an original csgn transition signed by the
installed predecessor. Envoy passes bytes unchanged to `Charter::install`.
Browser adapters must authenticate the configured origin, reject redirects and
bypass body caches, while leaving all publishing authority checks to Charter.
The optional `browser` feature supplies `browser::Browser` in a Window/isolated
iframe. Its constructor takes the same Charter Configuration; execute accepts
only the original Follower Request and trusted now, then returns bounded raw
Received bytes. Fetch omits credentials/referrer, refuses redirects, bypasses
caches and streams each Uint8Array within the original feed/announcement bound.
An AbortController aborts on future drop or the original request deadline.
Follower still rejects obsolete completions; Charter alone authenticates bytes.
Worker networking and TLS deployment remain embedding responsibilities.

## States and failure semantics

Stopped → Fetching → Following. Network, invalid trust, malformed announcement,
wrong response kind or timeout → Backoff. No known fresh charter, expired signed
material, newer announced revision, clock regression or exceeded maximum staleness
makes the consumer Unavailable. Follower's state describes I/O; consumer readiness is
derived from both children, not a second persisted trust state.

A poll response is at most 1024 bytes, rejects duplicate/unknown fields and must
have a positive epoch, nondecreasing revision and `changed` exactly matching a
higher revision. Announcements are hints, not signed material or permanent floors.
A newer revision or member-presented policy epoch fences a pending poll and
requests a feed promptly; hints cannot
bypass backoff. A fetched publication behind either revision or epoch hint cannot restore readiness.
A periodic fetch runs even when announcements are lost. Empty/unchanged polls
are spaced by the configured minimum interval, preventing immediate-reply loops.

Exponential retry delay starts at the explicit initial value, doubles to the
configured cap, and uses injected entropy for bounded positive jitter on every failure path, including automatic timeout and rejected feed/announcement. The runtime supplies fresh per-instance entropy in Config; `failed` may replace it with newly sampled entropy. No random generator is implemented here.
At most one request can exist; caps are enforced for any supplied jitter.
Successful verified refresh resets backoff. Rejected candidates preserve Charter's
old valid material; no error extends its signed deadline or the last successful
refresh time. During a transient outage, current material remains usable only
until both signed expiry and maximum staleness permit it, and only while no newer
revision is pending. Stop refuses all reads and fences completions; restart
requires fresh authenticated bootstrap while retaining the existing Charter's
in-process rollback floors. New process state never reuses a cached feed alone.

Clock regression fails closed and cancels pending work. A monotonic operation
counter never wraps; exhaustion refuses work. Configuration caps request timeout
at 300 seconds (minimum 26 for cvld's 25-second long poll), maximum backoff and
staleness at one day; refresh must fit the staleness budget. Checked deadline
arithmetic refuses overflow. Fixed errors omit payloads, identifiers and causes.

## Validation and open work

CI checks main declarations and unique locked first-party revisions, fmt, strict Clippy, native tests,
wasm32 build and identical deterministic state-machine vectors on wasm. Tests
exercise startup, changes, expiry, loss, delay, repeated failures, bounded storms,
clock regression, cancellation/stop/restart, obsolete replies, malformed events,
corruption and signed rollback. Native HTTPS tests use real local TLS and verify
request privacy, origin bootstrap, redirect and oversized-response refusal,
including chunked bodies without Content-Length.

Native and wasm also replay unchanged bytes from the running cvld door producer
in cchr CI: unchanged feed after registration/issuance/renewal, followed by
all/any/threshold policy edits, schema change and last-passkey revocation with
their actual announcements. These exercise the full follower→Charter path.
The cnvy job replays captures; cchr separately launches the actual door.
`tests/fixtures/provenance.json` records immutable producer/door revisions, CI
source and per-file SHA-256; CI verifies these hashes before testing. No mocked
verification verdicts or cvld server dependency enter the consumer graph.

Charter exposes a borrowed Guard policy containing the original signed full
settings and separate revocations. Guard verifies these envelopes, the device
signature and the profile, then delegates rich policy to Rulebook. These captures
verify byte preservation; they do not assert admission without a valid bundle.
Charter and Assurance separately execute actual door-authorized device bundles
through Guard on native and wasm. Production forum wiring remains downstream.
Production publishing-root provisioning and TLS setup, browser qualification and embedding
cancellation/scheduling are adapter integration responsibilities. No deployment,
registry publication, payments or external test accounts are involved.

## Balance material follows the existing feed

Envoy transports the extra Balance snapshot without inspecting, transforming or
verifying it. Charter verifies the manifest-bound kind and retains its floor.
A stripped or invalid Balance document follows the ordinary failed-refresh and
readiness rules; no second endpoint or member identifier is introduced. Shared
native/Wasm vectors exercise original signed bytes, a required-kind downgrade,
and recovery through the same fetch route.

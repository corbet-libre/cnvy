# Actual cvld public trust publications

These unchanged JSON responses were captured by the running global/community
cvld door in [cchr CI](https://github.com/corbet-foss/cchr/actions/runs/36911320705).
The services use a temporary libSQL store and synthetic users, real software
WebAuthn credentials, and real BBS proofs. No external account is involved.

`initial.json` and `ordinary.json` are byte-identical despite registration, lobby,
voucher admission, credential issuance and renewal. The remaining responses and
announcements follow all/any/threshold settings, schema version 2, then revocation
of the member's last passkey. Native and wasm tests consume these original bytes
through the follower and Charter; they do not substitute verification verdicts.

`provenance.json` records producer commit, cvld commit, successful CI URL and
SHA-256 for every capture. `scripts/check-fixtures.py` detects altered captures.
`import-door-evidence.py` imports a downloaded successful cchr `door-evidence`
artifact. The hash manifest is an audit record, not a cryptographic trust root.

Envoy tests replay this evidence; they do not start cvld themselves. The cchr CI
producer runs real HTTP door services afresh. The separate Envoy HTTP tests use
real local TLS to exercise origin authentication, refusal and response bounds.
These are public trust publications only, with synthetic fixture material.

The current captures also include actual authorized-device-signed bundles for
all, any, threshold and schema changes. `publishing-root.json` comes independently
from synthetic deployment configuration before the producer fetches a feed.
It is not derived from the returned `key_ring`. Only the public verification
ring is captured; the synthetic private seed is not part of this artifact.

Balance material: `balance-material.json` is the original public output of cblc b7449bdd32e884cfe29eff3bdbf787577bdda545, CI 36936767652, SHA-256 bb013abe6ce3e3f0e8c45481ced22e6cd099361b635af4b95c555511112e4f55. Its configuredFixtureRing is a separately supplied fictional test anchor. The activated issuer produced the material and real extension proof replay verified its artifact bindings. Tests which alter routing metadata explicitly re-sign with their test publisher; that tests feed authentication, not a new proof or record-currentness verdict.

`balance-material-v2.json` is the cplc canonical v2 balance material (the v1 capture above no longer parses: `admissionPolicyDigest` and `extensionPolicy.standingPolicyDigest` are required). It carries the cblc test standing digest and is re-signed with the unchanged fictional fixture key; the follower test re-signs it again with its own publisher.

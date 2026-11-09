# cnvy agent notes

Work in a lane (`~/agents/scratch/lane-new <lane>`); never edit `~/agents/scratch/ws`.
Run from the lane root (`<lane>/ws`); `cbuild <lane> ...` builds there.

Build and test (quote the verdict line, never infer from a pipe):
- `cbuild <lane> fmt -p cnvy -- --check` — must be clean
- `python3 scripts/check-fixtures.py` — provenance manifest matches, exit 0
- `cbuild <lane> test -p cnvy --locked --features http` — `door` `1`, `follower` `15`, `http` `11`, doc `0` passed, exit 0; long, background it
- `cbuild <lane> clippy -p cnvy --locked --all-targets --features http -- -D warnings` — red at HEAD from upstream `ckmg` lints (unused `SigningPhase`, dead `label`/`insert`), not this crate
- `cbuild <lane> check -p cnvy --locked --target wasm32-unknown-unknown` — exit 0
- Full gate (wasm-runner + shared wasm vectors in a real browser, 100% coverage) runs in CI; see `.github/workflows/ci.yml`, `docs/COVERAGE.md`.

Land: `cargo fmt` in touched repos, rebase on main (never force-push),
`cbuild <lane> xtask land --dry`, then `xtask land`.

Invariants (rule -> enforcing test):
- A stripped or invalid kind can never restore readiness
  -> `tests/it/follower.rs::balance_follows_the_same_feed_and_a_stripped_kind_cannot_restore_readiness`
- Epoch hints bounded to the next verified epoch; malformed feeds never become trust
  -> `::arbitrary_epoch_hint_is_bounded_by_the_next_verified_epoch`, `::malformed_announcements_and_corrupt_feed_cannot_become_trust`
- One bounded operation; retry storms and obsolete completions fenced
  -> `::timeout_and_retry_storms_have_one_bounded_operation`, `::obsolete_completion_after_replacement_keeps_new_fetch_alive`
- Clock regression fences pending work, recovers at high water
  -> `::clock_regression_fences_pending_and_recovers_at_high_water`
- HTTPS: exact origin only, no redirects/credentials, chunked bounds; a forged ring never authorizes
  -> `tests/it/http.rs::real_https_fetch_bootstraps_without_member_headers_or_redirects`, `::redirect_and_oversize_responses_are_refused`, `::valid_tls_does_not_authorize_a_forged_publishing_ring`

Pointers: `src/feed.rs`, `src/http.rs`, `src/browser.rs`, `docs/CONTRACT.md`, `docs/COVERAGE.md`.
Commits: explicit paths, imperative, author per `~/agents/AGENT.md`; no AI trailers.
`publish=false`; LGPL-3.0-only WITH linking-exception; no GPL/AGPL-only deps.

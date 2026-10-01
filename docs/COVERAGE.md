# Coverage gate

CI uses cargo-llvm-cov on nightly for upstream Rust branch instrumentation.
Stable Rust remains the compiler for formatting, Clippy, native and wasm checks.
The gate requires exactly 100% covered production lines and branches, from the
LLVM JSON counts, and refuses absent or empty measurements. There are no
production-code exclusions. Test harness and fixture files are excluded because
they are validation inputs rather than shipped behavior. A failed gate is an
open test gap, never evidence of complete coverage.

Dependencies are resolved once per CI run and the resulting Cargo.lock artifact
is reused by every job. CI refreshes within the declared ranges;
Dependabot proposes lockfile updates for review and merging after full CI.
The checked-in lock and per-run artifacts retain exact reproducible resolutions.

Every start, failed request, explicit refresh and newer revision/epoch hint
synchronously sets `next_refresh` to the observed clock. A successful feed clears
both targets before setting a future refresh. The redundant alternate fetch
conditions were removed; stopped callers return before this decision. Real
startup, failure, poll-cancellation and behind-announcement vectors retain these
invariants. No production coverage exclusion is used.

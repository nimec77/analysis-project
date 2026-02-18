# Phase 21: Property-based testing (`proptest`)

**Goal:** Add property-based tests using `proptest` and fill in missing unit test coverage for parsers.

## Tasks

- [x] 21.1 Add `proptest = "1"` to `[dev-dependencies]` in `Cargo.toml`
- [x] 21.2 Roundtrip test: `unquote_escaped(quote(s)) == Ok(("", s))` for arbitrary strings
- [x] 21.3 No-panic test: `LogLine::parser().parse(arbitrary_string)` never panics
- [x] 21.4 Suffix invariant: parser remaining output is always a suffix of input
- [x] 21.5 Add missing unit tests: `WithdrawCash`, `DeleteUser`, `UnregisterAsset` standalone parsing
- [x] 21.6 Add `Permutation` with 3 parsers coverage
- [x] 21.7 Add error cases for each domain type with malformed input

## Acceptance Criteria

**Test:** `cargo test && cargo test -- --nocapture`

## Dependencies

- Phase 20 complete

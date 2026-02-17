# Phase 12: Remove `OnceLock` singleton

**Goal:** Remove the `LOG_LINE_PARSER` `OnceLock` singleton, constructing the parser inline or passing it as a parameter.

## Tasks

- [x] 12.1 Remove `LOG_LINE_PARSER` `OnceLock` singleton
- [x] 12.2 Construct the parser inline or pass it as a parameter
- [x] 12.3 Update call site in `lib.rs`

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 1 complete (lightweight parser construction after `&str` migration)

## Implementation Notes

**Hint:** `src/parse.rs:1144` — `// подсказка: singleton, без которого можно обойтись`

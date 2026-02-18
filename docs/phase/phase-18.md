# Phase 18: Strategy pattern (`LogFilter` trait)

**Goal:** Extract the filtering logic from `read_log()` into a `LogFilter` trait, implement it for `ReadMode`, and update the `read_log()` signature to accept `impl LogFilter`.

## Tasks

- [x] 18.1 Define `LogFilter` trait in `src/lib.rs`: `fn accepts(&self, log: &LogLine) -> bool`
- [x] 18.2 Implement `LogFilter` for `ReadMode` (move existing match logic)
- [x] 18.3 Update `read_log()` signature: `filter: impl LogFilter` instead of `mode: ReadMode`
- [x] 18.4 Update call sites in `main.rs` and tests

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

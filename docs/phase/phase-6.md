# Phase 6: `match` instead of `if` chain

**Goal:** Replace the `if mode == ... else if mode == ...` chain with a `match` expression on `ReadMode`, making the filtering logic more idiomatic and exhaustive.

## Tasks

- [x] 6.1 Replace the `if mode == ... else if mode == ...` chain with `match` on `ReadMode`

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 5 complete

## Implementation Notes

**Hint:** `src/lib.rs:88` — `// подсказка: лучше match`

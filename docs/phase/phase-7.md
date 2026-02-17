# Phase 7: `Result` instead of `panic!`

**Goal:** Replace `panic!` on unknown mode with exhaustive `match` (no default arm needed after Phase 5), return `Result` from `read_log()` for any remaining fallible operations, and adapt tests accordingly.

## Tasks

- [x] 7.1 Replace `panic!` on unknown mode with exhaustive `match` (no default arm needed after Phase 5)
- [x] 7.2 Return `Result` from `read_log()` for any remaining fallible operations
- [x] 7.3 Adapt `test_all` in `lib.rs` if `read_log()` now returns `Result`: unwrap or use `?` in test

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 5 complete
- Phase 6 complete (exhaustive `match` already eliminates the `panic!` arm)

## Implementation Notes

**Hint:** `src/lib.rs:114` — `// подсказка: паниковать в библиотечном коде - нехорошо`

Note: Phase 6 already removed the `panic!("unknown mode {:?}", mode)` by replacing the `if`/`else if` chain with an exhaustive `match`. This phase should focus on any remaining `panic!` sites and on converting `read_log()` to return `Result` for broader error handling.

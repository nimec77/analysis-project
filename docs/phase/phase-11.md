# Phase 11: `NonZeroU32` tight type

**Goal:** Use `std::num::NonZeroU32` for `request_id` instead of `u32` + runtime check.

## Tasks

- [x] 11.1 Use `std::num::NonZeroU32` for `request_id` instead of `u32` + runtime check

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

## Implementation Notes

**Hint:** `src/parse.rs:39` — `// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32`

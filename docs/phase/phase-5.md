# Phase 5: `u8` constants -> `enum ReadMode`

**Goal:** Replace the `u8` mode constants (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) with a proper `enum ReadMode`, improving type safety and enabling `match` in later phases.

## Tasks

- [ ] 5.1 Replace `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` constants with `enum ReadMode`
- [ ] 5.2 Update `read_log()` and all call sites
- [ ] 5.3 Adapt `test_all` in `lib.rs`: replace `READ_MODE_ALL` with `ReadMode::All`

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

## Implementation Notes

**Hint:** `src/lib.rs:4` — `// подсказка: лучше использовать enum и match`

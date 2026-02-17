# Phase 10: `Box` the large enum variant

**Goal:** Wrap `AuthData` (or whichever variant is oversized) in `Box<>` to reduce `LogKind` stack size.

## Tasks

- [x] 10.1 Wrap `AuthData` (or whichever variant is oversized) in `Box<>` to reduce `LogKind` stack size
- [x] 10.2 Adapt `test_authdata` and `test_log_kind` in `parse.rs`: wrap `AuthData(...)` in `Box::new(...)` in expected values where the variant is `Connect(Box<AuthData>)`

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

## Implementation Notes

**Hint:** `src/parse.rs:621` — `// подсказка: довольно много места на стэке`
**Hint:** `src/parse.rs:852` — `// подсказка: а поля не слишком много места на стэке занимают?`

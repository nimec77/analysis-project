# Phase 13: Bug fix + dead code cleanup

**Goal:** Fix the `WithdrawCash` parsing bug and remove all unused code (dead types, unused constructors).

## Tasks

- [x] 13.1 Fix `WithdrawCash` bug: `src/parse.rs:1320` maps to `DepositCash` instead of `WithdrawCash`
- [x] 13.2 Add dedicated `WithdrawCash` parsing test to prevent regression
- [x] 13.3 Remove unused `AsIs` struct + Parser impl (~line 138)
- [x] 13.4 Remove unused `Either<L,R>` enum (~line 731)
- [x] 13.5 Remove unused `Status` enum + Parsable impl (~line 737)
- [x] 13.6 Remove unused `all3()` constructor (~line 331)
- [x] 13.7 Remove unused `all4()` constructor (~line 356)

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- None (standalone phase)

## Implementation Notes

- The `WithdrawCash` bug at line 1320 incorrectly maps to `DepositCash` — this is a copy-paste error
- Dead code items (`AsIs`, `Either`, `Status`, `all3`, `all4`) should be confirmed unused before removal
- Line numbers are approximate and may have shifted from earlier refactoring phases

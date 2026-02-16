# PRD: Phase 5 — `u8` constants -> `enum ReadMode`

**Status:** PRD_READY
**Ticket:** PH-5 "Phase 5: `u8` constants -> `enum ReadMode`"
**Phase:** 5 of 12 (see `docs/tasklist.md`)
**Dependencies:** None (independent phase)
**Blocked by:** Nothing
**Blocks:** Phase 6 (`match` instead of `if` chain), Phase 7 (`Result` instead of `panic!`)

---

## Context / Idea

Phase 5 targets the replacement of the three `u8` mode constants (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) with a proper `enum ReadMode`, improving type safety and enabling `match`-based dispatch in later phases (Phase 6, Phase 7).

The original author left a hint at `src/lib.rs:5`: `// подсказка: лучше использовать enum и match` ("hint: better to use an enum and match"), indicating this was recognized technical debt from the start.

### Authoritative specification from `docs/phase/phase-5.md`

**Goal:** Replace the `u8` mode constants (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) with a proper `enum ReadMode`, improving type safety and enabling `match` in later phases.

**Tasks:**

- [ ] 5.1 Replace `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` constants with `enum ReadMode`
- [ ] 5.2 Update `read_log()` and all call sites
- [ ] 5.3 Adapt `test_all` in `lib.rs`: replace `READ_MODE_ALL` with `ReadMode::All`

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** None (independent phase)

**Implementation Notes:**

- **Hint:** `src/lib.rs:5` -- `// подсказка: лучше использовать enum и match`

### Current codebase state (gap analysis)

After Phases 1-4, the current state of `src/lib.rs` relevant to this phase:

- **`u8` constants** (lines 5-11): Three public constants define read modes:
  ```rust
  // подсказка: лучше использовать enum и match
  pub const READ_MODE_ALL: u8 = 0;
  pub const READ_MODE_ERRORS: u8 = 1;
  pub const READ_MODE_EXCHANGES: u8 = 2;
  ```
- **`read_log()` signature** (line 47): `pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>` -- the `mode` parameter is typed as `u8`.
- **`if` chain** (lines 62-90): Mode filtering uses `if mode == READ_MODE_ALL { ... } else if mode == READ_MODE_ERRORS { ... } else if mode == READ_MODE_EXCHANGES { ... } else { panic!("unknown mode {}", mode) }`. The `panic!` on line 89 exists because the compiler cannot verify exhaustiveness of `u8` comparisons -- any value outside 0, 1, 2 is a runtime error.
- **Hint comment** (line 5): `// подсказка: лучше использовать enum и match` -- this is addressed by this phase.
- **Hint comment** (line 62): `// подсказка: лучше match` -- this hint relates to Phase 6, but introducing the enum is a prerequisite.
- **`main.rs`** (line 67): `analysis::read_log(file, analysis::READ_MODE_ALL, vec![])` -- uses the constant.
- **Tests** (lines 170-171): Both test calls use `READ_MODE_ALL`.

All of these must be updated to use the new `ReadMode` enum.

---

## Goals

1. **Introduce `enum ReadMode`.** Define a public enum with variants `All`, `Errors`, and `Exchanges`, replacing the three `u8` constants `READ_MODE_ALL`, `READ_MODE_ERRORS`, and `READ_MODE_EXCHANGES`.
2. **Improve type safety.** With an enum, the compiler prevents passing arbitrary `u8` values to `read_log()`. Invalid modes are caught at compile time, not at runtime.
3. **Enable exhaustive matching.** After this phase, Phase 6 can replace the `if`/`else if` chain with a `match` on `ReadMode`, and Phase 7 can remove the `panic!` (since exhaustive `match` on an enum needs no default arm).
4. **Update the `read_log()` signature.** Change the `mode` parameter from `u8` to `ReadMode`.
5. **Update all call sites.** Both `main.rs` and the test module must use the new enum variants.
6. **Remove the hint comment.** The hint `// подсказка: лучше использовать enum и match` at line 5 is addressed and should be removed.
7. **Remove the three `u8` constants.** `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` are deleted entirely.
8. **Preserve behavior.** Same input produces same output. All existing tests pass; no tests are deleted.

---

## User Stories

1. **As a library consumer**, I want `read_log()` to accept a `ReadMode` enum instead of a raw `u8`, so that I cannot accidentally pass an invalid mode value and get a runtime panic.
2. **As a developer working on Phase 6**, I want the mode expressed as an enum so that I can write `match mode { ReadMode::All => ..., ReadMode::Errors => ..., ReadMode::Exchanges => ... }` with compiler-verified exhaustiveness.
3. **As a developer working on Phase 7**, I want the `panic!("unknown mode {}", mode)` to become unnecessary by construction, so that library code does not panic on invalid input.
4. **As a maintainer**, I want the public API to use meaningful types instead of magic numbers, improving discoverability and self-documentation.

---

## Scenarios

### Scenario 1: `enum ReadMode` replaces constants

**Before:**
```rust
// подсказка: лучше использовать enum и match
pub const READ_MODE_ALL: u8 = 0;
pub const READ_MODE_ERRORS: u8 = 1;
pub const READ_MODE_EXCHANGES: u8 = 2;
```

**After:**
```rust
/// Read mode for filtering log entries.
pub enum ReadMode {
    /// Return all log entries.
    All,
    /// Return only error entries (System::Error and App::Error).
    Errors,
    /// Return only exchange/journal operation entries.
    Exchanges,
}
```

The hint comment is removed. The three `pub const` declarations are deleted.

### Scenario 2: `read_log()` signature change

**Before:**
```rust
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

**After:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

The `mode` parameter type changes from `u8` to `ReadMode`.

### Scenario 3: Filtering logic updated (minimal -- still uses `if` chain)

**Before:**
```rust
if mode == READ_MODE_ALL {
    true
}
else if mode == READ_MODE_ERRORS {
    ...
}
else if mode == READ_MODE_EXCHANGES {
    ...
}
else {
    panic!("unknown mode {}", mode)
}
```

**After (Phase 5 scope -- keep `if` chain for now):**
```rust
if mode == ReadMode::All {
    true
}
else if mode == ReadMode::Errors {
    ...
}
else if mode == ReadMode::Exchanges {
    ...
}
else {
    panic!("unknown mode {:?}", mode)
}
```

Note: The `if` chain is intentionally preserved in Phase 5; Phase 6 converts it to `match`. The `panic!` is preserved; Phase 7 addresses it. The enum must derive or implement `PartialEq` and `Debug` for the `==` comparisons and the `panic!` format string to compile. Alternatively, the `if` chain can already be converted to `match` in this phase since it is a natural consequence of using an enum, but the strict scope boundary leaves this to Phase 6.

### Scenario 4: `main.rs` adaptation

**Before:**
```rust
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

**After:**
```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
```

### Scenario 5: Test adaptation (`test_all` in `lib.rs`)

**Before:**
```rust
assert_eq!(read_log(SOURCE1.as_bytes(), READ_MODE_ALL, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);
```

**After:**
```rust
assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), ReadMode::All, vec![]);
```

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| `READ_MODE_ALL` occurrences in `src/` | Zero |
| `READ_MODE_ERRORS` occurrences in `src/` | Zero |
| `READ_MODE_EXCHANGES` occurrences in `src/` | Zero |
| `u8` mode constants in `src/lib.rs` | Zero |
| `enum ReadMode` definition | Present in `src/lib.rs`, public |
| `ReadMode` variants | `All`, `Errors`, `Exchanges` |
| `read_log()` `mode` parameter type | `ReadMode` (not `u8`) |
| Hint comment `// подсказка: лучше использовать enum и match` | Removed |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** Same input must produce same output.
3. **No test deletions.** Existing tests are adapted, not deleted.
4. **Scope boundary.** This phase addresses only the `u8` constants to `enum ReadMode` conversion. The `if`/`else if` chain to `match` conversion is Phase 6. The `panic!` removal is Phase 7. Neither is in scope here.
5. **Enum must support equality comparison.** The enum must derive `PartialEq` (needed while the `if` chain remains in Phase 5; Phase 6 will replace it with `match`).
6. **Enum must support debug formatting.** The enum must derive `Debug` (needed for the `panic!` format string which uses `{:?}` or `{}`; Phase 7 removes the `panic!`).
7. **Compiler-driven refactoring.** Change the type of `mode` in `read_log()` first, then follow compiler errors to update all call sites and comparisons.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `if` chain with `==` requires `PartialEq` derive on the enum | Certain | Low | Derive `PartialEq` on `ReadMode`. This is a one-line addition and is idiomatic Rust. Phase 6 will replace the `==` comparisons with `match`, at which point `PartialEq` may become unnecessary but causes no harm. |
| `panic!` format string needs `Debug` or `Display` | Certain | Low | Derive `Debug` on `ReadMode`. The `panic!` message currently uses `{}` for the `u8` mode; after the change, use `{:?}` for the enum. Phase 7 removes the `panic!` entirely. |
| `main.rs` breaks due to removed constants | Certain | Low | Update `main.rs` as part of task 5.2. This is a mechanical change: replace `analysis::READ_MODE_ALL` with `analysis::ReadMode::All`. |
| Downstream phases (6, 7) depend on this enum existing | Certain | None | This is the intended dependency. Phase 5 must be completed before Phases 6 and 7. |
| External callers depend on the `u8` constants as public API | Very Low | Medium | This is an internal project with no known external consumers. The constants can be safely removed. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. This phase has no dependencies on prior phases. The refactoring is mechanical: define the enum, change the parameter type, replace constant references with enum variants, and follow compiler errors.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | Remove hint comment `// подсказка: лучше использовать enum и match` (line 5). Remove `pub const READ_MODE_ALL: u8 = 0;` (line 7). Remove `pub const READ_MODE_ERRORS: u8 = 1;` (line 9). Remove `pub const READ_MODE_EXCHANGES: u8 = 2;` (line 11). Add `pub enum ReadMode { All, Errors, Exchanges }` with `#[derive(Debug, PartialEq)]`. Change `read_log()` signature: `mode: u8` becomes `mode: ReadMode`. Update `if` comparisons: `mode == READ_MODE_ALL` becomes `mode == ReadMode::All`, etc. Update `panic!` format: `{}` becomes `{:?}`. Update test calls: `READ_MODE_ALL` becomes `ReadMode::All`. |
| `src/main.rs` | Replace `analysis::READ_MODE_ALL` with `analysis::ReadMode::All` (line 67). |

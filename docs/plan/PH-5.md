# Implementation Plan: PH-5 -- `u8` constants -> `enum ReadMode`

**Status:** PLAN_APPROVED
**Ticket:** PH-5 "Phase 5: `u8` constants -> `enum ReadMode`"
**PRD:** `docs/prd/PH-5.prd.md`
**Research:** `docs/research/PH-5.md`
**Phase spec:** `docs/phase/phase-5.md`

---

## Components

### 1. `u8` mode constants (to be removed)

**File:** `src/lib.rs`, lines 5-11

Three public `u8` constants define read modes. A Russian hint comment at line 5 marks this as recognized technical debt. All six lines (1 hint comment, 3 doc comments, 3 constant definitions) are removed and replaced by the `ReadMode` enum.

```rust
// REMOVE (lines 5-11):
// подсказка: лучше использовать enum и match
/// Режим чтения из логов всего подряд
pub const READ_MODE_ALL: u8 = 0;
/// Режим чтения из логов только ошибок
pub const READ_MODE_ERRORS: u8 = 1;
/// Режим чтения из логов только операций, касающихся деген
pub const READ_MODE_EXCHANGES: u8 = 2;
```

### 2. `enum ReadMode` (new definition)

**File:** `src/lib.rs`, replaces lines 5-11

A public enum with three variants: `All`, `Errors`, `Exchanges`. Derives `Debug` and `PartialEq`.

```rust
// ADD:
/// Read mode for filtering log entries.
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    /// Return all log entries.
    All,
    /// Return only error entries (System::Error and App::Error).
    Errors,
    /// Return only exchange/journal operation entries.
    Exchanges,
}
```

- **`PartialEq`** is required because the `if` chain at lines 63-86 uses `==` comparisons (e.g., `mode == ReadMode::All`). Phase 6 will replace the `if` chain with `match`, at which point `PartialEq` may become unnecessary, but it causes no harm.
- **`Debug`** is required because the `panic!` at line 89 will use `{:?}` to format the enum value. Phase 7 removes the `panic!` entirely.

### 3. `read_log()` function signature

**File:** `src/lib.rs`, line 47

The `mode` parameter type changes from `u8` to `ReadMode`. No other signature changes.

### 4. Mode filtering logic (`if`/`else if` chain)

**File:** `src/lib.rs`, lines 62-90

The three `READ_MODE_*` constant references in the `if`/`else if` comparisons are replaced with `ReadMode::All`, `ReadMode::Errors`, and `ReadMode::Exchanges`. The `if`/`else if`/`else` structure itself is preserved (Phase 6 scope). The `panic!` arm is preserved (Phase 7 scope). The `panic!` format string changes from `{}` to `{:?}`.

### 5. Test code (call site adaptation)

**File:** `src/lib.rs`, lines 170-171

Both `READ_MODE_ALL` references in the `test_all` function are replaced with `ReadMode::All`. The test module already has `use super::*;`, so no additional import is needed.

### 6. CLI binary (call site adaptation)

**File:** `src/main.rs`, line 67

`analysis::READ_MODE_ALL` is replaced with `analysis::ReadMode::All`.

---

## API Contract

### Before

```rust
// Public constants (src/lib.rs)
pub const READ_MODE_ALL: u8 = 0;
pub const READ_MODE_ERRORS: u8 = 1;
pub const READ_MODE_EXCHANGES: u8 = 2;

// Public function (src/lib.rs)
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

Callers pass a raw `u8` value. Any value outside {0, 1, 2} causes a runtime `panic!`.

### After

```rust
// Public enum (src/lib.rs)
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    All,
    Errors,
    Exchanges,
}

// Public function (src/lib.rs)
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

- Three `pub const` declarations are removed entirely.
- `ReadMode` is a new public enum exported from the library crate.
- Callers pass a `ReadMode` variant. Invalid modes are caught at compile time, not runtime.
- Return type `Vec<LogLine>` is unchanged.
- `LogIterator<R>` remains private (not `pub`); unaffected by this phase.

---

## Data Flows

```
Caller (main.rs or test)
  |
  |  passes ReadMode enum variant (compile-time type-safe)
  v
read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
  |
  |  ownership transfer: input moved into LogIterator::new()
  v
LogIterator<R>::new(reader: R) -> BufReader<R> -> Lines -> Filter
  |
  |  for each LogLine:
  |    1. check request_id filter
  |    2. check mode filter:
  |       if mode == ReadMode::All { true }
  |       else if mode == ReadMode::Errors { matches!(...) }
  |       else if mode == ReadMode::Exchanges { matches!(...) }
  |       else { panic!(...) }  // unreachable in practice, removed in Phase 7
  v
Vec<LogLine> returned to caller
```

The data flow is identical to the pre-refactoring version. The only change is the type of the `mode` parameter: `u8` becomes `ReadMode`.

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|---|---|
| Zero external dependencies | No new crates. Only `std` types used. |
| No behavior changes | Same input produces same output. All tests preserved and adapted. |
| No test deletions | Tests are adapted (constant names to enum variants), not deleted. All 16 tests pass. |
| Type safety | Compile-time enforcement of valid mode values. No arbitrary `u8` accepted. |
| Scope boundary | Only the `u8`-to-enum conversion. The `if`/`else if` chain (Phase 6), `panic!` removal (Phase 7), and other hint comments are untouched. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `PartialEq` derive needed for `==` comparisons | Certain | Low | `#[derive(PartialEq)]` is one line and idiomatic Rust. Required while the `if` chain remains. Phase 6 may make it unnecessary, but it causes no harm to keep. |
| `Debug` derive needed for `panic!` format string | Certain | Low | The `panic!` currently uses `{}` (Display) for `u8`. After the change, `{:?}` (Debug) must be used. `#[derive(Debug)]` is required. Phase 7 removes the `panic!` entirely. |
| `main.rs` breaks due to removed constants | Certain | Low | Mechanical fix: replace `analysis::READ_MODE_ALL` with `analysis::ReadMode::All`. Covered by task 5.2b. |
| Tests break due to removed constants | Certain | Low | Mechanical fix: replace `READ_MODE_ALL` with `ReadMode::All`. Covered by task 5.3. |
| `else { panic!(...) }` arm becomes unreachable in practice | Expected | None | After Phase 5, no caller can construct an invalid mode value, but the `if`/`else if` chain is not a `match`, so the compiler does not know the `else` is unreachable. Phase 6 replaces with `match`, Phase 7 removes the default arm. No action needed in Phase 5. |
| External callers depend on the `u8` constants | Very Low | Medium | Internal project with no known external consumers. Constants can be safely removed. |

---

## Deviations to Fix

None. The research document (section 9) confirms the current codebase state matches the PRD's "before" state exactly across all 10 usage sites (3 constant definitions, 1 hint comment, 3 `if` comparisons, 2 test usages, 1 `main.rs` usage). No code deviates from requirements.

---

## Implementation Tasks

### Task 5.1: Define `enum ReadMode` and remove constants

**File:** `src/lib.rs`

Remove lines 5-11 (hint comment, three doc comments, three constant definitions) and replace with the `ReadMode` enum:

```rust
// REMOVE (lines 5-11):
// подсказка: лучше использовать enum и match
/// Режим чтения из логов всего подряд
pub const READ_MODE_ALL: u8 = 0;
/// Режим чтения из логов только ошибок
pub const READ_MODE_ERRORS: u8 = 1;
/// Режим чтения из логов только операций, касающихся деген
pub const READ_MODE_EXCHANGES: u8 = 2;

// REPLACE WITH:
/// Read mode for filtering log entries.
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    /// Return all log entries.
    All,
    /// Return only error entries (System::Error and App::Error).
    Errors,
    /// Return only exchange/journal operation entries.
    Exchanges,
}
```

**Verify:** `cargo build` -- should fail on all call sites referencing the removed constants.

### Task 5.2a: Update `read_log()` signature and `if` chain

**File:** `src/lib.rs`

1. Change the `mode` parameter type (line 47):

   ```rust
   // FROM:
   pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine> {

   // TO:
   pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine> {
   ```

2. Update the three `if` comparisons (lines 63, 66, 74):

   ```rust
   // FROM:
   && if mode == READ_MODE_ALL {
   // TO:
   && if mode == ReadMode::All {

   // FROM:
   else if mode == READ_MODE_ERRORS {
   // TO:
   else if mode == ReadMode::Errors {

   // FROM:
   else if mode == READ_MODE_EXCHANGES {
   // TO:
   else if mode == ReadMode::Exchanges {
   ```

3. Update the `panic!` format string (line 89):

   ```rust
   // FROM:
   panic!("unknown mode {}", mode)

   // TO:
   panic!("unknown mode {:?}", mode)
   ```

**Preserved (not in Phase 5 scope):**
- The `if`/`else if`/`else` chain structure (Phase 6 converts to `match`)
- The `panic!` arm itself (Phase 7 removes it)
- Hint comment at line 62: `// подсказка: лучше match` (Phase 6 scope)
- Hint comment at line 88: `// подсказка: паниковать в библиотечном коде - нехорошо` (Phase 7 scope)
- Hint comment at line 50: `// подсказка: можно обойтись итераторами` (Phase 9 scope)

**Verify:** `cargo build` -- should fail only on test and `main.rs` call sites.

### Task 5.2b: Adapt `main.rs`

**File:** `src/main.rs`, line 67

```rust
// FROM:
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);

// TO:
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
```

**Verify:** `cargo build` succeeds for the binary crate.

### Task 5.3: Adapt test call sites

**File:** `src/lib.rs`, lines 170-171

```rust
// FROM:
assert_eq!(read_log(SOURCE1.as_bytes(), READ_MODE_ALL, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);

// TO:
assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), ReadMode::All, vec![]);
```

The test module already has `use super::*;` at line 100, so `ReadMode::All` is in scope.

**Verify:** `cargo test` -- all 16 tests pass.

---

## Verification Checklist

After all tasks are complete, run the following:

```bash
# All tests pass (no deletions)
cargo test

# CLI output identical to pre-refactoring
cargo run -- example.log

# Zero READ_MODE_* occurrences in source
grep -r "READ_MODE_ALL" src/
# Expected: zero hits
grep -r "READ_MODE_ERRORS" src/
# Expected: zero hits
grep -r "READ_MODE_EXCHANGES" src/
# Expected: zero hits

# Zero u8 mode constants
grep -r "pub const.*u8" src/lib.rs
# Expected: zero hits

# Hint comment removed
grep -r "подсказка: лучше использовать enum и match" src/
# Expected: zero hits

# Enum defined
grep "enum ReadMode" src/lib.rs
# Expected: pub enum ReadMode

# read_log uses ReadMode
grep "pub fn read_log" src/lib.rs
# Expected: pub fn read_log(input: impl Read, mode: ReadMode, ...)

# ReadMode derives Debug and PartialEq
grep -A1 "derive" src/lib.rs | grep "Debug, PartialEq"
# Expected: one hit

# Phase 6 hint preserved
grep "подсказка: лучше match" src/lib.rs
# Expected: one hit (line 62)

# Phase 7 hint preserved
grep "подсказка: паниковать" src/lib.rs
# Expected: one hit (line 88)

# Phase 9 hint preserved
grep "подсказка: можно обойтись" src/lib.rs
# Expected: one hit (line 50)
```

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is a mechanical, compiler-driven refactoring. No architectural alternatives exist -- an enum is the idiomatic Rust replacement for `u8` mode constants.

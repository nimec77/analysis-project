# Research: PH-5 -- `u8` constants -> `enum ReadMode`

**Ticket:** PH-5 "Phase 5: `u8` constants -> `enum ReadMode`"
**PRD:** `docs/prd/PH-5.prd.md`
**Phase spec:** `docs/phase/phase-5.md`

---

## 1. Existing Code Analysis

### 1.1 `u8` Mode Constants (lines 5-11 of `src/lib.rs`)

Three public constants define read modes as plain `u8` values, with a Russian hint comment indicating the intent to replace them with an enum:

```rust
// подсказка: лучше использовать enum и match
/// Режим чтения из логов всего подряд
pub const READ_MODE_ALL: u8 = 0;
/// Режим чтения из логов только ошибок
pub const READ_MODE_ERRORS: u8 = 1;
/// Режим чтения из логов только операций, касающихся деген
pub const READ_MODE_EXCHANGES: u8 = 2;
```

The hint at line 5 translates to: "hint: better to use an enum and match." This is the technical debt marker that Phase 5 resolves.

**All occurrences across the codebase (9 total):**

| Location | Usage | Constant |
|---|---|---|
| `src/lib.rs:5` | Hint comment about enum | N/A |
| `src/lib.rs:7` | Constant definition | `READ_MODE_ALL` |
| `src/lib.rs:9` | Constant definition | `READ_MODE_ERRORS` |
| `src/lib.rs:11` | Constant definition | `READ_MODE_EXCHANGES` |
| `src/lib.rs:63` | `if` comparison in filter | `READ_MODE_ALL` |
| `src/lib.rs:66` | `else if` comparison in filter | `READ_MODE_ERRORS` |
| `src/lib.rs:74` | `else if` comparison in filter | `READ_MODE_EXCHANGES` |
| `src/lib.rs:170` | Test call #1 | `READ_MODE_ALL` |
| `src/lib.rs:171` | Test call #2 | `READ_MODE_ALL` |
| `src/main.rs:67` | CLI call | `READ_MODE_ALL` (qualified as `analysis::READ_MODE_ALL`) |

**Verdict:** All 10 references (3 definitions + 1 hint comment + 3 comparisons + 2 test usages + 1 main.rs usage) are affected by this phase.

### 1.2 `read_log()` Function Signature (line 47 of `src/lib.rs`)

```rust
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine> {
```

The `mode` parameter is typed as `u8`. This means the compiler cannot prevent callers from passing arbitrary values like `42` or `255`, which would hit the `panic!` at runtime.

### 1.3 Mode Filtering Logic (lines 62-90 of `src/lib.rs`)

The `if`/`else if` chain compares `mode` against each `u8` constant:

```rust
// подсказка: лучше match
&& if mode == READ_MODE_ALL {
        true
    }
    else if mode == READ_MODE_ERRORS {
        matches!(
            &log.kind,
            LogKind::System(
                SystemLogKind::Error(_)) | LogKind::App(AppLogKind::Error(_)
            )
        )
    }
    else if mode == READ_MODE_EXCHANGES {
        matches!(
            &log.kind,
            LogKind::App(AppLogKind::Journal(
                AppLogJournalKind::BuyAsset(_)
                | AppLogJournalKind::SellAsset(_)
                | AppLogJournalKind::CreateUser{..}
                | AppLogJournalKind::RegisterAsset{..}
                | AppLogJournalKind::DepositCash(_)
                | AppLogJournalKind::WithdrawCash(_)
            ))
        )
    }
    else {
        // подсказка: паниковать в библиотечном коде - нехорошо
        panic!("unknown mode {}", mode)
    }
```

Key observations:
- **Line 62** has a second hint comment: `// подсказка: лучше match` ("hint: better match"). This hint relates to Phase 6 and must be **kept** in Phase 5.
- **Line 88** has a hint comment: `// подсказка: паниковать в библиотечном коде - нехорошо` ("hint: panicking in library code is not good"). This hint relates to Phase 7 and must be **kept** in Phase 5.
- The `panic!` at line 89 uses `{}` format specifier for the `u8` mode value. After changing to the enum, this must become `{:?}` (which requires `Debug` on the enum).
- The `==` comparisons require the enum to derive `PartialEq`.

### 1.4 Test Usage (lines 168-179 of `src/lib.rs`)

```rust
#[test]
fn test_all() {
    assert_eq!(read_log(SOURCE1.as_bytes(), READ_MODE_ALL, vec![]).len(), 1);
    let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);
    ...
}
```

Both test invocations use `READ_MODE_ALL`. They must be updated to `ReadMode::All`.

### 1.5 CLI Usage (`src/main.rs`, line 67)

```rust
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

Uses the fully-qualified constant. Must be updated to `analysis::ReadMode::All`.

---

## 2. Patterns Used

| Pattern | Where | Notes |
|---|---|---|
| `u8` constants as enum substitute | `READ_MODE_ALL/ERRORS/EXCHANGES` | Classic C-style pattern. Lacks exhaustiveness checking and type safety. |
| `if`/`else if` chain for dispatch | `read_log()` filter logic | Since `u8` cannot be exhaustively matched, requires a `panic!` default arm. |
| `panic!` on invalid input | `read_log()` line 89 | Runtime crash on unexpected mode value. Exists because the compiler cannot verify that all `u8` values are covered. |
| Hint comments as technical debt markers | Lines 5, 62, 88 | Russian-language comments flagging known improvements. |

---

## 3. Implementation Path

### 3.1 Task 5.1: Define `enum ReadMode` and remove constants

**Remove (lines 5-11 of `src/lib.rs`):**
```rust
// подсказка: лучше использовать enum и match
/// Режим чтения из логов всего подряд
pub const READ_MODE_ALL: u8 = 0;
/// Режим чтения из логов только ошибок
pub const READ_MODE_ERRORS: u8 = 1;
/// Режим чтения из логов только операций, касающихся деген
pub const READ_MODE_EXCHANGES: u8 = 2;
```

**Replace with:**
```rust
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

Notes on derive macros:
- **`PartialEq`** is required because the `if` chain uses `==` comparisons (e.g., `mode == ReadMode::All`). This becomes unnecessary after Phase 6 converts to `match`, but causes no harm.
- **`Debug`** is required because the `panic!` format string at line 89 will use `{:?}` to format the enum value. This becomes unnecessary after Phase 7 removes the `panic!`, but causes no harm.

### 3.2 Task 5.2: Update `read_log()` signature

**From:**
```rust
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

**To:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

Only the type of `mode` changes: `u8` becomes `ReadMode`.

### 3.3 Task 5.2 (continued): Update `if` chain comparisons

**From:**
```rust
if mode == READ_MODE_ALL {
```
```rust
else if mode == READ_MODE_ERRORS {
```
```rust
else if mode == READ_MODE_EXCHANGES {
```

**To:**
```rust
if mode == ReadMode::All {
```
```rust
else if mode == ReadMode::Errors {
```
```rust
else if mode == ReadMode::Exchanges {
```

The `if`/`else if` structure, including the `else { panic!(...) }` arm, is preserved. Only the constant names change to enum variants. This is the strict Phase 5 scope boundary -- Phase 6 converts the entire chain to `match`.

### 3.4 Task 5.2 (continued): Update `panic!` format string

**From:**
```rust
panic!("unknown mode {}", mode)
```

**To:**
```rust
panic!("unknown mode {:?}", mode)
```

The `u8` type implements `Display` (used by `{}`), but enum types by default only implement `Debug` (used by `{:?}`). Since the PRD does not require implementing `Display`, the idiomatic approach is to use `{:?}`.

### 3.5 Task 5.3: Adapt tests

**From:**
```rust
assert_eq!(read_log(SOURCE1.as_bytes(), READ_MODE_ALL, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);
```

**To:**
```rust
assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), ReadMode::All, vec![]);
```

The test module already has `use super::*;`, so `ReadMode::All` is in scope without additional imports.

### 3.6 Task 5.2 (continued): Adapt `main.rs`

**From:**
```rust
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

**To:**
```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
```

Uses fully-qualified `analysis::ReadMode::All` path since `main.rs` does not import the enum with a `use` statement.

---

## 4. What Gets Removed

| Entity | File | Line(s) | Action |
|---|---|---|---|
| Hint comment `// подсказка: лучше использовать enum и match` | `src/lib.rs` | 5 | **Remove** -- debt resolved by this phase |
| Russian doc comment for `READ_MODE_ALL` | `src/lib.rs` | 6 | **Remove** -- constant being deleted |
| `pub const READ_MODE_ALL: u8 = 0;` | `src/lib.rs` | 7 | **Remove** -- replaced by `ReadMode::All` |
| Russian doc comment for `READ_MODE_ERRORS` | `src/lib.rs` | 8 | **Remove** -- constant being deleted |
| `pub const READ_MODE_ERRORS: u8 = 1;` | `src/lib.rs` | 9 | **Remove** -- replaced by `ReadMode::Errors` |
| Russian doc comment for `READ_MODE_EXCHANGES` | `src/lib.rs` | 10 | **Remove** -- constant being deleted |
| `pub const READ_MODE_EXCHANGES: u8 = 2;` | `src/lib.rs` | 11 | **Remove** -- replaced by `ReadMode::Exchanges` |

Total: 7 lines removed (1 hint comment + 3 doc comments + 3 constant definitions), replaced by the `ReadMode` enum definition (approximately 8 lines including doc comments and derives).

---

## 5. What Gets Kept

| Entity | File | Line | Reason |
|---|---|---|---|
| Hint comment `// подсказка: лучше match` | `src/lib.rs` | 62 | Phase 6 scope -- not resolved by this phase |
| Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` | `src/lib.rs` | 88 | Phase 7 scope -- not resolved by this phase |
| Hint comment `// подсказка: можно обойтись итераторами` | `src/lib.rs` | 50 | Phase 9 scope |
| The `if`/`else if`/`else` chain structure | `src/lib.rs` | 63-90 | Phase 6 converts to `match` |
| The `panic!("unknown mode {:?}", mode)` arm | `src/lib.rs` | 89 | Phase 7 removes it |
| All 16 existing tests | `src/lib.rs`, `src/parse.rs` | Various | No tests deleted per constraints |
| `LogIterator` struct and impls | `src/lib.rs` | 14-43 | Unrelated to this phase |
| The `for` loop with manual filter | `src/lib.rs` | 51-93 | Phase 9 scope |

---

## 6. Dependencies and Layers

```
main.rs  -->  lib.rs::read_log(impl Read, ReadMode, Vec<u32>)
                |
                +--> LogIterator<R>::new(R) --> BufReader<R> --> Lines --> Filter
                |
                +--> mode filtering: if mode == ReadMode::All { ... }
                     else if mode == ReadMode::Errors { ... }
                     else if mode == ReadMode::Exchanges { ... }
                     else { panic!(...) }
```

The change propagates outward from the constant definitions:
1. Remove constants and hint comment, define `enum ReadMode`
2. Change `read_log()` signature: `mode: u8` becomes `mode: ReadMode`
3. Follow compiler errors to update all comparisons in the `if` chain
4. Follow compiler errors to update `main.rs` call site
5. Follow compiler errors to update test call sites

This is a "compiler-driven refactoring" as noted in the PRD -- change the type, then fix every error the compiler reports.

---

## 7. Scope Boundaries with Adjacent Phases

This phase has strict scope boundaries with Phases 6 and 7:

| Concern | Phase 5 (this phase) | Phase 6 | Phase 7 |
|---|---|---|---|
| `u8` constants -> `enum ReadMode` | **In scope** | Out of scope | Out of scope |
| `if`/`else if` chain -> `match` | Out of scope | **In scope** | Out of scope |
| `panic!` removal / `Result` return | Out of scope | Out of scope | **In scope** |
| `PartialEq` derive on `ReadMode` | **Required** (for `==`) | May become unnecessary | May become unnecessary |
| `Debug` derive on `ReadMode` | **Required** (for `panic!` `{:?}`) | Still needed | May become unnecessary |

Phase 6 depends on Phase 5 (needs the enum to exist). Phase 7 depends on Phase 5 (needs the enum to make `match` exhaustive, eliminating the `panic!`).

---

## 8. Limitations and Risks

| Risk | Assessment |
|---|---|
| `PartialEq` derive needed for `==` comparisons | **Certain, low impact.** Required while the `if` chain remains. `#[derive(PartialEq)]` is one line and idiomatic Rust. Phase 6 may make it unnecessary, but it causes no harm to keep. |
| `Debug` derive needed for `panic!` format string | **Certain, low impact.** The `panic!` at line 89 currently uses `{}` (Display) for `u8`. After the change, the enum does not implement `Display` by default, so `{:?}` (Debug) must be used instead. `#[derive(Debug)]` is required. Phase 7 removes the `panic!` entirely. |
| `main.rs` breaks due to removed constants | **Certain, trivial fix.** Replace `analysis::READ_MODE_ALL` with `analysis::ReadMode::All`. |
| Tests break due to removed constants | **Certain, trivial fix.** Replace `READ_MODE_ALL` with `ReadMode::All`. |
| The `else { panic!(...) }` arm becomes unreachable in practice | **Expected but harmless.** After Phase 5, no caller can construct an invalid mode value (the enum has only 3 variants). However, the `if`/`else if` chain is not a `match`, so the compiler does not know the `else` is unreachable. Phase 6 replaces with `match`, and Phase 7 removes the default arm. No action needed in Phase 5. |
| External callers depend on `READ_MODE_ALL`, `READ_MODE_ERRORS`, or `READ_MODE_EXCHANGES` | **Very low risk.** This is an internal project with no known external consumers. The constants can be safely removed. |

---

## 9. Deviations from Requirements

None. The current codebase state matches the PRD's "Current codebase state" section exactly:

- Three `u8` constants are defined at lines 7, 9, 11 -- matches PRD
- Hint comment at line 5: `// подсказка: лучше использовать enum и match` -- matches PRD
- `read_log()` accepts `mode: u8` at line 47 -- matches PRD
- `if`/`else if` chain comparing `mode` to constants at lines 63-90 -- matches PRD
- `panic!("unknown mode {}", mode)` at line 89 -- matches PRD
- `main.rs` uses `analysis::READ_MODE_ALL` at line 67 -- matches PRD
- Tests use `READ_MODE_ALL` at lines 170-171 -- matches PRD

All 16 existing tests pass (`cargo test` verified). No code deviates from the requirements.

---

## 10. Resolved Questions

The PRD has no open questions. The user confirmed proceeding with default requirements.

---

## 11. New Technical Questions Discovered During Research

None. This is a mechanical, well-scoped refactoring. The implementation path is straightforward:

1. Define the enum with `#[derive(Debug, PartialEq)]`.
2. Remove the three constants and the hint comment.
3. Change `read_log()` parameter type from `u8` to `ReadMode`.
4. Update the three `if` comparisons to use enum variants.
5. Change the `panic!` format from `{}` to `{:?}`.
6. Update both test calls and the `main.rs` call.

Every step is a direct substitution with no ambiguity.

---

## 12. Verification

Per the acceptance criteria:

```bash
cargo test                # All 16 tests pass (no deletions)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:
- `grep -r "READ_MODE_ALL" src/` returns zero hits
- `grep -r "READ_MODE_ERRORS" src/` returns zero hits
- `grep -r "READ_MODE_EXCHANGES" src/` returns zero hits
- `grep -r "pub const.*u8" src/lib.rs` returns zero hits for mode constants
- `grep -r "подсказка: лучше использовать enum и match" src/` returns zero hits
- `grep -r "enum ReadMode" src/lib.rs` returns one hit (the definition)
- `read_log()` parameter type is `ReadMode` (not `u8`)
- `ReadMode` has variants `All`, `Errors`, `Exchanges`
- `ReadMode` derives `Debug` and `PartialEq`
- Hint comment `// подсказка: лучше match` (line 62) is preserved
- Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` (line 88) is preserved

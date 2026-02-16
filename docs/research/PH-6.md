# Research: PH-6 -- `match` instead of `if` chain

**Ticket:** PH-6 "Phase 6: `match` instead of `if` chain"
**PRD:** `docs/prd/PH-6.prd.md`
**Phase spec:** `docs/phase/phase-6.md`

---

## 1. Existing Code Analysis

### 1.1 `ReadMode` Enum (lines 5-14 of `src/lib.rs`)

Phase 5 already replaced the `u8` constants with a proper enum:

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

Key observations:
- `ReadMode` does **not** derive `Copy` or `Clone`.
- `PartialEq` is derived -- required by the current `if` chain's `==` comparisons. After Phase 6, `PartialEq` is no longer structurally required by the filtering logic (pattern matching does not use `==`), but may be retained for tests or future use.
- `Debug` is derived -- required by the `panic!("unknown mode {:?}", mode)` at line 92. After Phase 6 eliminates the `panic!` arm (as a natural consequence of exhaustive `match`), `Debug` is no longer structurally required by the filtering logic, but may be retained for Phase 7 or general utility.

### 1.2 `read_log()` Function Signature (line 50 of `src/lib.rs`)

```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

The `mode` parameter is taken **by value** (owned `ReadMode`). This is important for the `match` conversion -- see Section 3.1 for ownership analysis.

### 1.3 The `if`/`else if` Chain (lines 65-93 of `src/lib.rs`)

The full filtering expression inside the `for` loop body:

```rust
if request_ids.is_empty() || {
    let mut request_id_found = false;
    for request_id in &request_ids {
        if *request_id == log.request_id {
            request_id_found = true;
            break;
        }
    }
    request_id_found
}
// подсказка: лучше match
&& if mode == ReadMode::All {
        true
    }
    else if mode == ReadMode::Errors {
        matches!(
            &log.kind,
            LogKind::System(
                SystemLogKind::Error(_)) | LogKind::App(AppLogKind::Error(_)
            )
        )
    }
    else if mode == ReadMode::Exchanges {
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
        panic!("unknown mode {:?}", mode)
    }
{
    collected.push(log);
}
```

Key observations:
- The mode-filtering `if` expression is the **right-hand operand** of a `&&` within the outer `if` condition. The replacement `match` expression will occupy the same syntactic position.
- The hint comment at line 65 (`// подсказка: лучше match` -- "hint: better to use match") is the technical debt marker that Phase 6 resolves.
- The `else { panic!(...) }` arm at lines 90-93 contains a second hint comment (`// подсказка: паниковать в библиотечном коде - нехорошо` -- "hint: panicking in library code is not good"). This `panic!` arm and its hint become structurally unnecessary when the `match` is exhaustive over all three variants.

### 1.4 Hint Comments in `src/lib.rs`

Three hint comments remain in `src/lib.rs` after Phase 5:

| Line | Comment | Phase |
|---|---|---|
| 53 | `// подсказка: можно обойтись итераторами` | Phase 9 |
| 65 | `// подсказка: лучше match` | **Phase 6 (this phase)** |
| 91 | `// подсказка: паниковать в библиотечном коде - нехорошо` | Phase 7 (but annotates the `panic!` arm eliminated by Phase 6) |

### 1.5 Call Sites (unchanged by Phase 6)

| Location | Code | Impact |
|---|---|---|
| `src/main.rs:67` | `analysis::read_log(file, analysis::ReadMode::All, vec![])` | No change needed -- function signature unchanged |
| `src/lib.rs:173` | `read_log(SOURCE1.as_bytes(), ReadMode::All, vec![])` | No change needed |
| `src/lib.rs:174` | `read_log(SOURCE.as_bytes(), ReadMode::All, vec![])` | No change needed |

### 1.6 Sole `panic!` in `src/lib.rs`

```
src/lib.rs:92:  panic!("unknown mode {:?}", mode)
```

This is the **only** `panic!` in `src/lib.rs`. It exists solely because the `if`/`else if` chain cannot benefit from compiler-verified exhaustiveness. With an exhaustive `match` on a three-variant enum using three explicit arms (no wildcard), this arm becomes structurally impossible and is removed as a natural consequence of the conversion.

---

## 2. Patterns Used

| Pattern | Where | Notes |
|---|---|---|
| `if`/`else if` chain for enum dispatch | `read_log()` lines 66-93 | Non-idiomatic for Rust enums; does not benefit from exhaustiveness checking. |
| `==` comparison on enum variants | `mode == ReadMode::All`, etc. | Requires `PartialEq` derive; unnecessary with `match`. |
| `matches!` macro for nested pattern matching | Lines 70-75, 78-88 | Idiomatic; remains unchanged inside the `match` arms. |
| `panic!` as unreachable fallback | Line 92 | Exists because the `if` chain lacks exhaustiveness. Eliminated by exhaustive `match`. |
| Hint comments as technical debt markers | Lines 53, 65, 91 | Russian-language comments flagging known improvements. |
| Inline `if` expression as `&&` operand | Lines 66-93 | The `if` block evaluates to `bool`; the `match` block will also evaluate to `bool` in the same position. |

---

## 3. Implementation Path

### 3.1 Ownership Analysis: `match mode` vs. `match &mode`

The `mode` parameter is `ReadMode` (owned, by value). `ReadMode` does **not** derive `Copy` or `Clone`.

The mode-filtering expression is inside a `for log in logs { ... }` loop. The `if mode == ReadMode::All` comparison works on every iteration because `PartialEq::eq` borrows `&self` and `&other` -- it does not consume `mode`.

However, `match mode { ... }` would **move** `mode` on the first iteration of the loop, causing a compile error on subsequent iterations ("use of moved value: `mode`").

**Solution:** Match on a reference: `match &mode { ... }`. Rust's match ergonomics allow the arm patterns to be written as `ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges` (without `&`) even when matching on `&mode`. This is the idiomatic approach.

Alternatively, adding `#[derive(Copy, Clone)]` to `ReadMode` would allow `match mode { ... }` without the reference, since copying a simple fieldless enum is trivial. However, the PRD's "After" example shows `match mode { ... }` (not `match &mode`). Since the PRD does not explicitly require or forbid adding `Copy`, either approach is valid. The reference approach (`match &mode`) requires the fewest changes and does not modify the enum definition.

**Recommended approach:** Use `match &mode { ... }` to avoid adding new derives and to minimize scope.

### 3.2 Task 6.1: Replace the `if`/`else if` chain with `match`

**Remove (lines 65-93 of `src/lib.rs`):**
```rust
        // подсказка: лучше match
        && if mode == ReadMode::All {
                true
            }
            else if mode == ReadMode::Errors {
                matches!(
                    &log.kind,
                    LogKind::System(
                        SystemLogKind::Error(_)) | LogKind::App(AppLogKind::Error(_)
                    )
                )
            }
            else if mode == ReadMode::Exchanges {
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
                panic!("unknown mode {:?}", mode)
            }
```

**Replace with:**
```rust
        && match &mode {
            ReadMode::All => true,
            ReadMode::Errors => matches!(
                &log.kind,
                LogKind::System(
                    SystemLogKind::Error(_)) | LogKind::App(AppLogKind::Error(_)
                )
            ),
            ReadMode::Exchanges => matches!(
                &log.kind,
                LogKind::App(AppLogKind::Journal(
                    AppLogJournalKind::BuyAsset(_)
                    | AppLogJournalKind::SellAsset(_)
                    | AppLogJournalKind::CreateUser{..}
                    | AppLogJournalKind::RegisterAsset{..}
                    | AppLogJournalKind::DepositCash(_)
                    | AppLogJournalKind::WithdrawCash(_)
                ))
            ),
        }
```

Changes in this replacement:
1. **Hint comment `// подсказка: лучше match` (line 65) removed** -- the debt it marks is resolved.
2. **`if`/`else if`/`else` structure replaced with `match &mode { ... }`** -- three explicit arms, no wildcard.
3. **Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` (line 91) removed** -- the `panic!` it annotated no longer exists.
4. **`panic!("unknown mode {:?}", mode)` (line 92) removed** -- the exhaustive match makes it structurally unreachable.
5. **Filtering logic in each arm is identical** -- the `matches!` macro bodies are copied verbatim from the `if` branches.

---

## 4. What Gets Removed

| Entity | File | Line(s) | Action |
|---|---|---|---|
| Hint comment `// подсказка: лучше match` | `src/lib.rs` | 65 | **Remove** -- debt resolved by this phase |
| `if mode == ReadMode::All { true }` | `src/lib.rs` | 66-68 | **Replace** with `ReadMode::All => true,` |
| `else if mode == ReadMode::Errors { matches!(...) }` | `src/lib.rs` | 69-76 | **Replace** with `ReadMode::Errors => matches!(...),` |
| `else if mode == ReadMode::Exchanges { matches!(...) }` | `src/lib.rs` | 77-89 | **Replace** with `ReadMode::Exchanges => matches!(...),` |
| `else { panic!("unknown mode {:?}", mode) }` | `src/lib.rs` | 90-93 | **Remove** -- no longer needed (exhaustive match) |
| Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` | `src/lib.rs` | 91 | **Remove** -- annotated `panic!` is gone |

---

## 5. What Gets Kept

| Entity | File | Line | Reason |
|---|---|---|---|
| `ReadMode` enum definition with `#[derive(Debug, PartialEq)]` | `src/lib.rs` | 5-14 | Unchanged; `PartialEq` and `Debug` are no longer structurally required by filtering logic but are harmless to retain |
| Hint comment `// подсказка: можно обойтись итераторами` | `src/lib.rs` | 53 | Phase 9 scope |
| `read_log()` function signature | `src/lib.rs` | 50 | Unchanged |
| The outer `if` condition structure (request ID filtering + `&&`) | `src/lib.rs` | 55-64 | Unchanged |
| The `for` loop with manual collect | `src/lib.rs` | 54-97 | Phase 9 scope |
| `LogIterator` struct and impls | `src/lib.rs` | 17-47 | Unrelated to this phase |
| All call sites (`main.rs:67`, test at `lib.rs:173-174`) | `src/main.rs`, `src/lib.rs` | Various | Function signature unchanged |
| All existing tests | `src/lib.rs`, `src/parse.rs` | Various | No tests deleted per constraints |

---

## 6. Dependencies and Layers

```
main.rs  -->  lib.rs::read_log(impl Read, ReadMode, Vec<u32>)
                |
                +--> LogIterator<R>::new(R) --> BufReader<R> --> Lines --> Filter
                |
                +--> for log in logs {
                |        if (request_id_filter) && match &mode {
                |            ReadMode::All => true,
                |            ReadMode::Errors => matches!(...),
                |            ReadMode::Exchanges => matches!(...),
                |        } { collected.push(log) }
                |    }
```

The change is entirely internal to the `read_log()` function body. No call sites, signatures, or return types change. The refactoring is confined to the mode-filtering expression within the `for` loop.

---

## 7. Scope Boundaries with Adjacent Phases

| Concern | Phase 5 (done) | Phase 6 (this phase) | Phase 7 (next) | Phase 9 (future) |
|---|---|---|---|---|
| `u8` constants -> `enum ReadMode` | Resolved | -- | -- | -- |
| `if`/`else if` chain -> `match` | -- | **In scope** | -- | -- |
| Remove hint `лучше match` | -- | **In scope** | -- | -- |
| Remove `panic!` arm (natural consequence) | -- | **In scope** | -- | -- |
| Remove hint `паниковать в библиотечном коде` | -- | **In scope** (annotated code is gone) | -- | -- |
| Return `Result` instead of `panic!` | -- | -- | **In scope** | -- |
| Loops -> iterators | -- | -- | -- | **In scope** |
| `PartialEq` derive on `ReadMode` | Required (for `==`) | No longer required but retained | May become unnecessary | -- |
| `Debug` derive on `ReadMode` | Required (for `{:?}` in `panic!`) | No longer required but retained | May become unnecessary | -- |

Phase 7 ("Result instead of panic!") benefits from Phase 6 because the exhaustive `match` eliminates the only `panic!` in the filtering logic. Phase 7 can then focus on any remaining `panic!` sites elsewhere or on changing `read_log()`'s return type to `Result` for other fallible operations.

---

## 8. Limitations and Risks

| Risk | Assessment |
|---|---|
| `match` in expression position as `&&` operand may require careful formatting | **Low impact.** Rust allows `match` in any expression position. `rustfmt` will handle indentation. The `match` block replaces an `if` block in the same syntactic position. |
| Ownership of `mode`: `match mode` would move the value in a loop | **Medium likelihood, trivially mitigated.** Since `ReadMode` does not derive `Copy`, `match mode { ... }` inside the `for` loop would move `mode` on the first iteration. Use `match &mode { ... }` instead. Rust's match ergonomics allow arm patterns without `&`. |
| Removing the `panic!` arm overlaps with Phase 7 scope | **Low impact.** The `panic!` disappears naturally because an exhaustive `match` on three variants needs no default arm. Phase 7 addresses `Result` returns for broader error handling, not specifically this `panic!`. |
| `PartialEq` removal could break code | **No risk.** This phase does not remove `PartialEq`. It simply notes that `PartialEq` is no longer structurally required. |
| `Debug` removal could break code | **No risk.** This phase does not remove `Debug`. It simply notes that `Debug` is no longer structurally required after the `panic!` is gone. |
| Trailing commas after `matches!()` arms | **Low impact, important for correctness.** In a `match` expression, each arm must end with a comma (or be the last arm followed by `}`). The `matches!()` macro invocations must be followed by `,` to separate the arms. |

---

## 9. Deviations from Requirements

None. The current codebase state matches the PRD's "Current codebase state (gap analysis)" section exactly:

- `ReadMode` enum defined at lines 7-14 with `#[derive(Debug, PartialEq)]` and three variants (`All`, `Errors`, `Exchanges`) -- matches PRD.
- `if`/`else if` chain at lines 66-93 using `==` comparisons against `ReadMode` variants -- matches PRD.
- Hint comment `// подсказка: лучше match` at line 65 -- matches PRD.
- `panic!("unknown mode {:?}", mode)` at line 92 with hint comment at line 91 -- matches PRD.
- `read_log()` signature takes `mode: ReadMode` at line 50 -- matches PRD.
- `mode` is passed by value (owned `ReadMode`) -- matches PRD's risk analysis.

All existing tests pass (`cargo test`). No code deviates from the requirements.

---

## 10. Resolved Questions

The PRD has no open questions. The user confirmed proceeding with default requirements.

---

## 11. New Technical Questions Discovered During Research

### 11.1 `match mode` vs. `match &mode`: Ownership in a Loop

The PRD's "After" example in Scenario 1 shows:
```rust
&& match mode {
    ReadMode::All => true,
    ...
}
```

However, `mode` is owned (`ReadMode`, no `Copy`), and this `match` is inside a `for` loop. `match mode` would move `mode` on the first iteration, causing a compile error on subsequent iterations.

**Two valid resolutions:**
1. **`match &mode { ... }`** -- match on a reference. No changes to the enum definition. Match ergonomics allow patterns like `ReadMode::All` without explicit `&`. This is the minimal-change approach.
2. **Add `#[derive(Copy, Clone)]` to `ReadMode`** and use `match mode { ... }`. This allows copying the enum on each iteration. `ReadMode` is a simple fieldless enum, so `Copy` is trivial. This matches the PRD's example more literally.

Both approaches produce identical runtime behavior. The choice is a style decision. The PRD does not explicitly address this, so either is acceptable.

---

## 12. Verification

Per the acceptance criteria:

```bash
cargo test                # All tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:

| Metric | Expected |
|---|---|
| `if mode == ReadMode::` occurrences in `src/lib.rs` | Zero |
| `match` on `mode` occurrences in `src/lib.rs` | One (in `read_log()`) |
| Hint comment `// подсказка: лучше match` in `src/lib.rs` | Removed |
| `panic!("unknown mode` occurrences in `src/lib.rs` | Zero |
| Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` in `src/lib.rs` | Removed |
| Hint comment `// подсказка: можно обойтись итераторами` in `src/lib.rs` | Preserved (Phase 9) |
| All call sites (`main.rs`, tests) | Unchanged |
| `ReadMode` enum definition | Unchanged (or with added `Copy, Clone` if option 2 chosen) |
| Number of `match` arms | Exactly 3 (no wildcard/default) |

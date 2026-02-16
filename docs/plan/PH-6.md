# Implementation Plan: PH-6 -- `match` instead of `if` chain

**Status:** PLAN_APPROVED
**Ticket:** PH-6 "Phase 6: `match` instead of `if` chain"
**PRD:** `docs/prd/PH-6.prd.md`
**Research:** `docs/research/PH-6.md`
**Phase spec:** `docs/phase/phase-6.md`
**ADR:** `docs/adr/PH-6.md` (match ownership strategy)

---

## Components

### 1. Hint comment `// подсказка: лучше match` (to be removed)

**File:** `src/lib.rs`, line 65

This Russian-language hint comment marks the `if`/`else if` chain as recognized technical debt. It is removed because Phase 6 resolves the debt it identifies.

### 2. The `if`/`else if`/`else` chain (to be replaced)

**File:** `src/lib.rs`, lines 66-93

The mode-filtering logic currently uses an `if mode == ReadMode::All { ... } else if mode == ReadMode::Errors { ... } else if mode == ReadMode::Exchanges { ... } else { panic!(...) }` chain. This is replaced by a `match &mode { ... }` expression with three explicit arms (no wildcard), in the same syntactic position (right-hand operand of `&&`).

```rust
// REMOVE (lines 65-93):
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

```rust
// REPLACE WITH:
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

### 3. `panic!` arm and its hint comment (naturally eliminated)

**File:** `src/lib.rs`, lines 90-93

The `else { panic!("unknown mode {:?}", mode) }` arm and its hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` both disappear as a structural consequence of the exhaustive `match`. With three explicit arms covering all three `ReadMode` variants, there is no need for a default/wildcard arm.

### 4. `ReadMode` enum definition (unchanged)

**File:** `src/lib.rs`, lines 5-14

The enum retains its existing `#[derive(Debug, PartialEq)]`. After this phase, `PartialEq` is no longer structurally required by the filtering logic (pattern matching does not use `==`), and `Debug` is no longer structurally required (the `panic!` that used `{:?}` is gone). Both derives are retained as they are harmless and may serve future uses or tests.

### 5. `read_log()` function signature (unchanged)

**File:** `src/lib.rs`, line 50

```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

No signature change. The refactoring is entirely within the function body.

### 6. Call sites (unchanged)

| Location | Code | Impact |
|---|---|---|
| `src/main.rs:67` | `analysis::read_log(file, analysis::ReadMode::All, vec![])` | No change needed |
| `src/lib.rs:173` | `read_log(SOURCE1.as_bytes(), ReadMode::All, vec![])` | No change needed |
| `src/lib.rs:174` | `read_log(SOURCE.as_bytes(), ReadMode::All, vec![])` | No change needed |

### 7. Preserved hint comment (out of scope)

**File:** `src/lib.rs`, line 53

The hint `// подсказка: можно обойтись итераторами` is Phase 9 scope and must remain untouched.

---

## API Contract

No API changes. The public interface remains identical:

```rust
// Public enum (unchanged)
#[derive(Debug, PartialEq)]
pub enum ReadMode {
    All,
    Errors,
    Exchanges,
}

// Public function (signature unchanged)
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

The refactoring is entirely internal to the `read_log()` function body. No callers need modification.

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
  |    1. check request_id filter (unchanged)
  |    2. check mode filter:
  |       match &mode {
  |           ReadMode::All => true,
  |           ReadMode::Errors => matches!(...),
  |           ReadMode::Exchanges => matches!(...),
  |       }
  v
Vec<LogLine> returned to caller
```

The data flow is identical to the pre-refactoring version. The only change is the internal control flow structure: `if`/`else if`/`else` becomes `match`.

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|---|---|
| Zero external dependencies | No new crates. Only `std` types used. |
| No behavior changes | Same input produces same output. Each `match` arm returns the same boolean value as its `if` branch counterpart. |
| No test deletions | No tests are modified or deleted. Function signature unchanged, so tests compile without adaptation. |
| Compiler-verified exhaustiveness | The `match` has three explicit arms for three variants, no wildcard. Adding a fourth variant to `ReadMode` will produce a compile error at this `match` site. |
| Scope boundary | Only the `if`/`else if` chain to `match` conversion. The `Result` return type (Phase 7), iterator refactoring (Phase 9), and other concerns are untouched. |
| Hint comment resolution | `// подсказка: лучше match` is removed (debt resolved). `// подсказка: паниковать в библиотечном коде - нехорошо` is removed (annotated code is gone). `// подсказка: можно обойтись итераторами` is preserved (Phase 9). |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `match mode` moves `mode` in the loop, causing compile error on second iteration | Certain (if `match mode` used) | High (won't compile) | Use `match &mode` instead. See ADR `docs/adr/PH-6.md` for the decision. Rust's match ergonomics allow arm patterns without `&`. |
| `match` expression in `&&` operand position requires careful formatting | Medium | Low | Rust allows `match` in any expression position. `rustfmt` handles indentation. The `match` replaces an `if` in the same syntactic position. |
| Trailing commas after `matches!()` arms are easy to forget | Low | Medium (won't compile) | Each arm except the last must end with `,`. In practice, all three arms should use trailing commas for consistency. Compiler will catch missing commas. |
| Removing the `panic!` arm overlaps with Phase 7 scope | Low | Low | The `panic!` disappears naturally because an exhaustive `match` needs no default arm. Phase 7 addresses `Result` returns for broader error handling, which is a separate concern. |
| `PartialEq` removal could break code | None | None | This phase does not remove `PartialEq`. It is retained on the enum. |

---

## Deviations to Fix

None. The research document (section 9) confirms the current codebase state matches the PRD's gap analysis exactly:

- `ReadMode` enum at lines 5-14 with `#[derive(Debug, PartialEq)]` and three variants -- matches PRD.
- `if`/`else if` chain at lines 66-93 using `==` comparisons -- matches PRD.
- Hint comment `// подсказка: лучше match` at line 65 -- matches PRD.
- `panic!("unknown mode {:?}", mode)` at line 92 with hint at line 91 -- matches PRD.
- `read_log()` signature takes `mode: ReadMode` by value at line 50 -- matches PRD.

No code deviates from requirements.

---

## Implementation Tasks

### Task 6.1: Replace the `if`/`else if` chain with `match` on `ReadMode`

**File:** `src/lib.rs`

This is the sole task for this phase. It is a single atomic change to lines 65-93.

**Remove** lines 65-93 (the hint comment, the entire `if`/`else if`/`else` chain including the `panic!` arm and its hint comment).

**Replace with** a `match &mode` expression with three explicit arms:

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

Key details:
1. **`match &mode`** (not `match mode`): Matches on a reference to avoid moving `mode` inside the `for` loop. See ADR `docs/adr/PH-6.md`.
2. **Three explicit arms, no wildcard**: `ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges`. Compiler verifies exhaustiveness.
3. **Hint comment `// подсказка: лучше match` removed**: The debt it marks is resolved.
4. **Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` removed**: The `panic!` it annotated no longer exists.
5. **`panic!("unknown mode {:?}", mode)` removed**: No default arm needed in an exhaustive match.
6. **Filtering logic per arm is identical**: The `matches!` macro bodies are copied verbatim from the `if` branches.
7. **Trailing commas**: Each arm ends with `,` (including the last, for consistency).

**Verify:**

```bash
cargo build   # Compiles without error
cargo test    # All tests pass
cargo run -- example.log  # Output identical to pre-refactoring
```

---

## Verification Checklist

After the task is complete, verify all metrics from the PRD:

```bash
# All tests pass (no deletions)
cargo test

# CLI output identical to pre-refactoring
cargo run -- example.log

# Zero 'if mode == ReadMode::' occurrences
grep "if mode == ReadMode::" src/lib.rs
# Expected: zero hits

# One 'match' on mode
grep "match.*mode" src/lib.rs
# Expected: one hit (match &mode)

# Hint comment removed
grep "подсказка: лучше match" src/lib.rs
# Expected: zero hits

# panic! arm removed
grep 'panic!("unknown mode' src/lib.rs
# Expected: zero hits

# Panic hint comment removed
grep "подсказка: паниковать" src/lib.rs
# Expected: zero hits

# Phase 9 hint preserved
grep "подсказка: можно обойтись" src/lib.rs
# Expected: one hit (line 53)

# No wildcard arm in match
grep "_ =>" src/lib.rs
# Expected: zero hits (no default arm)

# ReadMode enum unchanged
grep "derive.*Debug.*PartialEq" src/lib.rs
# Expected: one hit

# All call sites unchanged
grep "ReadMode::All" src/lib.rs src/main.rs
# Expected: hits at test lines and main.rs, unchanged
```

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is a single mechanical replacement. The only architectural decision (`match &mode` vs. `match mode` with `Copy` derive) is resolved in the ADR.

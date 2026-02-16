# PRD: Phase 6 — `match` instead of `if` chain

**Status:** PRD_READY
**Ticket:** PH-6 "Phase 6: `match` instead of `if` chain"
**Phase:** 6 of 12 (see `docs/tasklist.md`)
**Dependencies:** Phase 5 (complete)
**Blocked by:** Nothing (Phase 5 is done)
**Blocks:** Phase 7 (`Result` instead of `panic!` -- benefits from exhaustive `match` removing the need for a default arm)

---

## Context / Idea

Phase 6 targets the replacement of the `if mode == ... else if mode == ...` chain in `read_log()` with a `match` expression on `ReadMode`, making the filtering logic more idiomatic and exhaustive.

The original author left a hint at `src/lib.rs:65`: `// подсказка: лучше match` ("hint: better to use match"), indicating this was recognized technical debt from the start.

### Authoritative specification from `docs/phase/phase-6.md`

**Goal:** Replace the `if mode == ... else if mode == ...` chain with a `match` expression on `ReadMode`, making the filtering logic more idiomatic and exhaustive.

**Tasks:**

- [ ] 6.1 Replace the `if mode == ... else if mode == ...` chain with `match` on `ReadMode`

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** Phase 5 complete

**Implementation Notes:**

- **Hint:** `src/lib.rs:88` -- `// подсказка: лучше match`

### Current codebase state (gap analysis)

After Phase 5, `ReadMode` is already a proper enum with `#[derive(Debug, PartialEq)]`. The current state of `src/lib.rs` relevant to this phase:

- **`ReadMode` enum** (lines 7-14): Already defined as:
  ```rust
  #[derive(Debug, PartialEq)]
  pub enum ReadMode {
      All,
      Errors,
      Exchanges,
  }
  ```
- **`if` chain** (lines 66-93): Mode filtering uses an `if`/`else if` chain with `==` comparisons:
  ```rust
  // подсказка: лучше match
  && if mode == ReadMode::All {
          true
      }
      else if mode == ReadMode::Errors {
          matches!(
              &log.kind,
              LogKind::System(SystemLogKind::Error(_)) | LogKind::App(AppLogKind::Error(_))
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
- **Hint comment** (line 65): `// подсказка: лучше match` -- this is directly addressed by this phase.
- **`panic!` arm** (lines 91-93): The `else { panic!(...) }` arm exists because the `if`/`else if` chain does not benefit from compiler-verified exhaustiveness. With a `match` on an enum, this arm becomes unreachable by construction. However, **removing the `panic!` is Phase 7's scope** -- this phase replaces the `if` chain with `match` but may retain a wildcard/default arm with the `panic!` to stay within scope, or naturally eliminate it since `match` on a three-variant enum with three explicit arms is exhaustive.
- **`PartialEq` derive** (line 7): Currently needed for the `==` comparisons in the `if` chain. After converting to `match`, `PartialEq` is no longer needed for the mode-filtering logic (but may be retained for other uses such as tests).

The `if` chain must be replaced with an equivalent `match` expression.

---

## Goals

1. **Replace the `if`/`else if` chain with `match`.** Convert the mode-filtering logic in `read_log()` from `if mode == ReadMode::All { ... } else if mode == ReadMode::Errors { ... } else if mode == ReadMode::Exchanges { ... } else { panic!(...) }` to `match mode { ReadMode::All => ..., ReadMode::Errors => ..., ReadMode::Exchanges => ... }`.
2. **Achieve compiler-verified exhaustiveness.** With a `match` on the `ReadMode` enum, the compiler guarantees all variants are handled. If a new variant is added in the future, the compiler will emit an error at this match site.
3. **Remove the hint comment.** The hint `// подсказка: лучше match` is addressed and should be removed.
4. **Eliminate the unreachable `panic!` arm (if in scope).** With an exhaustive `match` on three explicit variants, there is no need for a default/wildcard arm. The `else { panic!("unknown mode {:?}", mode) }` becomes structurally impossible. Note: The hint `// подсказка: паниковать в библиотечном коде - нехорошо` on the `panic!` is Phase 7's concern (which addresses `Result` instead of `panic!` more broadly), but the `panic!` arm naturally disappears when the `match` is exhaustive with no wildcard.
5. **Preserve behavior.** Same input produces same output. All existing tests pass; no tests are deleted.

---

## User Stories

1. **As a developer reading `read_log()`**, I want the mode-filtering logic to use a `match` expression instead of an `if`/`else if` chain, so that the code is idiomatic Rust and easier to understand at a glance.
2. **As a maintainer adding a new `ReadMode` variant**, I want the compiler to force me to handle the new variant in the filtering logic, so that I cannot accidentally forget a case (which was not enforced by the `if` chain).
3. **As a developer working on Phase 7**, I want the `panic!` arm to be gone (or clearly isolated), so that replacing it with a `Result` return is straightforward.
4. **As a code reviewer**, I want the filtering logic to use pattern matching, so that each arm's intent is immediately visible without comparing equality against enum variants.

---

## Scenarios

### Scenario 1: `if`/`else if` chain replaced with `match`

**Before:**
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

**After:**
```rust
&& match mode {
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

The hint comment `// подсказка: лучше match` is removed. The `else { panic!(...) }` arm is no longer needed because the `match` is exhaustive over all three `ReadMode` variants. The hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` is also removed since the `panic!` it annotated is gone.

### Scenario 2: `PartialEq` derive becomes optional

**Before (Phase 5):** `PartialEq` is required on `ReadMode` because the `if` chain uses `mode == ReadMode::All` equality comparisons.

**After (Phase 6):** `PartialEq` is no longer needed for the `match` expression (pattern matching does not require `PartialEq`). However, `PartialEq` may be retained on the enum if it is useful elsewhere (e.g., tests, future API). This PRD does not require removing `PartialEq` -- it simply notes that it is no longer structurally required by the filtering logic.

### Scenario 3: No changes to `main.rs` or tests

The `match` conversion is entirely internal to the `read_log()` function body. The function signature, return type, and behavior are unchanged. No call sites need updating. No test assertions change.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| `if mode == ReadMode::` occurrences in `src/lib.rs` | Zero |
| `match mode {` occurrences in `src/lib.rs` | One (in `read_log()`) |
| Hint comment `// подсказка: лучше match` | Removed |
| `panic!("unknown mode` occurrences in `src/lib.rs` | Zero (naturally eliminated by exhaustive match) |
| Hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` | Removed (the `panic!` it annotated is gone) |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** Same input must produce same output.
3. **No test deletions.** Existing tests are not deleted or modified (no changes needed since the function signature is unchanged).
4. **Scope boundary.** This phase addresses only the `if`/`else if` chain to `match` conversion. The broader question of returning `Result` instead of panicking is Phase 7's scope. However, since the exhaustive `match` naturally eliminates the only `panic!` in this code path, the `panic!` removal is a natural consequence of this phase rather than a separate action.
5. **`match` must be exhaustive.** All three `ReadMode` variants (`All`, `Errors`, `Exchanges`) must have explicit arms. No wildcard/default arm (`_ => ...`) should be used, to preserve the compiler's exhaustiveness checking.
6. **Filtering logic must be identical.** Each `match` arm must produce exactly the same boolean result as the corresponding `if` branch.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `match` in expression position within a complex boolean `&&` chain may require careful formatting for readability | Medium | Low | The `match` expression is used inline in the same position as the previous `if` expression. Rust allows `match` in expression position; formatting tools (`rustfmt`) will handle indentation. |
| Removing the `panic!` arm overlaps with Phase 7 scope | Low | Low | The `panic!` disappears naturally because an exhaustive `match` on three variants needs no default arm. This is not "replacing `panic!` with `Result`" (Phase 7's goal) -- it is simply a structural consequence of the `match` conversion. Phase 7 can focus on any remaining `panic!` sites or on changing `read_log()`'s return type to `Result`. |
| `PartialEq` removal could break downstream code | Low | Low | This PRD does not require removing `PartialEq` from `ReadMode`. It remains derived on the enum. If a future phase decides to remove it, that is a separate decision. |
| The `mode` variable is borrowed/moved in `match` | Very Low | Low | `ReadMode` derives `PartialEq` and `Debug`, and the `match` is on a reference or value. Since `ReadMode` is a simple enum without data, it implements `Copy` implicitly if derived, or can be matched by reference. In the current code, `mode` is passed by value (`ReadMode`), so `match mode { ... }` works directly. If `mode` were a reference, `match &mode` or `match *mode` would be needed. The current `read_log()` signature takes `mode: ReadMode` by value, so this is straightforward. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. Phase 5 (the sole dependency) is already done. The refactoring is mechanical: replace the `if`/`else if` chain with a `match` expression on `ReadMode`, remove the hint comment, and verify all tests pass.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | Remove hint comment `// подсказка: лучше match` (line 65). Replace the `if mode == ReadMode::All { ... } else if mode == ReadMode::Errors { ... } else if mode == ReadMode::Exchanges { ... } else { panic!(...) }` chain (lines 66-93) with `match mode { ReadMode::All => true, ReadMode::Errors => matches!(...), ReadMode::Exchanges => matches!(...) }`. Remove the `else { panic!("unknown mode {:?}", mode) }` arm and its hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` (both naturally eliminated by the exhaustive match). |

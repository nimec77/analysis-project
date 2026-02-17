# Summary: PH-13 -- Bug Fix + Dead Code Cleanup

**Ticket:** PH-13 "Phase 13: Bug fix + dead code cleanup"
**Status:** Complete
**Files changed:** `src/parse.rs`

---

## What Was Done

Phase 13 is the first phase in the optimization and improvement track (phases 13-22). It addressed a correctness bug in the `WithdrawCash` parser arm and removed five unused code items from `src/parse.rs`. The work was split into two commits per project convention (one issue category = one commit): a bug fix commit and a dead code cleanup commit.

### Changes

1. **Fixed the `WithdrawCash` parser mapping bug (task 13.1).** In the `Parsable` impl for `AppLogJournalKind`, the `WithdrawCash` parser arm correctly parsed the `"WithdrawCash"` tag and a `UserCash` payload, but the mapping closure produced `AppLogJournalKind::DepositCash(user_cash)` instead of `AppLogJournalKind::WithdrawCash(user_cash)`. This was a copy-paste error from the `DepositCash` arm directly above. The fix changed a single token in the mapping closure.

2. **Added `test_withdraw_cash` regression test (task 13.2).** A new test function was added to the `mod test` block that parses `App::Journal WithdrawCash UserCash{"user_id":"Alice","count":500,}` and asserts the result is `AppLogJournalKind::WithdrawCash(...)`. The test uses the correct `"count"` field name (per the actual `UserCash` parser) and the `nz()` helper consistent with other tests. This test would have failed before the bug fix, preventing regression.

3. **Removed unused `AsIs` struct and its `Parser` impl (task 13.3).** Deleted the `AsIs` struct definition, its `#[derive(Debug, Clone)]` attribute, its doc comment (`/// Парсер, возвращающий результат как есть`), and the `impl Parser for AsIs` block (9 lines total). The struct consumed the entire remaining input and returned it as a `String`, but was never referenced anywhere in the codebase.

4. **Removed unused `all3()` constructor function (task 13.6).** Deleted the `all3` function definition and its doc comments (7 lines total). The function was a convenience constructor for `All<(A0, A1, A2)>` but was never called. The `impl Parser for All<(A0, A1, A2)>` block was retained, as it provides the generic implementation used via direct `All { parser: (...) }` construction.

5. **Removed unused `all4()` constructor function (task 13.7).** Deleted the `all4` function definition and its doc comments (12 lines total). The function was a convenience constructor for `All<(A0, A1, A2, A3)>` but was never called. The `impl Parser for All<(A0, A1, A2, A3)>` block was retained.

6. **Removed unused `Either<Left, Right>` enum (task 13.4).** Deleted the `Either` enum definition and its doc comment (5 lines total). The enum was a generic two-variant `Left`/`Right` type but was never used -- the parser combinators use `Alt` instead.

7. **Removed unused `Status` enum and its `Parsable` impl (task 13.5).** Deleted the `Status` enum definition (with `Ok` and `Err(String)` variants), its doc comment, and the entire `impl Parsable for Status` block (23 lines total). The enum and its parser were never referenced anywhere in the codebase.

---

## Decisions Made

1. **Two separate commits per project convention.** The bug fix (tasks 13.1, 13.2) and the dead code cleanup (tasks 13.3-13.7) are separate issue categories and were committed separately: commit `261175f` for the bug fix + regression test, and commit `cd4ff9d` for the dead code removal.

2. **Dead code deleted in reverse line-number order.** To avoid line-shift issues during editing, dead code items were deleted from bottom to top: `Status` and `Either` (lines 730+), then `all4` and `all3` (lines 329+), then `AsIs` (line 136).

3. **`Parser` trait impls for tuple arities retained.** The `impl Parser for All<(A0, A1, A2)>` and `impl Parser for All<(A0, A1, A2, A3)>` blocks were not deleted -- only the convenience constructor functions (`all3`, `all4`) were removed. The generic implementations may be used via direct struct construction.

4. **Test uses correct field name `"count"` (not `"cash"`).** The PRD scenario illustratively used `"cash":500` as a field, but the actual `UserCash` parser uses `"count"` as the key name. The regression test uses the authoritative parser field name.

5. **Only `src/parse.rs` modified.** No changes were needed in `src/lib.rs` or `src/main.rs`. All removed items were private (no `pub` visibility), so no external consumers were affected.

---

## Verification

- `cargo build` -- compiles without errors.
- `cargo test` -- all 26 tests pass (25 existing + 1 new `test_withdraw_cash`). No test cases deleted.
- `cargo run -- example.log` -- output identical to pre-phase (no `WithdrawCash` entries in `example.log`).
- The `WithdrawCash` parser arm maps to `AppLogJournalKind::WithdrawCash`, not `DepositCash`.
- `test_withdraw_cash` test is present and passing.
- Zero occurrences of `AsIs` in `src/parse.rs`.
- Zero occurrences of `Either` in `src/parse.rs`.
- Zero occurrences of `enum Status` in `src/parse.rs`.
- Zero occurrences of `fn all3` in `src/parse.rs`.
- Zero occurrences of `fn all4` in `src/parse.rs`.
- No external dependencies added.
- Two commits: `261175f` (bug fix + regression test) and `cd4ff9d` (dead code cleanup).

---

## Impact on Downstream Phases

Phase 13 is the first phase in the optimization and improvement track. The dead code removal reduces cognitive load and removes false leads for subsequent phases. Specifically:

- **Phase 14 (Naming improvements):** Fewer items to review and rename. The removed dead code (`AsIs`, `Either`, `Status`, `all3`, `all4`) will not need naming consideration.
- **Phase 15 (Modularity -- split `parse.rs`):** Fewer items to distribute across sub-modules. The module split will be cleaner with dead code already removed.
- **General:** The `WithdrawCash` bug fix ensures correctness before further refactoring proceeds. Any future changes to the journal parser will operate on correct behavior.

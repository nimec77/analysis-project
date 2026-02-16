# PRD: Phase 3 — Remove `unsafe` transmute

**Status:** APPROVED
**Ticket:** Phase 3: Remove `unsafe` transmute
**Phase:** 3 of 12 (see `docs/tasklist.md`)
**Dependencies:** Phase 2 (complete)
**Blocked by:** Nothing (Phase 2 is done)
**Blocks:** None

---

## Context / Idea

Phase 3 targets the removal of the `unsafe { transmute(...) }` block from `LogIterator::new()` in `src/lib.rs`. The original author left a hint at what was line 40: `// подсказка: unsafe избыточен, да и весь rc - тоже` ("hint: unsafe is redundant, and the entire Rc is too"). The transmute existed to extend the lifetime of a `RefMut` borrow from `RefCell`, creating a `'static` reference that could be stored in the `Lines` iterator alongside the `Rc` that owned the data. Once `Rc<RefCell>` was removed in Phase 2, the transmute became unnecessary.

### Authoritative specification from `docs/phase/phase-3.md`

**Goal:** Replace the `unsafe { transmute(...) }` with safe code, now possible since `Rc<RefCell>` has been removed in Phase 2.

**Tasks:**

- [ ] 3.1 Replace the `unsafe { transmute(...) }` with safe code (possible once `Rc<RefCell>` is gone)

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** Phase 2 complete

**Implementation Notes:**

- **Hint:** `src/lib.rs:40` -- `// подсказка: unsafe избыточен, да и весь rc - тоже`

### Current codebase state (gap analysis)

The `unsafe { transmute(...) }` block was already removed as a side-effect of the Phase 2 refactoring. When `Rc<RefCell>` was eliminated, the `RefMut` borrow that required the transmute no longer existed, so the unsafe block was naturally removed along with it. This is documented in `docs/phase-2-summary.md`:

> **Phase 3 (Remove `unsafe` transmute):** Scope significantly reduced. The `unsafe { transmute }` block was already eliminated as a side-effect of this phase. Phase 3 may only need to verify that no `unsafe` code remains.

Verification confirms:
- `grep -r "unsafe" src/` -- no hits
- `grep -r "transmute" src/` -- no hits

---

## Goals

1. **Verify absence of `unsafe` code.** Confirm that no `unsafe` blocks or `transmute` calls remain anywhere in the codebase (`src/lib.rs`, `src/parse.rs`, `src/main.rs`).
2. **Remove the associated hint comment.** The hint `// подсказка: unsafe избыточен, да и весь rc - тоже` at the original location should be removed if it still exists (it was already removed during Phase 2).
3. **Validate correctness.** Ensure all tests pass and CLI output is unchanged after confirming the removal.
4. **Update project tracking.** Mark Phase 3 as complete in `docs/tasklist.md`.

---

## User Stories

1. **As a maintainer**, I want to confirm that no `unsafe` code remains in `src/lib.rs` so that the codebase is fully safe Rust and does not require auditing for undefined behavior.
2. **As a developer working on later phases**, I want Phase 3 formally closed so that the project tracking accurately reflects the current state of the refactoring effort.

---

## Scenarios

### Scenario 1: Verify no `unsafe` code remains

**Action:** Run `grep -r "unsafe\|transmute" src/` across all source files.

**Expected result:** Zero matches. No `unsafe` blocks, no `transmute` calls.

### Scenario 2: All tests pass

**Action:** Run `cargo test`.

**Expected result:** All existing tests pass. No test cases deleted or modified (the code is unchanged from the end of Phase 2).

### Scenario 3: CLI output unchanged

**Action:** Run `cargo run -- example.log`.

**Expected result:** Output identical to pre-Phase-2 baseline.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| `unsafe` blocks in `src/` | Zero |
| `transmute` calls in `src/` | Zero |
| Hint comments referencing `unsafe` | Zero remaining |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** Same input must produce same output.
3. **No test deletions.** Existing tests are preserved.
4. **Scope boundary.** This phase addresses only `unsafe` / `transmute` removal. Other technical debt (`Box<dyn MyReader>`, `u8` mode constants, `panic!`, etc.) is out of scope.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Work already completed in Phase 2 -- Phase 3 is effectively a no-op | Certain | Low | Phase 3 becomes a verification-only phase. Confirm absence of `unsafe`, run tests, update tracking, and close the ticket. No code changes needed. |
| Hint comment `// подсказка: unsafe избыточен, да и весь rc - тоже` still present | Low | Low | Grep for the hint. If found, remove it as part of this phase. If already removed (confirmed during Phase 2), no action needed. |

---

## Open Questions

1. **Should Phase 3 be marked as already complete?** The `unsafe { transmute }` was removed as a side-effect of Phase 2. Phase 3 may only require formal verification (grep + test run) and updating `docs/tasklist.md`. The team should decide whether this warrants a separate commit/phase or can simply be marked done with a verification note.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | No code changes expected (unsafe already removed in Phase 2). Verify only. |
| `docs/tasklist.md` | Mark Phase 3 as complete (:green_circle:). |

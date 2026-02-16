# Tasklist: Phase 3 -- Remove `unsafe` transmute

**Ticket:** Phase 3: Remove `unsafe` transmute
**PRD:** `docs/prd/phase-3.prd.md`
**Plan:** `docs/plan/phase-3.md`
**Status:** IMPLEMENT_STEP_OK

---

## Context

Phase 3 is a verification-only phase. The `unsafe { transmute(...) }` block in `LogIterator::new()` was already removed as a side-effect of the Phase 2 refactoring, which eliminated the `Rc<RefCell>` wrapper and the `RefMut` borrow that required the transmute. No source code changes are required. This phase formally verifies the absence of `unsafe` code, confirms hint comment removal, runs acceptance criteria, and updates project tracking.

---

## Tasks

- [x] **3.1 Verify absence of `unsafe` blocks in source code**
  Run `grep -r "unsafe" src/` across all source files (`src/lib.rs`, `src/parse.rs`, `src/main.rs`). Confirm zero matches. The `unsafe` block that previously existed in `LogIterator::new()` was removed during Phase 2.
  - **AC1:** `grep -r "unsafe" src/` returns zero matches.
  - **AC2:** No `unsafe` blocks exist anywhere in `src/lib.rs`, `src/parse.rs`, or `src/main.rs`.

- [x] **3.2 Verify absence of `transmute` calls in source code**
  Run `grep -r "transmute" src/` across all source files. Confirm zero matches. The `transmute` call that extended a `RefMut` lifetime to `'static` was removed during Phase 2.
  - **AC1:** `grep -r "transmute" src/` returns zero matches.
  - **AC2:** No `std::mem::transmute` or `transmute` calls exist anywhere in `src/`.

- [x] **3.3 Verify hint comment about `unsafe` is removed**
  Confirm the hint comment `// подсказка: unsafe избыточен, да и весь rc - тоже` no longer exists in any source file under `src/`. This comment was removed during Phase 2 along with the code it referenced.
  - **AC1:** `grep -r "unsafe избыточен" src/` returns zero matches.
  - **AC2:** No hint comments referencing `unsafe` or `transmute` remain in source files.

- [x] **3.4 Run acceptance criteria: tests and CLI**
  Run `cargo test` and `cargo run -- example.log`. All existing tests must pass without deletions or modifications. CLI output must be identical to pre-Phase-2 baseline.
  - **AC1:** `cargo test` passes with all existing tests (no test cases deleted or modified).
  - **AC2:** `cargo run -- example.log` produces output identical to pre-refactoring baseline.

- [x] **3.5 Update `docs/tasklist.md` to mark Phase 3 complete**
  Apply three updates to `docs/tasklist.md`: (1) change Phase 3 status from `:white_circle:` to `:green_circle:` in the progress table, (2) check off task 3.1 by changing `- [ ]` to `- [x]`, (3) advance "Current Phase" from `3` to `4`.
  - **AC1:** Phase 3 row in the progress table shows `:green_circle:`.
  - **AC2:** Task 3.1 in the Phase 3 section shows `- [x]`.
  - **AC3:** "Current Phase" reads `4`.

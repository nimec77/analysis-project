# Tasklist: PH-9 -- Loops to Iterators

**Status:** IMPLEMENT_STEP_OK
**Ticket:** PH-9
**PRD:** `docs/prd/PH-9.prd.md`
**Plan:** `docs/plan/PH-9.md`

---

## Context

Phase 9 replaces the two manual `for` loops in the `read_log()` function body (`src/lib.rs`) with idiomatic Rust iterator chains. The inner loop (manual request ID search with a mutable flag and `break`) becomes `request_ids.contains()`. The outer loop (manual collect-with-push) becomes `.collect::<Result<Vec<_>, _>>()?` followed by `.into_iter().filter(...).collect()`. The hint comment identifying this technical debt is removed. The `read_log()` function signature and all observable behavior are unchanged.

No deviations identified in the plan -- the codebase matches the PRD's "before" state exactly.

---

## Tasks

- [x] **9.1 Replace the entire `read_log()` body with an iterator chain**

  In `src/lib.rs`, replace lines 65-100 of the `read_log()` function body. This single task addresses all three components at once (inner loop, outer loop, and hint comment removal) because they are tightly coupled -- the outer loop replacement naturally subsumes the inner loop replacement.

  Specifically:
  - Replace `let logs = LogIterator::new(input);` / `let mut collected = Vec::new();` / `for log_result in logs { ... }` / `Ok(collected)` with `let collected = LogIterator::new(input).collect::<Result<Vec<_>, _>>()?;` followed by `Ok(collected.into_iter().filter(|log| { ... }).collect())`.
  - Replace the inner `for` loop (mutable `request_id_found` flag, `for request_id in &request_ids`, `break`) with `request_ids.contains(&log.request_id)`.
  - Remove the hint comment `// подсказка: можно обойтись итераторами`.
  - The `match &mode` filtering logic (with `ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges` arms and their nested `matches!()` macros) moves into the `.filter()` closure unchanged.

  **Acceptance criteria:**
  1. Zero `for` loops exist in the `read_log()` function body.
  2. Zero occurrences of `let mut collected` in `read_log()`.
  3. Zero occurrences of `let mut request_id_found` in `read_log()`.
  4. Zero occurrences of `подсказка: можно обойтись итераторами` in `src/lib.rs`.
  5. `request_ids.contains(&log.request_id)` is present in the filter closure.
  6. `.filter(` and `.collect()` are present in the iterator chain.
  7. The `read_log()` function signature is unchanged: `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>`.

- [x] **9.2 Verify**

  Run acceptance checks to confirm the refactoring is complete and correct:
  - `cargo test` passes all 25 tests (no test cases deleted or modified).
  - `cargo run -- example.log` produces output identical to pre-refactoring.
  - Verify all code-level metrics from task 9.1 acceptance criteria.
  - Confirm no files other than `src/lib.rs` were modified (`LogIterator`, `parse.rs`, `main.rs`, and `Cargo.toml` are untouched).
  - Confirm zero external dependencies were added.

  **Acceptance criteria:**
  1. `cargo test` passes all 25 tests. No test cases deleted or modified.
  2. `cargo run -- example.log` succeeds and output is identical to pre-refactoring.
  3. All PRD metrics are met: zero manual `for` loops in `read_log()`, mutable `collected` removed, mutable `request_id_found` removed, hint comment removed, `contains()` used, `.filter().collect()` used, function signature unchanged, 25 tests unchanged, zero external dependencies.

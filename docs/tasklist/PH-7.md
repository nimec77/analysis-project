# Tasklist: PH-7 -- `Result` instead of `panic!`

**Status:** IMPLEMENT_STEP_OK
**Ticket:** PH-7
**PRD:** `docs/prd/PH-7.prd.md`
**Plan:** `docs/plan/PH-7.md`

---

## Context

Phase 7 converts `read_log()` from returning `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>`, and changes `LogIterator`'s `Item` type to `Result<LogLine, std::io::Error>` so that I/O errors from `BufReader::lines()` are propagated to callers instead of being silently swallowed by `.ok()?`. Parse errors (unparseable lines) continue to be silently skipped. Task 7.1 (`panic!` elimination) was already completed by Phase 6's exhaustive `match` and requires no further work.

No deviations identified in the plan -- the codebase matches the PRD's "before" state exactly.

---

## Tasks

- [x] **7.1 Confirm `panic!` elimination (no work needed)**

  Phase 6 already replaced the `if`/`else if`/`else` chain (including `panic!("unknown mode {:?}", mode)`) with an exhaustive `match` on `ReadMode`. This task is formally acknowledged as complete with no code changes required.

  **Acceptance criteria:**
  1. Zero occurrences of `panic!` in `src/lib.rs`.
  2. Zero occurrences of `unreachable!` in `src/lib.rs`.
  3. The `match` expression in `read_log()` has three explicit arms (`ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges`) and no wildcard arm.

- [x] **7.2 Change `LogIterator::Item` to `Result<LogLine, std::io::Error>`**

  In `src/lib.rs`, replace the `Iterator` implementation for `LogIterator<R>` (lines 40-47):
  1. Change `type Item` from `parse::LogLine` to `Result<parse::LogLine, std::io::Error>`.
  2. Replace the chained-`.ok()?` body of `next()` with a `loop` that:
     - Returns `None` when the underlying iterator is exhausted (`self.lines.next()` returns `None`).
     - Returns `Some(Err(e))` for I/O errors from the `BufReader`.
     - Uses `continue` to skip parse errors and non-empty-remainder lines.
     - Returns `Some(Ok(result))` for successfully parsed lines.

  **Acceptance criteria:**
  1. `type Item = Result<parse::LogLine, std::io::Error>` is declared in the `Iterator` impl for `LogIterator<R>`.
  2. Zero occurrences of `.ok()?` in `LogIterator::next()` -- I/O errors are no longer silently swallowed.
  3. The `next()` method uses a `loop` with explicit `match` on the I/O result and `continue` on parse failure.

- [x] **7.3a Change `read_log()` return type and loop body**

  In `src/lib.rs`, update the `read_log()` function (line 50 onward):
  1. Change the return type from `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>`.
  2. Rename the loop variable from `log` to `log_result`.
  3. Add `let log = log_result?;` as the first line inside the loop body to propagate I/O errors.
  4. Change the final `collected` to `Ok(collected)`.
  5. Leave the hint comment `// подсказка: можно обойтись итераторами` untouched.
  6. Leave the request_id filter and mode `match` expression unchanged.

  **Acceptance criteria:**
  1. `read_log()` signature is `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>`.
  2. I/O errors are propagated via `?` in the loop body.
  3. The function returns `Ok(collected)` on success.
  4. Exactly one occurrence of `подсказка: можно обойтись итераторами` in `src/lib.rs` (Phase 9 hint preserved).

- [x] **7.3b Adapt test functions to handle `Result`**

  In `src/lib.rs`, add `.unwrap()` to all six `read_log()` calls across the three test functions:
  - `test_all`: 2 calls (lines 160, 161).
  - `test_errors_mode`: 2 calls (lines 174, 180).
  - `test_exchanges_mode`: 2 calls (lines 197, 203).

  No test functions are deleted. No test function signatures are changed.

  **Acceptance criteria:**
  1. All six `read_log()` calls in tests are followed by `.unwrap()`.
  2. No test functions are deleted or have their signatures changed.
  3. `cargo test` passes all tests.

- [x] **7.3c Adapt `main.rs` call site**

  In `src/main.rs` (line 67), add `.unwrap()` to the `read_log()` call, consistent with the existing `.unwrap()` pattern used elsewhere in `main.rs`.

  **Acceptance criteria:**
  1. The `read_log()` call in `src/main.rs` is followed by `.unwrap()`.
  2. `cargo build` compiles without errors.

- [x] **7.4 Final verification**

  Run acceptance checks to confirm the refactoring is complete and correct:
  - `cargo test` passes all tests (no test deletions).
  - `cargo run -- example.log` produces output identical to pre-refactoring.
  - `read_log()` return type includes `Result<Vec<LogLine>`.
  - `LogIterator::Item` is `Result`.
  - No `.ok()?` remaining in `LogIterator::next()`.
  - Phase 9 hint comment preserved.
  - Zero `panic!` in `src/lib.rs`.
  - `main.rs` handles `Result` with `.unwrap()`.

  **Acceptance criteria:**
  1. `cargo test` passes all tests. `cargo run -- example.log` output is identical to pre-refactoring (success path unchanged).
  2. All PRD metrics are met: `read_log()` returns `Result`, `LogIterator::Item` is `Result`, I/O errors propagated, parse errors still skipped, zero `panic!` in library code, all call sites adapted.

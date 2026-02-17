# PRD: Phase 7 — `Result` instead of `panic!`

**Status:** PRD_READY
**Ticket:** PH-7 "Phase 7: Result instead of panic!"
**Phase:** 7 of 12 (see `docs/tasklist.md`)
**Dependencies:** Phase 5 (complete), Phase 6 (complete)
**Blocked by:** Nothing (Phase 5 and Phase 6 are both done)
**Blocks:** None directly (downstream phases are independent)

---

## Context / Idea

Phase 7 targets the conversion of `read_log()` from returning `Vec<LogLine>` to returning `Result<Vec<LogLine>, E>`, eliminating any remaining `panic!` sites in the library crate and ensuring that fallible operations are surfaced to callers through the type system rather than through runtime panics.

The original author left a hint: `// подсказка: паниковать в библиотечном коде - нехорошо` ("hint: panicking in library code is bad practice"), indicating this was recognized technical debt from the start.

### Authoritative specification from `docs/phase/phase-7.md`

**Goal:** Replace `panic!` on unknown mode with exhaustive `match` (no default arm needed after Phase 5), return `Result` from `read_log()` for any remaining fallible operations, and adapt tests accordingly.

**Tasks:**

- [ ] 7.1 Replace `panic!` on unknown mode with exhaustive `match` (no default arm needed after Phase 5)
- [ ] 7.2 Return `Result` from `read_log()` for any remaining fallible operations
- [ ] 7.3 Adapt `test_all` in `lib.rs` if `read_log()` now returns `Result`: unwrap or use `?` in test

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** Phase 5 complete, Phase 6 complete (exhaustive `match` already eliminates the `panic!` arm)

**Implementation Notes:**

- **Hint:** `src/lib.rs:114` -- `// подсказка: паниковать в библиотечном коде - нехорошо`
- Note: Phase 6 already removed the `panic!("unknown mode {:?}", mode)` by replacing the `if`/`else if` chain with an exhaustive `match`. This phase should focus on any remaining `panic!` sites and on converting `read_log()` to return `Result` for broader error handling.

### Current codebase state (gap analysis)

After Phase 6, the codebase has the following relevant state:

- **No `panic!` calls remain in `src/lib.rs`.** Phase 6 replaced the `if`/`else if` chain with an exhaustive `match` on `ReadMode`, which naturally eliminated the `else { panic!("unknown mode {:?}", mode) }` arm. Task 7.1 from the phase description is already satisfied by Phase 6's work.

- **`read_log()` signature** (line 50): Currently returns `Vec<LogLine>` directly:
  ```rust
  pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine> {
  ```
  This does not communicate any possibility of failure to callers.

- **`LogIterator::next()` silently swallows errors** (lines 42-46):
  ```rust
  fn next(&mut self) -> Option<Self::Item> {
      let line = self.lines.next()?.ok()?;
      let (remaining, result) = LOG_LINE_PARSER.parse(line.trim()).ok()?;
      remaining.trim().is_empty().then_some(result)
  }
  ```
  I/O errors from `self.lines.next()` are converted to `None` via `.ok()?`, silently dropping the error. Parse failures are similarly discarded. While this "skip unparseable lines" behavior may be intentional for parse errors, silently swallowing I/O errors (e.g., disk failures, permission errors mid-read) is a defect for a library function.

- **`main.rs` call site** (line 67): Uses `read_log()` without any error handling:
  ```rust
  let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
  ```
  This will need to handle the `Result` (e.g., `.unwrap()`, `?`, or proper error handling).

- **Test call sites** (lines 160, 161, 174, 180, 197): All tests call `read_log()` and use the returned `Vec<LogLine>` directly. These will need to unwrap or use `?` when `read_log()` returns `Result`.

- **Remaining hint comments in `src/lib.rs`** (line 53): `// подсказка: можно обойтись итераторами` -- this is Phase 9's scope, not Phase 7's.

- **No remaining `panic!` or `unreachable!` calls in `src/lib.rs`.**

---

## Goals

1. **Return `Result` from `read_log()`.** Change the return type of `read_log()` from `Vec<LogLine>` to `Result<Vec<LogLine>, E>` where `E` is an appropriate error type, so that I/O errors encountered during log reading are propagated to callers rather than silently discarded.

2. **Propagate I/O errors from `LogIterator`.** The current `LogIterator::next()` uses `.ok()?` to silently swallow `std::io::Error` values from the underlying `BufReader`. After this phase, I/O errors should be surfaced through the `Result` return type of `read_log()`.

3. **Maintain the "skip unparseable lines" behavior for parse errors.** Lines that fail to parse (returning `Err(())` from the parser) should continue to be silently skipped, as this is intentional behavior -- the log stream may contain lines that do not match the expected format.

4. **Adapt all call sites.** Update `main.rs` and all test functions to handle the `Result` return type from `read_log()`.

5. **Confirm task 7.1 is already satisfied.** The `panic!("unknown mode {:?}", mode)` was already eliminated by Phase 6's exhaustive `match`. This phase formally acknowledges that task 7.1 requires no further work.

6. **Preserve behavior for the success path.** When no I/O errors occur, the output must be identical to the current implementation. All existing tests must continue to pass.

---

## User Stories

1. **As a library consumer calling `read_log()`**, I want the function to return a `Result` so that I can distinguish between "no matching logs found" (`Ok(vec![])`) and "an I/O error prevented reading" (`Err(...)`), rather than having errors silently swallowed.

2. **As a developer writing error handling code**, I want `read_log()` to use Rust's `Result` type so that I can use the `?` operator to propagate errors upward, following standard Rust error handling conventions.

3. **As a maintainer of `main.rs`**, I want the compiler to force me to handle potential errors from `read_log()`, so that I cannot accidentally ignore I/O failures when reading log files.

4. **As a developer working on future phases**, I want `read_log()` to follow the Rust convention of returning `Result` for fallible operations, establishing a clean API surface that later phases (e.g., Phase 9: iterator refactoring) can build upon.

---

## Scenarios

### Scenario 1: `read_log()` return type changes to `Result`

**Before:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine> {
```

**After:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error> {
```

The function returns `Ok(vec![...])` on success and `Err(io::Error)` if an I/O error occurs during reading. The error type is `std::io::Error` because the only fallible operation that should be propagated (rather than skipped) is I/O reading from the `BufReader`.

### Scenario 2: I/O errors are propagated, parse errors are still skipped

**Before (in `LogIterator::next()`):**
```rust
let line = self.lines.next()?.ok()?;
let (remaining, result) = LOG_LINE_PARSER.parse(line.trim()).ok()?;
remaining.trim().is_empty().then_some(result)
```

Both I/O errors and parse errors are converted to `None` and silently skipped.

**After:** I/O errors from `self.lines.next()` are propagated as `Err(std::io::Error)`. Parse failures (from `LOG_LINE_PARSER.parse()`) continue to be skipped (returning `None`), as unparseable lines are expected in the log stream.

One approach: Change `LogIterator`'s `Item` type to `Result<LogLine, std::io::Error>`, yielding `Err(...)` for I/O errors and `Ok(log_line)` for successfully parsed lines, while returning `None` (end of iteration) or skipping on parse failures. Alternatively, the iteration can be performed inside `read_log()` with explicit error handling. The implementer should choose the cleanest approach.

### Scenario 3: Tests adapted to handle `Result`

**Before:**
```rust
#[test]
fn test_all() {
    assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len(), 1);
    let all_parsed = read_log(SOURCE.as_bytes(), ReadMode::All, vec![]);
    // ...
}
```

**After (option A -- `.unwrap()`):**
```rust
#[test]
fn test_all() {
    assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).unwrap().len(), 1);
    let all_parsed = read_log(SOURCE.as_bytes(), ReadMode::All, vec![]).unwrap();
    // ...
}
```

**After (option B -- `?` with `Result` return):**
```rust
#[test]
fn test_all() -> Result<(), std::io::Error> {
    assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![])?.len(), 1);
    let all_parsed = read_log(SOURCE.as_bytes(), ReadMode::All, vec![])?;
    // ...
    Ok(())
}
```

Both approaches are acceptable. Using `.unwrap()` is simpler and more conventional in tests; using `?` with a `Result`-returning test function is also idiomatic Rust.

### Scenario 4: `main.rs` adapted to handle `Result`

**Before:**
```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
```

**After:**
```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]).unwrap();
```

Or with proper error handling:
```rust
let logs = match analysis::read_log(file, analysis::ReadMode::All, vec![]) {
    Ok(logs) => logs,
    Err(e) => {
        eprintln!("Error reading logs: {}", e);
        std::process::exit(1);
    }
};
```

Since `main.rs` is the binary crate (not library code), using `.unwrap()` or `.expect()` is acceptable here.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring (success path) |
| `panic!` occurrences in `src/lib.rs` | Zero (already zero after Phase 6; confirmed) |
| `read_log()` return type | `Result<Vec<LogLine>, std::io::Error>` (or equivalent) |
| I/O errors from `BufReader` | Propagated via `Result`, not silently swallowed |
| Parse errors (unparseable lines) | Still silently skipped (intentional behavior preserved) |
| Tests using `read_log()` | All adapted to handle `Result` (via `.unwrap()` or `?`) |
| `main.rs` call site | Adapted to handle `Result` |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`. Use `std::io::Error` as the error type rather than introducing `thiserror`, `anyhow`, or similar.
2. **No behavior changes on the success path.** When no I/O errors occur, the output must be byte-for-byte identical to the current implementation.
3. **No test deletions.** Existing tests are not deleted. They are adapted to handle the `Result` return type.
4. **Parse errors remain silent.** Lines that fail to parse are still skipped. Only I/O errors (from the underlying reader) are propagated.
5. **Scope boundary.** This phase addresses only the `read_log()` return type and I/O error propagation. Refactoring loops to iterators is Phase 9's scope. The hint `// подсказка: можно обойтись итераторами` is not addressed here.
6. **Error type must be `std::io::Error` (or a simple wrapper).** Since the only propagated errors are I/O errors from `BufReader::lines()`, `std::io::Error` is the natural and sufficient error type. A custom error enum is acceptable but not required.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Changing `LogIterator::Item` to `Result` complicates the iterator logic | Medium | Low | The implementer can choose between modifying `LogIterator`'s `Item` type or handling errors in the `read_log()` loop directly. Both approaches are valid; the simpler one should be preferred. |
| Tests using `as_bytes()` on string slices never produce I/O errors, so the `Result` path is not exercised in tests | Medium | Low | The primary goal is API correctness (returning `Result` instead of silently swallowing errors). The in-memory `&[u8]` reader used in tests will always succeed, which confirms the success path works. A dedicated test with a failing reader could be added but is not required by the phase specification. |
| `main.rs` changes could mask errors if `.unwrap()` is used carelessly | Low | Low | `.unwrap()` or `.expect("failed to read logs")` in `main.rs` is acceptable since this is the binary crate. The key improvement is that the *library* function now communicates fallibility through its type signature. |
| Changing a public function signature is a breaking API change | Low | Medium | `read_log()` is the library crate's primary public API. Changing its return type from `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>` is intentionally breaking -- callers must now handle the `Result`. This is the explicit goal of the phase. The only known call site outside tests is `main.rs`, which is in the same crate. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. Task 7.1 (replace `panic!` with exhaustive `match`) is already satisfied by Phase 6. The remaining work is task 7.2 (return `Result` from `read_log()`) and task 7.3 (adapt tests and call sites). Both Phase 5 and Phase 6 (the dependencies) are already done.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | Change `read_log()` return type from `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>`. Update `LogIterator` or the `read_log()` loop to propagate I/O errors instead of silently swallowing them via `.ok()?`. Wrap the final collected vector in `Ok(...)`. Adapt all test functions (`test_all`, `test_errors_mode`, `test_exchanges_mode`) to handle the `Result` return (`.unwrap()` or `?`). |
| `src/main.rs` | Update the `read_log()` call site to handle the `Result` (e.g., `.unwrap()`, `.expect(...)`, or `match`). |

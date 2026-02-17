# Research: PH-7 -- `Result` instead of `panic!`

**Ticket:** PH-7 "Phase 7: Result instead of panic!"
**PRD:** `docs/prd/PH-7.prd.md`
**Phase spec:** `docs/phase/phase-7.md`

---

## 1. Resolved Questions

The PRD has no open questions. The user confirmed proceeding with default requirements only -- no additional constraints or preferences.

---

## 2. Related Modules/Services

### 2.1 `src/lib.rs` -- Primary Target

This is the sole file containing the library crate's public API (`read_log()`) and the internal `LogIterator<R>` implementation. All changes for this phase are concentrated here.

**Current `read_log()` signature (line 50):**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine> {
```

**Current `LogIterator` implementation (lines 17-46):**
```rust
struct LogIterator<R: Read> {
    #[allow(clippy::type_complexity)]
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<R>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
impl<R: Read> LogIterator<R> {
    fn new(reader: R) -> Self {
        use std::io::BufRead;
        Self {
            lines: std::io::BufReader::with_capacity(4096, reader)
                .lines()
                .filter(|line_res| {
                    !line_res
                        .as_ref()
                        .ok()
                        .map(|line| line.trim().is_empty())
                        .unwrap_or(false)
                }),
        }
    }
}
impl<R: Read> Iterator for LogIterator<R> {
    type Item = parse::LogLine;
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?.ok()?;
        let (remaining, result) = LOG_LINE_PARSER.parse(line.trim()).ok()?;
        remaining.trim().is_empty().then_some(result)
    }
}
```

**Key observation -- two `.ok()?` calls in `next()` (line 43-44):**

1. `self.lines.next()?.ok()?` -- The `.ok()?` on line 43 converts `Result<String, std::io::Error>` to `Option<String>`, silently discarding I/O errors (e.g., disk failures, permission errors mid-read). This is the defect that Phase 7 must address.

2. `LOG_LINE_PARSER.parse(line.trim()).ok()?` -- The `.ok()?` on line 44 converts `Result<(&str, LogLine), ()>` to `Option<(&str, LogLine)>`, silently skipping unparseable lines. This is **intentional behavior** that must be preserved.

### 2.2 `src/main.rs` -- Call Site (line 67)

```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
```

This is the only call site outside of tests. It must be updated to handle the `Result` return type. Since this is binary crate code, `.unwrap()` or `.expect(...)` is acceptable.

### 2.3 `src/lib.rs` -- Test Call Sites (lines 160-221)

Five `read_log()` calls across three test functions:

| Test Function | Line(s) | Call |
|---|---|---|
| `test_all` | 160 | `read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len()` |
| `test_all` | 161 | `read_log(SOURCE.as_bytes(), ReadMode::All, vec![])` |
| `test_errors_mode` | 174 | `read_log(SOURCE1.as_bytes(), ReadMode::Errors, vec![1])` |
| `test_errors_mode` | 180 | `read_log(SOURCE.as_bytes(), ReadMode::Errors, all_ids)` |
| `test_exchanges_mode` | 197 | `read_log(SOURCE1.as_bytes(), ReadMode::Exchanges, vec![1])` |
| `test_exchanges_mode` | 203 | `read_log(SOURCE.as_bytes(), ReadMode::Exchanges, all_ids)` |

All six calls use `as_bytes()` on string slices, which produces `&[u8]` readers that never produce I/O errors. These will need `.unwrap()` or `?` added.

### 2.4 `src/parse.rs` -- Not Modified

The parser module (`LogLineParser`, `LOG_LINE_PARSER`, `LogLine`, etc.) is not affected by this phase. The parser's `parse()` method returns `Result<(&str, LogLine), ()>`, and parse failures continue to be silently skipped.

### 2.5 `Cargo.toml` -- Not Modified

The project has zero external dependencies (edition 2024, no crates). This phase uses only `std::io::Error`, requiring no new dependencies.

---

## 3. Current Endpoints and Contracts

### 3.1 Public API Surface

The library crate exposes:

| Item | Current Signature |
|---|---|
| `read_log()` | `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>` |
| `ReadMode` enum | `pub enum ReadMode { All, Errors, Exchanges }` |
| `parse` module | Various parser types and data model types (unchanged) |

After Phase 7, only `read_log()` changes:

| Item | New Signature |
|---|---|
| `read_log()` | `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>` |

This is an intentionally breaking API change. The only known external call site is `src/main.rs:67`.

### 3.2 `LogIterator` -- Internal Contract

`LogIterator` is `pub(crate)` (actually just `struct`, not `pub`), so it is internal to the library. Its `Item` type can be changed without any public API impact.

**Current:** `type Item = parse::LogLine;`

**After (if approach A is chosen):** `type Item = Result<parse::LogLine, std::io::Error>;`

---

## 4. Patterns Used

### 4.1 Error Handling Patterns in the Codebase

| Pattern | Location | Notes |
|---|---|---|
| `.ok()?` to silently skip errors | `src/lib.rs:43-44` | Used for both I/O errors and parse errors. Phase 7 must differentiate between the two. |
| `.unwrap()` in binary crate | `src/main.rs:56, 64, 66` | `main.rs` already uses `.unwrap()` for `just_parse_anouncements()`, `current_dir()`, and `File::open()`. Adding `.unwrap()` for `read_log()` is consistent. |
| `Result<(remaining, T), ()>` for parse results | `src/parse.rs:1430` | Parser uses unit error type `()`. This is the error that should remain silently skipped. |
| `std::io::Lines` yielding `Result<String, std::io::Error>` | `src/lib.rs:20` | The underlying iterator already produces `Result` values. The I/O error is currently discarded by `.ok()?`. |
| `filter()` on `Lines` to skip blank lines | `src/lib.rs:30-36` | The blank-line filter currently passes through I/O errors (non-blank lines return `true` for errors because the `.ok()?.map(...)?.unwrap_or(false)` chain returns `false` only for empty-line `Ok` results, meaning `!false = true` -- errors pass through the filter). |

### 4.2 Blank-Line Filter Behavior with Errors

The filter closure in `LogIterator::new()` (lines 30-36) has subtle behavior with `Err` values:

```rust
.filter(|line_res| {
    !line_res
        .as_ref()
        .ok()                           // Err -> None
        .map(|line| line.trim().is_empty()) // None stays None
        .unwrap_or(false)               // None -> false
})
// For Err: !false = true -> Err values PASS THROUGH the filter
```

This means I/O errors are not filtered out -- they pass through to `next()`, where `.ok()?` currently discards them. After Phase 7, these errors will reach the caller. This is correct behavior: the filter is for blank lines, not for error suppression.

---

## 5. Implementation Approaches

The PRD identifies two valid approaches for propagating I/O errors. Both are analyzed here.

### 5.1 Approach A: Change `LogIterator::Item` to `Result<LogLine, std::io::Error>`

**Change `Iterator` impl:**
```rust
impl<R: Read> Iterator for LogIterator<R> {
    type Item = Result<parse::LogLine, std::io::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line_result = self.lines.next()?;  // None = end of stream
            let line = match line_result {
                Ok(line) => line,
                Err(e) => return Some(Err(e)),      // I/O error -> propagate
            };
            let Ok((remaining, result)) = LOG_LINE_PARSER.parse(line.trim()) else {
                continue;  // parse error -> skip, try next line
            };
            if remaining.trim().is_empty() {
                return Some(Ok(result));
            }
            // remaining not empty -> skip, try next line
        }
    }
}
```

**Change `read_log()` to collect and propagate:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error> {
    let logs = LogIterator::new(input);
    let mut collected = Vec::new();
    for log_result in logs {
        let log = log_result?;  // propagate I/O errors with ?
        // ... existing filtering logic unchanged ...
        if /* filter passes */ {
            collected.push(log);
        }
    }
    Ok(collected)
}
```

**Pros:**
- Clean separation of concerns: `LogIterator` handles I/O and parsing; `read_log()` handles filtering.
- `LogIterator` faithfully reports what happened (I/O error vs. parsed line).
- The `loop` + `continue` pattern for skipping parse errors is idiomatic.

**Cons:**
- Requires changing the `Iterator::Item` type.
- Slightly more complex `next()` implementation (loop instead of chained `?`).
- The `LogIterator` now has a different semantic contract: it skips unparseable lines internally (loop/continue) rather than exposing them.

### 5.2 Approach B: Handle Errors Directly in `read_log()` Loop

Keep `LogIterator::Item` as `parse::LogLine` but change `read_log()` to iterate over the underlying `BufReader::lines()` directly, handling I/O errors and parse errors separately.

This would require restructuring `read_log()` significantly, essentially inlining the `LogIterator` logic. This is more invasive and less clean.

**Assessment:** Approach A is preferred -- it keeps the existing separation between iteration (LogIterator) and filtering (read_log), and is the more natural Rust pattern.

### 5.3 Test Adaptation Strategy

Since all tests use `as_bytes()` readers (which never produce I/O errors), the simplest adaptation is adding `.unwrap()` to each `read_log()` call:

```rust
// Before:
assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len(), 1);
// After:
assert_eq!(read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).unwrap().len(), 1);
```

Using `.unwrap()` is simpler and more conventional in test code than converting each test function to return `Result`. Both approaches are acceptable per the PRD.

### 5.4 `main.rs` Adaptation Strategy

The simplest approach is `.unwrap()` or `.expect(...)`, consistent with existing error handling in `main.rs` (lines 56, 64, 66 all use `.unwrap()`).

---

## 6. Confirmation: Task 7.1 Already Satisfied

The PRD states that task 7.1 ("Replace `panic!` on unknown mode with exhaustive `match`") was already completed by Phase 6. Research confirms this:

- **Zero `panic!` calls in `src/lib.rs`.** Grep for `panic!` returns no matches.
- **Zero `unreachable!` calls in `src/lib.rs`.** Grep confirms.
- **The `match` expression on `ReadMode` at lines 64-81 is exhaustive** with three explicit arms (`All`, `Errors`, `Exchanges`) and no wildcard/default arm.
- **The hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` is already gone** -- it was removed along with the `panic!` arm in Phase 6.

Task 7.1 requires no further work.

---

## 7. Remaining Hint Comments in `src/lib.rs`

After Phase 6, only one hint comment remains in `src/lib.rs`:

| Line | Comment | Phase |
|---|---|---|
| 53 | `// подсказка: можно обойтись итераторами` | Phase 9 (out of scope) |

The hint that Phase 7 was originally associated with (`// подсказка: паниковать в библиотечном коде - нехорошо`) was already removed by Phase 6, since it annotated the `panic!` arm that the exhaustive `match` eliminated. Phase 7's remaining work (tasks 7.2 and 7.3) does not have a corresponding hint comment in the current codebase -- its scope is driven by the phase specification and PRD, not by a source-level hint.

---

## 8. Dependencies and Layers

```
main.rs  -->  lib.rs::read_log(impl Read, ReadMode, Vec<u32>) -> Result<Vec<LogLine>, io::Error>
                |
                +--> LogIterator<R>::new(R) --> BufReader<R> --> Lines --> Filter (blank lines)
                |         |
                |         +--> Iterator::next() yields Result<LogLine, io::Error>
                |                  |
                |                  +--> I/O error from BufReader -> Some(Err(e))
                |                  +--> Parse error -> skip (continue loop)
                |                  +--> Successful parse -> Some(Ok(log_line))
                |                  +--> End of stream -> None
                |
                +--> for log_result in logs {
                |        let log = log_result?;    // propagate I/O errors
                |        if (request_id_filter) && match &mode {
                |            ReadMode::All => true,
                |            ReadMode::Errors => matches!(...),
                |            ReadMode::Exchanges => matches!(...),
                |        } { collected.push(log) }
                |    }
                |    Ok(collected)
```

---

## 9. Limitations and Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Changing `LogIterator::Item` to `Result` requires a `loop` + `continue` pattern in `next()` to skip parse errors | Certain (structural requirement) | Low -- the implementation is straightforward | The `loop` with explicit `match` on I/O result and `continue` on parse failure is a well-known Rust iterator pattern. |
| All tests use `&[u8]` readers that never produce I/O errors, so the error path is not exercised in tests | Medium | Low -- the primary goal is API correctness | The PRD explicitly notes this and states a dedicated failing-reader test is not required. The success path is fully tested by existing tests. |
| Changing `read_log()` return type is a breaking API change | Certain (by design) | Low -- the only external call site is `main.rs` in the same crate | This is the explicit goal of the phase. |
| The blank-line filter already passes through `Err` values (as analyzed in Section 4.2) | N/A -- existing behavior | None -- this is correct behavior | The filter correctly allows errors to pass through; only the subsequent `.ok()?` was incorrectly discarding them. |
| Phase 9 (loops -> iterators) will need to account for `Result` in the `for` loop | Low | Low | Phase 9 can use `.collect::<Result<Vec<_>, _>>()` or similar iterator patterns. The `Result` return type established by Phase 7 is a clean foundation. |

---

## 10. Deviations from Requirements

**None.** The current codebase state matches the PRD's gap analysis exactly:

- `read_log()` returns `Vec<LogLine>` (not `Result`) -- confirmed at line 50.
- `LogIterator::next()` uses `.ok()?` to silently swallow I/O errors -- confirmed at line 43.
- No `panic!` calls remain in `src/lib.rs` -- confirmed (task 7.1 already satisfied by Phase 6).
- `main.rs` uses `read_log()` without error handling -- confirmed at line 67.
- Tests call `read_log()` and use the returned `Vec<LogLine>` directly -- confirmed at lines 160, 161, 174, 180, 197, 203.
- The remaining hint comment `// подсказка: можно обойтись итераторами` at line 53 is Phase 9 scope -- confirmed.

All 18 existing tests pass (`cargo test`).

---

## 11. Scope Boundaries

| Concern | Phase 6 (done) | Phase 7 (this phase) | Phase 9 (future) |
|---|---|---|---|
| `panic!` -> exhaustive `match` | Resolved | Confirmed (task 7.1 -- no work needed) | -- |
| `read_log()` returns `Result` | -- | **In scope (task 7.2)** | -- |
| I/O errors propagated from `LogIterator` | -- | **In scope (task 7.2)** | -- |
| Parse errors silently skipped | Existing behavior | **Preserved (requirement)** | -- |
| Tests adapted for `Result` return | -- | **In scope (task 7.3)** | -- |
| `main.rs` adapted for `Result` return | -- | **In scope (task 7.3)** | -- |
| Loops -> iterators | -- | -- | In scope |
| Hint `можно обойтись итераторами` | -- | Preserved (out of scope) | In scope |

---

## 12. New Technical Questions Discovered During Research

### 12.1 `LogIterator::next()` Must Handle Skip-and-Retry for Parse Errors

The current `next()` implementation uses chained `?` operators, which means a parse failure on one line causes `next()` to return `None` (end of iteration) rather than trying the next line. This means:

```rust
fn next(&mut self) -> Option<Self::Item> {
    let line = self.lines.next()?.ok()?;              // line 43
    let (remaining, result) = LOG_LINE_PARSER.parse(line.trim()).ok()?;  // line 44
    remaining.trim().is_empty().then_some(result)      // line 45
}
```

If `LOG_LINE_PARSER.parse()` fails on a line, `.ok()?` returns `None` from `next()`, stopping iteration entirely. The **only reason this works currently** is that all lines in the test data and example log are parseable (unparseable lines are blank lines, which are already filtered out by the `Filter` in `LogIterator::new()`).

However, the PRD states: "Lines that fail to parse should continue to be silently skipped." This implies the iterator should **continue** to the next line after a parse failure rather than stopping. The new implementation with `loop`/`continue` (Approach A in Section 5.1) naturally fixes this latent behavior issue while satisfying the Phase 7 requirements.

**Assessment:** This is not a deviation from requirements -- the PRD explicitly requires skip behavior for parse errors. The `loop`/`continue` pattern in the new `next()` implementation provides this correctly. The current behavior (stopping on first parse failure) is a latent bug that happens to be masked by the fact that all non-blank lines in test data are parseable.

### 12.2 Filter Closure Type Annotation

The `LogIterator` struct stores the filter as `fn(&Result<String, std::io::Error>) -> bool` (a function pointer type). This works because the filter closure in `new()` does not capture any environment. When changing `next()` to propagate I/O errors, the filter remains unchanged -- it filters blank lines only, and errors pass through. No changes to the filter or its type are needed.

---

## 13. Verification Checklist

Per the acceptance criteria:

```bash
cargo test                # All 18 tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring (success path)
```

Additionally verify:

| Metric | Expected |
|---|---|
| `panic!` occurrences in `src/lib.rs` | Zero (already zero; unchanged) |
| `read_log()` return type | `Result<Vec<LogLine>, std::io::Error>` |
| I/O errors from `BufReader` | Propagated via `Result`, not silently swallowed |
| Parse errors (unparseable lines) | Silently skipped (intentional behavior preserved) |
| `LogIterator::Item` type | `Result<parse::LogLine, std::io::Error>` (if Approach A) |
| Tests using `read_log()` | All adapted with `.unwrap()` (or `?`) |
| `main.rs` call site | Adapted with `.unwrap()` (or `.expect(...)`) |
| Hint `можно обойтись итераторами` | Preserved at line 53 |
| External dependencies | Zero (uses only `std::io::Error`) |
| Number of tests | 18 (unchanged) |

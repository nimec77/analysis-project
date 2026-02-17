# Implementation Plan: PH-7 -- `Result` instead of `panic!`

**Status:** PLAN_APPROVED
**Ticket:** PH-7 "Phase 7: Result instead of panic!"
**PRD:** `docs/prd/PH-7.prd.md`
**Research:** `docs/research/PH-7.md`
**Phase spec:** `docs/phase/phase-7.md`

---

## Components

### 1. `LogIterator<R>` -- Iterator `Item` type change

**File:** `src/lib.rs`, lines 40-47

The `Iterator` implementation for `LogIterator<R>` currently declares `type Item = parse::LogLine`. This is changed to `type Item = Result<parse::LogLine, std::io::Error>` so that I/O errors from the underlying `BufReader` are propagated to the caller rather than silently swallowed by `.ok()?`.

**Current:**
```rust
impl<R: Read> Iterator for LogIterator<R> {
    type Item = parse::LogLine;
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?.ok()?;
        let (remaining, result) = LOG_LINE_PARSER.parse(line.trim()).ok()?;
        remaining.trim().is_empty().then_some(result)
    }
}
```

**After:**
```rust
impl<R: Read> Iterator for LogIterator<R> {
    type Item = Result<parse::LogLine, std::io::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line_result = self.lines.next()?;   // None = end of stream
            let line = match line_result {
                Ok(line) => line,
                Err(e) => return Some(Err(e)),       // I/O error -> propagate
            };
            let Ok((remaining, result)) = LOG_LINE_PARSER.parse(line.trim()) else {
                continue;                             // parse error -> skip
            };
            if remaining.trim().is_empty() {
                return Some(Ok(result));
            }
            // remaining not empty -> skip, try next line
        }
    }
}
```

Key details:
1. **`loop` + `continue`**: Parse errors and non-empty-remainder lines are skipped by continuing to the next line. This preserves the intentional "skip unparseable lines" behavior and also fixes a latent bug in the current implementation where a parse failure causes `next()` to return `None` (stopping iteration entirely) rather than trying the next line.
2. **`Some(Err(e))` for I/O errors**: I/O errors from `BufReader::lines()` are yielded as `Some(Err(e))`, allowing the caller to detect and propagate them.
3. **`Some(Ok(result))` for parsed lines**: Successfully parsed lines are yielded wrapped in `Ok`.
4. **`None` for end of stream**: When the underlying iterator is exhausted, `self.lines.next()` returns `None`, which propagates out via `?`.

### 2. `LogIterator<R>` -- Blank-line filter (unchanged)

**File:** `src/lib.rs`, lines 24-38

The blank-line filter in `LogIterator::new()` is not modified. As analyzed in the research (Section 4.2), `Err` values pass through the filter (they are not blank lines), which is correct behavior. The filter only removes `Ok(line)` values where `line.trim().is_empty()`.

### 3. `read_log()` -- Return type change

**File:** `src/lib.rs`, line 50

The return type changes from `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>`.

**Current:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine> {
```

**After:**
```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error> {
```

### 4. `read_log()` -- Loop body with error propagation

**File:** `src/lib.rs`, lines 51-86

The `for log in logs` loop must be adapted because `LogIterator` now yields `Result<LogLine, std::io::Error>` items. The loop body uses `?` to propagate I/O errors and wraps the final result in `Ok(...)`.

**Current:**
```rust
    let logs = LogIterator::new(input);
    let mut collected = Vec::new();
    // подсказка: можно обойтись итераторами
    for log in logs {
        if request_ids.is_empty() || {
            // ... request_id filter ...
        } && match &mode {
            // ... mode filter ...
        } {
            collected.push(log);
        }
    }
    collected
```

**After:**
```rust
    let logs = LogIterator::new(input);
    let mut collected = Vec::new();
    // подсказка: можно обойтись итераторами
    for log_result in logs {
        let log = log_result?;
        if request_ids.is_empty() || {
            // ... request_id filter (unchanged) ...
        } && match &mode {
            // ... mode filter (unchanged) ...
        } {
            collected.push(log);
        }
    }
    Ok(collected)
```

Key details:
1. **`for log_result in logs`**: The loop variable is renamed from `log` to `log_result` to reflect the `Result` type.
2. **`let log = log_result?;`**: I/O errors are propagated with `?`. On `Err`, the function returns `Err(io::Error)` immediately.
3. **`Ok(collected)`**: The successful return wraps the collected vector in `Ok`.
4. **Hint comment preserved**: `// подсказка: можно обойтись итераторами` at line 53 is Phase 9 scope and remains untouched.
5. **All filtering logic unchanged**: The request_id filter and mode `match` expression are identical to the current code.

### 5. `main.rs` -- Call site adaptation

**File:** `src/main.rs`, line 67

The `read_log()` call site must handle the `Result` return type.

**Current:**
```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]);
```

**After:**
```rust
let logs = analysis::read_log(file, analysis::ReadMode::All, vec![]).unwrap();
```

Using `.unwrap()` is consistent with the existing error handling pattern in `main.rs` (lines 56, 64, 66 all use `.unwrap()`). Since this is binary crate code, `.unwrap()` is acceptable.

### 6. Test functions -- Adapted to handle `Result`

**File:** `src/lib.rs`, test module (lines 158-222)

Six `read_log()` calls across three test functions need `.unwrap()` added:

| Test Function | Line | Current | After |
|---|---|---|---|
| `test_all` | 160 | `read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).len()` | `read_log(SOURCE1.as_bytes(), ReadMode::All, vec![]).unwrap().len()` |
| `test_all` | 161 | `read_log(SOURCE.as_bytes(), ReadMode::All, vec![])` | `read_log(SOURCE.as_bytes(), ReadMode::All, vec![]).unwrap()` |
| `test_errors_mode` | 174 | `read_log(SOURCE1.as_bytes(), ReadMode::Errors, vec![1])` | `read_log(SOURCE1.as_bytes(), ReadMode::Errors, vec![1]).unwrap()` |
| `test_errors_mode` | 180 | `read_log(SOURCE.as_bytes(), ReadMode::Errors, all_ids)` | `read_log(SOURCE.as_bytes(), ReadMode::Errors, all_ids).unwrap()` |
| `test_exchanges_mode` | 197 | `read_log(SOURCE1.as_bytes(), ReadMode::Exchanges, vec![1])` | `read_log(SOURCE1.as_bytes(), ReadMode::Exchanges, vec![1]).unwrap()` |
| `test_exchanges_mode` | 203 | `read_log(SOURCE.as_bytes(), ReadMode::Exchanges, all_ids)` | `read_log(SOURCE.as_bytes(), ReadMode::Exchanges, all_ids).unwrap()` |

Using `.unwrap()` is preferred over converting test functions to return `Result` because it is simpler, more conventional in test code, and avoids changing test function signatures unnecessarily.

### 7. Confirmed: Task 7.1 already satisfied (no work needed)

Phase 6 replaced the `if`/`else if` chain with an exhaustive `match` on `ReadMode`, eliminating the `panic!("unknown mode {:?}", mode)` arm. The current codebase has:
- Zero `panic!` calls in `src/lib.rs`.
- Zero `unreachable!` calls in `src/lib.rs`.
- An exhaustive `match` at lines 64-81 with three explicit arms and no wildcard.
- The hint comment `// подсказка: паниковать в библиотечном коде - нехорошо` already removed.

Task 7.1 requires no further work.

---

## API Contract

### Before (current)

```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>
```

### After

```rust
pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>
```

This is an intentionally breaking API change. The function now communicates fallibility through the type system:
- `Ok(vec![...])` -- successful read with zero or more matching log entries.
- `Err(io::Error)` -- an I/O error occurred during reading (e.g., disk failure, permission error mid-read).

The error type is `std::io::Error` because the only propagated errors originate from `BufReader::lines()`, which yields `Result<String, std::io::Error>`. No custom error type is needed.

---

## Data Flows

```
Caller (main.rs or test)
  |
  |  passes ReadMode enum variant + request IDs
  v
read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>)
    -> Result<Vec<LogLine>, std::io::Error>
  |
  |  ownership transfer: input moved into LogIterator::new()
  v
LogIterator<R>::new(reader: R) -> BufReader<R> -> Lines -> Filter (blank lines)
  |
  |  Iterator yields Result<LogLine, io::Error>:
  |    - I/O error from BufReader   -> Some(Err(e))    -> propagated by ? in read_log
  |    - Parse error                -> skip (continue)  -> not visible to caller
  |    - Non-empty remaining        -> skip (continue)  -> not visible to caller
  |    - Successful parse           -> Some(Ok(log))   -> filtered by mode + request_ids
  |    - End of stream              -> None             -> loop ends
  |
  v
for log_result in logs {
    let log = log_result?;           // I/O error -> early return Err(e)
    if (request_id_filter) && match &mode {
        ReadMode::All => true,
        ReadMode::Errors => matches!(...),
        ReadMode::Exchanges => matches!(...),
    } { collected.push(log) }
}
Ok(collected)                        // success -> Ok(Vec<LogLine>)
```

The data flow is identical to the pre-refactoring version for the success path. The only new path is the I/O error propagation: `BufReader` -> `LogIterator::next()` -> `Some(Err(e))` -> `log_result?` in `read_log()` -> `Err(e)` returned to caller.

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|---|---|
| Zero external dependencies | No new crates. Uses only `std::io::Error` as the error type. |
| No behavior changes on success path | When no I/O errors occur, output is identical. The `Ok(...)` wrapper is the only difference at the API boundary. |
| No test deletions | All three test functions (`test_all`, `test_errors_mode`, `test_exchanges_mode`) are preserved. Only `.unwrap()` is added to `read_log()` calls. |
| I/O error propagation | I/O errors from `BufReader::lines()` are yielded as `Some(Err(e))` by `LogIterator` and propagated via `?` in `read_log()`. No longer silently swallowed. |
| Parse errors remain silent | Parse failures and non-empty-remainder lines are skipped via `continue` in the `loop` inside `LogIterator::next()`. |
| Scope boundary | Only `read_log()` return type and `LogIterator::next()` error handling are changed. The hint `// подсказка: можно обойтись итераторами` is preserved (Phase 9). No iterator refactoring of the `for` loop in `read_log()`. |
| Compiler-driven migration | After changing the `read_log()` return type, the compiler identifies all call sites that need adaptation. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Changing `LogIterator::Item` to `Result` requires `loop` + `continue` pattern in `next()` | Certain (structural) | Low | The `loop` with explicit `match` on I/O result and `continue` on parse failure is a well-known Rust iterator pattern. Straightforward implementation. |
| All tests use `&[u8]` readers that never produce I/O errors, so the error path is not exercised | Medium | Low | The primary goal is API correctness. The PRD explicitly states a dedicated failing-reader test is not required. The success path is fully tested by existing tests. |
| Changing `read_log()` return type is a breaking API change | Certain (by design) | Low | This is the explicit goal of the phase. The only external call site is `main.rs` in the same crate. |
| The latent parse-error-stops-iteration bug in current `next()` means behavior changes for inputs with unparseable non-blank lines | Low | Low | The new `loop`/`continue` pattern correctly implements the specified "skip unparseable lines" behavior. All existing tests pass because test data has no unparseable non-blank lines. The change aligns behavior with requirements. |
| Phase 9 (loops -> iterators) will need to account for `Result` in the `for` loop | Low | Low | Phase 9 can use `.collect::<Result<Vec<_>, _>>()` or similar. The `Result` return type is a clean foundation. |

---

## Deviations to Fix

None. The research document (Section 10) confirms the current codebase state matches the PRD's gap analysis exactly:

- `read_log()` returns `Vec<LogLine>` (not `Result`) -- confirmed at line 50.
- `LogIterator::next()` uses `.ok()?` to silently swallow I/O errors -- confirmed at line 43.
- No `panic!` calls remain in `src/lib.rs` -- confirmed (task 7.1 already satisfied by Phase 6).
- `main.rs` uses `read_log()` without error handling -- confirmed at line 67.
- Tests call `read_log()` and use the returned `Vec<LogLine>` directly -- confirmed at lines 160, 161, 174, 180, 197, 203.
- The remaining hint comment `// подсказка: можно обойтись итераторами` at line 53 is Phase 9 scope -- confirmed.

No code deviates from requirements. No corrective tasks are needed.

---

## Implementation Tasks

### Task 7.1: Confirm `panic!` elimination (no work needed)

Phase 6 already replaced the `if`/`else if`/`else` chain (including `panic!("unknown mode {:?}", mode)`) with an exhaustive `match` on `ReadMode`. This task is formally acknowledged as complete with no further action required.

### Task 7.2: Change `LogIterator::Item` to `Result<LogLine, std::io::Error>`

**File:** `src/lib.rs`, lines 40-47

Replace the `Iterator` implementation for `LogIterator<R>`:

1. Change `type Item` from `parse::LogLine` to `Result<parse::LogLine, std::io::Error>`.
2. Replace the chained-`?` body of `next()` with a `loop` that:
   - Returns `None` when the underlying iterator is exhausted.
   - Returns `Some(Err(e))` for I/O errors.
   - Uses `continue` to skip parse errors and non-empty-remainder lines.
   - Returns `Some(Ok(result))` for successfully parsed lines.

### Task 7.3a: Change `read_log()` return type and loop

**File:** `src/lib.rs`, lines 50-86

1. Change the return type from `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>`.
2. Rename the loop variable from `log` to `log_result`.
3. Add `let log = log_result?;` as the first line inside the loop body to propagate I/O errors.
4. Change `collected` (line 85) to `Ok(collected)`.
5. Leave the hint comment `// подсказка: можно обойтись итераторами` untouched.
6. Leave the request_id filter and mode `match` expression unchanged.

### Task 7.3b: Adapt test functions

**File:** `src/lib.rs`, test module

Add `.unwrap()` to all six `read_log()` calls in the three test functions:
- `test_all` (2 calls, lines 160-161)
- `test_errors_mode` (2 calls, lines 174, 180)
- `test_exchanges_mode` (2 calls, lines 197, 203)

No test functions are deleted. No test function signatures are changed.

### Task 7.3c: Adapt `main.rs` call site

**File:** `src/main.rs`, line 67

Add `.unwrap()` to the `read_log()` call, consistent with the existing `.unwrap()` pattern in `main.rs`.

### Task 7.4: Verify

Run the acceptance criteria:

```bash
cargo test                # All tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring (success path)
```

Additionally verify:

```bash
# read_log return type includes Result
grep "Result<Vec<LogLine>" src/lib.rs
# Expected: one hit (the function signature)

# LogIterator::Item is Result
grep "type Item = Result" src/lib.rs
# Expected: one hit

# No .ok()? remaining in LogIterator::next()
grep "\.ok()?" src/lib.rs
# Expected: zero hits

# Phase 9 hint preserved
grep "подсказка: можно обойтись" src/lib.rs
# Expected: one hit (line 53)

# Zero panic! in lib.rs
grep "panic!" src/lib.rs
# Expected: zero hits

# main.rs handles Result
grep "read_log.*unwrap()" src/main.rs
# Expected: one hit
```

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. The research recommends Approach A (change `LogIterator::Item` to `Result`) over Approach B (inline iteration into `read_log()`), and this plan implements Approach A. No architectural decision record is needed because the alternative (Approach B) is strictly inferior -- it would inline `LogIterator` logic and break separation of concerns without any compensating benefit.

# Research: PH-9 -- Loops to Iterators

**Ticket:** PH-9 "Phase 9: Loops to iterators"
**PRD:** `docs/prd/PH-9.prd.md`
**Phase spec:** `docs/phase/phase-9.md`

---

## 1. Resolved Questions

The PRD has no open questions. The user confirmed proceeding with default requirements only -- no additional constraints or preferences.

---

## 2. Related Modules/Services

### 2.1 `src/lib.rs` -- Primary Target

This is the sole file containing the loops targeted by this phase. The `read_log()` function (lines 60-101) contains both the outer `for` loop and the inner `for` loop that must be replaced with iterator chains.

**Current `read_log()` function (lines 60-101):**

```rust
pub fn read_log(
    input: impl Read,
    mode: ReadMode,
    request_ids: Vec<u32>,
) -> Result<Vec<LogLine>, std::io::Error> {
    let logs = LogIterator::new(input);
    let mut collected = Vec::new();
    // подсказка: можно обойтись итераторами
    for log_result in logs {
        let log = log_result?;
        if request_ids.is_empty() || {
            let mut request_id_found = false;
            for request_id in &request_ids {
                if *request_id == log.request_id {
                    request_id_found = true;
                    break;
                }
            }
            request_id_found
        } && match &mode {
            ReadMode::All => true,
            ReadMode::Errors => matches!(
                &log.kind,
                LogKind::System(SystemLogKind::Error(_)) | LogKind::App(AppLogKind::Error(_))
            ),
            ReadMode::Exchanges => matches!(
                &log.kind,
                LogKind::App(AppLogKind::Journal(
                    AppLogJournalKind::BuyAsset(_)
                        | AppLogJournalKind::SellAsset(_)
                        | AppLogJournalKind::CreateUser { .. }
                        | AppLogJournalKind::RegisterAsset { .. }
                        | AppLogJournalKind::DepositCash(_)
                        | AppLogJournalKind::WithdrawCash(_)
                ))
            ),
        } {
            collected.push(log);
        }
    }
    Ok(collected)
}
```

**Inner loop (lines 71-78) -- manual request ID search:**

```rust
let mut request_id_found = false;
for request_id in &request_ids {
    if *request_id == log.request_id {
        request_id_found = true;
        break;
    }
}
request_id_found
```

This is a textbook `contains()` pattern -- a linear search with a mutable flag and early `break`.

**Outer loop (lines 68-99) -- manual collect with push:**

```rust
let mut collected = Vec::new();
for log_result in logs {
    let log = log_result?;
    if <condition> {
        collected.push(log);
    }
}
Ok(collected)
```

This is a textbook `.filter().collect()` pattern -- iterating, filtering, and pushing into a mutable `Vec`.

### 2.2 `src/lib.rs` -- `LogIterator` (lines 17-56) -- NOT Modified

The `loop` inside `Iterator::next()` for `LogIterator` (line 43) is idiomatic for an iterator that needs to skip unparseable lines. It uses `self.lines.next()?` for `Option` propagation and `continue` to skip invalid parses. This is the standard pattern for implementing a stateful iterator and is explicitly out of scope per the PRD.

### 2.3 `src/parse.rs` -- NOT Modified

Three loop constructs exist in `parse.rs`, all out of scope:

| Location | Pattern | Why Out of Scope |
|---|---|---|
| Line 101 (`do_unquote`) | `while let Some(c) = chars.next()` | Idiomatic character-by-character state machine for unquoting strings. Cannot be replaced with a simple iterator chain due to the escape-state tracking and early return. |
| Line 517 (`List::parse`) | `while !remaining.is_empty()` | Parser combinator internal -- processes a bracketed list `[a, b, c,]` with mutable `remaining` tracking. Standard for parser combinators. |
| Line 707 (`Take::parse`) | `for _ in 0..self.count` | Fixed-count parsing loop. Standard for parser combinators. |

### 2.4 `src/main.rs` -- NOT Modified

No loops in `main.rs`. Uses `.for_each()` on an iterator (line 70), which is already idiomatic.

### 2.5 `Cargo.toml` -- NOT Modified

Zero external dependencies. This phase uses only standard Rust iterator methods. No new crates needed.

---

## 3. Current Endpoints and Contracts

### 3.1 Public API Surface -- Unchanged

| Item | Before PH-9 | After PH-9 |
|---|---|---|
| `read_log()` signature | `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>` | **Unchanged** |
| `ReadMode` enum | `pub enum ReadMode { All, Errors, Exchanges }` | **Unchanged** |
| `LogIterator` | Private struct, generic `<R: Read>` | **Unchanged** (not touched) |

The public API is completely unchanged. This is a pure internal refactoring of the `read_log()` function body.

### 3.2 Test Coverage

- **25 tests pass currently** (22 in `parse.rs`, 3 in `lib.rs`).
- **3 tests in `lib.rs` exercise `read_log()` directly:**
  - `test_all` -- Tests `ReadMode::All` with empty request_ids and full SOURCE data.
  - `test_errors_mode` -- Tests `ReadMode::Errors` with specific request_ids (verifies only error entries are returned).
  - `test_exchanges_mode` -- Tests `ReadMode::Exchanges` with specific request_ids (verifies only journal entries are returned).
- **Test assertions in `test_errors_mode` and `test_exchanges_mode`** use `for log in &errors` / `for log in &exchanges` loops with `assert!(matches!(...))`. These test loops iterate over results for assertion purposes and are not targeted by this phase.
- **No tests should be modified.** The function signature and return values are unchanged.

---

## 4. Patterns Used

### 4.1 Inner Loop Replacement: `contains()` vs `any()`

The inner `for` loop is a linear search for a matching `request_id` in a `Vec<u32>`. Two idiomatic replacements exist:

**Option A -- `Vec::contains()` (recommended):**
```rust
request_ids.contains(&log.request_id)
```
- Simplest and most readable.
- `Vec::contains(&T)` performs exactly the same linear search as the manual loop.
- No closure overhead.

**Option B -- `Iterator::any()`:**
```rust
request_ids.iter().any(|&id| id == log.request_id)
```
- Also idiomatic, slightly more verbose.
- Useful when the comparison is more complex than simple equality, but overkill here.

**Recommendation:** Use `contains()` for maximum conciseness. The PRD's Scenario 1 presents both options and notes that `contains()` is "the most concise and readable for simple equality checks."

### 4.2 Outer Loop Replacement: Iterator Chain

The outer `for` loop with `push` follows the pattern: iterate, unwrap `Result`, filter, collect. Several iterator chain designs are viable:

**Approach A -- Collect all Results first, then filter (two-pass):**
```rust
LogIterator::new(input)
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .filter(|log| {
        (request_ids.is_empty() || request_ids.contains(&log.request_id))
            && match &mode {
                ReadMode::All => true,
                ReadMode::Errors => matches!(...),
                ReadMode::Exchanges => matches!(...),
            }
    })
    .collect()
```
- Cleanest separation of concerns: error handling first, then filtering.
- Reads all lines into memory before filtering. For log files this is negligible since all lines must be read from the input stream sequentially anyway.
- Uses `collect::<Result<Vec<_>, _>>()` which short-circuits on the first `Err`, preserving the current error semantics.
- After the `?`, we have `Vec<LogLine>` and can filter purely.

**Approach B -- Interleaved error handling and filtering (single-pass):**
```rust
LogIterator::new(input)
    .map(|log_result| {
        let log = log_result?;
        Ok((request_ids.is_empty() || request_ids.contains(&log.request_id))
            && match &mode { ... }
        ).then_some(log))
    })
    .filter_map(|r| match r {
        Ok(Some(log)) => Some(Ok(log)),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    })
    .collect::<Result<Vec<_>, _>>()
```
- Single-pass but significantly more complex.
- The `Result<Option<LogLine>, Error>` nesting makes the code harder to read.

**Recommendation:** Approach A (two-pass) is cleaner and more readable. The PRD explicitly states this approach in Scenario 2 and notes in the Risks section that "The practical difference is negligible since all lines must be read from the input stream sequentially regardless."

### 4.3 Error Propagation with `collect::<Result<Vec<_>, _>>()`

The `Iterator::collect()` method has a special implementation for `Result` types: when collecting an iterator of `Result<T, E>` into `Result<Vec<T>, E>`, it short-circuits on the first `Err`. This preserves the current behavior of the `for` loop with `?` -- the function returns the first I/O error immediately.

The subtle behavioral difference with Approach A is that the current code filters while parsing (a line that parses successfully but is filtered out is never pushed to `collected`), whereas Approach A parses all lines first and filters afterward. The **observable behavior** is identical: same `Result<Vec<LogLine>, std::io::Error>` return value for any input. The only difference is memory usage (all parsed lines are in memory briefly before filtering), which is negligible for typical log files.

### 4.4 Existing Idiomatic Patterns in the Codebase

The codebase already uses iterator patterns extensively:

| Location | Pattern |
|---|---|
| `src/lib.rs:28-36` | `LogIterator::new()` uses `.lines().filter(...)` chain |
| `src/parse.rs:81-93` | `quote()` uses `.chars().map(...).flatten()` |
| `src/parse.rs:23-34` | `U32::parse()` uses `.char_indices().find_map(...)` |
| `src/main.rs:70` | `logs.iter().for_each(...)` |

The `read_log()` function is the last holdout using manual loop-and-push instead of iterator chains.

---

## 5. Detailed Implementation Plan

### Step 1: Replace inner `for` loop with `contains()`

**File:** `src/lib.rs`, lines 70-78

**Before:**
```rust
if request_ids.is_empty() || {
    let mut request_id_found = false;
    for request_id in &request_ids {
        if *request_id == log.request_id {
            request_id_found = true;
            break;
        }
    }
    request_id_found
}
```

**After:**
```rust
if request_ids.is_empty() || request_ids.contains(&log.request_id)
```

This eliminates:
- The mutable `request_id_found` flag
- The inner `for` loop
- The `break` statement
- The block expression wrapping the inner loop

### Step 2: Replace outer `for` loop with iterator chain

**File:** `src/lib.rs`, lines 65-100

**Before (after Step 1 applied):**
```rust
let logs = LogIterator::new(input);
let mut collected = Vec::new();
// подсказка: можно обойтись итераторами
for log_result in logs {
    let log = log_result?;
    if (request_ids.is_empty() || request_ids.contains(&log.request_id))
        && match &mode { ... }
    {
        collected.push(log);
    }
}
Ok(collected)
```

**After:**
```rust
let collected = LogIterator::new(input)
    .collect::<Result<Vec<_>, _>>()?;
Ok(collected
    .into_iter()
    .filter(|log| {
        (request_ids.is_empty() || request_ids.contains(&log.request_id))
            && match &mode {
                ReadMode::All => true,
                ReadMode::Errors => matches!(
                    &log.kind,
                    LogKind::System(SystemLogKind::Error(_))
                        | LogKind::App(AppLogKind::Error(_))
                ),
                ReadMode::Exchanges => matches!(
                    &log.kind,
                    LogKind::App(AppLogKind::Journal(
                        AppLogJournalKind::BuyAsset(_)
                            | AppLogJournalKind::SellAsset(_)
                            | AppLogJournalKind::CreateUser { .. }
                            | AppLogJournalKind::RegisterAsset { .. }
                            | AppLogJournalKind::DepositCash(_)
                            | AppLogJournalKind::WithdrawCash(_)
                    ))
                ),
            }
    })
    .collect())
```

This eliminates:
- The mutable `collected` variable
- The outer `for` loop
- The `push` call
- The hint comment

### Step 3: Remove the hint comment

The `// подсказка: можно обойтись итераторами` comment at line 67 is removed as part of Step 2 (it lives between the variable declaration and the `for` loop, both of which are eliminated).

### Step 4: Verify

```bash
cargo test                # All 25 tests pass (no test cases deleted or modified)
cargo run -- example.log  # Output identical to pre-refactoring
```

---

## 6. Limitations and Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Two-pass vs single-pass: Approach A collects all parsed lines before filtering, using slightly more memory than the current interleaved approach | Low | Low | Negligible for log files. All lines must be read sequentially from the input stream regardless. The PRD explicitly acknowledges this. |
| Short-circuit error semantics: `collect::<Result<Vec<_>, _>>()` stops at the first `Err`, same as the current `?` in the `for` loop | N/A | None | Semantics are preserved -- first I/O error aborts. |
| Complex filter closure readability: the `match` with nested `matches!()` macros inside a `.filter()` closure | Low | Low | The same complexity exists in the current `for` loop. Moving it into a closure does not add complexity. If desired, the filter predicate could be extracted into a named closure or helper function, but this is optional and the PRD does not require it. |
| Test loops in `test_errors_mode` and `test_exchanges_mode` (lines 202, 226) use `for log in &errors` | None | None | These are assertion loops in tests, not targeted by this phase. They iterate over already-collected results to verify each entry matches the expected pattern. |

---

## 7. Deviations from Requirements

**None.** The current codebase state matches the PRD's gap analysis exactly:

- The outer `for` loop exists at lines 68-99 in `read_log()` -- confirmed.
- The inner `for` loop exists at lines 71-78 with mutable `request_id_found` flag and `break` -- confirmed.
- The mutable `collected` variable exists at line 66 -- confirmed.
- The hint comment `// подсказка: можно обойтись итераторами` exists at line 67 -- confirmed.
- The `read_log()` signature is `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>` -- confirmed.
- The `loop` in `LogIterator::next()` is idiomatic and out of scope -- confirmed.
- The `while let` in `do_unquote` (line 101), `while` in `List::parse` (line 517), and `for` in `Take::parse` (line 707) are parser internals and out of scope -- confirmed.
- All 25 existing tests pass with `cargo test`.
- `cargo run -- example.log` succeeds.

---

## 8. New Technical Questions Discovered During Research

### 8.1 Whether to Extract the Filter Predicate

The filter closure contains a non-trivial condition: a request ID check combined with a `match` on `ReadMode` containing nested `matches!()` macros. While this is functionally correct and no more complex than the current `for` loop body, it could optionally be extracted into a named closure or helper function for clarity:

```rust
let matches_filter = |log: &LogLine| {
    (request_ids.is_empty() || request_ids.contains(&log.request_id))
        && match &mode {
            ReadMode::All => true,
            ReadMode::Errors => matches!(...),
            ReadMode::Exchanges => matches!(...),
        }
};
```

The PRD does not require this extraction; it is an optional readability improvement. The core requirement is that the manual loops and mutable state are eliminated.

### 8.2 Alternative: Single-Expression Function Body

The entire `read_log()` body could be collapsed into a single expression:

```rust
pub fn read_log(
    input: impl Read,
    mode: ReadMode,
    request_ids: Vec<u32>,
) -> Result<Vec<LogLine>, std::io::Error> {
    Ok(LogIterator::new(input)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|log| { ... })
        .collect())
}
```

This is maximally concise but may reduce readability slightly. Either form (with or without intermediate `let` binding) satisfies the requirements.

---

## 9. Scope Boundaries

| Concern | PH-9 (this phase) | Other Phases |
|---|---|---|
| Replace inner `for` loop with `contains()` | **In scope** | -- |
| Replace outer `for` loop with `.filter().collect()` | **In scope** | -- |
| Remove mutable `collected` variable | **In scope** | -- |
| Remove mutable `request_id_found` flag | **In scope** | -- |
| Remove hint comment `подсказка: можно обойтись итераторами` | **In scope** | -- |
| `loop` in `LogIterator::next()` | Out of scope (idiomatic) | -- |
| `while let` in `do_unquote()` | Out of scope (idiomatic) | -- |
| `while` in `List::parse()` | Out of scope (idiomatic) | -- |
| `for` in `Take::parse()` | Out of scope (idiomatic) | -- |
| `for` loops in test assertions (`test_errors_mode`, `test_exchanges_mode`) | Out of scope (test code) | -- |
| `Box` the large enum variant | Out of scope | Phase 10 |
| `NonZeroU32` tight type | Out of scope | Phase 11 |
| Remove `OnceLock` singleton | Out of scope | Phase 12 |

---

## 10. Verification Checklist

Per the acceptance criteria:

```bash
cargo test                # All 25 tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:

| Metric | Expected |
|---|---|
| Manual `for` loops in `read_log()` | Zero (both outer and inner loops replaced) |
| Mutable `collected` variable | Removed |
| Mutable `request_id_found` flag | Removed |
| Hint comment `подсказка: можно обойтись итераторами` | Removed |
| Lines of code in `read_log()` | Reduced |
| `read_log()` function signature | Unchanged |
| External dependencies added | Zero |
| Tests modified or deleted | Zero |

# Implementation Plan: PH-9 -- Loops to Iterators

**Status:** PLAN_APPROVED
**Ticket:** PH-9 "Phase 9: Loops to iterators"
**PRD:** `docs/prd/PH-9.prd.md`
**Research:** `docs/research/PH-9.md`
**Phase spec:** `docs/phase/phase-9.md`

---

## Components

### 1. Inner `for` loop -- Replace with `contains()`

**File:** `src/lib.rs`, lines 70-78

The inner `for` loop manually searches `request_ids` for a matching `log.request_id` using a mutable boolean flag (`request_id_found`) and `break`. This is a textbook `Vec::contains()` pattern.

**Current (lines 70-78):**
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
- The inner `for` loop with `break`
- The block expression wrapping the loop

`contains()` is chosen over `iter().any()` because it is the most concise and readable for simple equality checks, as noted in the PRD Scenario 1.

### 2. Outer `for` loop -- Replace with iterator chain

**File:** `src/lib.rs`, lines 65-100

The outer `for` loop iterates over `LogIterator` results, manually unwrapping each `Result` with `?`, applying a filter condition, and pushing matching items into a mutable `Vec`. This is a textbook `.filter().collect()` pattern.

The replacement uses a two-pass approach (Approach A from the research):
1. First, collect all `Result` items from `LogIterator` into a `Result<Vec<LogLine>, _>` using `collect::<Result<Vec<_>, _>>()?`. This short-circuits on the first `Err`, preserving the current error propagation semantics.
2. Then, filter the successfully collected `Vec<LogLine>` with `.into_iter().filter(...)`.collect()`.

**Current (lines 65-100):**
```rust
let logs = LogIterator::new(input);
let mut collected = Vec::new();
// подсказка: можно обойтись итераторами
for log_result in logs {
    let log = log_result?;
    if <filtering condition> {
        collected.push(log);
    }
}
Ok(collected)
```

**After (with inner loop replacement from Component 1 already applied):**
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
- The manual `push` call

### 3. Hint comment -- Remove

**File:** `src/lib.rs`, line 67

The `// подсказка: можно обойтись итераторами` ("hint: you can get by with iterators") comment is removed as part of Component 2. The technical debt it identifies is resolved by the iterator chain replacement.

---

## API Contract

### Before and After -- Unchanged

```rust
pub fn read_log(
    input: impl Read,
    mode: ReadMode,
    request_ids: Vec<u32>,
) -> Result<Vec<LogLine>, std::io::Error>
```

The function signature is unchanged. The return type is unchanged. For any given input, the function produces identical output before and after the refactoring. This is a pure internal restructuring of the function body.

### Error semantics

`collect::<Result<Vec<_>, _>>()` stops at the first `Err`, identical to the current `?` operator in the `for` loop. If any line from `LogIterator` yields an I/O error, `read_log()` returns that error immediately.

---

## Data Flows

```
Caller (main.rs, test, or library consumer)
  |
  |  read_log(input, mode, request_ids)
  v
LogIterator::new(input)
  |
  |  yields Result<LogLine, io::Error> items
  v
.collect::<Result<Vec<_>, _>>()?
  |
  |  short-circuits on first Err; produces Vec<LogLine> on success
  v
.into_iter().filter(|log| ...)
  |
  |  filters by request_ids (contains) AND by mode (match)
  v
.collect::<Vec<LogLine>>()
  |
  v
Ok(filtered_vec) returned to caller
```

The data flow is functionally identical to the pre-refactoring version. The only difference is that all lines are parsed before filtering (two-pass) rather than filtering during parsing (single-pass). The observable return value is identical for any input. The PRD explicitly acknowledges this difference in the Risks section and notes that "The practical difference is negligible since all lines must be read from the input stream sequentially regardless."

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|---|---|
| Zero external dependencies | No new crates. Uses only standard Rust iterator methods (`collect`, `into_iter`, `filter`, `contains`). |
| No behavior changes | Same input produces same output. Error propagation semantics preserved (first error aborts). |
| No test deletions | All 25 existing tests are preserved unmodified. The `read_log()` signature is unchanged, so no test adaptation is needed. |
| Scope boundary | Only the `read_log()` function body in `src/lib.rs` is modified. `LogIterator`, `parse.rs`, `main.rs`, and `Cargo.toml` are untouched. |
| Hint comment resolved | The `// подсказка: можно обойтись итераторами` comment is removed because the technical debt it identifies is resolved by the iterator chain. |
| Conventions compliance | Follows `docs/conventions.md` "Idiomatic Rust" table: "Instead of Manual loops with `push` -> Use Iterators: `.filter()`, `.map()`, `.collect()`". |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Two-pass vs single-pass memory usage: the current code filters during parsing (one line in memory at a time beyond the accumulator), while the refactored code parses all lines first and then filters | Low | Low | Negligible for log files. All lines must be read sequentially from the input stream regardless. The intermediate `Vec` holding all parsed lines before filtering is short-lived. The PRD explicitly acknowledges this. |
| Short-circuit error semantics: `collect::<Result<Vec<_>, _>>()` stops at the first `Err`, same as the current `?` in the `for` loop | N/A | None | Semantics are preserved. First I/O error aborts the function. |
| Complex filter closure readability: the `match` with nested `matches!()` macros inside a `.filter()` closure | Low | Low | The same complexity exists in the current `for` loop. Moving it into a closure does not add complexity. Extracting the filter predicate into a named closure is an optional readability improvement not required by the PRD. |

---

## Deviations to Fix

**None.** The current codebase state matches the PRD's gap analysis exactly:

- The outer `for` loop exists at lines 68-99 in `read_log()` -- confirmed in `src/lib.rs`.
- The inner `for` loop exists at lines 71-78 with mutable `request_id_found` flag and `break` -- confirmed.
- The mutable `collected` variable exists at line 66 -- confirmed.
- The hint comment `// подсказка: можно обойтись итераторами` exists at line 67 -- confirmed.
- The `read_log()` signature is `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>` -- confirmed.
- The `loop` in `LogIterator::next()` is idiomatic and out of scope -- confirmed.
- Loops in `parse.rs` (`do_unquote`, `List::parse`, `Take::parse`) are parser internals and out of scope -- confirmed.
- All 25 existing tests pass with `cargo test`.

No code deviates from requirements. No corrective tasks are needed.

---

## Implementation Tasks

### Task 9.1: Replace the entire `read_log()` body with an iterator chain

**File:** `src/lib.rs`, lines 65-100

This single task replaces the complete function body, addressing all three components at once (inner loop, outer loop, and hint comment removal). The changes are tightly coupled -- replacing the inner loop while keeping the outer loop would be a partial step that introduces unnecessary churn, and the outer loop replacement naturally subsumes the inner loop replacement.

Replace lines 65-100 of `read_log()`:

**Current:**
```rust
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

### Task 9.2: Verify

Run the acceptance criteria:

```bash
cargo test                # All 25 tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:

| Check | Expected |
|---|---|
| `for` loops in `read_log()` body | Zero |
| `let mut collected` in `read_log()` | Gone |
| `let mut request_id_found` in `read_log()` | Gone |
| `// подсказка: можно обойтись итераторами` | Gone |
| `request_ids.contains(` present | Yes (one hit) |
| `.filter(` and `.collect()` present | Yes |
| `read_log()` function signature | Unchanged |
| Number of tests | 25 (unchanged) |
| External dependencies | Zero (unchanged) |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. Both loops in `read_log()` map directly to standard iterator patterns (`contains()` for the inner loop, `.filter().collect()` for the outer loop). The two-pass approach with `collect::<Result<Vec<_>, _>>()` is the cleanest and most readable option for handling error propagation. No architectural decision record is needed because there is only one viable, idiomatic approach.

# PRD: Phase 9 — Loops to Iterators

**Status:** PRD_READY
**Ticket:** PH-9 "Phase 9: Loops to iterators"
**Phase:** 9 of 12 (see `docs/tasklist.md`)
**Dependencies:** None (independent phase)
**Blocked by:** Nothing
**Blocks:** None directly (downstream phases are independent)

---

## Context / Idea

Phase 9 targets the replacement of manual `for` / `while` loops with idiomatic Rust iterator chains in `src/lib.rs`. The original author left a hint at line 67: `// подсказка: можно обойтись итераторами` ("hint: you can get by with iterators"), indicating this was recognized as technical debt from the start.

### Authoritative specification from `docs/phase/phase-9.md`

**Goal:** Replace manual `for` / `while` loops with iterator chains where idiomatic.

**Tasks:**

- [ ] 9.1 Replace manual `for` / `while` loops with iterator chains where idiomatic

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** None (independent phase)

**Implementation Notes:**

- **Hint:** `src/lib.rs:76` -- `// подсказка: можно обойтись итераторами` (note: actual line number in current codebase is 67)

### Current codebase state (gap analysis)

The `read_log()` function in `src/lib.rs` (lines 60-101) contains the primary loop-based code targeted by this phase. The function currently uses:

1. **An outer `for` loop** (line 68) that iterates over `LogIterator` results, manually building a `Vec` with `collected.push(log)`:

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
        ReadMode::Errors => matches!(...),
        ReadMode::Exchanges => matches!(...),
    } {
        collected.push(log);
    }
}
Ok(collected)
```

2. **An inner `for` loop** (line 72) that manually searches `request_ids` for a matching ID, using a mutable boolean flag (`request_id_found`) and `break`:

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

This inner loop is a textbook case for `request_ids.contains(&log.request_id)` or `request_ids.iter().any(|&id| id == log.request_id)`.

The outer loop with manual `push` is a textbook case for `.filter().collect()` on the iterator.

**Note on `LogIterator::next()`** (line 43): The `loop` inside the `Iterator::next()` implementation is idiomatic for an iterator that needs to skip unparseable lines and is not targeted by this phase. It is the standard pattern for implementing a filtering/transforming iterator by hand.

**Note on `parse.rs` loops:** The `while let` loop in the string unquoting parser (line 101) and the `for` loop in the `Take` combinator (line 707) are parser internals where the loop structure is appropriate and idiomatic. They are not targeted by this phase.

---

## Goals

1. **Replace the inner `for` loop** (manual request ID search with mutable flag and `break`) with an idiomatic iterator method such as `request_ids.contains(&log.request_id)` or `request_ids.iter().any(|&id| id == log.request_id)`.

2. **Replace the outer `for` loop with `push`** (manual collection pattern) with an iterator chain using `.filter()` and `.collect()`, eliminating the mutable `collected` variable.

3. **Handle the `Result` propagation** within the iterator chain. Since `LogIterator` yields `Result<LogLine, std::io::Error>`, the `?` operator currently used inside the `for` loop must be replaced with an appropriate iterator pattern such as `.collect::<Result<Vec<_>, _>>()` with filtering applied before collection, or `.filter_map()` / `.map()` combined with `collect`.

4. **Remove the hint comment.** The `// подсказка: можно обойтись итераторами` comment at line 67 should be removed, as the technical debt it identifies will be resolved.

5. **Preserve all existing behavior.** The refactored iterator chain must produce identical results for all read modes and request ID filters. All existing tests must pass without modification.

---

## User Stories

1. **As a maintainer of `src/lib.rs`**, I want the `read_log()` function to use idiomatic Rust iterator chains instead of manual loops with mutable state, so that the code is more concise, readable, and less prone to off-by-one or state-management bugs.

2. **As a Rust developer reading this codebase**, I want to see standard iterator patterns (`.filter()`, `.collect()`, `.contains()`, `.any()`) instead of manual flag-and-break loops, so that the intent of the code is immediately clear.

3. **As a contributor extending filtering logic**, I want the filtering to be expressed as composable iterator adapters, so that adding new filter criteria is a matter of chaining another `.filter()` rather than nesting another loop inside a complex `if` condition.

---

## Scenarios

### Scenario 1: Inner loop replaced with `contains()` or `any()`

**Before:**
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

**After (option A -- `contains`):**
```rust
request_ids.contains(&log.request_id)
```

**After (option B -- `any`):**
```rust
request_ids.iter().any(|&id| id == log.request_id)
```

Both are idiomatic. `contains()` is the most concise and readable for simple equality checks.

### Scenario 2: Outer loop replaced with iterator chain

**Before:**
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

**After (iterator chain with `collect::<Result<_, _>>`):**
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
    .collect::<Vec<_>>()
```

Or alternatively, with error handling interleaved in the chain:

```rust
LogIterator::new(input)
    .filter_map(|log_result| {
        // transpose Result and Option for filtering
    })
    .collect::<Result<Vec<_>, _>>()
```

The exact form is an implementation choice; the key requirement is that the manual loop, mutable accumulator, and manual flag-and-break pattern are all eliminated in favor of iterator combinators.

### Scenario 3: Hint comment removed

**Before:**
```rust
// подсказка: можно обойтись итераторами
```

**After:** This comment is removed. The iterator chain speaks for itself.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| Manual `for` loops in `read_log()` | Zero (both outer and inner loops replaced) |
| Mutable `collected` variable | Removed |
| Mutable `request_id_found` flag | Removed |
| Hint comment `подсказка: можно обойтись итераторами` | Removed |
| Lines of code in `read_log()` | Reduced (fewer lines of boilerplate loop machinery) |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** The iterator chain must filter and collect identically to the manual loops it replaces. All existing tests must pass without modification.
3. **No test deletions.** Existing tests are not deleted or modified unless strictly necessary to accommodate the API change (none should be necessary since `read_log()` signature is unchanged).
4. **Error propagation preserved.** `read_log()` currently returns `Result<Vec<LogLine>, std::io::Error>`. The `?` operator propagation of I/O errors from `LogIterator` must be preserved in the iterator chain -- if any line yields an `Err`, the function must return that error immediately (short-circuit behavior).
5. **Scope boundary.** This phase addresses only the loops in `read_log()` in `src/lib.rs`. The `loop` in `LogIterator::next()`, the `while let` in `parse.rs` string unquoting, and the `for` in the `Take` combinator are not in scope -- they are idiomatic for their context.
6. **Public API unchanged.** The signature of `read_log()` remains `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Result<Vec<LogLine>, std::io::Error>`.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Short-circuit error semantics change: the current `for` loop with `?` stops at the first I/O error. An iterator chain using `.collect::<Result<Vec<_>, _>>()` on the entire iterator before filtering would also stop at the first error but would parse all lines before filtering, which is a behavioral change (filtering happens after full parse instead of interleaved). | Medium | Low | Either accept the slight behavioral difference (all lines parsed before filtering) or use a chain that interleaves error handling with filtering, e.g., using `.map()` then `.filter()` then `.collect::<Result<_, _>>()`. Both approaches preserve the "first error aborts" semantics. The practical difference is negligible since all lines must be read from the input stream sequentially regardless. |
| Iterator chain readability for complex filtering conditions (nested `match` + `matches!` macros inside a `.filter()` closure) | Low | Low | The filtering logic is already complex in the current `for` loop. Moving it into a closure does not add complexity -- it may actually improve readability by separating the iteration machinery from the filtering predicate. Consider extracting the filter predicate into a named helper closure or function if it becomes unwieldy. |
| Performance: collecting all results first and then filtering (two-pass) vs. filtering during iteration (single-pass) | Low | Low | For log files of typical size, the difference is negligible. If single-pass is desired, the iterator chain can use `.filter_map()` or process `Result` items inline. The current code already processes all lines sequentially. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. The two loops in `read_log()` map directly to standard iterator patterns (`contains()`/`any()` for the inner loop, `.filter().collect()` for the outer loop). The only design choice is the exact form of error handling in the iterator chain, which is an implementation detail with multiple valid approaches.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | Refactor `read_log()` to replace the outer `for` loop (lines 68-99) and inner `for` loop (lines 71-78) with an idiomatic iterator chain using `.filter()` and `.collect()`. Replace the manual request ID search with `contains()` or `any()`. Remove the mutable `collected` variable and `request_id_found` flag. Remove the `// подсказка: можно обойтись итераторами` hint comment. |

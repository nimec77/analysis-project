# Implementation Plan: PH-4 -- Generic `R: Read` instead of trait object

**Status:** PLAN_APPROVED
**Ticket:** PH-4 "Phase 4: Generic R: Read instead of trait object"
**PRD:** `docs/prd/PH-4.prd.md`
**Research:** `docs/research/PH-4.md`
**Phase spec:** `docs/phase/phase-4.md`

---

## Components

### 1. `MyReader` trait (to be removed)

**File:** `src/lib.rs`, lines 12-17

The supertrait combining `Read + Debug + 'static` exists solely as a workaround for Rust's E0225 restriction. Once `LogIterator` is generic over `R: Read`, the trait object is gone, and the workaround is unnecessary. The trait definition (line 15), blanket impl (line 16), E0225 comment block (lines 12-14), and hint comment (line 17) are all removed.

### 2. `LogIterator` struct (to be parameterized)

**File:** `src/lib.rs`, lines 19-26

Currently stores `lines: Filter<Lines<BufReader<Box<dyn MyReader>>>, ...>`. The inner `Box<dyn MyReader>` is replaced with a generic type parameter `R`, making the struct `LogIterator<R: Read>`.

### 3. `LogIterator::new()` (signature change)

**File:** `src/lib.rs`, lines 27-41

Currently accepts `reader: Box<dyn MyReader>`. After the change, accepts `reader: R` where `R: Read`.

### 4. `impl Iterator for LogIterator` (to be parameterized)

**File:** `src/lib.rs`, lines 43-50

Currently `impl Iterator for LogIterator`. After the change, becomes `impl<R: Read> Iterator for LogIterator<R>`.

### 5. `read_log()` function (signature change)

**File:** `src/lib.rs`, line 53

Currently `pub fn read_log(input: Box<dyn MyReader>, ...)`. After the change, `pub fn read_log(input: impl Read, ...)`.

### 6. Test code (call site adaptation)

**File:** `src/lib.rs`, lines 176-178

Two `Box<dyn MyReader>` type annotations and `Box::new(...)` wrappers in the `test_all` function. Replaced with direct `as_bytes()` calls.

### 7. CLI binary (call site adaptation)

**File:** `src/main.rs`, line 66

`Box<dyn analysis::MyReader>` type annotation and `Box::new(...)` wrapper. Replaced with passing `File` directly.

---

## API Contract

### Before

```rust
// Public trait (src/lib.rs)
pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}

// Public function (src/lib.rs)
pub fn read_log(input: Box<dyn MyReader>, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

Callers must box their readers into `Box<dyn MyReader>` and import or reference the `MyReader` trait.

### After

```rust
// Public function (src/lib.rs)
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

- `MyReader` trait is removed entirely (no longer `pub`).
- Callers pass any `R: Read` directly -- no boxing, no type annotation.
- Return type `Vec<LogLine>` is unchanged and does not depend on the generic.
- `LogIterator<R>` remains private (not `pub`), so it does not affect the public API surface beyond `read_log()`.

---

## Data Flows

```
Caller (main.rs or test)
  │
  │  passes R: Read (e.g., File, &[u8])
  │  ── no boxing, no trait object ──
  ▼
read_log(input: impl Read, mode, request_ids) -> Vec<LogLine>
  │
  │  ownership transfer: input moved into LogIterator::new()
  ▼
LogIterator<R>::new(reader: R)
  │
  │  wraps: BufReader::with_capacity(4096, reader)
  │  chains: .lines().filter(...)
  ▼
LogIterator<R>.lines  ── static dispatch, monomorphized per R ──
  │
  │  each .next() call:
  │    1. reads a line from BufReader<R> (static dispatch)
  │    2. parses via LOG_LINE_PARSER
  │    3. yields Option<LogLine>
  ▼
read_log() collects filtered LogLine values into Vec<LogLine>
```

The data flow is identical to the pre-refactoring version. The only change is the dispatch mechanism: dynamic (vtable) becomes static (monomorphization).

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|---|---|
| Zero external dependencies | No new crates. Only `std::io::Read` (already transitively used). |
| No behavior changes | Same input produces same output. All tests preserved and adapted. |
| No test deletions | Tests are adapted (remove boxing), not deleted. |
| Performance | Static dispatch replaces dynamic dispatch, enabling inlining and monomorphization. |
| Scope boundary | Only the trait-object-to-generic conversion. Other hints (lines 4, 56, 68, 94) are untouched. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `#[derive(Debug)]` fails without `R: Debug` bound | Certain (if kept) | Low | Remove `#[derive(Debug)]` from `LogIterator`. The struct is private and never formatted via `Debug`. This is the cleanest option (see Research section 4). |
| `'static` bound lost | Certain | None | Dropping `'static` is strictly more permissive. All current callers use `'static`-compatible types. No breakage possible. |
| `main.rs` breaks due to removed `MyReader` | Certain | Low | Mechanical fix: remove type annotation, remove `Box::new()`, pass `File` directly. Covered by task 4.4. |
| Monomorphization code size increase | Low | Negligible | Only two concrete reader types (`File`, `&[u8]`). Two specializations vs. one vtable -- trivial. |
| External consumers depend on `pub MyReader` | Very Low | Medium | Internal project, no known external consumers. Safe to remove. |

---

## Deviations to Fix

None. The research document (section 11) confirms the current codebase state after Phases 2 and 3 matches the PRD's "before" state exactly across all 8 `MyReader` usage sites. No code deviates from requirements.

---

## Implementation Tasks

### Task 4.1: Make `LogIterator` generic

**File:** `src/lib.rs`

1. Add `use std::io::Read;` import near the top of the file (after `use parse::*;`).

2. Remove the entire `MyReader` block (lines 12-17):
   ```rust
   // REMOVE:
   /// Для `Box<dyn много трейтов, помимо auto-трейтов>`, (`rustc E0225`)
   /// `only auto traits can be used as additional traits in a trait object`
   /// `consider creating a new trait with all of these as supertraits and using that trait here instead`
   pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}
   impl<T: std::io::Read + std::fmt::Debug + 'static> MyReader for T {}
   // подсказка: вместо trait-объекта можно дженерик
   ```

3. Remove `#[derive(Debug)]` from `LogIterator` (line 19). The struct is private and never formatted. This avoids requiring `R: Debug` as a bound.

4. Parameterize the struct definition (lines 20-26):
   ```rust
   // FROM:
   struct LogIterator {
       #[allow(clippy::type_complexity)]
       lines: std::iter::Filter<
           std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>,
           fn(&Result<String, std::io::Error>) -> bool,
       >,
   }

   // TO:
   struct LogIterator<R: Read> {
       #[allow(clippy::type_complexity)]
       lines: std::iter::Filter<
           std::io::Lines<std::io::BufReader<R>>,
           fn(&Result<String, std::io::Error>) -> bool,
       >,
   }
   ```

5. Parameterize the `impl` block and `new()` (lines 27-41):
   ```rust
   // FROM:
   impl LogIterator {
       fn new(reader: Box<dyn MyReader>) -> Self {

   // TO:
   impl<R: Read> LogIterator<R> {
       fn new(reader: R) -> Self {
   ```

6. Parameterize the `Iterator` impl (line 43):
   ```rust
   // FROM:
   impl Iterator for LogIterator {

   // TO:
   impl<R: Read> Iterator for LogIterator<R> {
   ```

**Verify:** `cargo build` -- should fail only on call sites (`read_log`, tests, `main.rs`).

### Task 4.2: Remove `Box<dyn MyReader>` / `MyReader` trait

This is accomplished as part of Task 4.1 step 2 above. After the `MyReader` trait definition, blanket impl, and associated comments are removed, there are zero `MyReader` references remaining in `src/lib.rs` aside from call sites (which are updated in tasks 4.3 and below).

**Verify:** `grep -r "MyReader" src/lib.rs` returns only the test call sites (lines 176, 178) and `read_log` (line 53), which are addressed next.

### Task 4.3: Update `read_log()` signature

**File:** `src/lib.rs`, line 53

```rust
// FROM:
pub fn read_log(input: Box<dyn MyReader>, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine> {

// TO:
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine> {
```

The function body is unchanged -- `LogIterator::new(input)` now receives an `impl Read` which satisfies `R: Read`.

**Verify:** `cargo build` -- should fail only on test and `main.rs` call sites.

### Task 4.3a: Update test call sites

**File:** `src/lib.rs`, lines 176-178

```rust
// FROM:
let reader1: Box<dyn MyReader> = Box::new(SOURCE1.as_bytes());
assert_eq!(read_log(reader1, READ_MODE_ALL, vec![]).len(), 1);
let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);

// TO:
assert_eq!(read_log(SOURCE1.as_bytes(), READ_MODE_ALL, vec![]).len(), 1);
let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);
```

`&[u8]` implements `Read` directly. No boxing or type annotation needed.

**Verify:** `cargo test` -- all 16 tests pass.

### Task 4.4: Adapt `main.rs`

**File:** `src/main.rs`, line 66

```rust
// FROM:
let file: Box<dyn analysis::MyReader> = Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);

// TO:
let file = std::fs::File::open(filename).unwrap();
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

No boxing, no type annotation, no `MyReader` import. `File` implements `Read` directly.

**Verify:** `cargo build` succeeds. `cargo run -- example.log` produces identical output.

---

## Verification Checklist

After all tasks are complete, run the following:

```bash
# All tests pass (no deletions)
cargo test

# CLI output identical to pre-refactoring
cargo run -- example.log

# Zero MyReader occurrences in source
grep -r "MyReader" src/
# Expected: zero hits

# Zero Box<dyn occurrences in lib.rs
grep -r "Box<dyn" src/lib.rs
# Expected: zero hits

# Hint comment removed
grep -r "подсказка: вместо trait-объекта" src/
# Expected: zero hits

# LogIterator is generic
grep "struct LogIterator" src/lib.rs
# Expected: struct LogIterator<R: Read>

# read_log uses impl Read
grep "pub fn read_log" src/lib.rs
# Expected: pub fn read_log(input: impl Read, ...)
```

---

## Open Questions

1. **`#[derive(Debug)]` decision:** The plan removes `#[derive(Debug)]` from `LogIterator` since the struct is private and never formatted via `Debug`. This is the cleanest approach (avoids adding an `R: Debug` bound). If the user prefers keeping `#[derive(Debug)]` with `R: Read + Debug` as the bound, this is also acceptable -- all concrete reader types (`File`, `&[u8]`) implement `Debug`. **Default decision: remove `#[derive(Debug)]`.**

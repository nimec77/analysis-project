# Research: PH-4 -- Generic `R: Read` instead of trait object

**Ticket:** PH-4 "Phase 4: Generic R: Read instead of trait object"
**PRD:** `docs/prd/PH-4.prd.md`
**Phase spec:** `docs/phase/phase-4.md`

---

## 1. Existing Code Analysis

### 1.1 `MyReader` trait (lines 12-16 of `src/lib.rs`)

A supertrait combining `std::io::Read + std::fmt::Debug + 'static`, introduced to work around Rust's `E0225` restriction (only auto traits can be used as additional traits in a trait object). It has a blanket impl so any `T: Read + Debug + 'static` automatically implements `MyReader`.

```rust
/// For `Box<dyn many traits besides auto-traits>`, (`rustc E0225`)
pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}
impl<T: std::io::Read + std::fmt::Debug + 'static> MyReader for T {}
```

**Usages across the codebase (8 occurrences):**

| Location | Usage |
|---|---|
| `src/lib.rs:15` | Trait definition |
| `src/lib.rs:16` | Blanket impl |
| `src/lib.rs:23` | `LogIterator.lines` field type: `BufReader<Box<dyn MyReader>>` |
| `src/lib.rs:28` | `LogIterator::new()` parameter: `reader: Box<dyn MyReader>` |
| `src/lib.rs:53` | `read_log()` parameter: `input: Box<dyn MyReader>` |
| `src/lib.rs:176` | `test_all`: `let reader1: Box<dyn MyReader> = Box::new(SOURCE1.as_bytes());` |
| `src/lib.rs:178` | `test_all`: `let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());` |
| `src/main.rs:66` | CLI: `let file: Box<dyn analysis::MyReader> = Box::new(std::fs::File::open(filename).unwrap());` |

**Verdict:** All 8 occurrences are removed in this phase. The trait definition, blanket impl, and every usage site are eliminated.

### 1.2 Hint comment (line 17 of `src/lib.rs`)

```rust
// подсказка: вместо trait-объекта можно дженерик
```

Translation: "hint: instead of a trait object, you can use a generic." This is the technical debt marker that Phase 4 resolves. It must be removed once the refactoring is complete.

### 1.3 `LogIterator` struct (lines 19-26 of `src/lib.rs`)

Current definition after Phases 2 and 3:

```rust
#[derive(Debug)]
struct LogIterator {
    #[allow(clippy::type_complexity)]
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
```

The struct is not generic. The inner reader type is hardcoded as `Box<dyn MyReader>`, causing dynamic dispatch through a vtable on every read call. The struct derives `Debug`, which currently works because `Box<dyn MyReader>` implements `Debug` (the `MyReader` supertrait requires `Debug`).

### 1.4 `LogIterator::new()` (lines 27-42 of `src/lib.rs`)

```rust
impl LogIterator {
    fn new(reader: Box<dyn MyReader>) -> Self {
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
```

Accepts a boxed trait object. The `BufReader` takes ownership of the `Box<dyn MyReader>`, wrapping it with a 4096-byte buffer. The `.lines().filter(...)` chain produces the full iterator type stored in the `lines` field.

### 1.5 `impl Iterator for LogIterator` (lines 43-50 of `src/lib.rs`)

```rust
impl Iterator for LogIterator {
    type Item = parse::LogLine;
    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?.ok()?;
        let (remaining, result) = LOG_LINE_PARSER.parse(line.trim()).ok()?;
        remaining.trim().is_empty().then_some(result)
    }
}
```

This impl block is not generic. It must become `impl<R: Read> Iterator for LogIterator<R>` after the refactoring.

### 1.6 `read_log()` function (lines 52-102 of `src/lib.rs`)

```rust
pub fn read_log(input: Box<dyn MyReader>, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine> {
    let logs = LogIterator::new(input);
    ...
}
```

Public API function. Takes a boxed trait object, constructs a `LogIterator`, and filters/collects results. The signature changes to accept `impl Read`.

### 1.7 `test_all` (lines 174-188 of `src/lib.rs`)

```rust
let reader1: Box<dyn MyReader> = Box::new(SOURCE1.as_bytes());
assert_eq!(read_log(reader1, READ_MODE_ALL, vec![]).len(), 1);
let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

Two test invocations both create `Box<dyn MyReader>` from byte slices. After the refactoring, `&[u8]` implements `Read` directly and can be passed without boxing.

### 1.8 `main.rs` (line 66 of `src/main.rs`)

```rust
let file: Box<dyn analysis::MyReader> = Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

The CLI creates a boxed `File` with a `MyReader` type annotation and passes it to `read_log()`. After the refactoring, `File` implements `Read` directly and can be passed without boxing or type annotation.

---

## 2. Patterns Used

| Pattern | Where | Notes |
|---|---|---|
| Supertrait trait object | `MyReader` trait + `Box<dyn MyReader>` | Workaround for E0225. Enables combining `Read + Debug + 'static` in a single trait object. Eliminated by this phase. |
| Dynamic dispatch | `LogIterator.lines` field | Every `BufReader` read call goes through vtable indirection. Replaced with static dispatch/monomorphization. |
| `#[derive(Debug)]` on iterator struct | `LogIterator` | Requires the inner reader to implement `Debug`. With generics, this requires `R: Debug` bound or a manual `Debug` impl. |
| `impl Trait` in argument position | Target for `read_log()` | Desugars to a generic type parameter, keeping the public API concise. |
| Ownership transfer | `read_log()` -> `LogIterator::new()` -> `BufReader` | The reader is moved through the chain. No borrowing or reference counting involved. |

---

## 3. Implementation Path

### 3.1 Task 4.1: Make `LogIterator` generic

**Struct definition changes from:**
```rust
#[derive(Debug)]
struct LogIterator {
    #[allow(clippy::type_complexity)]
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
```

**To:**
```rust
#[derive(Debug)]
struct LogIterator<R: Read> {
    #[allow(clippy::type_complexity)]
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<R>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
```

The `impl` block changes from `impl LogIterator` to `impl<R: Read> LogIterator<R>`, and `new()` accepts `reader: R` instead of `reader: Box<dyn MyReader>`.

The `Iterator` impl changes from `impl Iterator for LogIterator` to `impl<R: Read> Iterator for LogIterator<R>`.

### 3.2 Task 4.2: Remove `MyReader` trait

Remove lines 12-17 from `src/lib.rs`:
```rust
/// Для `Box<dyn много трейтов, помимо auto-трейтов>`, (`rustc E0225`)
/// `only auto traits can be used as additional traits in a trait object`
/// `consider creating a new trait with all of these as supertraits and using that trait here instead`
pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}
impl<T: std::io::Read + std::fmt::Debug + 'static> MyReader for T {}
// подсказка: вместо trait-объекта можно дженерик
```

All six lines (three comment lines, trait definition, blanket impl, hint comment) are removed.

### 3.3 Task 4.3: Update `read_log()` signature

**From:**
```rust
pub fn read_log(input: Box<dyn MyReader>, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

**To:**
```rust
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

Using `impl Read` in argument position desugars to a hidden generic parameter, keeping the signature concise.

### 3.4 Task 4.4: Adapt `main.rs`

**From:**
```rust
let file: Box<dyn analysis::MyReader> = Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

**To:**
```rust
let file = std::fs::File::open(filename).unwrap();
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

No boxing, no type annotation, no `MyReader` import.

### 3.5 Test adaptation

**From:**
```rust
let reader1: Box<dyn MyReader> = Box::new(SOURCE1.as_bytes());
assert_eq!(read_log(reader1, READ_MODE_ALL, vec![]).len(), 1);
let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

**To:**
```rust
let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);
```

Or with explicit bindings:
```rust
let reader1 = SOURCE1.as_bytes();
assert_eq!(read_log(reader1, READ_MODE_ALL, vec![]).len(), 1);
let reader = SOURCE.as_bytes();
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

`&[u8]` implements `Read` directly. No boxing needed.

---

## 4. The `#[derive(Debug)]` Question

This is the most nuanced technical detail in this phase.

**Current state:** `LogIterator` derives `Debug`. This works because `Box<dyn MyReader>` implements `Debug` (the `MyReader` supertrait requires `Debug`).

**After the change:** `LogIterator<R: Read>` derives `Debug`. The derive macro expands to:
```rust
impl<R: Read + Debug> Debug for LogIterator<R> { ... }
```

The derive macro adds a `Debug` bound on `R` automatically. This means `LogIterator<R>` only implements `Debug` when `R: Debug`.

**Impact assessment:**

- `LogIterator` is a **private** struct (no `pub` modifier). It is only constructed inside `read_log()` and never exposed to external callers.
- No code in the codebase prints or formats `LogIterator` via `{:?}`. There are no `format!`, `println!`, `dbg!`, or `assert_eq!` calls involving `LogIterator` itself (only its `Item` type `LogLine` is printed).
- The `#[derive(Debug)]` is therefore currently unused at runtime -- it exists only as a development convenience.

**Options per the PRD's risk analysis:**

1. **Add `R: Read + Debug` bound** -- preserves the `#[derive(Debug)]` but adds a `Debug` requirement to all callers. Since `File`, `&[u8]`, and `Cursor<Vec<u8>>` all implement `Debug`, this is not restrictive in practice.
2. **Remove `#[derive(Debug)]`** -- simplest approach since `Debug` is unused. The bound on the struct is just `R: Read`.
3. **Implement `Debug` manually** -- write a custom `impl<R: Read> Debug for LogIterator<R>` that does not require `R: Debug` (e.g., prints a placeholder for the `lines` field).

**Recommendation:** Option 2 (remove `#[derive(Debug)]`) is the cleanest. The struct is private and never formatted. If debugging is needed later, a manual impl can be added. However, option 1 is also acceptable since all concrete reader types in the codebase implement `Debug`.

---

## 5. The `'static` Bound Question

**Current state:** `MyReader` requires `'static`. This means only owned types or `&'static` references can be used as readers.

**After the change:** With `R: Read` (no `'static` bound), types like `&'a [u8]` with non-`'static` lifetimes can be passed to `read_log()`. This is strictly more permissive.

**Impact:** No breakage. All existing callers pass either:
- `File` (owned, `'static`)
- `SOURCE.as_bytes()` where `SOURCE: &'static str` (produces `&'static [u8]`, which is `'static`)

Dropping the `'static` bound is safe and makes the API more flexible. The PRD's risk analysis agrees: "the `'static` bound ... is likely unnecessary."

---

## 6. `std::io::Read` Import

The current `src/lib.rs` does not have a top-level `use std::io::Read;` statement. The `Read` trait is referenced via its full path only inside `MyReader`'s definition (`std::io::Read`). After making `LogIterator` generic over `R: Read`, a `use std::io::Read;` import is needed at the top of the file (or the bound must be written as `R: std::io::Read`).

The idiomatic approach is to add `use std::io::Read;` near the top of `src/lib.rs`.

---

## 7. What Gets Removed

| Entity | File | Line(s) | Action |
|---|---|---|---|
| Comment block for E0225 workaround | `src/lib.rs` | 12-14 | **Remove** -- no longer applicable |
| `MyReader` trait definition | `src/lib.rs` | 15 | **Remove** -- no longer needed |
| `MyReader` blanket impl | `src/lib.rs` | 16 | **Remove** -- no longer needed |
| Hint comment `// подсказка: вместо trait-объекта можно дженерик` | `src/lib.rs` | 17 | **Remove** -- debt resolved |
| `Box<dyn MyReader>` in `LogIterator.lines` type | `src/lib.rs` | 23 | **Replace with** `R` |
| `Box<dyn MyReader>` in `LogIterator::new()` param | `src/lib.rs` | 28 | **Replace with** `R` |
| `Box<dyn MyReader>` in `read_log()` param | `src/lib.rs` | 53 | **Replace with** `impl Read` |
| `Box<dyn MyReader>` type annotations in `test_all` | `src/lib.rs` | 176, 178 | **Remove** -- pass readers directly |
| `Box<dyn analysis::MyReader>` in `main.rs` | `src/main.rs` | 66 | **Remove** -- pass `File` directly |

---

## 8. What Gets Kept

| Entity | Reason |
|---|---|
| `LogIterator` struct (modified to generic) | Still needed; now parameterized over `R: Read` |
| `LogIterator::new()` (modified signature) | Still needed; now accepts `R` instead of `Box<dyn MyReader>` |
| `impl Iterator for LogIterator` (modified to generic) | Still needed; now `impl<R: Read> Iterator for LogIterator<R>` |
| `read_log()` function (modified signature) | Still needed; now accepts `impl Read` |
| All test assertions and test data constants | No test cases deleted per constraints |
| `#[allow(clippy::type_complexity)]` on `lines` field | The type is still complex, just with `R` instead of `Box<dyn MyReader>` |
| All other hint comments (lines 4, 56, 68, 94) | Address different phases (5, 9, 6, 7 respectively) |

---

## 9. Dependencies and Layers

```
main.rs  -->  lib.rs::read_log(impl Read)  -->  LogIterator<R>::new(R)  -->  BufReader<R>
                                                                          -->  Lines iterator
                                                                          -->  Filter iterator
```

The change propagates from the inside out:
1. Remove `MyReader` trait and hint comment (prerequisite for all other changes)
2. Add `use std::io::Read;` import
3. Make `LogIterator` generic over `R: Read` (struct + impl + Iterator impl)
4. Update `read_log()` to accept `impl Read`
5. Update `test_all` (test caller)
6. Update `main.rs` (binary caller)

Steps 5 and 6 are mechanical follow-the-compiler-errors changes after steps 1-4.

---

## 10. Limitations and Risks

| Risk | Assessment |
|---|---|
| `Debug` bound on generic | **Low risk.** `LogIterator` is private and never formatted via `Debug`. Either add `R: Read + Debug` as the bound or remove `#[derive(Debug)]`. See section 4 for detailed analysis. |
| `'static` bound lost | **No risk.** Dropping `'static` is strictly more permissive. All current callers use `'static` types anyway. See section 5. |
| `main.rs` breaks due to removed `MyReader` | **Certain but trivial.** Remove the type annotation and `Box::new(...)`, pass `File` directly. |
| Monomorphization code size increase | **Negligible.** Only two concrete reader types exist in the codebase (`File` in `main.rs`, `&[u8]` in tests). The compiler generates two specializations instead of one vtable-dispatched version. |
| `read_log()` return type becomes dependent on generic | **Not a risk.** `read_log()` uses `impl Read` in argument position, which creates a hidden generic parameter. The return type `Vec<LogLine>` is concrete and does not depend on `R`. |
| External consumers depend on `MyReader` as `pub` trait | **Very low risk.** This is an internal project with no known external consumers. The trait can be safely removed. |

---

## 11. Deviations from Requirements

None. The current codebase state after Phases 2 and 3 matches the PRD's "before" state exactly:

- `MyReader` trait is defined at line 15 with blanket impl at line 16 -- matches PRD section "Current codebase state"
- `LogIterator` stores `Box<dyn MyReader>` -- matches PRD
- `LogIterator::new()` takes `Box<dyn MyReader>` -- matches PRD
- `read_log()` takes `Box<dyn MyReader>` -- matches PRD
- `main.rs` uses `Box<dyn analysis::MyReader>` -- matches PRD
- Tests use `Box<dyn MyReader>` -- matches PRD
- Hint comment `// подсказка: вместо trait-объекта можно дженерик` is present at line 17 -- matches PRD

All 16 existing tests pass. No code deviates from the requirements.

---

## 12. Resolved Questions

The PRD has no open questions. The user confirmed proceeding with default requirements.

---

## 13. New Technical Questions Discovered During Research

1. **`#[derive(Debug)]` decision:** Should `LogIterator<R>` keep `#[derive(Debug)]` (requiring `R: Debug`), remove it entirely, or use a manual impl? The struct is private and never formatted, making removal the simplest option. However, adding `R: Read + Debug` preserves existing behavior and all concrete reader types satisfy it. This is a low-stakes decision that does not affect correctness.

---

## 14. Verification

Per the acceptance criteria:

```bash
cargo test                # All 16 tests pass (no deletions)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:
- `grep -r "MyReader" src/` returns zero hits
- `grep -r "Box<dyn" src/lib.rs` returns zero hits
- `grep -r "подсказка: вместо trait-объекта" src/` returns zero hits
- `LogIterator` struct definition contains `<R: Read>` generic parameter
- `read_log()` parameter type is `impl Read`

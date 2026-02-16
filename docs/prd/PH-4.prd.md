# PRD: Phase 4 — Generic `R: Read` instead of trait object

**Status:** PRD_READY
**Ticket:** PH-4 "Phase 4: Generic R: Read instead of trait object"
**Phase:** 4 of 12 (see `docs/tasklist.md`)
**Dependencies:** Phase 2 (complete)
**Blocked by:** Nothing (Phase 2 is done)
**Blocks:** None

---

## Context / Idea

Phase 4 targets the replacement of the `Box<dyn MyReader>` trait object in `LogIterator` and `read_log()` with a generic type parameter `R: Read`. The original author left a hint at `src/lib.rs:17`: `// подсказка: вместо trait-объекта можно дженерик` ("hint: instead of a trait object, you can use a generic").

Currently, `LogIterator` stores its reader as `Box<dyn MyReader>`, requiring dynamic dispatch through the `MyReader` supertrait (which combines `std::io::Read + std::fmt::Debug + 'static`). The `MyReader` trait itself exists solely to work around Rust's E0225 restriction (only auto traits can be used as additional traits in a trait object). By making `LogIterator` generic over `R: Read`, the trait object indirection, the `Box` allocation, and the `MyReader` trait can all be eliminated, enabling static dispatch and monomorphization.

### Authoritative specification from `docs/phase/phase-4.md`

**Goal:** Make `LogIterator` generic over `R: Read` instead of using a trait object, removing the `Box<dyn MyReader>` / `MyReader` trait indirection.

**Tasks:**

- [ ] 4.1 Make `LogIterator` generic: `LogIterator<R: Read>`
- [ ] 4.2 Remove `Box<dyn MyReader>` / `MyReader` trait
- [ ] 4.3 Update `read_log()` signature to accept `impl Read`
- [ ] 4.4 Adapt `main.rs` to the new signature

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** Phase 2 complete

**Implementation Notes:**

- **Hint:** `src/lib.rs:17` -- `// подсказка: вместо trait-объекта можно дженерик`

### Current codebase state (gap analysis)

After Phases 2 and 3, `LogIterator` directly owns a `Box<dyn MyReader>` without `Rc<RefCell>` or `unsafe` code. The current state of `src/lib.rs`:

- **`MyReader` trait** (line 15): `pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}` with a blanket impl. This is `pub` and used by `main.rs` for type annotation.
- **`LogIterator` struct** (line 20): Stores `lines: std::iter::Filter<std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>, fn(...)>`. The `Box<dyn MyReader>` appears as the inner type of `BufReader`.
- **`LogIterator::new()`** (line 28): Takes `reader: Box<dyn MyReader>`.
- **`read_log()`** (line 53): Public API, takes `input: Box<dyn MyReader>`.
- **`main.rs`** (line 66): `let file: Box<dyn analysis::MyReader> = Box::new(std::fs::File::open(filename).unwrap());`
- **Tests** (lines 176-178): `let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());`

All of these must be updated to use the generic parameter instead.

---

## Goals

1. **Eliminate dynamic dispatch.** Replace `Box<dyn MyReader>` with a generic `R: Read` type parameter on `LogIterator`, enabling static dispatch and monomorphization.
2. **Remove the `MyReader` trait.** The supertrait workaround is no longer needed once the trait object is gone. Both the trait definition and its blanket impl are removed.
3. **Simplify the public API.** `read_log()` accepts `impl Read` instead of `Box<dyn MyReader>`, allowing callers to pass any reader without boxing.
4. **Remove the hint comment.** The hint `// подсказка: вместо trait-объекта можно дженерик` at line 17 is addressed and should be removed.
5. **Preserve behavior.** Same input produces same output. All existing tests pass; no tests are deleted.

---

## User Stories

1. **As a library consumer**, I want `read_log()` to accept any type implementing `Read` (e.g., `File`, `&[u8]`, `Cursor<Vec<u8>>`) without requiring me to box it, so that the API is more ergonomic and avoids unnecessary heap allocation.
2. **As a developer working on later phases**, I want `MyReader` removed so that the codebase has no unnecessary trait abstractions, making it easier to reason about.
3. **As a maintainer**, I want `LogIterator` to use static dispatch so that the compiler can inline and optimize read calls, improving performance for large log files.

---

## Scenarios

### Scenario 1: `LogIterator` becomes generic

**Before:**
```rust
struct LogIterator {
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
impl LogIterator {
    fn new(reader: Box<dyn MyReader>) -> Self { ... }
}
```

**After:**
```rust
struct LogIterator<R: Read> {
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<R>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
impl<R: Read> LogIterator<R> {
    fn new(reader: R) -> Self { ... }
}
```

The generic parameter `R` replaces `Box<dyn MyReader>` everywhere within the struct and its impl block.

### Scenario 2: `read_log()` signature change

**Before:**
```rust
pub fn read_log(input: Box<dyn MyReader>, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

**After:**
```rust
pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>
```

Using `impl Read` in argument position desugars to a generic parameter, keeping the signature concise while accepting any reader type.

### Scenario 3: `main.rs` adaptation

**Before:**
```rust
let file: Box<dyn analysis::MyReader> = Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

**After:**
```rust
let file = std::fs::File::open(filename).unwrap();
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

No boxing, no type annotation, no `MyReader` import needed.

### Scenario 4: Test adaptation (`test_all` in `lib.rs`)

**Before:**
```rust
let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

**After:**
```rust
let all_parsed = read_log(SOURCE.as_bytes(), READ_MODE_ALL, vec![]);
```

Or with an explicit binding:
```rust
let reader = SOURCE.as_bytes();
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

No boxing required. `&[u8]` implements `Read` directly.

### Scenario 5: `MyReader` trait removed

**Before:**
```rust
pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}
impl<T: std::io::Read + std::fmt::Debug + 'static> MyReader for T {}
```

**After:** Both lines deleted. The trait is no longer needed and is no longer `pub`.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| `MyReader` trait definition | Removed |
| `Box<dyn MyReader>` occurrences in `src/` | Zero |
| `MyReader` occurrences in `src/` | Zero |
| Hint comment `// подсказка: вместо trait-объекта можно дженерик` | Removed |
| `LogIterator` generic parameter | `R: Read` present |
| `read_log()` parameter type | `impl Read` (not `Box<dyn ...>`) |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** Same input must produce same output.
3. **No test deletions.** Existing tests are adapted, not deleted.
4. **Scope boundary.** This phase addresses only the trait object to generic conversion. Other technical debt (`u8` mode constants, `panic!`, `if` chain instead of `match`, etc.) is out of scope.
5. **Compiler-driven refactoring.** Change `LogIterator`'s type parameter and `read_log()` signature first, then follow compiler errors to update all call sites.
6. **`Iterator` impl must also be generic.** The `impl Iterator for LogIterator` block becomes `impl<R: Read> Iterator for LogIterator<R>`.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `Debug` bound lost: current `LogIterator` derives `Debug`, which requires the inner type to implement `Debug`. With `R: Read` alone, `#[derive(Debug)]` will fail unless `R: Debug` is also required. | Medium | Low | Either add `R: Read + Debug` as the bound, or remove `#[derive(Debug)]` from `LogIterator` if the `Debug` impl is not needed. The current `MyReader` requires `Debug`, so adding the bound preserves existing behavior. Alternatively, implement `Debug` manually. |
| `'static` bound lost: `MyReader` requires `'static`. Removing it could allow references with shorter lifetimes to be passed to `read_log()`. | Low | Low | In practice, callers pass owned types (`File`, `Vec<u8>` via `Cursor`, `&'static [u8]`). If the `'static` bound is needed, it can be added to the generic constraint, but it is likely unnecessary. |
| `main.rs` breaks due to removed `MyReader` trait | Certain | Low | Update `main.rs` as part of task 4.4. This is a known, mechanical change: remove the `Box<dyn analysis::MyReader>` type annotation and pass the `File` directly. |
| Callers outside this repo depend on `MyReader` as a public trait | Very Low | Medium | This is an internal project with no known external consumers. The trait can be safely removed. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. Phase 2 (the sole dependency) is already done. The refactoring is mechanical: parameterize the struct, update the signatures, remove the trait, and follow compiler errors.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | Remove `MyReader` trait and blanket impl (lines 15-16). Remove hint comment (line 17). Make `LogIterator` generic: `LogIterator<R: Read>`. Update `LogIterator::new()` to accept `R` instead of `Box<dyn MyReader>`. Update `impl Iterator for LogIterator` to `impl<R: Read> Iterator for LogIterator<R>`. Change `read_log()` signature from `Box<dyn MyReader>` to `impl Read`. Update `test_all`: remove `Box<dyn MyReader>` type annotations, pass readers directly. |
| `src/main.rs` | Remove `Box<dyn analysis::MyReader>` type annotation. Pass `File` directly to `read_log()` without boxing. Remove any `analysis::MyReader` import/usage. |

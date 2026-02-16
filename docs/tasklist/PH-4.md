# Tasklist: PH-4 -- Generic `R: Read` instead of trait object

**Status:** IMPLEMENT_STEP_OK
**Ticket:** PH-4 "Phase 4: Generic R: Read instead of trait object"
**PRD:** `docs/prd/PH-4.prd.md`
**Plan:** `docs/plan/PH-4.md`

---

## Context

Replace `Box<dyn MyReader>` trait object in `LogIterator` and `read_log()` with a generic type parameter `R: Read`. This eliminates dynamic dispatch, removes the `MyReader` supertrait workaround, and simplifies the public API. All behavior is preserved; no tests are deleted.

---

## Tasks

- [x] **4.1 Remove `MyReader` trait and related comments**
  - Remove the `MyReader` trait definition (`pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}`) and its blanket impl from `src/lib.rs`.
  - Remove the E0225 comment block (lines 12-14) and the hint comment `// подсказка: вместо trait-объекта можно дженерик` (line 17).
  - Add `use std::io::Read;` import near the top of the file.
  - **Acceptance criteria:**
    - Zero occurrences of `MyReader` in `src/lib.rs` (trait definition, blanket impl, comments).
    - Zero occurrences of `подсказка: вместо trait-объекта` in `src/`.
    - `use std::io::Read;` is present in `src/lib.rs`.

- [x] **4.2 Make `LogIterator` generic over `R: Read`**
  - Change `struct LogIterator` to `struct LogIterator<R: Read>`.
  - Replace `Box<dyn MyReader>` with `R` in the `lines` field type: `BufReader<R>` instead of `BufReader<Box<dyn MyReader>>`.
  - Remove `#[derive(Debug)]` from `LogIterator` (struct is private, never formatted via `Debug`).
  - Parameterize `impl LogIterator` to `impl<R: Read> LogIterator<R>`.
  - Change `LogIterator::new()` parameter from `reader: Box<dyn MyReader>` to `reader: R`.
  - Parameterize `impl Iterator for LogIterator` to `impl<R: Read> Iterator for LogIterator<R>`.
  - **Acceptance criteria:**
    - `struct LogIterator<R: Read>` is present in `src/lib.rs`.
    - `impl<R: Read> LogIterator<R>` is present in `src/lib.rs`.
    - `impl<R: Read> Iterator for LogIterator<R>` is present in `src/lib.rs`.
    - Zero occurrences of `Box<dyn` in `src/lib.rs`.
    - No `#[derive(Debug)]` on `LogIterator`.

- [x] **4.3 Update `read_log()` signature to accept `impl Read`**
  - Change `pub fn read_log(input: Box<dyn MyReader>, ...)` to `pub fn read_log(input: impl Read, ...)`.
  - Function body is unchanged.
  - **Acceptance criteria:**
    - `pub fn read_log(input: impl Read, mode: u8, request_ids: Vec<u32>) -> Vec<LogLine>` signature in `src/lib.rs`.
    - Zero occurrences of `Box<dyn MyReader>` in the `read_log` signature.

- [x] **4.4 Update test call sites in `src/lib.rs`**
  - Remove `Box<dyn MyReader>` type annotations and `Box::new(...)` wrappers in test functions.
  - Pass `SOURCE.as_bytes()` and `SOURCE1.as_bytes()` directly to `read_log()`.
  - **Acceptance criteria:**
    - Zero occurrences of `MyReader` in test code.
    - Zero occurrences of `Box::new(` wrapping readers in test code.
    - `&[u8]` passed directly to `read_log()` -- no boxing.
    - `cargo test` -- all existing tests pass, no tests deleted.

- [x] **4.5 Adapt `main.rs` to the new API**
  - Remove `Box<dyn analysis::MyReader>` type annotation and `Box::new(...)` wrapper on line 66.
  - Pass `std::fs::File::open(filename).unwrap()` directly to `analysis::read_log()`.
  - Remove any `analysis::MyReader` usage/import.
  - **Acceptance criteria:**
    - Zero occurrences of `MyReader` in `src/main.rs`.
    - Zero occurrences of `Box<dyn` in `src/main.rs`.
    - `File` is passed directly to `read_log()` without boxing.
    - `cargo build` succeeds.

- [x] **4.6 Final verification**
  - Run `cargo test` -- all tests pass (no test cases deleted).
  - Run `cargo run -- example.log` -- output is identical to pre-refactoring.
  - Verify zero `MyReader` occurrences in `src/`.
  - Verify zero `Box<dyn` occurrences in `src/lib.rs`.
  - Verify `struct LogIterator<R: Read>` is present.
  - Verify `read_log(input: impl Read, ...)` signature is present.
  - **Acceptance criteria:**
    - All verification checklist items from the plan pass.
    - No behavior changes; same input produces same output.

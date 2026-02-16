# Tasklist: Phase 2 -- Remove `Rc<RefCell>`

**Ticket:** Phase 2: Remove `Rc<RefCell>`
**PRD:** `docs/prd/phase-2.prd.md`
**Plan:** `docs/plan/phase-2.md`
**Status:** IMPLEMENT_STEP_OK

---

## Context

`LogIterator` in `src/lib.rs` wraps its reader in `Rc<RefCell<Box<dyn MyReader>>>`, but it is the sole owner. The shared ownership (`Rc`) and interior mutability (`RefCell`) are unnecessary. This phase removes them, giving `LogIterator` direct ownership of the reader. The change propagates to `read_log()`, tests, and `main.rs`.

---

## Tasks

- [x] **2.1 Remove `RefMutWrapper` struct and its `Read` impl**
  Remove the `RefMutWrapper` struct definition and its `impl Read` block from `src/lib.rs` (lines 12-22). This adapter exists solely to satisfy `BufReader`'s `T: Read` requirement when borrowing through `RefCell` and becomes dead code once `Rc<RefCell>` is removed.
  - **AC1:** `RefMutWrapper` struct and its `impl Read` block no longer exist in `src/lib.rs`.
  - **AC2:** `grep -r "RefMutWrapper" src/` returns no hits.

- [x] **2.2 Rewrite `LogIterator` struct to use direct ownership**
  Remove the `reader_rc` field from `LogIterator`. Change the `lines` type parameter from `BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>` to `BufReader<Box<dyn MyReader>>`. The struct becomes a single-field wrapper around the iterator chain.
  - **AC1:** `LogIterator` has a single field `lines` with type `Filter<Lines<BufReader<Box<dyn MyReader>>>, ...>`.
  - **AC2:** No `Rc` or `RefCell` types appear in the `LogIterator` struct definition.

- [x] **2.3 Rewrite `LogIterator::new()` to accept owned reader**
  Change the parameter from `Rc<RefCell<Box<dyn MyReader>>>` to `Box<dyn MyReader>`. Pass the owned `Box` directly into `BufReader::with_capacity()`. Remove the `borrow_mut()` call, the `unsafe { transmute }` block, the `RefMutWrapper` wrapping, the `reader_rc` field initialization, and all associated Russian comments about the borrow motivation (lines 42-48).
  - **AC1:** `LogIterator::new()` accepts `Box<dyn MyReader>` and passes it directly to `BufReader::with_capacity()`.
  - **AC2:** No `unsafe`, `transmute`, `borrow_mut`, or `RefMutWrapper` calls remain in `LogIterator::new()`.

- [x] **2.4 Update `read_log()` signature and remove hint comment**
  Change the `input` parameter type from `Rc<RefCell<Box<dyn MyReader>>>` to `Box<dyn MyReader>`. Remove the hint comment `// подсказка: RefCell вообще не нужен`. The body remains unchanged (it already passes `input` to `LogIterator::new()`).
  - **AC1:** `read_log()` signature uses `input: Box<dyn MyReader>`.
  - **AC2:** The hint comment `// подсказка: RefCell вообще не нужен` is removed.

- [x] **2.5 Adapt `test_all` test to pass reader directly**
  Remove `Rc::new(RefCell::new(...))` wrapping and `.clone()` calls in `test_all`. Change variable types to `Box<dyn MyReader>` and pass them directly to `read_log()`. No test assertions are deleted or changed.
  - **AC1:** No `Rc`, `RefCell`, or `.clone()` calls remain in `test_all`.
  - **AC2:** All existing test assertions are preserved unchanged.

- [x] **2.6 Adapt `main.rs` to pass reader directly**
  Remove `Rc::new(RefCell::new(...))` wrapping and `.clone()` in `main.rs`. Change the variable type to `Box<dyn analysis::MyReader>` and pass it directly to `analysis::read_log()`.
  - **AC1:** No `Rc`, `RefCell`, or `.clone()` calls remain in `main.rs` around the reader/`read_log()` call.
  - **AC2:** `main.rs` compiles and runs correctly with `cargo run -- example.log`.

- [x] **2.7 Verify: all tests pass and CLI output is unchanged**
  Run `cargo test` and `cargo run -- example.log`. Confirm no `Rc`, `RefCell`, `RefMutWrapper`, or `transmute` references remain in `src/`.
  - **AC1:** `cargo test` passes with all existing tests (no test deletions).
  - **AC2:** `cargo run -- example.log` produces output identical to pre-refactoring.
  - **AC3:** `grep -r "Rc\|RefCell" src/lib.rs` returns no hits.
  - **AC4:** `grep -r "RefMutWrapper" src/` returns no hits.
  - **AC5:** `grep -r "transmute" src/` returns no hits.

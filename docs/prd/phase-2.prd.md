# PRD: Phase 2 — Remove `Rc<RefCell>`

**Status:** PRD_READY
**Ticket:** Phase 2: Remove `Rc<RefCell>`
**Phase:** 2 of 12 (see `docs/tasklist.md`)
**Dependencies:** None (independent phase)
**Blocked by:** Nothing
**Blocks:** Phase 3 (Remove `unsafe` transmute), Phase 4 (Generic `R: Read` instead of trait object)

---

## Context / Idea

The `LogIterator` in `src/lib.rs` currently wraps its reader in `Rc<RefCell<Box<dyn MyReader>>>`. This wrapping is unnecessary: `LogIterator` is the sole owner of the reader, so shared ownership (`Rc`) and interior mutability (`RefCell`) serve no purpose. The original author acknowledged this with two inline hints:

- `src/lib.rs:42` -- `// подсказка: unsafe избыточен, да и весь rc - тоже` ("hint: unsafe is redundant, and the entire Rc is too")
- `src/lib.rs:74` -- `// подсказка: RefCell вообще не нужен` ("hint: RefCell is not needed at all")

The `Rc<RefCell>` wrapping is the root cause of several downstream problems: it forces the existence of `RefMutWrapper` (a newtype around `RefMut` to satisfy `Read`), it necessitates an `unsafe { transmute }` to extend the lifetime of the borrow, and it requires storing both the `Rc` and the `lines` iterator in the same struct (with careful field ordering to avoid use-after-free). Removing `Rc<RefCell>` eliminates all of these issues and is a prerequisite for Phase 3 (removing `unsafe`) and Phase 4 (introducing a generic `R: Read`).

### Authoritative specification from `docs/phase/phase-2.md`

**Goal:** Remove the unnecessary `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`, giving it direct ownership of the reader.

**Tasks:**

- [ ] 2.1 Remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`
- [ ] 2.2 Give `LogIterator` direct ownership of the reader
- [ ] 2.3 Remove `RefMutWrapper` and `MyReader` trait if they become unused
- [ ] 2.4 Adapt `test_all` in `lib.rs`: remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping, pass reader directly to `read_log()`

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Implementation Notes:**

- **Hint:** `src/lib.rs:71` -- `// подсказка: RefCell вообще не нужен`
- **Hint:** `src/lib.rs:40` -- `// подсказка: unsafe избыточен, да и весь rc - тоже`

---

## Goals

1. **Simplify ownership model.** `LogIterator` should directly own its `Box<dyn MyReader>` without `Rc<RefCell>` indirection.
2. **Remove unnecessary abstractions.** Eliminate `RefMutWrapper` and `MyReader` trait if they become unused after the change.
3. **Unblock downstream phases.** Enable Phase 3 (remove `unsafe` transmute) and Phase 4 (generic `R: Read`).
4. **Preserve behavior.** Same input produces same output. All existing tests pass; no tests are deleted.

---

## User Stories

1. **As a developer working on Phase 3**, I need `Rc<RefCell>` removed so that the `unsafe { transmute }` in `LogIterator::new` can be replaced with safe code.
2. **As a developer working on Phase 4**, I need `Rc<RefCell>` removed so that `LogIterator` can be parameterized with a generic `R: Read` instead of a trait object.
3. **As a maintainer**, I want the code to use direct ownership instead of `Rc<RefCell>` so the ownership model is obvious and there is no unnecessary runtime overhead.

---

## Scenarios

### Scenario 1: Library usage (`read_log()`)

**Before:**
```rust
let refcell: Rc<RefCell<Box<dyn MyReader>>> =
    Rc::new(RefCell::new(Box::new(source.as_bytes())));
let logs = read_log(refcell.clone(), READ_MODE_ALL, vec![]);
```

**After:**
```rust
let reader: Box<dyn MyReader> = Box::new(source.as_bytes());
let logs = read_log(reader, READ_MODE_ALL, vec![]);
```

The signature of `read_log()` changes from accepting `Rc<RefCell<Box<dyn MyReader>>>` to accepting `Box<dyn MyReader>` (or equivalent owned reader).

### Scenario 2: CLI usage (`main.rs`)

**Before:**
```rust
let file: Rc<RefCell<Box<dyn MyReader>>> =
    Rc::new(RefCell::new(Box::new(std::fs::File::open(filename).unwrap())));
let logs = analysis::read_log(file.clone(), READ_MODE_ALL, vec![]);
```

**After:**
```rust
let file: Box<dyn analysis::MyReader> =
    Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, READ_MODE_ALL, vec![]);
```

### Scenario 3: Test adaptation (`test_all` in `lib.rs`)

All `Rc::new(RefCell::new(...))` wrapping in `test_all` is removed. The test passes reader values directly to `read_log()`. No test cases are deleted; only type signatures are adapted.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| `unsafe` blocks | Remains in this phase (removed in Phase 3) |
| `Rc` / `RefCell` imports | Zero in `src/lib.rs` after this phase |
| `RefMutWrapper` struct | Removed if unused |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** Same input must produce same output.
3. **No test deletions.** Existing tests are adapted, not deleted.
4. **One issue category per commit.** This phase is a single commit.
5. **Compiler-driven.** Change the `LogIterator` field and `read_log()` signature first, then follow compiler errors.
6. **Scope boundary.** This phase removes `Rc<RefCell>` only. The `unsafe { transmute }` may still be present (addressed in Phase 3). The `Box<dyn MyReader>` trait object may still be present (addressed in Phase 4). The `MyReader` trait should only be removed if it becomes truly unused after this phase's changes.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `LogIterator` self-referential struct issue: once `Rc<RefCell>` is removed, the struct may still need to hold both the owned reader and the `BufReader`/`Lines` iterator that borrows from it | Medium | High | The `BufReader` takes ownership of its inner reader via `Read`, so passing the owned `Box<dyn MyReader>` directly into `BufReader` avoids self-referential borrowing entirely. The `unsafe { transmute }` becomes unnecessary. |
| `RefMutWrapper` or `MyReader` trait is used elsewhere | Low | Low | Grep the codebase for all usages before removing. `MyReader` is currently `pub` and used in `main.rs`. |
| `main.rs` breaks due to changed `read_log()` signature | Certain | Low | Update `main.rs` call site as part of task 2.4. This is a known, mechanical change. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. The phase has no external dependencies and is unblocked.

---

## Files Affected

| File | Changes |
|---|---|
| `src/lib.rs` | Remove `RefMutWrapper` struct and its `Read` impl. Remove `Rc<RefCell>` from `LogIterator` fields. Simplify `LogIterator::new()` to take owned reader directly. Change `read_log()` signature. Adapt `test_all`. |
| `src/main.rs` | Remove `Rc<RefCell>` wrapping when calling `read_log()`. Pass owned `Box<dyn MyReader>` directly. |

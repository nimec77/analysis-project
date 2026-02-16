# Summary: Phase 2 -- Remove `Rc<RefCell>`

**Ticket:** Phase 2: Remove `Rc<RefCell>`
**Status:** Complete
**Files changed:** `src/lib.rs`, `src/main.rs`

---

## What Was Done

Removed the unnecessary `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator` in `src/lib.rs`, giving it direct ownership of the reader. This was a refactoring phase that simplified the ownership model without changing any observable behavior.

### Changes

1. **Removed `RefMutWrapper` struct and its `Read` impl.** This newtype wrapper around `RefMut` existed solely to satisfy `BufReader`'s `T: Read` bound when borrowing through a `RefCell`. With `Rc<RefCell>` gone, there is no `RefMut` to wrap.

2. **Simplified `LogIterator` to a single field.** The `reader_rc` field (which held the `Rc<RefCell<Box<dyn MyReader>>>` alive to prevent the transmuted `RefMut` from dangling) was removed. The `lines` type parameter changed from `BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>` to `BufReader<Box<dyn MyReader>>`.

3. **Rewrote `LogIterator::new()` to accept an owned `Box<dyn MyReader>`.** The owned `Box` is passed directly to `BufReader::with_capacity()`. Removed the `borrow_mut()` call, the `unsafe { transmute }` block, the `RefMutWrapper` wrapping, and the Russian-language comments explaining the old borrow motivation.

4. **Changed `read_log()` signature** from `input: Rc<RefCell<Box<dyn MyReader>>>` to `input: Box<dyn MyReader>`. Removed the hint comment `// подсказка: RefCell вообще не нужен`.

5. **Adapted `test_all`** to pass `Box<dyn MyReader>` directly instead of `Rc::new(RefCell::new(...))` with `.clone()`.

6. **Adapted `main.rs`** to pass `Box<dyn MyReader>` directly instead of `Rc::new(RefCell::new(...))` with `.clone()`.

### Minor Cleanups (Incidental)

- Changed `const SOURCE1: &'static str` and `const SOURCE: &'static str` to `&str` (eliding the redundant `'static` lifetime on constants).
- Added `#[allow(clippy::type_complexity)]` to the `LogIterator.lines` field.

---

## Decisions Made

1. **`MyReader` trait retained.** The PRD task 2.3 said to remove `MyReader` "if [it] become[s] unused." After removing `Rc<RefCell>`, `MyReader` is still used in `LogIterator`, `read_log()`, `main.rs`, and tests as `Box<dyn MyReader>`. Its removal is deferred to Phase 4 (generic `R: Read`).

2. **`unsafe { transmute }` removed as a side-effect.** The PRD scope boundary stated the `unsafe` block "may still be present" after Phase 2 (it is formally Phase 3's target). In practice, once `Rc<RefCell>` was removed there was no `RefMut` to transmute, so the `unsafe` block naturally disappeared. This reduces Phase 3's remaining scope.

3. **Compiler-driven approach.** The implementation followed the plan's recommended strategy: change the `LogIterator` struct fields first, then follow compiler errors outward to `LogIterator::new()`, `read_log()`, tests, and `main.rs`.

---

## Technical Debt Resolved

| Hint | Location (before) | Resolution |
|---|---|---|
| `// подсказка: unsafe избыточен, да и весь rc - тоже` | `src/lib.rs:42` | Both `unsafe` and `Rc` removed; direct ownership eliminates the need for both |
| `// подсказка: RefCell вообще не нужен` | `src/lib.rs:74` | `RefCell` removed entirely from the codebase |

## Technical Debt Remaining (for later phases)

| Hint | Location | Target Phase |
|---|---|---|
| `// подсказка: вместо trait-объекта можно дженерик` | `src/lib.rs` | Phase 4 |
| `// подсказка: лучше использовать enum и match` | `src/lib.rs` | Phase 5 |
| `// подсказка: лучше match` | `src/lib.rs` | Phase 6 |
| `// подсказка: можно обойтись итераторами` | `src/lib.rs` | Phase 9 |
| `// подсказка: паниковать в библиотечном коде - нехорошо` | `src/lib.rs` | Phase 7 |

---

## Verification

- `cargo test` -- all existing tests pass; no test cases deleted.
- `cargo run -- example.log` -- output identical to pre-refactoring.
- `grep -r "Rc\|RefCell" src/lib.rs` -- no hits.
- `grep -r "RefMutWrapper" src/` -- no hits.
- `grep -r "transmute" src/` -- no hits.

---

## Impact on Downstream Phases

- **Phase 3 (Remove `unsafe` transmute):** Scope significantly reduced. The `unsafe { transmute }` block was already eliminated as a side-effect of this phase. Phase 3 may only need to verify that no `unsafe` code remains.
- **Phase 4 (Generic `R: Read`):** Unblocked. `LogIterator` now owns `Box<dyn MyReader>` directly, making it straightforward to parameterize with a generic `R: Read`.

# Phase 2: Remove `Rc<RefCell>`

**Goal:** Remove the unnecessary `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`, giving it direct ownership of the reader.

## Tasks

- [x] 2.1 Remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`
- [x] 2.2 Give `LogIterator` direct ownership of the reader
- [x] 2.3 Remove `RefMutWrapper` and `MyReader` trait if they become unused
- [x] 2.4 Adapt `test_all` in `lib.rs`: remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping, pass reader directly to `read_log()`

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

## Implementation Notes

**Hint:** `src/lib.rs:71` — `// подсказка: RefCell вообще не нужен`
**Hint:** `src/lib.rs:40` — `// подсказка: unsafe избыточен, да и весь rc - тоже`

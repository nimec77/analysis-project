# Changelog

All notable changes to this project will be documented in this file.

---

## Phase 2: Remove `Rc<RefCell>`

**Scope:** `src/lib.rs`, `src/main.rs`

Removed the unnecessary `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`, giving it direct ownership of the reader via `Box<dyn MyReader>`. This eliminated the `RefMutWrapper` adapter struct, the `unsafe { transmute }` lifetime extension, and the self-referential struct pattern. The `read_log()` API now accepts `Box<dyn MyReader>` instead of `Rc<RefCell<Box<dyn MyReader>>>`. No behavior changes; all existing tests pass unchanged.

## Phase 1: `String` -> `&str` in `Parser` trait

**Scope:** `src/parse.rs`, `src/lib.rs`, `src/main.rs`

Migrated the `Parser` trait and all combinator implementations from operating on owned `String` values to borrowed `&str` slices with lifetimes. This reduces unnecessary heap allocations throughout the parsing pipeline. Updated all 15 tests in `parse.rs` and adapted call sites in `lib.rs` and `main.rs`.

# Phase 4: Generic `R: Read` instead of trait object

**Goal:** Make `LogIterator` generic over `R: Read` instead of using a trait object, removing the `Box<dyn MyReader>` / `MyReader` trait indirection.

## Tasks

- [x] 4.1 Make `LogIterator` generic: `LogIterator<R: Read>`
- [x] 4.2 Remove `Box<dyn MyReader>` / `MyReader` trait
- [x] 4.3 Update `read_log()` signature to accept `impl Read`
- [x] 4.4 Adapt `main.rs` to the new signature

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 2 complete

## Implementation Notes

**Hint:** `src/lib.rs:30` — `// подсказка: вместо trait-объекта можно дженерик`

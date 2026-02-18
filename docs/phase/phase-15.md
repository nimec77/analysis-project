# Phase 15: Modularity (split `parse.rs`)

**Goal:** Split the monolithic `src/parse.rs` into a module directory with separate files for combinators, domain types, and log hierarchy.

## Tasks

- [x] 15.1 Create `src/parse/` directory
- [x] 15.2 Move combinator framework (traits + structs) to `src/parse/combinators.rs`
- [x] 15.3 Move domain types (AuthData, AssetDsc, Backet, etc.) to `src/parse/domain.rs`
- [x] 15.4 Move log hierarchy (LogLine, LogKind, etc.) to `src/parse/log.rs`
- [x] 15.5 Convert `src/parse.rs` to module root: `mod combinators; mod domain; mod log;` with `pub use` re-exports
- [x] 15.6 Move `primitives` (ex-`stdp`) as private sub-module within `combinators.rs`
- [x] 15.7 Refine visibility: constructor functions to `pub(crate)`
- [x] 15.8 Move tests to `#[cfg(test)] mod tests` in each sub-module

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 14 complete

## Implementation Notes

Uses edition 2024 module paths (NO `mod.rs`).

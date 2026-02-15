# Phase 1: `String` -> `&str` in `Parser` trait

**Goal:** Change the `Parser` trait to operate on `&str` with lifetimes instead of `String`, eliminating unnecessary allocations throughout the parser combinator framework.

## Tasks

- [x] 1.1 Change `Parser` trait to operate on `&str` with lifetimes instead of `String`
- [x] 1.2 Update all combinator implementations (`Tag`, `Alt`, `Map`, `Delimited`, `Preceded`, `Permutation`, `List`, `Take`, etc.)
- [x] 1.3 Update `Parsable` trait and all implementations
- [x] 1.4 Remove now-unnecessary `.clone()` calls on input strings
- [x] 1.5 Adapt all 15 tests in `parse.rs`: remove `.into()` on parser inputs and on `remaining` in `Ok((...))` expectations; keep `.into()` for owned `String` output values (e.g. `Unquote`, `AssetDsc.id`, `Backet.asset_id`)

## Acceptance Criteria

**Verify:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

## Implementation Notes

**Hint:** `src/parse.rs:5` — `// подсказка: здесь можно переделать` (the `Parser` trait definition)

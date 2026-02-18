# Summary: PH-15 -- Modularity (split `parse.rs`)

**Ticket:** PH-15 "Phase 15: Modularity (split parse.rs)"
**Status:** Complete
**Files changed:** `src/parse.rs`, `src/parse/combinators.rs` (new), `src/parse/domain.rs` (new), `src/parse/log.rs` (new)

---

## What Was Done

Phase 15 split the monolithic `src/parse.rs` (1762 lines) into a module directory with three sub-module files and a thin module root, using Rust edition 2024 module path conventions (no `mod.rs` files). This is a pure structural refactor -- no behavior changes, no new dependencies, no test deletions.

### Changes

1. **Created `src/parse/combinators.rs` (842 lines).** Moved the `Parser` and `Parsable` trait definitions, the `primitives` sub-module (containing `U32`, `I32`, `Byte`), helper functions (`quote`, `unquote_escaped`, `unquote_simple`), and all 13 combinator structs (`Tag`, `Unquote`, `QuotedTag`, `StripWhitespace`, `Delimited`, `Map`, `Preceded`, `Tuple`, `KeyValue`, `Permutation`, `List`, `Alt`, `Take`) with their `Parser` trait impls and constructor functions. Added a `#[cfg(test)] mod tests` block with 11 combinator tests (`test_u32`, `test_i32`, `test_quote`, `test_unquote_simple`, `test_unquote`, `test_tag`, `test_quoted_tag`, `test_strip_whitespace`, `test_delimited`, `test_key_value`, `test_list`).

2. **Created `src/parse/domain.rs` (396 lines).** Moved the `AUTHDATA_SIZE` constant, 7 domain struct definitions (`AuthData`, `AssetDsc`, `Backet`, `UserCash`, `UserBacket`, `UserBackets`, `Announcements`) with their `Parsable` impls, and the generic `just_parse` function. Added `use super::combinators::*;` and `use super::combinators::primitives;` imports for access to combinator types and primitives. Added a `#[cfg(test)] mod tests` block with 10 domain tests (`test_authdata`, `test_asset_dsc`, `test_backet`, `test_just_parse_asset_dsc`, `test_just_parse_backet`, `test_just_parse_user_cash`, `test_just_parse_user_backet`, `test_just_parse_user_backets`, `test_just_parse_announcements`, `test_just_parse_error_cases`).

3. **Created `src/parse/log.rs` (557 lines).** Moved 9 enum/struct definitions (`LogKind`, `SystemLogKind`, `SystemLogTraceKind`, `SystemLogErrorKind`, `AppLogKind`, `AppLogErrorKind`, `AppLogTraceKind`, `AppLogJournalKind`, `LogLine`) with their `Parsable` impls. Added `use super::combinators::*;`, `use super::combinators::primitives;`, and `use super::domain::*;` imports for access to combinators, primitives, and domain types. Added a `#[cfg(test)] mod tests` block with 2 log tests (`test_log_kind`, `test_withdraw_cash`).

4. **Rewrote `src/parse.rs` as module root (7 lines).** Replaced the entire 1762-line monolithic file with `mod combinators; mod domain; mod log;` declarations plus `pub use combinators::*; pub use domain::*; pub use log::*;` re-exports, ensuring `use parse::*;` in `lib.rs` continues to resolve all public types.

5. **Adjusted visibility: constructor functions to `pub(crate)`, primitives to `pub(crate) mod`.** Changed `mod primitives` to `pub(crate) mod primitives` so that `domain.rs` and `log.rs` can access `primitives::U32`, `primitives::Byte`. Changed all 17 constructor functions from `fn` to `pub(crate) fn`: `unquote`, `tag`, `quoted_tag`, `strip_whitespace`, `delimited`, `map`, `preceded`, `tuple2`, `key_value`, `permutation2`, `permutation3`, `list`, `alt2`, `alt3`, `alt4`, `alt8`, `take`.

6. **Redistributed tests into per-sub-module test blocks.** Split the monolithic `#[cfg(test)] mod test` block into three `#[cfg(test)] mod tests` blocks -- one in each sub-module. The `fn nz(n: u32) -> NonZeroU32` helper is duplicated in each test block for self-containment. All 23 parse tests preserved; no test cases deleted.

---

## Decisions Made

1. **Single commit for the entire module split.** Per project convention ("one issue category = one commit"), the entire phase is a single modularity issue category.

2. **Edition 2024 module paths (no `mod.rs`).** The project uses Rust edition 2024, which natively supports having both `src/parse.rs` (module root) and `src/parse/` (sub-module directory) without requiring `mod.rs` files. Sub-modules are declared as `src/parse/combinators.rs`, `src/parse/domain.rs`, `src/parse/log.rs`.

3. **Wildcard re-exports (`pub use *`) in module root.** Using `pub use combinators::*; pub use domain::*; pub use log::*;` in the module root ensures that all public items are transparently re-exported. This keeps `src/lib.rs` unchanged (`pub mod parse; use parse::*;` continues to work).

4. **All constructors uniformly `pub(crate)`.** Even constructors not currently called from other sub-modules (e.g., `quoted_tag` is only used within `combinators.rs`) were made `pub(crate)` for consistency, following the plan's recommendation.

5. **`primitives` kept as inline sub-module within `combinators.rs`.** The `primitives` module (`U32`, `I32`, `Byte`) was kept as an inline `pub(crate) mod primitives { ... }` within `combinators.rs` rather than being extracted to a separate file. This keeps the module hierarchy shallow.

6. **Dependency graph strictly layered.** The sub-modules follow a strict dependency chain: `combinators` (standalone) <- `domain` (depends on combinators) <- `log` (depends on combinators + domain). No circular dependencies.

7. **Helper `nz()` duplicated rather than shared.** The test helper `fn nz(n: u32) -> NonZeroU32` is duplicated in each sub-module's test block rather than being shared, keeping each test module self-contained. This is a trivial one-liner.

---

## Verification

- `cargo build` -- compiles with zero errors.
- `cargo test` -- all 26 tests pass (23 parse + 3 lib). No test cases deleted.
- `cargo run -- example.log` -- output identical to pre-phase.
- Two pre-existing warnings (`quote` unused, `I32` unconstructed) persist in `combinators.rs`; no new warnings introduced.
- No `src/parse/mod.rs` file exists.
- `src/lib.rs` -- zero modifications.
- `src/main.rs` -- zero modifications.
- Module root (`src/parse.rs`) is 7 lines.
- `src/parse/combinators.rs` is 842 lines (target: ~800-900).
- `src/parse/domain.rs` is 396 lines (target: ~350-400).
- `src/parse/log.rs` is 557 lines (target: ~550-600).

---

## Impact on Downstream Phases

Phase 15 is a prerequisite for several subsequent phases that benefit from the smaller, focused sub-module files:

- **Phase 16 (Newtype pattern -- `UserId`, `AssetId`):** New newtypes can be added directly in `domain.rs` alongside the existing domain types, without scrolling through combinator infrastructure or log hierarchy code.
- **Phase 20 (Display trait for log types):** `Display` implementations can be added per-sub-module, keeping log formatting logic in `log.rs` and domain formatting in `domain.rs`.
- **Phase 22 (Parser fluent API):** Fluent method extensions to the `Parser` trait can be developed in `combinators.rs` in isolation.
- **General:** All future changes touch smaller, focused files (~400-850 lines each instead of 1762 lines), reducing cognitive load and merge conflict surface area.

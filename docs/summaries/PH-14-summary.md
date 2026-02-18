# Summary: PH-14 -- Naming Improvements

**Ticket:** PH-14 "Phase 14: Naming improvements"
**Status:** Complete
**Files changed:** `src/parse.rs`

---

## What Was Done

Phase 14 is a pure renaming refactor within `src/parse.rs`. Five identifiers were renamed to improve naming consistency and align with established conventions (e.g., nom combinator naming). No behavior changes, no new dependencies, and no files other than `src/parse.rs` were modified. All changes were performed as mechanical find-and-replace operations within a single file.

### Changes

1. **Renamed `do_unquote()` to `unquote_escaped()` (task 14.4).** The function definition at line 93, the call site in `Unquote::parse()` at line 129, and the doc comment at the definition site were updated. The `do_` prefix was a naming anti-pattern that added no semantic value. The new name `unquote_escaped()` clearly communicates that this function handles backslash escape sequences (e.g., `\"` and `\\`) and returns an owned `String`.

2. **Renamed `do_unquote_non_escaped()` to `unquote_simple()` (task 14.5).** The function definition, the call site in `QuotedTag::parse()`, three test assertions, and the test function name (`test_do_unquote_non_escaped` to `test_unquote_simple`) were updated. The intra-doc link in the doc comment was updated from `[do_unquote]` to `[unquote_escaped]`. The new name `unquote_simple()` clearly communicates the simpler behavior (no escape handling, returns a borrowed `&str`) compared to `unquote_escaped()`.

3. **Renamed `All` struct to `Tuple` (task 14.1).** The struct definition, all `Parser` trait implementations (for 2-tuple, 3-tuple, and 4-tuple arities), all type annotations in `Parsable` return types (for `KeyValue`, `AssetDsc`, `Backet`, `UserCash`, `UserBacket`, `UserBackets`, and `LogLine`), and doc comments were updated. The doc comment was changed from referencing ``all`` to referencing ``tuple`` as the nom analog. The new name `Tuple` aligns with nom's naming convention where `tuple()` is the combinator for sequential parsing that returns a tuple of results.

4. **Renamed `all2()` to `tuple2()` (task 14.2).** The function definition and all 8 call sites were updated. The doc comment was updated to reference `[Tuple]` instead of `[All]`. The new name `tuple2()` follows the same arity-suffix pattern already used by `alt2()`..`alt8()` and `permutation2()`..`permutation3()`.

5. **Renamed `stdp` module to `primitives` (task 14.3).** The module declaration and all 38 qualified references (`stdp::U32`, `stdp::I32`, `stdp::Byte`) were updated to use `primitives::U32`, `primitives::I32`, `primitives::Byte`. The abbreviation `stdp` did not communicate the module's purpose. The new name `primitives` makes the module's role (parsers for primitive/standard types) self-evident.

---

## Decisions Made

1. **Single commit for all renames.** Per project convention ("one issue category = one commit"), all five renames plus associated doc comment updates constitute a single naming-improvement issue category and go into one commit.

2. **Rename order: unquote functions first, then struct/constructor, then module.** The unquote functions were renamed first (fewest occurrences, localized changes), then the `All`/`all2` struct and constructor (14 + 9 occurrences), then the `stdp` module last (39 occurrences, highest count). This order minimized line-shift interference between edits.

3. **Doc comment text updated only where identifier names appear.** Existing Russian comment text was preserved as-is. Only identifier names within comments (e.g., `` `all` `` to `` `tuple` ``, `[All]` to `[Tuple]`, `[do_unquote]` to `[unquote_escaped]`) were updated to reflect the new names.

4. **Items explicitly not renamed.** Per the phase specification, the following were intentionally left unchanged: `A0`/`A1`/`A2` type parameters (standard tuple-impl pattern), `nz()` test helper, `AssetDsc.dsc` field (matches domain key), arity suffixes on other combinators (`alt2`, `permutation3`).

5. **Only `src/parse.rs` modified.** No changes were needed in `src/lib.rs` or `src/main.rs`. Although `Tuple` (formerly `All`) is declared `pub`, it is only referenced within `parse.rs`. The `use parse::*;` import in `lib.rs` transparently picks up the renamed types.

---

## Verification

- `cargo build` -- compiles without errors.
- `cargo test` -- all 26 tests pass. No test cases deleted.
- `cargo run -- example.log` -- output identical to pre-phase.
- Zero occurrences of `All<` in `src/parse.rs` (all renamed to `Tuple<`).
- Zero occurrences of `all2` in `src/parse.rs` (all renamed to `tuple2`).
- Zero occurrences of `stdp` in `src/parse.rs` (all renamed to `primitives`).
- Zero occurrences of `do_unquote` in `src/parse.rs` (renamed to `unquote_escaped`).
- Zero occurrences of `do_unquote_non_escaped` in `src/parse.rs` (renamed to `unquote_simple`).
- No new compiler warnings.
- No external dependencies added.

---

## Impact on Downstream Phases

Phase 14 prepares the codebase for phase 15 (module split of `parse.rs` into sub-modules). Specifically:

- **Phase 15 (Modularity -- split `parse.rs`):** The renamed identifiers (`Tuple`, `tuple2`, `primitives`, `unquote_escaped`, `unquote_simple`) are now in place before the code is distributed across sub-modules. Cleaner names before splitting yields a cleaner result.
- **Phase 16 (Newtype pattern):** The `primitives` module name is a better fit for future newtypes (`UserId`, `AssetId`) that may be added alongside the existing primitive parsers.
- **General:** Consistent naming across the parser module reduces cognitive load for all subsequent refactoring and feature work.

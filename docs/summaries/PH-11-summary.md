# Summary: PH-11 -- `NonZeroU32` Tight Type

**Ticket:** PH-11 "Phase 11: `NonZeroU32` tight type"
**Status:** Complete
**Files changed:** `src/parse.rs`, `src/lib.rs`

---

## What Was Done

Replaced the `u32` type used for `request_id` with `std::num::NonZeroU32` throughout the codebase, encoding the "request IDs are never zero" invariant directly in the type system rather than relying on a runtime `if value == 0 { return Err(()); }` check. The change propagated through the `stdp::U32` parser, the `LogLine` struct, the `Parsable` impl for `LogLine`, the `read_log()` function signature, and all test call sites. The three-line hint comment block identifying this technical debt was removed along with the runtime zero check.

### Changes

1. **Changed `stdp::U32` parser's `type Dest` from `u32` to `NonZeroU32`.** Added `use std::num::NonZeroU32;` inside the `mod stdp` block. The parser now returns `NonZeroU32` instead of `u32`, with zero-rejection handled by `NonZeroU32::new(value).ok_or(())?` instead of the previous `if value == 0 { return Err(()); }` runtime check.

2. **Removed hint comment block and runtime zero check.** The three-line hint comment (`// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32` / `// (ограничиться NonZeroU32::new(value).ok_or(()).get() - норм)` / `// или даже заиспользовать tightness`) and the `if value == 0 { return Err(()); }` check with its inline comment were removed and replaced by a single `let non_zero = NonZeroU32::new(value).ok_or(())?;` call.

3. **Updated `LogLine::request_id` field type from `u32` to `std::num::NonZeroU32`.** The struct field at line 1376 now carries the `NonZeroU32` type, making the "never zero" invariant visible in the public API.

4. **Updated the `Parsable` impl for `LogLine`.** The associated `Parser` type's function pointer changed from `fn((LogKind, u32)) -> Self` to `fn((LogKind, std::num::NonZeroU32)) -> Self`. The parser closure `|(kind, request_id)| LogLine { kind, request_id }` required no change because the `NonZeroU32` value flows through automatically.

5. **Updated `read_log()` signature in `src/lib.rs`.** Added `use std::num::NonZeroU32;` at the crate level. Changed the `request_ids` parameter from `Vec<u32>` to `Vec<NonZeroU32>`. The filtering logic `request_ids.contains(&log.request_id)` required no change because `NonZeroU32` implements `PartialEq`.

6. **Updated all test call sites.** In `src/parse.rs`, the `test_u32` function was updated with a `use std::num::NonZeroU32;` import and a helper function `fn nz(n: u32) -> NonZeroU32`, and all four `Ok(...)` assertions were updated to wrap expected values in `NonZeroU32::new(...).unwrap()`. Error test cases (`""`, `"-3"`, `"0x"`) remained unchanged. In `src/lib.rs`, `test_errors_mode` and `test_exchanges_mode` were updated to construct `NonZeroU32` values: `vec![NonZeroU32::new(1).unwrap()]` for single-ID vectors and `(1..=10).map(|n| NonZeroU32::new(n).unwrap()).collect()` for multi-ID vectors. `test_all` and `main.rs` required no changes because empty `vec![]` type-infers correctly from the updated function signature.

---

## Decisions Made

1. **`NonZeroU32::new(value).ok_or(())?` as the replacement pattern.** This is the idiomatic Rust pattern for converting a `u32` to `NonZeroU32` with fallible rejection of zero, matching the hint left by the original author. The `ok_or(())` converts `None` (zero value) to the existing `Err(())` error type used throughout the parser framework.

2. **Test helper function `nz()` introduced in `test_u32`.** A small helper `fn nz(n: u32) -> NonZeroU32 { NonZeroU32::new(n).unwrap() }` was added inside the test module to reduce verbosity when constructing `NonZeroU32` values in assertions, following the risk mitigation suggested in the PRD.

3. **No changes to `src/main.rs`.** The empty `vec![]` passed to `read_log()` in `main.rs` type-infers `Vec<NonZeroU32>` from the updated function signature, so no modification was needed.

4. **No changes to `stdp::I32` parser.** Per the PRD's scope boundary constraint, only the `stdp::U32` parser (used for `request_id`) was changed. The `I32` parser remains `i32` as it is used for different fields.

5. **`std::num::NonZeroU32` used as a fully-qualified path in struct fields.** The `LogLine::request_id` field and the `Parsable` impl use `std::num::NonZeroU32` as the fully-qualified type (rather than importing `NonZeroU32` at the module level in the outer scope), consistent with how other types like `NonZeroU32` are referenced in data model structs elsewhere in the file (e.g., `BuyAsset::count`, `SellAsset::count`).

---

## Technical Debt Resolved

| Hint | Location (before) | Resolution |
|---|---|---|
| `// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32` | `src/parse.rs:37-39` | Removed. The `stdp::U32` parser now returns `NonZeroU32` directly, replacing the runtime `if value == 0` check with `NonZeroU32::new(value).ok_or(())?`. |

## Technical Debt Remaining (for later phases)

| Hint | Location | Target Phase |
|---|---|---|
| `// подсказка: singleton, без которого можно обойтись` | `src/parse.rs:1408` | Phase 12 |

---

## Verification

- `cargo build` -- compiles without errors.
- `cargo test` -- all 25 tests pass; no test cases deleted.
- `cargo run -- example.log` -- output identical to pre-refactoring (because `NonZeroU32`'s `Debug` impl prints the inner `u32` value directly).
- `type Dest = NonZeroU32;` is present in the `stdp::U32` parser impl.
- `NonZeroU32::new(value).ok_or(())?` is present in the `stdp::U32` parser.
- `pub request_id: std::num::NonZeroU32` is present in the `LogLine` struct.
- `fn((LogKind, std::num::NonZeroU32)) -> Self` is present in the `Parsable` impl for `LogLine`.
- `request_ids: Vec<NonZeroU32>` is present in the `read_log()` signature.
- Zero occurrences of `подсказка: вместо if можно использовать tight-тип` in `src/parse.rs`.
- Zero occurrences of `if value == 0` in the `stdp::U32` parser.
- `use std::num::NonZeroU32;` imports are present in both `mod stdp` and `src/lib.rs`.
- Test assertions use `NonZeroU32::new(...).unwrap()` for expected values.
- No changes in `src/main.rs`.
- `stdp::I32` parser is unchanged.
- Zero external dependencies added.

---

## Impact on Downstream Phases

- **Phase 12 (Remove LogLineParser singleton):** Unaffected. The `LOG_LINE_PARSER` singleton and its hint comment are untouched.

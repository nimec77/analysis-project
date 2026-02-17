# Summary: PH-10 -- Box the Large Enum Variant

**Ticket:** PH-10 "Phase 10: Box the large enum variant"
**Status:** Complete
**Files changed:** `src/parse.rs`

---

## What Was Done

Reduced the stack size of the `AppLogTraceKind` enum (and the entire `LogKind` -> `LogLine` hierarchy) by wrapping the oversized `AuthData` payload in `Box<>` at the `Connect` variant. The `AuthData` struct contains a `[u8; 1024]` fixed-size array, which previously inflated every enum in the chain to ~1040 bytes on the stack. After boxing, `Connect(Box<AuthData>)` stores an 8-byte pointer inline and the 1024-byte payload on the heap, reducing `LogLine` from ~1040 bytes to ~40 bytes. The parser closure and test assertion were updated accordingly, and both hint comments identifying this technical debt were removed.

### Changes

1. **Changed `AppLogTraceKind::Connect` variant from `Connect(AuthData)` to `Connect(Box<AuthData>)`.** The variant at line 981 now holds a `Box<AuthData>` instead of an inline `AuthData`, storing an 8-byte pointer on the stack and the 1024-byte array on the heap. This size reduction propagates through the entire enum hierarchy: `AppLogTraceKind`, `AppLogKind`, `LogKind`, and `LogLine` all shrink from ~1040 bytes to ~40 bytes.

2. **Updated the parser closure to wrap `AuthData` in `Box::new()`.** The map closure in the `Parsable` impl for `AppLogTraceKind` at line 1179 was changed from `|authdata| AppLogTraceKind::Connect(authdata)` to `|authdata| AppLogTraceKind::Connect(Box::new(authdata))`. The associated type `fn(AuthData) -> AppLogTraceKind` remains unchanged because the non-capturing closure coerces to the function pointer type.

3. **Updated `test_log_kind` expected value.** The expected value in the `test_log_kind` test at line 1650 was changed from `AppLogTraceKind::Connect(AuthData([...]))` to `AppLogTraceKind::Connect(Box::new(AuthData([...])))`.

4. **Removed two hint comments.** The comment `// подсказка: довольно много места на стэке` ("hint: quite a lot of space on the stack") above the `AuthData` struct and the comment `// подсказка: а поля не слишком много места на стэке занимают?` ("hint: don't the fields take up too much stack space?") above the `AppLogTraceKind` enum were both removed, as the technical debt they identified is now resolved.

---

## Decisions Made

1. **Boxing at the variant level, not the struct level.** The `Box<>` wrapping was applied at the `Connect(Box<AuthData>)` variant rather than changing the `AuthData` struct itself. This is the standard Rust pattern: the struct remains a simple newtype (`pub struct AuthData([u8; AUTHDATA_SIZE])`), and only the enum variant that embeds it pays the indirection cost. Code that works with `AuthData` directly (such as `test_authdata`) is unaffected.

2. **Function pointer coercion preserved.** The closure `|authdata| AppLogTraceKind::Connect(Box::new(authdata))` is a non-capturing closure that coerces to the `fn(AuthData) -> AppLogTraceKind` function pointer type. This means the associated `Parser` type in the `Parsable` impl did not need to change, keeping the type signature simple.

3. **No changes to `src/lib.rs` or `src/main.rs`.** The `read_log()` filtering logic uses `matches!()` macros that check variant shapes without destructuring inner data, so no pattern-matching code needed updating. The boxing is entirely internal to `src/parse.rs`.

4. **No new tests added.** The refactoring is a pure memory layout change. The existing 25 tests (including `test_log_kind` which exercises the `Connect` variant end-to-end, and `test_authdata` which tests `AuthData` parsing directly) provide complete coverage. No additional test coverage was needed.

---

## Technical Debt Resolved

| Hint | Location (before) | Resolution |
|---|---|---|
| `// подсказка: довольно много места на стэке` | `src/parse.rs:722` | Removed. The `AuthData` payload is now heap-allocated via `Box<AuthData>` in the `Connect` variant, eliminating the 1024-byte stack bloat. |
| `// подсказка: а поля не слишком много места на стэке занимают?` | `src/parse.rs:979` | Removed. The `Connect` variant now uses `Box<AuthData>` (8 bytes) instead of inline `AuthData` (1024 bytes). |

## Technical Debt Remaining (for later phases)

| Hint | Location | Target Phase |
|---|---|---|
| `// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32` | `src/parse.rs:37` | Phase 10 (NonZeroU32) |
| `// подсказка: singleton, без которого можно обойтись` | `src/parse.rs:1412` | Phase 11 |

---

## Verification

- `cargo build` -- compiles without errors.
- `cargo test` -- all 25 tests pass; no test cases deleted or modified (only the expected value in `test_log_kind` is updated).
- `cargo run -- example.log` -- output identical to pre-refactoring (because `Box<T>` delegates `Debug` to `T::fmt()`).
- `Connect(Box<AuthData>)` is present in the `AppLogTraceKind` enum definition.
- `Box::new(authdata)` is present in the parser closure.
- `Box::new(AuthData([...]))` is present in the `test_log_kind` test assertion.
- Zero occurrences of `подсказка: довольно много места на стэке` in `src/parse.rs`.
- Zero occurrences of `подсказка: а поля не слишком много места на стэке занимают?` in `src/parse.rs`.
- The `AuthData` struct definition is unchanged: `pub struct AuthData([u8; AUTHDATA_SIZE])`.
- The `test_authdata` test is unchanged.
- No changes in `src/lib.rs` or `src/main.rs`.
- Zero external dependencies added.
- Only `src/parse.rs` was modified.

---

## Impact on Downstream Phases

- **Phase 10 (NonZeroU32):** Unaffected. The `NonZeroU32` hint in `src/parse.rs` is untouched.
- **Phase 11 (Remove LogLineParser singleton):** Unaffected. `LogLineParser` is untouched.

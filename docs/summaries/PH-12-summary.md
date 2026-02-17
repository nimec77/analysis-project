# Summary: PH-12 -- Remove `OnceLock` Singleton

**Ticket:** PH-12 "Phase 12: Remove `OnceLock` singleton"
**Status:** Complete
**Files changed:** `src/parse.rs`, `src/lib.rs`

---

## What Was Done

Removed the `LogLineParser` struct and `LOG_LINE_PARSER` `OnceLock` singleton from `src/parse.rs`, replacing the global static with a locally-owned parser field in `LogIterator`. This eliminates the last piece of hidden global mutable state in the codebase and removes the unnecessary `std::sync::OnceLock` synchronization primitive. The parser is now constructed once per `LogIterator` instance via `LogLine::parser()` and stored as a struct field, making the data flow explicit and the ownership model clear.

### Changes

1. **Deleted the `LogLineParser` struct and `impl` block from `parse.rs`.** The 10-line `LogLineParser` struct definition (wrapping `std::sync::OnceLock<<LogLine as Parsable>::Parser>`) and its `impl` block with the `parse()` method were removed entirely. No replacement code was added in `parse.rs`.

2. **Deleted the `pub static LOG_LINE_PARSER` declaration from `parse.rs`.** The global static singleton (`pub static LOG_LINE_PARSER: LogLineParser = LogLineParser { parser: std::sync::OnceLock::new() };`) was removed. The `OnceLock` lazy initialization pattern is no longer used anywhere in the codebase.

3. **Removed the last remaining hint comments.** The two-line hint block (`// подсказка: singleton, без которого можно обойтись` and `// парсеры не страшно вытащить в pub`) and the doc comment (`/// Единожды собранный парсер логов`) were deleted. These were the last `// подсказка:` hints in the entire codebase, marking the completion of all identified technical debt.

4. **Added a `parser` field to `LogIterator<R>` in `lib.rs`.** The field `parser: <LogLine as Parsable>::Parser` was added to the `LogIterator<R>` struct, giving the iterator direct ownership of the parser instance.

5. **Initialized the `parser` field in `LogIterator::new()`.** The constructor now initializes the field with `parser: LogLine::parser()`, constructing the parser once when the iterator is created and reusing it for every `next()` call.

6. **Replaced `LOG_LINE_PARSER.parse(...)` with `self.parser.parse(...)` in `LogIterator::next()`.** The call site in the `Iterator::next()` implementation was updated from `LOG_LINE_PARSER.parse(line.trim())` to `self.parser.parse(line.trim())`, using the locally-owned parser field instead of the global static.

---

## Decisions Made

1. **Parser stored as a field in `LogIterator`, not constructed inline per `next()` call.** The parser is built once in `LogIterator::new()` and reused across all `next()` calls. While constructing the parser is cheap (all combinators are lightweight stack-only structs after Phase 1), storing it as a field avoids redundant construction and makes the intent clear: one parser per iterator lifetime.

2. **No replacement code added in `parse.rs`.** The `LogLineParser` struct and `LOG_LINE_PARSER` static were deleted without adding any replacement abstraction. External consumers who previously used `LOG_LINE_PARSER.parse(input)` should now call `<LogLine as Parsable>::parser().parse(input)` directly, using the already-public `Parsable` and `Parser` traits (made public in Phase 8).

3. **Associated type syntax used for the field type.** The parser field uses `<LogLine as Parsable>::Parser` as its type, letting the compiler handle the deeply nested combinator type expansion. This is consistent with how all `Parsable` implementations define their associated `Parser` type throughout `parse.rs`.

4. **No changes to `src/main.rs` or test code.** The refactoring is invisible to tests and the CLI binary. Tests call `read_log()` which internally creates `LogIterator`, and the structural change does not affect the public API or behavior.

---

## Technical Debt Resolved

| Hint | Location (before) | Resolution |
|---|---|---|
| `// подсказка: singleton, без которого можно обойтись` | `src/parse.rs:1408` | Removed. The `OnceLock` singleton is deleted; the parser is constructed locally in `LogIterator::new()`. |
| `// парсеры не страшно вытащить в pub` | `src/parse.rs:1409` | Removed. The `Parsable` and `Parser` traits were already made public in Phase 8. The parser is now constructed directly via `LogLine::parser()`. |

## Technical Debt Remaining

**None.** This was the final phase (12 of 12). All `// подсказка:` hint comments have been resolved across the entire codebase.

---

## Verification

- `cargo build` -- compiles without errors.
- `cargo test` -- all 25 tests pass; no test cases deleted.
- `cargo run -- example.log` -- output identical to pre-refactoring.
- Zero occurrences of `LogLineParser` in `src/parse.rs`.
- Zero occurrences of `LOG_LINE_PARSER` in `src/parse.rs` and `src/lib.rs`.
- Zero occurrences of `std::sync::OnceLock` in `src/parse.rs`.
- Zero occurrences of `подсказка` anywhere in the source code.
- `parser: <LogLine as Parsable>::Parser` field is present in `LogIterator`.
- `parser: LogLine::parser()` is present in `LogIterator::new()`.
- `self.parser.parse(line.trim())` is present in `LogIterator::next()`.
- No changes in `src/main.rs`.
- No changes in test code.
- Zero external dependencies added.

---

## Impact on Downstream Phases

None. This is the final phase (12 of 12). All planned refactoring work is complete.

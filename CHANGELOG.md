# Changelog

All notable changes to this project will be documented in this file.

---

## Phase 15: Modularity (split `parse.rs`)

**Scope:** `src/parse.rs`, `src/parse/combinators.rs` (new), `src/parse/domain.rs` (new), `src/parse/log.rs` (new)

Split the monolithic `src/parse.rs` (1762 lines) into a module directory with three sub-module files and a thin module root, using Rust edition 2024 module path conventions (no `mod.rs` files). Created `src/parse/combinators.rs` (842 lines) containing the `Parser` and `Parsable` traits, the `primitives` sub-module, all combinator structs and their constructor functions, and 11 combinator tests. Created `src/parse/domain.rs` (396 lines) containing the 7 domain types (`AuthData`, `AssetDsc`, `Backet`, `UserCash`, `UserBacket`, `UserBackets`, `Announcements`), their `Parsable` impls, the `just_parse` function, and 10 domain tests. Created `src/parse/log.rs` (557 lines) containing the 9 log hierarchy types (`LogLine`, `LogKind`, `SystemLogKind`, `SystemLogTraceKind`, `SystemLogErrorKind`, `AppLogKind`, `AppLogErrorKind`, `AppLogTraceKind`, `AppLogJournalKind`), their `Parsable` impls, and 2 log tests. Rewrote `src/parse.rs` as a 7-line module root with `mod` declarations and `pub use` re-exports. Changed all 17 constructor functions to `pub(crate)` visibility and the `primitives` module to `pub(crate) mod` for cross-sub-module access. Redistributed the monolithic test block into per-sub-module `#[cfg(test)] mod tests` blocks. This is a pure structural refactor -- no behavior changes, no new dependencies, no files other than `src/parse.rs` and the new sub-modules modified. `src/lib.rs` and `src/main.rs` are unchanged. All 26 tests pass; no test cases deleted. `cargo run -- example.log` output unchanged.

## Phase 14: Naming Improvements

**Scope:** `src/parse.rs`

Renamed five identifiers across `src/parse.rs` to improve naming consistency and align with established conventions (e.g., nom combinator naming). Renamed the `All<T>` struct to `Tuple<T>` and its constructor `all2()` to `tuple2()` to match nom's `tuple` combinator naming and the existing arity-suffix pattern (`alt2`, `permutation2`). Renamed the `stdp` module to `primitives` to make its purpose (parsers for primitive/standard types) self-evident. Renamed `do_unquote()` to `unquote_escaped()` and `do_unquote_non_escaped()` to `unquote_simple()`, replacing the `do_` anti-pattern with names that clearly communicate the distinction: one handles escape sequences and allocates, the other does not. Updated all internal references including type annotations in `Parsable` return types, doc comments, intra-doc links, and test code. Renamed the `test_do_unquote_non_escaped` test function to `test_unquote_simple`. This is a pure renaming refactor -- no behavior changes, no new dependencies, no files other than `src/parse.rs` modified. All 26 tests pass; no test cases deleted. `cargo run -- example.log` output unchanged.

## Phase 13: Bug Fix + Dead Code Cleanup

**Scope:** `src/parse.rs`

Fixed a copy-paste bug in the `WithdrawCash` parser arm where the mapping closure incorrectly produced `AppLogJournalKind::DepositCash(user_cash)` instead of `AppLogJournalKind::WithdrawCash(user_cash)`, causing `WithdrawCash` journal entries to be silently misclassified as `DepositCash` in the parsed output. Added a `test_withdraw_cash` regression test that parses a `WithdrawCash` log line and asserts the result is `AppLogJournalKind::WithdrawCash(...)`. Removed five unused code items from the parser module: the `AsIs` struct and its `Parser` impl (9 lines), the `all3()` constructor function (7 lines), the `all4()` constructor function (12 lines), the `Either<Left, Right>` enum (5 lines), and the `Status` enum with its `Parsable` impl (23 lines) -- all confirmed unused via project-wide search. The `impl Parser for All<(A0, A1, A2)>` and `impl Parser for All<(A0, A1, A2, A3)>` generic trait implementations were retained. No changes to `src/lib.rs` or `src/main.rs`. No external dependencies added. All 26 tests pass (25 existing + 1 new); no test cases deleted. `cargo run -- example.log` output unchanged.

## Phase 12: Remove `OnceLock` Singleton

**Scope:** `src/parse.rs`, `src/lib.rs`

Removed the `LogLineParser` struct and `LOG_LINE_PARSER` `OnceLock` singleton from `src/parse.rs`, eliminating the last piece of hidden global mutable state and the unnecessary `std::sync::OnceLock` synchronization primitive. The parser is now constructed once per `LogIterator` instance via `LogLine::parser()` and stored as a `parser: <LogLine as Parsable>::Parser` field, replacing the previous `LOG_LINE_PARSER.parse(line.trim())` call with `self.parser.parse(line.trim())` in `LogIterator::next()`. Removed the last two `// подсказка:` hint comments in the codebase (`// подсказка: singleton, без которого можно обойтись` and `// парсеры не страшно вытащить в pub`), completing all identified technical debt across the 12-phase refactoring project. No changes to `src/main.rs` or test code. No external dependencies added. All existing 25 tests pass unchanged; no behavior changes.

## Phase 11: `NonZeroU32` Tight Type

**Scope:** `src/parse.rs`, `src/lib.rs`

Replaced the `u32` type used for `request_id` with `std::num::NonZeroU32` throughout the codebase, encoding the "request IDs are never zero" invariant directly in the type system. The `stdp::U32` parser's `type Dest` was changed from `u32` to `NonZeroU32`, and the runtime `if value == 0 { return Err(()); }` check was replaced with `NonZeroU32::new(value).ok_or(())?`. The `LogLine::request_id` field type was updated from `u32` to `std::num::NonZeroU32`, and the `Parsable` impl's function pointer type was updated from `fn((LogKind, u32)) -> Self` to `fn((LogKind, std::num::NonZeroU32)) -> Self`. The `read_log()` parameter was changed from `request_ids: Vec<u32>` to `request_ids: Vec<NonZeroU32>`. All test call sites were updated to construct `NonZeroU32` values; a helper `fn nz()` was added in the parse test module for conciseness. The three-line hint comment block (`// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32`) and the runtime zero check were removed. No changes to `src/main.rs` (empty `vec![]` type-infers correctly). No external dependencies added. All existing 25 tests pass unchanged; no behavior changes.

## Phase 10: Box the Large Enum Variant

**Scope:** `src/parse.rs`

Wrapped the oversized `AuthData` payload in `Box<>` at the `AppLogTraceKind::Connect` variant, changing it from `Connect(AuthData)` to `Connect(Box<AuthData>)`. The `AuthData` struct contains a `[u8; 1024]` fixed-size array that previously inflated every enum in the chain (`AppLogTraceKind` -> `AppLogKind` -> `LogKind` -> `LogLine`) to ~1040 bytes on the stack. After boxing, the `Connect` variant stores an 8-byte pointer inline and the 1024-byte payload on the heap, reducing `LogLine` from ~1040 bytes to ~40 bytes. Updated the parser map closure to wrap the parsed `AuthData` in `Box::new()` (the non-capturing closure coerces to the existing `fn(AuthData) -> AppLogTraceKind` function pointer type, so the associated `Parser` type is unchanged). Updated the `test_log_kind` expected value to use `Box::new(AuthData([...]))`. Removed both hint comments (`// подсказка: довольно много места на стэке` and `// подсказка: а поля не слишком много места на стэке занимают?`). No changes to `src/lib.rs` or `src/main.rs`. All existing 25 tests pass unchanged; no behavior changes.

## Phase 9: Loops to Iterators

**Scope:** `src/lib.rs`

Replaced the two manual `for` loops in the `read_log()` function body with idiomatic Rust iterator chains. The outer `for` loop (manual `push` into a mutable `Vec`) was replaced with a two-pass iterator chain: `.collect::<Result<Vec<_>, _>>()?` to parse all lines with short-circuit error propagation, followed by `.into_iter().filter(|log| { ... }).collect()` for filtering by request ID and mode. The inner `for` loop (manual request ID search using a mutable boolean flag `request_id_found` and `break`) was replaced with `request_ids.contains(&log.request_id)`. Removed the hint comment `// подсказка: можно обойтись итераторами`. The `read_log()` function signature is unchanged; all existing 25 tests pass unchanged; no behavior changes.

## Phase 8: Generic `just_parse<T>()`

**Scope:** `src/parse.rs`, `src/main.rs`

Collapsed six nearly identical `just_parse_*` / `just_user_*` wrapper functions (`just_parse_asset_dsc`, `just_parse_backet`, `just_user_cash`, `just_user_backet`, `just_user_backets`, `just_parse_anouncements`) into a single generic `pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()>` function. Made the `Parsable` and `Parser` traits public to satisfy Rust's visibility rules for the generic function's trait bound (E0445). Updated the sole external call site in `main.rs` to use turbofish syntax (`just_parse::<Announcements>(...)`). Removed the hint comments `// просто обёртки` and `// подсказка: почему бы не заменить на один дженерик?`. The generic function works for all 17 types implementing `Parsable`, not just the 6 that previously had dedicated wrappers. Added seven new tests for the generic function. All existing tests pass unchanged; no behavior changes.

## Phase 7: `Result` instead of `panic!`

**Scope:** `src/lib.rs`, `src/main.rs`

Converted `read_log()` from returning `Vec<LogLine>` to `Result<Vec<LogLine>, std::io::Error>`, and changed `LogIterator`'s `Item` type from `LogLine` to `Result<LogLine, std::io::Error>` so that I/O errors from `BufReader::lines()` are propagated to callers instead of being silently swallowed by `.ok()?`. The chained `.ok()?` calls in `LogIterator::next()` were replaced with a `loop`/`continue` pattern that explicitly yields `Some(Err(e))` for I/O errors, skips parse errors via `continue`, and yields `Some(Ok(result))` for successfully parsed lines. The `read_log()` loop body now uses `let log = log_result?;` to propagate I/O errors and returns `Ok(collected)` on success. All test functions and the `main.rs` call site were adapted with `.unwrap()`. Parse errors (unparseable log lines) continue to be silently skipped. The Phase 9 hint comment (`// подсказка: можно обойтись итераторами`) is preserved. All existing tests pass unchanged; no behavior changes on the success path.

## Phase 6: `match` instead of `if` chain

**Scope:** `src/lib.rs`

Replaced the `if mode == ReadMode::All { ... } else if mode == ReadMode::Errors { ... } else if mode == ReadMode::Exchanges { ... } else { panic!(...) }` chain in `read_log()` with an exhaustive `match &mode` expression containing three explicit arms and no wildcard/default arm. This makes the filtering logic idiomatic Rust and enables compiler-verified exhaustiveness. The `panic!("unknown mode {:?}", mode)` arm and both hint comments (`// подсказка: лучше match` and `// подсказка: паниковать в библиотечном коде - нехорошо`) were removed as a natural consequence of the exhaustive match. Added two new tests (`test_errors_mode` and `test_exchanges_mode`) to exercise the `ReadMode::Errors` and `ReadMode::Exchanges` filter paths. All existing tests pass unchanged; no behavior changes.

## Phase 5: `u8` constants -> `enum ReadMode`

**Scope:** `src/lib.rs`, `src/main.rs`

Replaced the three public `u8` mode constants (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) with a public `enum ReadMode` having variants `All`, `Errors`, and `Exchanges`, deriving `Debug` and `PartialEq`. Updated the `read_log()` function signature from `mode: u8` to `mode: ReadMode`, replaced all constant references in the filtering logic with enum variants, updated the `panic!` format string to use `{:?}`, and adapted call sites in `main.rs` and the test module. Removed the hint comment `// подсказка: лучше использовать enum и match`. Invalid mode values are now caught at compile time instead of causing a runtime panic. All existing tests pass unchanged; no behavior changes.

## Phase 4: Generic `R: Read` instead of trait object

**Scope:** `src/lib.rs`, `src/main.rs`

Replaced `Box<dyn MyReader>` with a generic type parameter `R: Read` on `LogIterator`, enabling static dispatch and monomorphization. Removed the `MyReader` supertrait (the E0225 workaround), its blanket impl, the associated comments, and the hint comment. The `read_log()` public API now accepts `impl Read` instead of `Box<dyn MyReader>`, allowing callers to pass any reader directly without boxing. Removed `#[derive(Debug)]` from the private `LogIterator` struct to avoid an unnecessary `R: Debug` bound. All existing tests pass unchanged; no behavior changes.

## Phase 3: Remove `unsafe` transmute

**Scope:** Verification only (no source code changes)

Formally verified that the `unsafe { transmute(...) }` block in `LogIterator::new()` was fully removed as a side-effect of Phase 2. Confirmed zero occurrences of `unsafe` and `transmute` across all source files, confirmed the associated hint comment (`// подсказка: unsafe избыточен, да и весь rc - тоже`) was already removed, and validated that all 16 tests pass and CLI output is unchanged. No Rust source code was modified in this phase.

## Phase 2: Remove `Rc<RefCell>`

**Scope:** `src/lib.rs`, `src/main.rs`

Removed the unnecessary `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`, giving it direct ownership of the reader via `Box<dyn MyReader>`. This eliminated the `RefMutWrapper` adapter struct, the `unsafe { transmute }` lifetime extension, and the self-referential struct pattern. The `read_log()` API now accepts `Box<dyn MyReader>` instead of `Rc<RefCell<Box<dyn MyReader>>>`. No behavior changes; all existing tests pass unchanged.

## Phase 1: `String` -> `&str` in `Parser` trait

**Scope:** `src/parse.rs`, `src/lib.rs`, `src/main.rs`

Migrated the `Parser` trait and all combinator implementations from operating on owned `String` values to borrowed `&str` slices with lifetimes. This reduces unnecessary heap allocations throughout the parsing pipeline. Updated all 15 tests in `parse.rs` and adapted call sites in `lib.rs` and `main.rs`.

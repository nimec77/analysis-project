# Changelog

All notable changes to this project will be documented in this file.

---

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

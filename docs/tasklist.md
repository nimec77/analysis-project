# Refactoring Tasklist

## Progress

### Part 1: Core Refactoring (completed)

| Status | Phase | Description | Files | Depends on |
|--------|-------|-------------|-------|------------|
| :green_circle: | 1 | `String` -> `&str` in `Parser` trait | `src/parse.rs` | — |
| :green_circle: | 2 | Remove `Rc<RefCell>` | `src/lib.rs` | — |
| :green_circle: | 3 | Remove `unsafe` transmute | `src/lib.rs` | Phase 2 |
| :green_circle: | 4 | Generic `R: Read` instead of trait object | `src/lib.rs`, `src/main.rs` | Phase 2 |
| :green_circle: | 5 | `u8` constants -> `enum ReadMode` | `src/lib.rs` | — |
| :green_circle: | 6 | `match` instead of `if` chain | `src/lib.rs` | Phase 5 |
| :green_circle: | 7 | `Result` instead of `panic!` | `src/lib.rs` | Phase 5 |
| :green_circle: | 8 | Generic `just_parse<T>()` | `src/parse.rs` | — |
| :green_circle: | 9 | Loops -> iterators | `src/lib.rs` | — |
| :green_circle: | 10 | `Box` the large enum variant | `src/parse.rs` | — |
| :green_circle: | 11 | `NonZeroU32` tight type | `src/parse.rs` | — |
| :green_circle: | 12 | Remove `OnceLock` singleton | `src/parse.rs`, `src/lib.rs` | Phase 1 |

### Part 2: Optimization and Improvement

| Status | Phase | Description | Files | Depends on |
|--------|-------|-------------|-------|------------|
| :green_circle: | 13 | Bug fix + dead code cleanup | `src/parse.rs` | — |
| :white_circle: | 14 | Naming improvements | `src/parse.rs` | — |
| :white_circle: | 15 | Modularity (split `parse.rs`) | `src/parse.rs`, `src/parse/*.rs` | Phase 14 |
| :white_circle: | 16 | Newtype pattern (`UserId`, `AssetId`) | `src/parse/*.rs` | Phase 15 |
| :white_circle: | 17 | Error handling (`ParseError`, `anyhow`) | `src/parse/*.rs`, `src/lib.rs`, `src/main.rs`, `Cargo.toml` | — |
| :white_circle: | 18 | Strategy pattern (`LogFilter` trait) | `src/lib.rs` | — |
| :white_circle: | 19 | CLI argument parsing (`clap`) | `src/main.rs`, `Cargo.toml` | Phase 18 |
| :white_circle: | 20 | `Display` trait for log types | `src/parse/*.rs` | Phase 15 |
| :white_circle: | 21 | Property-based testing (`proptest`) | `src/parse/*.rs`, `Cargo.toml` | Phase 20 |
| :white_circle: | 22 | Parser fluent API (stretch) | `src/parse/combinators.rs` | Phase 15 |

Legend: :white_circle: pending | :large_blue_circle: in progress | :green_circle: done

**Current Phase:** 14

---

## Phase 1: `String` -> `&str` in `Parser` trait

- [x] Change `Parser` trait to operate on `&str` with lifetimes instead of `String`
- [x] Update all combinator implementations (`Tag`, `Alt`, `Map`, `Delimited`, `Preceded`, `Permutation`, `List`, `Take`, etc.)
- [x] Update `Parsable` trait and all implementations
- [x] Remove now-unnecessary `.clone()` calls on input strings
- [x] Adapt all 15 tests in `parse.rs`: remove `.into()` on parser inputs and on `remaining` in `Ok((...))` expectations; keep `.into()` for owned `String` output values (e.g. `Unquote`, `AssetDsc.id`, `Backet.asset_id`)

**Hint:** `src/parse.rs:5` — `// подсказка: здесь можно переделать` (the `Parser` trait definition)

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 2: Remove `Rc<RefCell>`

- [x] Remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`
- [x] Give `LogIterator` direct ownership of the reader
- [x] Remove `RefMutWrapper` and `MyReader` trait if they become unused
- [x] Adapt `test_all` in `lib.rs`: remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping, pass reader directly to `read_log()`

**Hint:** `src/lib.rs:71` — `// подсказка: RefCell вообще не нужен`
**Hint:** `src/lib.rs:40` — `// подсказка: unsafe избыточен, да и весь rc - тоже`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 3: Remove `unsafe` transmute

- [x] Replace the `unsafe { transmute(...) }` with safe code (possible once `Rc<RefCell>` is gone)

**Hint:** `src/lib.rs:40` — `// подсказка: unsafe избыточен, да и весь rc - тоже`

**Depends on:** Phase 2

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 4: Generic `R: Read` instead of trait object

- [x] Make `LogIterator` generic: `LogIterator<R: Read>`
- [x] Remove `Box<dyn MyReader>` / `MyReader` trait
- [x] Update `read_log()` signature to accept `impl Read`
- [x] Adapt `main.rs` to the new signature

**Hint:** `src/lib.rs:30` — `// подсказка: вместо trait-объекта можно дженерик`

**Depends on:** Phase 2

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 5: `u8` constants -> `enum ReadMode`

- [x] Replace `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` constants with `enum ReadMode`
- [x] Update `read_log()` and all call sites
- [x] Adapt `test_all` in `lib.rs`: replace `READ_MODE_ALL` with `ReadMode::All`

**Hint:** `src/lib.rs:4` — `// подсказка: лучше использовать enum и match`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 6: `match` instead of `if` chain

- [x] Replace the `if mode == ... else if mode == ...` chain with `match` on `ReadMode`

**Hint:** `src/lib.rs:88` — `// подсказка: лучше match`

**Depends on:** Phase 5

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 7: `Result` instead of `panic!`

- [x] Replace `panic!` on unknown mode with exhaustive `match` (no default arm needed after Phase 5)
- [x] Return `Result` from `read_log()` for any remaining fallible operations
- [x] Adapt `test_all` in `lib.rs` if `read_log()` now returns `Result`: unwrap or use `?` in test

**Hint:** `src/lib.rs:114` — `// подсказка: паниковать в библиотечном коде - нехорошо`

**Depends on:** Phase 5

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 8: Generic `just_parse<T>()`

- [x] Collapse `just_parse_u32`, `just_parse_u64`, etc. into one generic `just_parse<T: Parsable>()`

**Hint:** `src/parse.rs:789` — `// подсказка: почему бы не заменить на один дженерик?`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 9: Loops -> iterators

- [x] Replace manual `for` / `while` loops with iterator chains where idiomatic

**Hint:** `src/lib.rs:76` — `// подсказка: можно обойтись итераторами`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 10: `Box` the large enum variant

- [x] Wrap `AuthData` (or whichever variant is oversized) in `Box<>` to reduce `LogKind` stack size
- [x] Adapt `test_authdata` and `test_log_kind` in `parse.rs`: wrap `AuthData(...)` in `Box::new(...)` in expected values where the variant is `Connect(Box<AuthData>)`

**Hint:** `src/parse.rs:621` — `// подсказка: довольно много места на стэке`
**Hint:** `src/parse.rs:852` — `// подсказка: а поля не слишком много места на стэке занимают?`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 11: `NonZeroU32` tight type

- [x] Use `std::num::NonZeroU32` for `request_id` instead of `u32` + runtime check

**Hint:** `src/parse.rs:39` — `// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 12: Remove `OnceLock` singleton

- [x] Remove `LOG_LINE_PARSER` `OnceLock` singleton
- [x] Construct the parser inline or pass it as a parameter
- [x] Update call site in `lib.rs`

**Hint:** `src/parse.rs:1144` — `// подсказка: singleton, без которого можно обойтись`

**Depends on:** Phase 1 (lightweight parser construction after `&str` migration)

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 13: Bug fix + dead code cleanup

- [x] Fix `WithdrawCash` bug: `src/parse.rs:1320` maps to `DepositCash` instead of `WithdrawCash`
- [x] Add dedicated `WithdrawCash` parsing test to prevent regression
- [x] Remove unused `AsIs` struct + Parser impl (~line 138)
- [x] Remove unused `Either<L,R>` enum (~line 731)
- [x] Remove unused `Status` enum + Parsable impl (~line 737)
- [x] Remove unused `all3()` constructor (~line 331)
- [x] Remove unused `all4()` constructor (~line 356)

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 14: Naming improvements

- [ ] Rename `All` struct → `Tuple` (matches nom's naming for sequential parsing returning a tuple)
- [ ] Rename `all2()` → `tuple2()` and update all call sites
- [ ] Rename `stdp` module → `primitives`
- [ ] Rename `do_unquote()` → `unquote_escaped()`
- [ ] Rename `do_unquote_non_escaped()` → `unquote_simple()`
- [ ] Update all internal references and tests

Not changing: `A0/A1/A2` type params (standard tuple-impl pattern), `nz()` test helper, `AssetDsc.dsc` (matches domain key), arity suffixes (`alt2`, `permutation3`).

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 15: Modularity (split `parse.rs`)

- [ ] Create `src/parse/` directory
- [ ] Move combinator framework (traits + structs) to `src/parse/combinators.rs`
- [ ] Move domain types (AuthData, AssetDsc, Backet, etc.) to `src/parse/domain.rs`
- [ ] Move log hierarchy (LogLine, LogKind, etc.) to `src/parse/log.rs`
- [ ] Convert `src/parse.rs` to module root: `mod combinators; mod domain; mod log;` with `pub use` re-exports
- [ ] Move `primitives` (ex-`stdp`) as private sub-module within `combinators.rs`
- [ ] Refine visibility: constructor functions to `pub(crate)`
- [ ] Move tests to `#[cfg(test)] mod tests` in each sub-module

Uses edition 2024 module paths (NO `mod.rs`).

**Depends on:** Phase 14

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 16: Newtype pattern (`UserId`, `AssetId`)

- [ ] Define `pub struct UserId(pub String)` with `Debug, Clone, PartialEq`
- [ ] Define `pub struct AssetId(pub String)` with `Debug, Clone, PartialEq`
- [ ] Implement `Parsable` for `UserId` (delegate to `Unquote` + `Map`)
- [ ] Implement `Parsable` for `AssetId` (delegate to `Unquote` + `Map`)
- [ ] Replace `user_id: String` → `user_id: UserId` in `UserCash`, `UserBacket`, `UserBackets`, `AppLogJournalKind::{CreateUser, DeleteUser, RegisterAsset, UnregisterAsset}`
- [ ] Replace `asset_id: String` / `id: String` → `AssetId` in `AssetDsc`, `Backet`, `AppLogJournalKind::{RegisterAsset, UnregisterAsset}`
- [ ] Update all parser implementations
- [ ] Update all tests

**Depends on:** Phase 15

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 17: Error handling (`ParseError`, `anyhow`)

- [ ] Define `ParseError` enum with variants: `UnexpectedInput`, `IncompleteInput`, `InvalidValue` (each with `&'static str` context)
- [ ] Add `thiserror = "2"` to `[dependencies]` in `Cargo.toml`
- [ ] Replace `Result<T, ()>` with `Result<T, ParseError>` in `Parser` trait and all implementations
- [ ] Update all `Err(())` → appropriate `ParseError` variants
- [ ] Update all `ok_or(())` and `map_err(|_| ())` calls
- [ ] Add `anyhow = "1"` to `[dependencies]` in `Cargo.toml`
- [ ] Fix `main.rs`: replace `args[1]` panic with `.get(1)` + usage message
- [ ] Fix `main.rs`: replace `.unwrap()` on file open with error message
- [ ] Remove hardcoded demo code from `main.rs` (lines 54-58)
- [ ] Change `main()` to `fn main() -> anyhow::Result<()>`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 18: Strategy pattern (`LogFilter` trait)

- [ ] Define `LogFilter` trait in `src/lib.rs`: `fn accepts(&self, log: &LogLine) -> bool`
- [ ] Implement `LogFilter` for `ReadMode` (move existing match logic)
- [ ] Update `read_log()` signature: `filter: impl LogFilter` instead of `mode: ReadMode`
- [ ] Update call sites in `main.rs` and tests

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 19: CLI argument parsing (`clap`)

- [ ] Add `clap = { version = "4", features = ["derive"] }` to `[dependencies]` in `Cargo.toml`
- [ ] Define CLI struct with `#[derive(clap::Parser)]`
- [ ] Support `--mode all|errors|exchanges` (default: `all`)
- [ ] Support `--request-id 1,2,3` (optional, comma-separated)
- [ ] Positional `<filename>` argument
- [ ] Free `--help` and `--version` support
- [ ] Update `main()` to use clap-parsed args

**Depends on:** Phase 18

**Verify:** `cargo test && cargo run -- example.log && cargo run -- --help`

---

## Phase 20: `Display` trait for log types

- [ ] Implement `Display` for `LogLine`
- [ ] Implement `Display` for `LogKind`, `SystemLogKind`, `AppLogKind`
- [ ] Implement `Display` for journal variants (`AppLogJournalKind`)
- [ ] Implement `Display` for domain types (`UserId`, `AssetId`, `UserCash`, `Backet`, etc.)
- [ ] Update `main.rs` to use `{}` instead of `{:?}` for output

**Depends on:** Phase 15

**Verify:** `cargo test && cargo run -- example.log` (output should be human-readable)

---

## Phase 21: Property-based testing (`proptest`)

- [ ] Add `proptest = "1"` to `[dev-dependencies]` in `Cargo.toml`
- [ ] Roundtrip test: `unquote_escaped(quote(s)) == Ok(("", s))` for arbitrary strings
- [ ] No-panic test: `LogLine::parser().parse(arbitrary_string)` never panics
- [ ] Suffix invariant: parser remaining output is always a suffix of input
- [ ] Add missing unit tests: `WithdrawCash`, `DeleteUser`, `UnregisterAsset` standalone parsing
- [ ] Add `Permutation` with 3 parsers coverage
- [ ] Add error cases for each domain type with malformed input

**Depends on:** Phase 20

**Verify:** `cargo test && cargo test -- --nocapture`

---

## Phase 22: Parser fluent API (stretch)

- [ ] Add `.map()` method to `Parser` trait as blanket extension
- [ ] Add `.preceded_by()` method
- [ ] Add `.strip_ws()` method
- [ ] Rewrite `Parsable` implementations using fluent style where it improves readability
- [ ] Example: `tag("Error").preceded_by(tag("System::")).map(|_| ...)`

**Depends on:** Phase 15

**Verify:** `cargo test && cargo run -- example.log`

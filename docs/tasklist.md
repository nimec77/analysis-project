# Refactoring Tasklist

## Progress

| Status | Phase | Description | Files | Depends on |
|--------|-------|-------------|-------|------------|
| :green_circle: | 1 | `String` -> `&str` in `Parser` trait | `src/parse.rs` | — |
| :green_circle: | 2 | Remove `Rc<RefCell>` | `src/lib.rs` | — |
| :green_circle: | 3 | Remove `unsafe` transmute | `src/lib.rs` | Phase 2 |
| :green_circle: | 4 | Generic `R: Read` instead of trait object | `src/lib.rs`, `src/main.rs` | Phase 2 |
| :white_circle: | 5 | `u8` constants -> `enum ReadMode` | `src/lib.rs` | — |
| :white_circle: | 6 | `match` instead of `if` chain | `src/lib.rs` | Phase 5 |
| :white_circle: | 7 | `Result` instead of `panic!` | `src/lib.rs` | Phase 5 |
| :white_circle: | 8 | Generic `just_parse<T>()` | `src/parse.rs` | — |
| :white_circle: | 9 | Loops -> iterators | `src/lib.rs` | — |
| :white_circle: | 10 | `Box` the large enum variant | `src/parse.rs` | — |
| :white_circle: | 11 | `NonZeroU32` tight type | `src/parse.rs` | — |
| :white_circle: | 12 | Remove `OnceLock` singleton | `src/parse.rs`, `src/lib.rs` | Phase 1 |

Legend: :white_circle: pending | :large_blue_circle: in progress | :green_circle: done

**Current Phase:** 5

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

- [ ] Replace `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` constants with `enum ReadMode`
- [ ] Update `read_log()` and all call sites
- [ ] Adapt `test_all` in `lib.rs`: replace `READ_MODE_ALL` with `ReadMode::All`

**Hint:** `src/lib.rs:4` — `// подсказка: лучше использовать enum и match`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 6: `match` instead of `if` chain

- [ ] Replace the `if mode == ... else if mode == ...` chain with `match` on `ReadMode`

**Hint:** `src/lib.rs:88` — `// подсказка: лучше match`

**Depends on:** Phase 5

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 7: `Result` instead of `panic!`

- [ ] Replace `panic!` on unknown mode with exhaustive `match` (no default arm needed after Phase 5)
- [ ] Return `Result` from `read_log()` for any remaining fallible operations
- [ ] Adapt `test_all` in `lib.rs` if `read_log()` now returns `Result`: unwrap or use `?` in test

**Hint:** `src/lib.rs:114` — `// подсказка: паниковать в библиотечном коде - нехорошо`

**Depends on:** Phase 5

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 8: Generic `just_parse<T>()`

- [ ] Collapse `just_parse_u32`, `just_parse_u64`, etc. into one generic `just_parse<T: Parsable>()`

**Hint:** `src/parse.rs:789` — `// подсказка: почему бы не заменить на один дженерик?`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 9: Loops -> iterators

- [ ] Replace manual `for` / `while` loops with iterator chains where idiomatic

**Hint:** `src/lib.rs:76` — `// подсказка: можно обойтись итераторами`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 10: `Box` the large enum variant

- [ ] Wrap `AuthData` (or whichever variant is oversized) in `Box<>` to reduce `LogKind` stack size
- [ ] Adapt `test_authdata` and `test_log_kind` in `parse.rs`: wrap `AuthData(...)` in `Box::new(...)` in expected values where the variant is `Connect(Box<AuthData>)`

**Hint:** `src/parse.rs:621` — `// подсказка: довольно много места на стэке`
**Hint:** `src/parse.rs:852` — `// подсказка: а поля не слишком много места на стэке занимают?`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 11: `NonZeroU32` tight type

- [ ] Use `std::num::NonZeroU32` for `request_id` instead of `u32` + runtime check

**Hint:** `src/parse.rs:39` — `// подсказка: вместо if можно использовать tight-тип std::num::NonZeroU32`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 12: Remove `OnceLock` singleton

- [ ] Remove `LOG_LINE_PARSER` `OnceLock` singleton
- [ ] Construct the parser inline or pass it as a parameter
- [ ] Update call site in `lib.rs`

**Hint:** `src/parse.rs:1144` — `// подсказка: singleton, без которого можно обойтись`

**Depends on:** Phase 1 (lightweight parser construction after `&str` migration)

**Verify:** `cargo test && cargo run -- example.log`

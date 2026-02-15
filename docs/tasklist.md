# Refactoring Tasklist

## Progress

| Status | Phase | Description | Files | Depends on |
|--------|-------|-------------|-------|------------|
| :white_circle: | 1 | `String` -> `&str` in `Parser` trait | `src/parse.rs` | — |
| :white_circle: | 2 | Remove `Rc<RefCell>` | `src/lib.rs` | — |
| :white_circle: | 3 | Remove `unsafe` transmute | `src/lib.rs` | Phase 2 |
| :white_circle: | 4 | Generic `R: Read` instead of trait object | `src/lib.rs`, `src/main.rs` | Phase 2 |
| :white_circle: | 5 | `u8` constants -> `enum ReadMode` | `src/lib.rs` | — |
| :white_circle: | 6 | `match` instead of `if` chain | `src/lib.rs` | Phase 5 |
| :white_circle: | 7 | `Result` instead of `panic!` | `src/lib.rs` | Phase 5 |
| :white_circle: | 8 | Generic `just_parse<T>()` | `src/parse.rs` | — |
| :white_circle: | 9 | Loops -> iterators | `src/lib.rs` | — |
| :white_circle: | 10 | `Box` the large enum variant | `src/parse.rs` | — |
| :white_circle: | 11 | `NonZeroU32` tight type | `src/parse.rs` | — |
| :white_circle: | 12 | Remove `OnceLock` singleton | `src/parse.rs`, `src/lib.rs` | Phase 1 |

Legend: :white_circle: pending | :large_blue_circle: in progress | :green_circle: done

---

## Phase 1: `String` -> `&str` in `Parser` trait

- [ ] Change `Parser` trait to operate on `&str` with lifetimes instead of `String`
- [ ] Update all combinator implementations (`Tag`, `Alt`, `Map`, `Delimited`, `Preceded`, `Permutation`, `List`, `Take`, etc.)
- [ ] Update `Parsable` trait and all implementations
- [ ] Remove now-unnecessary `.clone()` calls on input strings

**Hint:** `src/parse.rs:5` — `// подсказка: здесь можно переделать` (the `Parser` trait definition)

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 2: Remove `Rc<RefCell>`

- [ ] Remove `Rc<RefCell<Box<dyn MyReader>>>` wrapping from `LogIterator`
- [ ] Give `LogIterator` direct ownership of the reader
- [ ] Remove `RefMutWrapper` and `MyReader` trait if they become unused

**Hint:** `src/lib.rs:71` — `// подсказка: RefCell вообще не нужен`
**Hint:** `src/lib.rs:40` — `// подсказка: unsafe избыточен, да и весь rc - тоже`

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 3: Remove `unsafe` transmute

- [ ] Replace the `unsafe { transmute(...) }` with safe code (possible once `Rc<RefCell>` is gone)

**Hint:** `src/lib.rs:40` — `// подсказка: unsafe избыточен, да и весь rc - тоже`

**Depends on:** Phase 2

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 4: Generic `R: Read` instead of trait object

- [ ] Make `LogIterator` generic: `LogIterator<R: Read>`
- [ ] Remove `Box<dyn MyReader>` / `MyReader` trait
- [ ] Update `read_log()` signature to accept `impl Read`
- [ ] Adapt `main.rs` to the new signature

**Hint:** `src/lib.rs:30` — `// подсказка: вместо trait-объекта можно дженерик`

**Depends on:** Phase 2

**Verify:** `cargo test && cargo run -- example.log`

---

## Phase 5: `u8` constants -> `enum ReadMode`

- [ ] Replace `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` constants with `enum ReadMode`
- [ ] Update `read_log()` and all call sites

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

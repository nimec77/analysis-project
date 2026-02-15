# Idea: Rust Code Refactoring

## Task Description

### Project

A Rust log analysis tool for a trading/exchange platform (edition 2024, no external dependencies). It parses structured log lines, filters them by mode (all, errors, exchange operations), and by `request_id`.

Structure:
- `src/lib.rs` — library crate: exports `read_log()`, log iterator, filtering
- `src/parse.rs` — parser combinator framework (nom-like), log data model
- `src/main.rs` — CLI binary: reads a file and prints parsed logs

### Task Summary

The project contains working but non-idiomatic code. The goal is to refactor it by fixing 11 categories of issues. Hints (`// подсказка: ...`) are placed throughout the code, pointing to locations where problematic code is guaranteed to exist. There are enough hints to cover every issue type from the list.

### 11 Issue Categories

1. **Unnecessary `clone()` calls** instead of using references
2. **`Rc<RefCell<T>>`** where references alone would suffice
3. **Loops instead of iterators**
4. **Unnecessary use of `unsafe`**
5. **Singleton that can be eliminated**
6. **Excessive validation instead of tight types** (e.g., `NonZeroU32`)
7. **Code duplication instead of generics** for parameters of different types
8. **Trait objects** where generics would suffice
9. **Chain of `if` statements instead of a single `match`**
10. **Enum variant occupying several kilobytes on the stack**
11. **Panic instead of returning an error** in library code

---

## Approaches to Fixing and Things to Watch Out For

### General Principles

- **Start with the fundamental change.** The `Parser` trait accepts and returns `String`, which scatters `.to_string()` and `.clone()` throughout `parse.rs`. Switching to `&str` with lifetimes is the root change that cascadingly eliminates most unnecessary `clone()` calls in combinators (`Alt`, `Permutation`, etc.). This is the most labor-intensive change, but it should be done first so that subsequent fixes build on a clean foundation.
- **The compiler is your best ally.** After changing the trait signature, the compiler will point out every location that needs updating. You can progress from error to error.
- **Tests are your safety net.** Run `cargo test` after each logical step. Keep the test cases (adapting data types in tests is acceptable).

### By Category

1. **`clone()` → references.** The key approach is to use `&str` with lifetimes instead of `String` in parser signatures. The remaining input (`remaining`) becomes a slice of the original input — no allocations needed. In `Permutation` and `Alt`, cloning the input string before each parser attempt disappears: `&str` copies trivially (it's just a pointer + length).

2. **`Rc<RefCell<T>>` → references/ownership.** The reader is used linearly and is not shared between multiple owners. It suffices to pass it by value (move) or by mutable reference. This eliminates the need for `RefMutWrapper`, `RefMut`, and the entire wrapper simplifies.

3. **Loops → iterators.** A manual loop with `push` is replaced by a `.filter().collect()` chain. A nested loop searching for `request_id` is replaced with `.contains()` or `.any()`.

4. **Remove `unsafe`.** The `transmute` for lifetime extension is UB. After eliminating `Rc<RefCell>`, the need for it disappears entirely, since the reader is passed directly without borrowing through `RefCell`.

5. **Remove the singleton.** Parsers are lightweight stateless structures (zero-sized types). `OnceLock` for lazy initialization provides no benefit. You can make the `Parser`/`Parsable` traits public and call `LogLine::parser().parse(...)` directly, or export a simple function.

6. **Tight types.** Two aspects:
   - `u8` mode constants → `enum`. The compiler guarantees exhaustiveness; manual validation and `panic!` are unnecessary.
   - The check `if value == 0` → `NonZeroU32::new(value).ok_or(())`. The constraint is encoded in the type. You can immediately call `.get()` to avoid changing the entire data model.

7. **Generic instead of duplication.** The set of identical wrapper functions `just_parse_*` is replaced by a single generic function `just_parse<T: Parsable>(...)`.

8. **Generic instead of trait object.** `Box<dyn MyReader>` → type parameter `R: Read`. Eliminates dynamic dispatch and simplifies the `LogIterator` structure (no `dyn`, no `Box`).

9. **`match` instead of `if` chain.** The chain `if mode == X ... else if mode == Y ...` is replaced by a single `match`. Combined with `enum` (item 6), this yields an idiomatic exhaustive match without `else { panic!() }`.

10. **Large enum variant → `Box`.** `AuthData([u8; 1024])` bloats `AppLogTraceKind` (and its parents `AppLogKind`, `LogKind`, `LogLine`) to 1024+ bytes. Wrap it in `Box` — either `Connect(Box<AuthData>)` or `AuthData(Box<[u8; 1024]>)`. The first option is preferable as it has less impact on the `AuthData` structure.

11. **`Result` instead of `panic!`.** Library code should not panic on invalid input. After replacing `u8` with `enum ReadMode`, the branch with `panic!` disappears naturally (exhaustive match). The `read_log` signature can remain `-> Vec<LogLine>` (parse errors are already ignored via `.ok()?`), or it can return `Result` for I/O errors.

### Things to Watch Out For

- **Dependencies between fixes.** Switching `Parser` to `&str` is the foundation: removing `clone()`, updating `just_parse`, and removing the singleton all depend on it. Removing `Rc<RefCell>` cascades into removing `unsafe` and the trait object. `enum ReadMode` eliminates the `if` chain and `panic!` simultaneously.
- **`do_unquote()` returns a `String` as a value** (deserialized string), not as a remainder — this is correct and should not be changed.
- **Tests use `.into()` to create `String`** — after switching to `&str`, tests become simpler (remove `.into()`).
- **There are enough hints for every issue type.** No need to search for additional locations — focus on those marked with `// подсказка:` comments.

---

## Submission Requirements

### Mandatory Requirements

1. **Tests pass.** All existing test cases are preserved and pass (`cargo test -- --nocapture`). Adapting data types in tests is acceptable, but deleting test cases is not.
2. **Application works.** `cargo run -- example.log` produces the same output as before the refactoring.
3. **At least 80% of issues fixed.** Out of the 11 categories, each one that has a hint in the code must be fixed. Submission at 80% is acceptable if the deadline is close.

### Formatting and Submission Process

1. **Upload the original project** to a public GitHub repository as a separate commit (the original, unrefactored version).
2. **Format the refactoring** as subsequent commits (one or several). These are patches to the project.
3. **Submit the repository link** in the field on the project page in Practicum and click "Submit".
4. **Review.** The reviewer will check the code and leave comments in GitHub Issues.
5. **Fix Issues.** Each comment is fixed in a **separate commit** whose title includes the Issue number. After pushing, reply in the corresponding Issue, mentioning the reviewer, the commit, and the reason for the fix.
6. **Closure.** Once all Issues are closed, the project work is considered submitted.

### Useful Commands

```bash
cargo test -- --nocapture          # all tests with terminal output
cargo test test_all -- --nocapture # specific test
cargo run -- example.log           # run CLI
```

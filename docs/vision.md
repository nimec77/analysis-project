# Technical Vision

## 1. Development Principles

1. **KISS** — Minimal changes per fix. One issue category = one commit. No premature abstractions.
2. **Idiomatic Rust** — Prefer iterators over loops, `match` over if-chains, `Result` over `panic!`, tight types over runtime validation.
3. **No behavior changes** — Same input must produce same output. Refactoring changes structure, not functionality.
4. **Compiler-driven** — Change a signature, then follow compiler errors until it builds.
5. **Tests as safety net** — `cargo test` after every logical step. Keep all test cases; adapt types if needed, never delete tests.
6. **Follow the hints** — Focus on locations marked with `// подсказка:`. Fixing every hint is mandatory. There are enough hints for each issue category.
7. **Respect dependency order** — Foundational changes first (e.g., `&str` before removing `clone()`; `Rc<RefCell>` removal before `unsafe` removal).
8. **Zero dependencies** — No external crates (phases 1-12). Third-party crates allowed in phases 13+ where they provide clear value.

## 2. Project Structure

**File layout** (phases 1-12: unchanged; phases 13+: parse.rs split into sub-modules):

```
analysis-project/
  Cargo.toml            # crate "analysis", edition 2024
  example.log           # sample log data for manual testing
  src/
    lib.rs              # library crate: public API, LogFilter trait, filtering, iteration
    parse.rs            # module root: re-exports public API from sub-modules
    parse/
      combinators.rs    # Parser/Parsable traits + combinator structs (Tuple, Alt, Map, etc.)
      domain.rs         # domain types (AuthData, AssetDsc, Backet, UserId, AssetId, etc.)
      log.rs            # log hierarchy (LogLine, LogKind, SystemLogKind, AppLogKind, etc.)
    main.rs             # CLI binary ("cli"): reads file, prints parsed logs (clap-based)
  docs/
    idea.md             # refactoring task specification
    vision.md           # technical vision (this document)
```

**Module responsibilities:**

| Module | Responsibility |
|---|---|
| `lib.rs` | `read_log(reader, mode, request_ids)` — parse, filter, return logs |
| `parse.rs` | Module root. After phase 15: re-exports from `parse/combinators.rs`, `parse/domain.rs`, `parse/log.rs` |
| `parse/combinators.rs` | `Parser` / `Parsable` traits, combinators (`Tag`, `Alt`, `Map`, `Delimited`, `Tuple`, ...) |
| `parse/domain.rs` | Domain types (`AuthData`, `AssetDsc`, `Backet`, `UserId`, `AssetId`, `UserCash`, etc.) |
| `parse/log.rs` | Log hierarchy (`LogLine`, `LogKind`, `SystemLogKind`, `AppLogKind`, etc.) |
| `main.rs` | CLI: open file, call `read_log()`, print output |

**What changes after refactoring** (internals only, public behavior preserved):

Phases 1-12 (completed):
- `lib.rs`: `Rc<RefCell<Box<dyn MyReader>>>` replaced with generic `R: Read`. `RefMutWrapper` and `MyReader` trait removed. `u8` mode constants become `enum ReadMode`. `LogIterator` becomes `LogIterator<R>`. `panic!` replaced by exhaustive `match`.
- `parse.rs`: `Parser` trait switches from `String` to `&str` with lifetimes. Singleton `OnceLock` removed. `just_parse_*` functions collapsed into one generic. `AuthData` variant boxed.
- `main.rs`: Adapts to new `read_log()` signature (simpler — no `Rc<RefCell>`).

Phases 13-22 (optimization and improvement):
- `parse.rs`: Split into `parse/{combinators,domain,log}.rs`. `All` renamed to `Tuple`. Dead code removed. `UserId`/`AssetId` newtypes introduced. `Result<T, ()>` replaced with `Result<T, ParseError>`. Fluent parser API added.
- `lib.rs`: `LogFilter` trait extracted (strategy pattern). `ReadMode` implements it. Error propagation improved.
- `main.rs`: `clap`-based CLI with `--mode`, `--request-id`, `--help`. `anyhow` error handling. `Display` output instead of `Debug`.
- `Cargo.toml`: Dependencies added: `thiserror`, `anyhow`, `clap`. Dev-dependencies: `proptest`.

## 3. Architecture

**Data flow:**

```
log file / byte stream
        |
    LogIterator  (reads lines, skips blanks)
        |
    Parser       (parses each line into LogLine)
        |
    read_log()   (filters by mode + request_ids)
        |
    Vec<LogLine>
```

**Key abstractions:**

- **`Parser` trait** — core abstraction: `fn parse(&self, input) -> Result<(remaining, Dest), ()>`. Each combinator (`Tag`, `Alt`, `Delimited`, `Map`, `Preceded`, `Permutation`, `List`, etc.) implements this trait.
- **`Parsable` trait** — implemented by data model types. Each type knows how to construct its own parser via `fn parser() -> Self::Parser`.
- **Log data model** — `LogLine` -> `LogKind` -> `{SystemLogKind, AppLogKind}` -> specific variants (errors, traces, journal events). Journal events model exchange operations: `CreateUser`, `RegisterAsset`, `BuyAsset`, `SellAsset`, etc.

**Key design decisions:**

- Hand-rolled parser combinators (nom-like API)
- Parsers are zero-sized stateless structs — cheap to construct
- Combinator arities via tuple impls (`tuple2`..`tuple4`, `alt2`..`alt8`) since Rust lacks variadic generics
- `unquote_escaped()` intentionally returns `String` (deserialized value, not a slice)
- `UserId` and `AssetId` newtypes prevent argument-order bugs at compile time
- `LogFilter` trait (strategy pattern) enables extensible filtering without modifying `ReadMode`

**Post-refactoring target** (phases 1-12, completed):

- `Parser` trait operates on `&str` with lifetimes (no allocations for remaining input)
- `LogIterator<R: Read>` — generic, no dynamic dispatch
- `read_log()` takes `ReadMode` enum + `impl Read` — no `Rc`, no `RefCell`, no `Box<dyn>`
- Single generic `just_parse<T: Parsable>()` replaces duplicated functions

**Post-optimization target** (phases 13-22):

- Structured `ParseError` type with diagnostic context (replaces `()`)
- `parse.rs` split into focused sub-modules (combinators, domain, log)
- Domain IDs as newtypes (`UserId`, `AssetId`) — compile-time safety
- `LogFilter` trait — extensible filtering strategy
- `clap`-based CLI with mode/request-id arguments
- `Display` impls for human-readable log output
- Property-based tests for parser invariants

## 4. Workflows

**Build & test:**

```bash
cargo build                        # build
cargo test                         # run all tests
cargo test test_name               # run single test
cargo test -- --nocapture          # tests with stdout
cargo run -- example.log           # run CLI
```

**Refactoring workflow:**

1. Pick an issue category from the 11-item list
2. Find the `// подсказка:` hint(s) for that category
3. Make the change
4. `cargo test` — must pass
5. `cargo run -- example.log` — output must match pre-refactoring
6. Commit (one commit per issue category)

**Recommended order** (respecting dependencies):

| Phase | Fix | Why first |
|---|---|---|
| 1 | `Parser`: `String` -> `&str` | Foundation — cascadingly removes `clone()` |
| 2 | `Rc<RefCell>` -> ownership/references | Enables removing `unsafe` and trait object |
| 3 | Remove `unsafe` (`transmute`) | Depends on phase 2 |
| 4 | Generic `R: Read` instead of `Box<dyn MyReader>` | Depends on phase 2 |
| 5 | `u8` constants -> `enum ReadMode` | Enables `match` and removes `panic!` |
| 6 | `match` instead of `if` chain | Depends on phase 5 |
| 7 | `Result` instead of `panic!` | Depends on phase 5 |
| 8 | Generic `just_parse<T>()` | Independent, do anytime |
| 9 | Loops -> iterators | Independent |
| 10 | `Box` the large enum variant | Independent |
| 11 | `NonZeroU32` tight type | Independent |
| 12 | Remove singleton (`OnceLock`) | After `&str` migration |
| 13 | Bug fix + dead code cleanup | Correctness — must be first in phase 2 |
| 14 | Naming improvements | Before module split for cleaner result |
| 15 | Modularity (split `parse.rs`) | Before newtypes — smaller files are easier to refactor |
| 16 | Newtype pattern (`UserId`, `AssetId`) | Independent |
| 17 | Error handling (`ParseError`, `anyhow`) | Independent |
| 18 | Strategy pattern (`LogFilter` trait) | Independent |
| 19 | CLI argument parsing (`clap`) | After phase 18 (uses `LogFilter`) |
| 20 | `Display` trait for log types | Before property-based tests (enables roundtrip tests) |
| 21 | Property-based testing (`proptest`) | After phase 20 |
| 22 | Parser fluent API (stretch) | After all other phases |

**Submission workflow:**

1. Push original code as first commit
2. Push refactoring as subsequent commit(s)
3. Submit repo link in Practicum
4. Fix reviewer Issues in separate commits (title = issue number)

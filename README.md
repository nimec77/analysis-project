# analysis

Rust log analysis tool for a trading/exchange application. Parses structured log lines, filters them by mode and request ID.

## Usage

```bash
# Analyze all log entries
cargo run -- example.log

# Filter by mode (all, errors, exchanges)
cargo run -- example.log --mode errors

# Filter by request ID (comma-separated)
cargo run -- example.log --request-id 1,2

# Combine filters
cargo run -- example.log --mode exchanges --request-id 3,4

# Help
cargo run -- --help
```

## Log Format

Each log line follows the structure:

```
{System|App}::{Error|Trace|Journal} <Variant> <Payload> requestid=<N>
```

Example lines from `example.log`:

```
System::Error NetworkError "network interface is down" requestid=1
App::Error SystemError "network" requestid=1
System::Trace SendRequest "Jupiter->CreateUser{\"user_id\": \"Alice\", ...}" requestid=3
App::Journal CreateUser userid="Bob" cash=1000 requestid=4
```

**Log hierarchy:** `LogLine` → `LogKind` → `SystemLogKind` | `AppLogKind` → specific error/trace/journal variants.

**Journal events:** CreateUser, DeleteUser, RegisterAsset, UnregisterAsset, DepositCash, WithdrawCash, BuyAsset, SellAsset.

## Architecture

```
                 ┌──────────────┐
                 │  CLI (clap)  │
                 └──────┬───────┘
                        │
                 ┌──────▼───────┐
                 │   read_log() │  accepts impl LogFilter + request IDs
                 └──────┬───────┘
                        │
              ┌─────────▼──────────┐
              │   LogIterator<R>   │  generic over R: Read
              └─────────┬──────────┘
                        │
              ┌─────────▼──────────┐
              │  LogLine::parser() │  Parsable trait → Parser trait
              └─────────┬──────────┘
                        │
         ┌──────────────┼──────────────┐
         ▼              ▼              ▼
    combinators      domain          log
    (Parser trait,   (AuthData,     (LogLine,
     primitives,     AssetDsc,      LogKind,
     tuple/alt/      Backet, ...)   AppLogKind, ...)
     permutation)
```

**Key abstractions:**

- **`Parser` trait** — `fn parse(&self, input: &str) -> Result<(&str, Dest), ParseError>`. Combinators compose via structs. Fluent API: `.map()`, `.preceded_by()`, `.strip_ws()`.
- **`Parsable` trait** — Implemented by data model types. `fn parser() -> Self::Parser` constructs the parser declaratively.
- **`LogFilter` trait** — Strategy pattern for filtering. `ReadMode` provides built-in implementations (All, Errors, Exchanges).

## Build & Test

```bash
cargo build              # Build the project
cargo test               # Run all tests (42 tests across 3 modules)
cargo test test_u32      # Run a single test by name
cargo test -- --nocapture # Tests with stdout visible
cargo clippy --tests     # Lint check (must be zero warnings)
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 2 | Structured error types (`ParseError` enum) |
| `anyhow` | 1 | Ergonomic CLI error handling |
| `clap` | 4 | CLI argument parsing (derive mode) |
| `proptest` | 1 | Property-based testing (dev-dependency) |

## Project Structure

```
analysis-project/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Library: read_log(), LogIterator, LogFilter, ReadMode
│   ├── main.rs             # Binary (cli): clap CLI with --mode, --request-id
│   ├── parse.rs            # Module root: re-exports from sub-modules
│   └── parse/
│       ├── combinators.rs  # Parser trait, Parsable trait, all combinators
│       ├── domain.rs       # Domain types: AuthData, AssetDsc, Backet, UserId, etc.
│       └── log.rs          # Log types: LogLine, LogKind, SystemLogKind, AppLogKind, etc.
├── docs/
│   ├── vision.md           # Phased refactoring plan
│   ├── conventions.md      # Coding rules checklist
│   ├── tasklist.md         # Task tracking
│   └── idea.md             # Project concept
├── example.log             # Sample log file
├── CHANGELOG.md            # Detailed phase-by-phase changelog
└── CLAUDE.md               # AI assistant instructions
```

## Refactoring Phases

| Phase | Title | Description |
|-------|-------|-------------|
| 1 | `String` → `&str` in Parser | Migrated parsers from owned strings to borrowed slices |
| 2 | Remove `Rc<RefCell>` | Direct reader ownership, eliminated wrapper adapters |
| 3 | Remove `unsafe` transmute | Verified transmute removal (side-effect of Phase 2) |
| 4 | Generic `R: Read` | Static dispatch via type parameter instead of trait object |
| 5 | `u8` constants → `enum ReadMode` | Type-safe filtering mode enum |
| 6 | `match` instead of `if` chain | Exhaustive match with compiler-verified coverage |
| 7 | `Result` instead of `panic!` | Error propagation via `Result<T, io::Error>` |
| 8 | Generic `just_parse<T>()` | Single generic function replacing six wrappers |
| 9 | Loops to iterators | Idiomatic iterator chains in `read_log()` |
| 10 | Box the large enum variant | Boxed `AuthData` in `Connect` variant (1040B → 40B) |
| 11 | `NonZeroU32` tight type | Type-level "request IDs are never zero" invariant |
| 12 | Remove `OnceLock` singleton | Parser constructed per-iterator, no global state |
| 13 | Bug fix + dead code cleanup | Fixed WithdrawCash misclassification, removed 5 unused items |
| 14 | Naming improvements | Renamed combinators and functions for clarity |
| 15 | Modularity (split `parse.rs`) | Split monolith into combinators/domain/log sub-modules |
| 16 | Newtype pattern | `UserId(String)` and `AssetId(String)` newtypes |
| 17 | Error handling | `ParseError` with `thiserror`, `anyhow` for CLI |
| 18 | Strategy pattern | `LogFilter` trait, `read_log()` accepts `impl LogFilter` |
| 19 | CLI argument parsing | `clap` derive-based CLI with `--mode`, `--request-id` |
| 20 | `Display` for log types | Round-trippable `Display` impls for all types |
| 21 | Property-based testing | `proptest` roundtrip, no-panic, and suffix invariant tests |
| 22 | Parser fluent API | `.map()`, `.preceded_by()`, `.strip_ws()` chainable methods |
| 23 | Combinator macros | `impl_tuple!`, `impl_alt!`, `permutation_fn!` macros replacing hand-written arity impls |

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all tests (lib.rs and parse.rs)
cargo test test_u32      # Run a single test by name
cargo test -- --nocapture # Tests with stdout visible
cargo run -- example.log # Run CLI with a log file
```

## Project Overview

A Rust log analysis tool for a trading/exchange application. Parses structured log lines, filters them by mode and request ID. **Zero external dependencies** — uses Rust edition 2024.

### Crate Structure

- **Library crate** (`src/lib.rs`): Exports `read_log(impl Read, ReadMode, Vec<NonZeroU32>)` for parsing and filtering logs. `LogIterator<R: Read>` reads lines, skips blanks, parses each via `LogLine::parser()`.
- **Binary crate** (`src/main.rs`, bin name `cli`): Reads a log file from CLI args and prints parsed logs.
- **Parser module** (`src/parse.rs`): Hand-rolled parser combinator framework (nom-like). Defines the `Parser` trait, `Parsable` trait, and all combinators (`Tag`, `Delimited`, `Alt`, `Preceded`, `Map`, `Permutation`, `List`, `Take`, etc.). Contains all data model types and their parser implementations.

### Log Format

Each line follows: `{System|App}::{Error|Trace|Journal} <Variant> <Payload> requestid=<N>`

**Log data model hierarchy:** `LogLine` -> `LogKind` -> `{SystemLogKind, AppLogKind}` -> specific error/trace/journal variants. Journal events track exchange operations (CreateUser, RegisterAsset, BuyAsset, SellAsset, DepositCash, WithdrawCash, DeleteUser, UnregisterAsset).

**Domain types:** `AssetDsc`, `Backet`, `UserCash`, `UserBacket`, `UserBackets`, `Announcements` — each implements `Parsable` to self-construct its parser.

### Parser Architecture

- **`Parser` trait** — core: `fn parse(&self, input: &str) -> Result<(&str, Dest), ()>`. Each combinator struct implements this.
- **`Parsable` trait** — implemented by data model types. `fn parser() -> Self::Parser` constructs the parser declaratively.
- Parsers are zero-sized stateless structs — cheap to construct.
- Combinator arities via tuple impls (`all2`..`all4`, `alt2`..`alt8`, `permutation2`..`permutation3`) since Rust lacks variadic generics.
- `just_parse::<T>(input)` is the generic entry point for parsing any `Parsable` type.

### Code Conventions

- **Zero external dependencies.** No new crates in `Cargo.toml`.
- **No behavior changes.** Same input must produce same output. Refactoring changes structure, not functionality.
- **Never delete tests.** Adapting types in tests is fine; deleting test cases is not.
- Existing comments in Russian are left as-is. All new comments and documentation in **English**.
- Lines marked `// подсказка:` ("hint") indicate known technical debt left intentionally for refactoring — these are mandatory fix locations.
- One issue category = one commit.

### Phased Development

The project follows a phased refactoring plan documented in `docs/vision.md`. Each phase has associated docs in `docs/{prd,plan,research,tasklist,summaries,phase}/`. See `docs/conventions.md` for the full coding rules checklist.

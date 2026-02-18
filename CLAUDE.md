# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all tests (42 tests across 3 modules)
cargo test test_u32      # Run a single test by name
cargo test -- --nocapture # Tests with stdout visible
cargo clippy --tests     # Lint check (must be zero warnings)
cargo run -- example.log # Run CLI with a log file
cargo run -- example.log --mode errors --request-id 1,2  # CLI with filters
```

## Project Overview

A Rust log analysis tool for a trading/exchange application. Parses structured log lines, filters them by mode and request ID. Uses Rust edition 2024.

**Dependencies:** `thiserror` (structured errors), `anyhow` (CLI error handling), `clap` (CLI argument parsing). Dev: `proptest` (property-based tests).

### Crate Structure

- **Library crate** (`src/lib.rs`): Exports `read_log(impl Read, impl LogFilter, Vec<NonZeroU32>)` for parsing and filtering logs. `LogIterator<R: Read>` reads lines, skips blanks, parses each via `LogLine::parser()`. `LogFilter` trait (strategy pattern) with `ReadMode` implementing it.
- **Binary crate** (`src/main.rs`, bin name `cli`): `clap`-based CLI with `--mode` and `--request-id` arguments.
- **Parser module** (`src/parse.rs`): Re-exports from three sub-modules:
  - `parse/combinators.rs` — `Parser` trait, `Parsable` trait, all combinators and macros
  - `parse/domain.rs` — domain types (`AuthData`, `AssetDsc`, `Backet`, `UserId`, `AssetId`, `UserCash`, `UserBacket`, `UserBackets`, `Announcements`) with `Parsable` and `Display` impls
  - `parse/log.rs` — log hierarchy types (`LogLine`, `LogKind`, `SystemLogKind`, `AppLogKind`, etc.) with `Parsable` and `Display` impls

### Parser Architecture

- **`Parser` trait**: `fn parse(&self, input: &str) -> Result<(&str, Dest), ParseError>`. Each combinator struct implements this.
- **`Parsable` trait**: Implemented by data model types. `fn parser() -> Self::Parser` constructs the parser declaratively.
- **Fluent API**: `.map(f)`, `.preceded_by(prefix)`, `.strip_ws()` chain on any `Parser`.
- Parsers are zero-sized stateless structs — cheap to construct.
- Combinator arities via macros and tuple impls since Rust lacks variadic generics:
  - `impl_tuple!` — `tuple2` (with constructor), arities 3-4 (`@impl` only, trait impl without constructor)
  - `impl_alt!` — `alt2`..`alt4`, `alt8` (with constructors), arities 5-7 (`@impl` only)
  - `permutation_fn!` — `permutation2`, `permutation3` (hand-written `impl Parser` due to N!-branch matching)
- `@impl`-only macro invocations generate `impl Parser for T<(...)>` without a constructor function, avoiding dead_code warnings. To add a constructor later, change `@impl` to a named invocation.
- `just_parse::<T>(input)` is the generic entry point for parsing any `Parsable` type.

### Log Format

Each line follows: `{System|App}::{Error|Trace|Journal} <Variant> <Payload> requestid=<N>`

**Log data model hierarchy:** `LogLine` -> `LogKind` -> `{SystemLogKind, AppLogKind}` -> specific error/trace/journal variants. Journal events track exchange operations (CreateUser, RegisterAsset, BuyAsset, SellAsset, DepositCash, WithdrawCash, DeleteUser, UnregisterAsset).

**Domain types:** `AssetDsc`, `Backet`, `UserCash`, `UserBacket`, `UserBackets`, `Announcements` — each implements `Parsable` to self-construct its parser.

### Code Conventions

- **No new external dependencies** beyond what's already in `Cargo.toml`.
- **No behavior changes.** Same input must produce same output. Refactoring changes structure, not functionality.
- **Never delete tests.** Adapting types in tests is fine; deleting test cases is not.
- Existing comments in Russian are left as-is. All new comments and documentation in **English**.
- Lines marked `// подсказка:` ("hint") indicate known technical debt left intentionally for refactoring — these are mandatory fix locations.
- One issue category = one commit.

### Phased Development

The project follows a phased refactoring plan documented in `docs/vision.md`. Phases 1-22 are complete. See `docs/conventions.md` for the full coding rules checklist.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Build the project
cargo test               # Run all tests (lib.rs and parse.rs)
cargo test test_u32      # Run a single test by name
cargo run -- example.log # Run CLI with a log file
```

## Project Overview

A Rust log analysis tool for a trading/exchange application. Parses structured log lines, filters them by mode and request ID. No external dependencies — uses Rust edition 2024.

### Crate Structure

- **Library crate** (`src/lib.rs`): Exports `read_log()` for parsing and filtering logs. Defines read modes (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) and the `LogIterator` that wraps an `Rc<RefCell<Box<dyn MyReader>>>`.
- **Binary crate** (`src/main.rs`, bin name `cli`): Reads a log file from CLI args and prints parsed logs.
- **Parser module** (`src/parse.rs`): Hand-rolled parser combinator framework (nom-like). Defines the `Parser` trait, `Parsable` trait, and all combinators (`Tag`, `Delimited`, `Alt`, `Preceded`, `Map`, `Permutation`, `List`, `Take`, etc.). Contains all data model types and their parser implementations.

### Log Format

Each line follows: `{System|App}::{Error|Trace|Journal} <Variant> <Payload> requestid=<N>`

Key data types in the log model hierarchy: `LogLine` → `LogKind` → `{SystemLogKind, AppLogKind}` → specific error/trace/journal variants. Journal events track exchange operations (CreateUser, RegisterAsset, BuyAsset, SellAsset, etc.).

### Code Conventions

- Comments and documentation are in **Russian**.
- Lines marked `// подсказка:` ("hint") indicate known technical debt left intentionally for refactoring (e.g., unsafe transmute in `LogIterator::new`, unnecessary `Rc<RefCell>`, `u8` mode constants instead of enums, `panic!` in library code).
- Parser combinator arities are implemented via tuple impls (e.g., `all2`, `all3`, `all4`, `alt2`..`alt8`) since Rust lacks variadic generics.
- `LOG_LINE_PARSER` is a lazily-initialized singleton (`OnceLock`) — the single entry point for parsing log lines.

# Phase 17: Error handling (`ParseError`, `anyhow`)

**Goal:** Replace `Result<T, ()>` with a structured `ParseError` type using `thiserror`, and fix `main.rs` panics with proper error messages using `anyhow`.

## Tasks

- [ ] 17.1 Define `ParseError` enum with variants: `UnexpectedInput`, `IncompleteInput`, `InvalidValue` (each with `&'static str` context)
- [ ] 17.2 Add `thiserror = "2"` to `[dependencies]` in `Cargo.toml`
- [ ] 17.3 Replace `Result<T, ()>` with `Result<T, ParseError>` in `Parser` trait and all implementations
- [ ] 17.4 Update all `Err(())` → appropriate `ParseError` variants
- [ ] 17.5 Update all `ok_or(())` and `map_err(|_| ())` calls
- [ ] 17.6 Add `anyhow = "1"` to `[dependencies]` in `Cargo.toml`
- [ ] 17.7 Fix `main.rs`: replace `args[1]` panic with `.get(1)` + usage message
- [ ] 17.8 Fix `main.rs`: replace `.unwrap()` on file open with error message
- [ ] 17.9 Remove hardcoded demo code from `main.rs` (lines 54-58)
- [ ] 17.10 Change `main()` to `fn main() -> anyhow::Result<()>`

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- None (independent phase)

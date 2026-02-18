# Phase 19: CLI argument parsing (`clap`)

**Goal:** Replace manual `std::env::args()` parsing with `clap` derive-based CLI, supporting `--mode`, `--request-id`, positional filename, `--help`, and `--version`.

## Tasks

- [x] 19.1 Add `clap = { version = "4", features = ["derive"] }` to `[dependencies]` in `Cargo.toml`
- [x] 19.2 Define CLI struct with `#[derive(clap::Parser)]`
- [x] 19.3 Support `--mode all|errors|exchanges` (default: `all`)
- [x] 19.4 Support `--request-id 1,2,3` (optional, comma-separated)
- [x] 19.5 Positional `<filename>` argument
- [x] 19.6 Free `--help` and `--version` support
- [x] 19.7 Update `main()` to use clap-parsed args

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log && cargo run -- --help`

## Dependencies

- Phase 18 complete

# Phase 20: `Display` trait for log types

**Goal:** Implement `Display` for all log and domain types so that `main.rs` can use `{}` instead of `{:?}` for human-readable output.

## Tasks

- [x] 20.1 Implement `Display` for `LogLine`
- [x] 20.2 Implement `Display` for `LogKind`, `SystemLogKind`, `AppLogKind`
- [x] 20.3 Implement `Display` for journal variants (`AppLogJournalKind`)
- [x] 20.4 Implement `Display` for domain types (`UserId`, `AssetId`, `UserCash`, `Backet`, etc.)
- [x] 20.5 Update `main.rs` to use `{}` instead of `{:?}` for output

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log` (output should be human-readable)

## Dependencies

- Phase 15 complete

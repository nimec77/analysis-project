# Phase 16: Newtype pattern (`UserId`, `AssetId`)

**Goal:** Introduce `UserId` and `AssetId` newtype wrappers to replace raw `String` fields, improving type safety across domain and log types.

## Tasks

- [x] 16.1 Define `pub struct UserId(pub String)` with `Debug, Clone, PartialEq`
- [x] 16.2 Define `pub struct AssetId(pub String)` with `Debug, Clone, PartialEq`
- [x] 16.3 Implement `Parsable` for `UserId` (delegate to `Unquote` + `Map`)
- [x] 16.4 Implement `Parsable` for `AssetId` (delegate to `Unquote` + `Map`)
- [x] 16.5 Replace `user_id: String` -> `user_id: UserId` in `UserCash`, `UserBacket`, `UserBackets`, `AppLogJournalKind::{CreateUser, DeleteUser, RegisterAsset, UnregisterAsset}`
- [x] 16.6 Replace `asset_id: String` / `id: String` -> `AssetId` in `AssetDsc`, `Backet`, `AppLogJournalKind::{RegisterAsset, UnregisterAsset}`
- [x] 16.7 Update all parser implementations
- [x] 16.8 Update all tests

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 15 complete

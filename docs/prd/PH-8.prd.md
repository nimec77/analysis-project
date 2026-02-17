# PRD: Phase 8 — Generic `just_parse<T>()`

**Status:** PRD_READY
**Ticket:** PH-8 "Phase 8: Generic just_parse<T>()"
**Phase:** 8 of 12 (see `docs/tasklist.md`)
**Dependencies:** None (independent phase)
**Blocked by:** Nothing
**Blocks:** None directly (downstream phases are independent)

---

## Context / Idea

Phase 8 targets the elimination of code duplication in the `just_parse_*` family of public convenience functions in `src/parse.rs`. Currently there are six nearly identical wrapper functions, each calling `<T as Parsable>::parser().parse(input)` for a specific concrete type. The original author left a hint at `src/parse.rs:938`: `// подсказка: почему бы не заменить на один дженерик?` ("hint: why not replace with a single generic?"), indicating this was recognized as technical debt from the start.

### Authoritative specification from `docs/phase/phase-8.md`

**Goal:** Collapse `just_parse_u32`, `just_parse_u64`, etc. into one generic `just_parse<T: Parsable>()` function, eliminating code duplication.

**Tasks:**

- [ ] 8.1 Collapse `just_parse_u32`, `just_parse_u64`, etc. into one generic `just_parse<T: Parsable>()`

**Acceptance Criteria:** `cargo test && cargo run -- example.log`

**Dependencies:** None (independent phase)

**Implementation Notes:**

- **Hint:** `src/parse.rs:789` -- `// подсказка: почему бы не заменить на один дженерик?` (note: actual line number in current codebase is 938)

### Current codebase state (gap analysis)

The current codebase contains six public wrapper functions in `src/parse.rs` (lines 940-962), each with an identical body pattern:

```rust
// просто обёртки
// подсказка: почему бы не заменить на один дженерик?
/// Обёртка для парсинга [AssetDsc]
pub fn just_parse_asset_dsc(input: &str) -> Result<(&str, AssetDsc), ()> {
    <AssetDsc as Parsable>::parser().parse(input)
}
/// Обёртка для парсинга [Backet]
pub fn just_parse_backet(input: &str) -> Result<(&str, Backet), ()> {
    <Backet as Parsable>::parser().parse(input)
}
/// Обёртка для парсинга [UserCash]
pub fn just_user_cash(input: &str) -> Result<(&str, UserCash), ()> {
    <UserCash as Parsable>::parser().parse(input)
}
/// Обёртка для парсинга [UserBacket]
pub fn just_user_backet(input: &str) -> Result<(&str, UserBacket), ()> {
    <UserBacket as Parsable>::parser().parse(input)
}
/// Обёртка для парсинга [UserBackets]
pub fn just_user_backets(input: &str) -> Result<(&str, UserBackets), ()> {
    <UserBackets as Parsable>::parser().parse(input)
}
/// Обёртка для парсинга [Announcements]
pub fn just_parse_anouncements(input: &str) -> Result<(&str, Announcements), ()> {
    <Announcements as Parsable>::parser().parse(input)
}
```

Every function follows the exact same pattern: `<T as Parsable>::parser().parse(input)`. The only difference is the concrete type `T`. This is a textbook case for a single generic function.

**External call site:** `src/main.rs:56` calls `just_parse_anouncements`:
```rust
let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
```
This call site must be migrated to the new generic function.

**The `Parsable` trait** (lines 9-12) already provides the necessary abstraction:
```rust
trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;
    fn parser() -> Self::Parser;
}
```

However, `Parsable` is currently **not public** (no `pub` keyword). It must be made public for the generic `just_parse<T>()` to be usable from outside the module.

**No tests directly call the `just_parse_*` functions.** Tests in `parse.rs` construct parsers directly via `T::parser()` or use the specific parser structs. The only external call site is `main.rs`.

---

## Goals

1. **Introduce a single generic `just_parse<T: Parsable>()` function** that replaces all six concrete wrapper functions, eliminating the code duplication.

2. **Make the `Parsable` trait public.** The `Parsable` trait is currently `pub(crate)` (no `pub` keyword). It must be marked `pub` so that the generic `just_parse<T>()` function's trait bound is usable by external callers.

3. **Remove the six individual `just_parse_*` / `just_user_*` wrapper functions** once the generic replacement is in place.

4. **Update all call sites.** Migrate `src/main.rs` to call `just_parse::<Announcements>()` (or equivalent turbofish syntax) instead of `just_parse_anouncements()`.

5. **Remove the hint comment.** The `// подсказка: почему бы не заменить на один дженерик?` comment at line 938 and the `// просто обёртки` comment above the wrappers should be removed, as the technical debt they identify will be resolved.

6. **Preserve all existing behavior.** The generic function must produce identical results for all types that previously had dedicated wrappers.

---

## User Stories

1. **As a library consumer**, I want a single `just_parse::<T>()` function so that I can parse any `Parsable` type without having to discover and remember separate function names like `just_parse_asset_dsc`, `just_user_cash`, `just_parse_backet`, etc.

2. **As a developer adding new `Parsable` types**, I want the generic `just_parse<T>()` to work automatically for any type that implements `Parsable`, so that I do not have to write a new boilerplate wrapper function every time I add a type.

3. **As a maintainer of `src/parse.rs`**, I want to eliminate the six duplicated wrapper functions so that the codebase is smaller, easier to read, and free of unnecessary repetition.

---

## Scenarios

### Scenario 1: Generic function replaces all wrappers

**Before (six functions):**
```rust
pub fn just_parse_asset_dsc(input: &str) -> Result<(&str, AssetDsc), ()> {
    <AssetDsc as Parsable>::parser().parse(input)
}
pub fn just_parse_backet(input: &str) -> Result<(&str, Backet), ()> {
    <Backet as Parsable>::parser().parse(input)
}
pub fn just_user_cash(input: &str) -> Result<(&str, UserCash), ()> {
    <UserCash as Parsable>::parser().parse(input)
}
pub fn just_user_backet(input: &str) -> Result<(&str, UserBacket), ()> {
    <UserBacket as Parsable>::parser().parse(input)
}
pub fn just_user_backets(input: &str) -> Result<(&str, UserBackets), ()> {
    <UserBackets as Parsable>::parser().parse(input)
}
pub fn just_parse_anouncements(input: &str) -> Result<(&str, Announcements), ()> {
    <Announcements as Parsable>::parser().parse(input)
}
```

**After (one generic function):**
```rust
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
    T::parser().parse(input)
}
```

### Scenario 2: `Parsable` trait made public

**Before:**
```rust
trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;
    fn parser() -> Self::Parser;
}
```

**After:**
```rust
pub trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;
    fn parser() -> Self::Parser;
}
```

This is required because `just_parse<T: Parsable>()` is a `pub` function with a trait bound on `Parsable`. Rust requires that all types and traits referenced in a public function's signature are themselves public.

### Scenario 3: `main.rs` call site updated

**Before:**
```rust
let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
```

**After:**
```rust
let announcements = analysis::parse::just_parse::<analysis::parse::Announcements>(parsing_demo).unwrap();
```

Or with a `use` import:
```rust
use analysis::parse::{just_parse, Announcements};
let announcements = just_parse::<Announcements>(parsing_demo).unwrap();
```

### Scenario 4: Hint comments removed

**Before:**
```rust
// просто обёртки
// подсказка: почему бы не заменить на один дженерик?
```

**After:** These comments are removed. The generic function speaks for itself.

---

## Metrics

| Metric | Target |
|---|---|
| `cargo test` | All tests pass (no test cases deleted) |
| `cargo run -- example.log` | Output identical to pre-refactoring |
| Number of `just_parse_*` / `just_user_*` wrapper functions | Zero (all six removed) |
| Number of generic `just_parse` functions | Exactly one |
| `Parsable` trait visibility | `pub` |
| Lines of code removed (net) | Approximately 20 lines of boilerplate removed |
| Hint comment `подсказка: почему бы не заменить на один дженерик` | Removed |

---

## Constraints

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** The generic function must parse identically to the concrete wrappers it replaces. All existing tests must pass without modification (since tests do not call the `just_parse_*` functions directly).
3. **No test deletions.** Existing tests are not deleted or modified unless strictly necessary to accommodate the API change.
4. **Public API surface change.** The six concrete functions are removed and replaced by one generic function. The `Parsable` trait is made public. This is an intentional breaking change for any external code calling the old function names (the only known call site is `main.rs`, which is in the same crate).
5. **Scope boundary.** This phase addresses only the `just_parse_*` wrapper functions in `src/parse.rs` and their call sites. No other refactoring (e.g., loops-to-iterators, `NonZeroU32`, etc.) is in scope.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Making `Parsable` public exposes internal trait to downstream consumers | Low | Low | `Parsable` is already effectively part of the public API surface since the concrete types it is implemented on are public. Making it `pub` formalizes this and enables the generic function. The `Parser` trait may also need to be made public if the compiler requires it for the associated type bound. |
| Turbofish syntax (`just_parse::<Announcements>(...)`) is less ergonomic than the old named functions | Low | Low | The turbofish syntax is standard Rust. In many call sites, type inference will eliminate the need for explicit turbofish (e.g., when the result is assigned to a variable with a known type). The benefit of a single function outweighs the minor syntax cost. |
| The `Parser` trait may also need to be made `pub` to satisfy Rust's visibility rules for `Parsable`'s associated type | Medium | Low | If the compiler requires `Parser` to be public because `Parsable::Parser` references it in its bound, then `Parser` should also be made `pub`. This is a straightforward change with no behavioral impact. |
| Internal tests call `T::parser().parse(input)` directly and will not be affected, but if any test uses the old function names they would break | Low | Low | Grep confirms no tests call `just_parse_*` or `just_user_*` functions. The only external call site is `main.rs:56`. |

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. All six wrapper functions follow an identical pattern that maps directly to a single generic function. The only call site outside `parse.rs` is in `main.rs` and is straightforward to migrate.

---

## Files Affected

| File | Changes |
|---|---|
| `src/parse.rs` | Make the `Parsable` trait `pub` (and potentially `Parser` trait as well, if required by visibility rules). Add a single generic `pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()>`. Remove the six concrete wrapper functions (`just_parse_asset_dsc`, `just_parse_backet`, `just_user_cash`, `just_user_backet`, `just_user_backets`, `just_parse_anouncements`). Remove the associated hint comments. |
| `src/main.rs` | Update the `just_parse_anouncements(...)` call to use `just_parse::<Announcements>(...)`. |

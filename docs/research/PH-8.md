# Research: PH-8 -- Generic `just_parse<T>()`

**Ticket:** PH-8 "Phase 8: Generic just_parse<T>()"
**PRD:** `docs/prd/PH-8.prd.md`
**Phase spec:** `docs/phase/phase-8.md`

---

## 1. Resolved Questions

The PRD has no open questions. The user confirmed proceeding with default requirements only -- no additional constraints or preferences.

---

## 2. Related Modules/Services

### 2.1 `src/parse.rs` -- Primary Target

This is the sole file containing the six duplicated wrapper functions that must be collapsed into one generic function. All structural changes for this phase are concentrated here.

**Current wrapper functions (lines 940-962):**

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

Every function follows the identical pattern: `<T as Parsable>::parser().parse(input)`. The only difference is the concrete type `T`.

**Current `Parsable` trait (lines 9-12) -- NOT public:**

```rust
trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;
    fn parser() -> Self::Parser;
}
```

**Current `Parser` trait (lines 2-6) -- NOT public:**

```rust
trait Parser {
    type Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()>;
}
```

### 2.2 `src/main.rs` -- Call Site (line 56)

The only external call site for any `just_parse_*` function:

```rust
let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
```

This must be migrated to use the generic `just_parse::<Announcements>()`.

### 2.3 `src/lib.rs` -- Not Modified

The library crate root (`src/lib.rs`) re-exports the `parse` module via `pub mod parse;` and `use parse::*;`. No changes are needed here. The `read_log()` function, `LogIterator`, and tests in `lib.rs` do not reference any `just_parse_*` functions.

### 2.4 `Cargo.toml` -- Not Modified

Zero external dependencies. This phase uses only standard Rust features (trait bounds, generics). No new crates needed.

---

## 3. Current Endpoints and Contracts

### 3.1 Public API Surface Affected

| Item | Current State | After PH-8 |
|---|---|---|
| `just_parse_asset_dsc()` | `pub fn` | **Removed** |
| `just_parse_backet()` | `pub fn` | **Removed** |
| `just_user_cash()` | `pub fn` | **Removed** |
| `just_user_backet()` | `pub fn` | **Removed** |
| `just_user_backets()` | `pub fn` | **Removed** |
| `just_parse_anouncements()` | `pub fn` | **Removed** |
| `just_parse::<T>()` | Does not exist | **Added** -- `pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()>` |
| `Parsable` trait | `trait Parsable` (private) | **`pub trait Parsable`** |
| `Parser` trait | `trait Parser` (private) | **Potentially `pub trait Parser`** (see Section 5.1) |

### 3.2 Types Implementing `Parsable`

All 17 types that implement `Parsable` in `src/parse.rs`:

| Type | Line | Public? |
|---|---|---|
| `AuthData` | 726 | `pub struct` |
| `Status` | 746 | Not public (no `pub`) |
| `AssetDsc` | 772 | `pub struct` |
| `Backet` | 802 | `pub struct` |
| `UserCash` | 831 | `pub struct` |
| `UserBacket` | 859 | `pub struct` |
| `UserBackets` | 891 | `pub struct` |
| `Announcements` | 927 | `pub struct` |
| `SystemLogErrorKind` | 1034 | `pub enum` |
| `SystemLogTraceKind` | 1070 | `pub enum` |
| `SystemLogKind` | 1106 | `pub enum` |
| `AppLogErrorKind` | 1136 | `pub enum` |
| `AppLogTraceKind` | 1169 | `pub enum` |
| `AppLogJournalKind` | 1230 | `pub enum` |
| `AppLogKind` | 1362 | `pub enum` |
| `LogKind` | 1386 | `pub enum` |
| `LogLine` | 1406 | `pub struct` |

The six wrapper functions currently wrap exactly the first six public data types (`AssetDsc`, `Backet`, `UserCash`, `UserBacket`, `UserBackets`, `Announcements`). The generic `just_parse::<T>()` will work for **all** 17 types that implement `Parsable` -- a significant ergonomic improvement.

### 3.3 Test Coverage

- **18 tests pass currently** (15 in `parse.rs`, 3 in `lib.rs`).
- **No tests call any `just_parse_*` or `just_user_*` function.** Grep confirms zero references in test code.
- All parser tests use `T::parser().parse(input)` directly.
- The only external call site is `main.rs:56`.

---

## 4. Patterns Used

### 4.1 Generic Function with Trait Bound

The target pattern is a standard Rust generic function:

```rust
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
    T::parser().parse(input)
}
```

The `T::parser()` call invokes `<T as Parsable>::parser()`, which returns `T::Parser` (the associated parser type). Then `.parse(input)` invokes `<T::Parser as Parser>::parse()`, returning `Result<(&str, T), ()>`.

### 4.2 Turbofish Syntax at Call Sites

When the return type cannot be inferred, callers must use turbofish:

```rust
let result = just_parse::<Announcements>(input)?;
```

When assigned to a typed variable, inference may work:

```rust
let result: Result<(&str, Announcements), ()> = just_parse(input);
```

In the `main.rs` call site, the result is passed to `.unwrap()` with `println!("{:?}", ...)`, so turbofish will be needed.

### 4.3 Existing Public Trait Pattern

The `LogLineParser` struct (line 1426) already demonstrates a public API that internally uses `<LogLine as Parsable>::parser()`:

```rust
pub struct LogLineParser {
    parser: std::sync::OnceLock<<LogLine as Parsable>::Parser>,
}
impl LogLineParser {
    pub fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, LogLine), ()> {
        self.parser
            .get_or_init(|| <LogLine as Parsable>::parser())
            .parse(input)
    }
}
```

This compiles today even though `Parsable` and `Parser` are private, because `LogLineParser::parse()` uses concrete types in its signature (`LogLine`) rather than exposing `Parsable` as a trait bound. The generic `just_parse<T: Parsable>()` is different: it exposes `Parsable` in the trait bound of a `pub fn`, which requires `Parsable` itself to be `pub`.

---

## 5. Visibility Analysis -- Critical Implementation Detail

### 5.1 `Parsable` Must Be Made `pub`

Rust's visibility rules (RFC 136, E0445) require that all traits referenced in a public function's signature are themselves public. Since `just_parse<T: Parsable>()` is a `pub fn` with trait bound `T: Parsable`, the `Parsable` trait **must** be made `pub`.

### 5.2 `Parser` Trait -- May or May Not Need `pub`

The `Parsable` trait has an associated type with a bound on `Parser`:

```rust
pub trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;
    fn parser() -> Self::Parser;
}
```

The associated type `Parser` is bounded by `Parser<Dest = Self>`. Rust's visibility rules (E0445) may require that `Parser` also be made `pub` because it appears as a trait bound on a public associated type in a public trait. The compiler will flag this if it is required.

**Assessment:** It is likely that `Parser` will also need `pub`. This is a straightforward change with zero behavioral impact. The PRD explicitly identifies this as a known risk (Risks table, row 3).

### 5.3 Combinator Types (Map, List, Alt, etc.) Do NOT Need `pub`

The combinator types (`Map`, `Delimited`, `All`, `Permutation`, `KeyValue`, `StripWhitespace`, `List`, `Alt`, `Take`, `Tag`, `Preceded`, `Unquote`) are used in `Parsable::Parser` associated type definitions (e.g., `type Parser = Map<Delimited<...>, ...>`). However, these are **concrete type assignments** for the associated type, not part of the public function signature.

The public signature of `just_parse<T: Parsable>()` is:

```rust
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()>
```

The associated type `T::Parser` is never exposed in this signature. It is only used internally in the function body (`T::parser().parse(input)`). Therefore, the combinator types do not need to be public.

**Critical clarification:** Even though `Parsable::Parser` is a public associated type, callers of `just_parse()` never directly name or use `T::Parser`. The associated type is an implementation detail. Rust does not require that the concrete types assigned to a public associated type are themselves public -- it only requires that the *trait bound* on the associated type (`Parser<Dest = Self>`) uses public traits.

**Conclusion:** If `Parser` is made `pub`, then `Parsable::Parser: Parser<Dest = Self>` satisfies visibility rules. The concrete combinator types remain private.

---

## 6. Implementation Plan

### Step 1: Make `Parsable` trait `pub`

**File:** `src/parse.rs`, line 9

**Before:**
```rust
trait Parsable: Sized {
```

**After:**
```rust
pub trait Parsable: Sized {
```

### Step 2: Make `Parser` trait `pub` (if required by compiler)

**File:** `src/parse.rs`, line 3

**Before:**
```rust
trait Parser {
```

**After:**
```rust
pub trait Parser {
```

This should be attempted only if the compiler errors on Step 1. The PRD anticipates this may be necessary.

### Step 3: Add generic `just_parse<T: Parsable>()` function

**File:** `src/parse.rs`, insert at line 937 (replacing the wrapper block)

```rust
/// Generic wrapper for parsing any [Parsable] type.
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
    T::parser().parse(input)
}
```

### Step 4: Remove the six concrete wrapper functions

**File:** `src/parse.rs`, remove lines 937-962 (the two comment lines, six functions, and their doc comments)

Lines to remove:
- Line 937: `// просто обёртки`
- Line 938: `// подсказка: почему бы не заменить на один дженерик?`
- Lines 939-962: All six `just_parse_*` / `just_user_*` functions

### Step 5: Update `main.rs` call site

**File:** `src/main.rs`, line 56

**Before:**
```rust
let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
```

**After:**
```rust
let announcements = analysis::parse::just_parse::<analysis::parse::Announcements>(parsing_demo).unwrap();
```

Or, for readability, add a `use` import at the top of `main()` or at the module level:

```rust
use analysis::parse::{just_parse, Announcements};
// ...
let announcements = just_parse::<Announcements>(parsing_demo).unwrap();
```

The PRD shows both options in Scenario 3. Either is acceptable.

### Step 6: Verify

```bash
cargo test                # All 18 tests pass
cargo run -- example.log  # Output identical to pre-refactoring
```

---

## 7. Limitations and Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Making `Parsable` public exposes an internal trait to downstream consumers | Low | Low | `Parsable` is already effectively part of the public API since the concrete types it is implemented on are public. Making it `pub` formalizes this. |
| The `Parser` trait may also need `pub` due to visibility rules on `Parsable::Parser` associated type bound | Medium | Low | Straightforward change. PRD explicitly acknowledges this. No behavioral impact. |
| Turbofish syntax (`just_parse::<T>(...)`) is slightly less ergonomic than named functions | Low | Low | Standard Rust practice. Type inference eliminates turbofish in many contexts. The benefit of one function outweighs the syntax cost. |
| Combinator types (`Map`, `List`, etc.) might need `pub` for `Parsable::Parser` visibility | Low | Low | Analysis in Section 5.3 concludes they do NOT need `pub`. The compiler will confirm. If unexpectedly needed, they can be made `pub` without behavioral changes. |
| External code calling old `just_parse_*` names will break | Certain | Low | Only one known call site (`main.rs:56`), which is migrated in Step 5. |

---

## 8. Deviations from Requirements

**None.** The current codebase state matches the PRD's gap analysis exactly:

- Six wrapper functions exist at lines 940-962 with identical `<T as Parsable>::parser().parse(input)` bodies -- confirmed.
- `Parsable` trait is private (no `pub` keyword) at line 9 -- confirmed.
- `Parser` trait is private (no `pub` keyword) at line 3 -- confirmed.
- The hint comments `// просто обёртки` and `// подсказка: почему бы не заменить на один дженерик?` exist at lines 937-938 -- confirmed.
- The only external call site is `main.rs:56` calling `just_parse_anouncements()` -- confirmed.
- No tests call any `just_parse_*` or `just_user_*` function -- confirmed.
- All 18 existing tests pass (`cargo test`).
- `cargo run -- example.log` succeeds with expected output.

---

## 9. New Technical Questions Discovered During Research

### 9.1 Whether `Parser` Trait Needs `pub`

The `Parsable` trait has `type Parser: Parser<Dest = Self>;` which uses the `Parser` trait as a bound. When `Parsable` becomes `pub`, the compiler may require `Parser` to also be `pub` (E0445: private trait in public interface). This should be verified by compiling after making `Parsable` pub, and if the compiler errors, `Parser` should be made `pub` as well.

The PRD acknowledges this in the Risks table: "The `Parser` trait may also need to be made `pub` to satisfy Rust's visibility rules for `Parsable`'s associated type."

### 9.2 No Impact on `LogLineParser`

The existing `LogLineParser` (line 1426) uses `<LogLine as Parsable>::parser()` internally and stores the parser in an `OnceLock`. Making `Parsable` public does not change `LogLineParser`'s behavior. `LogLineParser` will remain a valid, separate way to parse `LogLine` values (with caching via `OnceLock`), while `just_parse::<LogLine>()` provides a non-cached alternative.

### 9.3 Naming Convention Consistency

The six existing functions use inconsistent naming: `just_parse_asset_dsc`, `just_parse_backet`, `just_user_cash` (missing `parse_`), `just_user_backet` (missing `parse_`), `just_user_backets`, `just_parse_anouncements`. The generic `just_parse` function eliminates this inconsistency entirely -- callers specify the type, not a function name.

---

## 10. Scope Boundaries

| Concern | PH-8 (this phase) | Other Phases |
|---|---|---|
| Collapse `just_parse_*` into generic `just_parse<T>()` | **In scope** | -- |
| Make `Parsable` trait `pub` | **In scope** | -- |
| Make `Parser` trait `pub` (if compiler requires) | **In scope** | -- |
| Remove hint comments (lines 937-938) | **In scope** | -- |
| Update `main.rs` call site | **In scope** | -- |
| Make combinator types `pub` | **Out of scope** (not expected to be needed) | -- |
| Loops to iterators in `read_log()` | Out of scope | Phase 9 |
| `NonZeroU32` for parsed numbers | Out of scope | Phase 10 |
| Remove `LogLineParser` singleton | Out of scope | Phase 11 |

---

## 11. Verification Checklist

Per the acceptance criteria:

```bash
cargo test                # All 18 tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:

| Metric | Expected |
|---|---|
| Number of `just_parse_*` / `just_user_*` wrapper functions | Zero (all six removed) |
| Number of generic `just_parse` functions | Exactly one |
| `Parsable` trait visibility | `pub` |
| `Parser` trait visibility | `pub` (if compiler required it) |
| Hint comment `подсказка: почему бы не заменить на один дженерик` | Removed |
| Comment `просто обёртки` | Removed |
| Lines of code removed (net) | Approximately 20 lines of boilerplate removed |
| External dependencies added | Zero |
| Tests modified | Zero |
| `main.rs` call site | Updated to `just_parse::<Announcements>()` |

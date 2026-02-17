# Implementation Plan: PH-8 -- Generic `just_parse<T>()`

**Status:** PLAN_APPROVED
**Ticket:** PH-8 "Phase 8: Generic just_parse<T>()"
**PRD:** `docs/prd/PH-8.prd.md`
**Research:** `docs/research/PH-8.md`
**Phase spec:** `docs/phase/phase-8.md`

---

## Components

### 1. `Parsable` trait -- Make public

**File:** `src/parse.rs`, line 9

The `Parsable` trait is currently crate-private (no `pub` keyword). It must be made `pub` because the new generic function `just_parse<T: Parsable>()` is a public function with a trait bound on `Parsable`. Rust's visibility rules (E0445) require all traits referenced in a public function's signature to be themselves public.

**Current:**
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

### 2. `Parser` trait -- Make public (if compiler requires)

**File:** `src/parse.rs`, line 3

The `Parser` trait is currently crate-private. The `Parsable` trait has an associated type bounded by `Parser<Dest = Self>`. When `Parsable` is made `pub`, Rust's visibility rules (E0445) may require that `Parser` also be `pub` because it appears as a trait bound on a public associated type in a public trait.

**Current:**
```rust
trait Parser {
    type Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()>;
}
```

**After (if compiler requires):**
```rust
pub trait Parser {
    type Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()>;
}
```

The PRD explicitly identifies this as a known risk (Risks table, row 3) and accepts it. The research (Section 5.2) confirms the assessment. This is a zero-behavioral-impact change.

**Note:** Combinator types (`Map`, `Delimited`, `Alt`, `Preceded`, `List`, `Take`, `Tag`, `Permutation`, `KeyValue`, `StripWhitespace`, `Unquote`) do NOT need to be made public. They are concrete types assigned to `Parsable::Parser` associated types, not exposed in the public function signature. See research Section 5.3 for the full analysis.

### 3. Generic `just_parse<T: Parsable>()` function -- Add

**File:** `src/parse.rs`, replacing the six wrapper functions at lines 937-962

A single generic function replaces all six concrete wrapper functions. The body is identical to the body of every existing wrapper: call the `Parsable::parser()` method to get the parser, then invoke `.parse(input)` on it.

**New function:**
```rust
/// Generic wrapper for parsing any [Parsable] type.
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
    T::parser().parse(input)
}
```

This function works for all 17 types that implement `Parsable` (not just the 6 that previously had dedicated wrappers), which is a significant ergonomic improvement.

### 4. Six concrete wrapper functions -- Remove

**File:** `src/parse.rs`, lines 937-962

The following items are removed entirely:

| Item | Lines |
|---|---|
| Comment `// просто обёртки` | 937 |
| Hint comment `// подсказка: почему бы не заменить на один дженерик?` | 938 |
| `just_parse_asset_dsc()` | 939-942 |
| `just_parse_backet()` | 943-946 |
| `just_user_cash()` | 947-950 |
| `just_user_backet()` | 951-954 |
| `just_user_backets()` | 955-958 |
| `just_parse_anouncements()` | 959-962 |

These are replaced by the single generic function from Component 3.

### 5. `main.rs` call site -- Update

**File:** `src/main.rs`, line 56

The only external call site for `just_parse_anouncements()` is migrated to the generic function using turbofish syntax.

**Current:**
```rust
let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
```

**After:**
```rust
let announcements = analysis::parse::just_parse::<analysis::parse::Announcements>(parsing_demo).unwrap();
```

The turbofish syntax is necessary because the return type cannot be inferred in this context (the result is passed to `.unwrap()` and then `println!("{:?}", ...)`).

An alternative is to add a `use` import, but the existing code uses fully qualified paths throughout `main.rs`, so the turbofish approach is consistent with the file's style.

---

## API Contract

### Before (current)

Six separate public functions with inconsistent naming:

```rust
pub fn just_parse_asset_dsc(input: &str)  -> Result<(&str, AssetDsc), ()>
pub fn just_parse_backet(input: &str)     -> Result<(&str, Backet), ()>
pub fn just_user_cash(input: &str)        -> Result<(&str, UserCash), ()>
pub fn just_user_backet(input: &str)      -> Result<(&str, UserBacket), ()>
pub fn just_user_backets(input: &str)     -> Result<(&str, UserBackets), ()>
pub fn just_parse_anouncements(input: &str) -> Result<(&str, Announcements), ()>
```

Traits `Parsable` and `Parser` are crate-private.

### After

One generic public function:

```rust
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()>
```

The `Parsable` trait is `pub`. The `Parser` trait is `pub` (if the compiler requires it for the associated type bound in `Parsable`).

This is an intentional breaking change for any external code calling the old function names. The only known external call site is `main.rs:56`, which is migrated in Task 8.4.

---

## Data Flows

```
Caller (main.rs, test, or library consumer)
  |
  |  specifies concrete type T via turbofish or type inference
  v
just_parse::<T>(input: &str) -> Result<(&str, T), ()>
  |
  |  calls T::parser()  [Parsable::parser() -> T::Parser]
  v
T::Parser  (a concrete zero-sized combinator type, e.g., Map<Delimited<...>, ...>)
  |
  |  calls .parse(input)  [Parser::parse()]
  v
Result<(&str, T), ()>
  |
  |  Ok((remaining, parsed_value))  -- success
  |  Err(())                        -- parse failure
  v
Returned to caller
```

The data flow is identical to the pre-refactoring version. The only change is that the caller specifies the type `T` as a generic parameter instead of choosing a named function. The parsing logic, combinator chain, and error handling are unchanged.

---

## NFR (Non-Functional Requirements)

| Requirement | How Met |
|---|---|
| Zero external dependencies | No new crates. Uses only standard Rust generics and trait bounds. |
| No behavior changes | `just_parse::<T>(input)` produces identical results to `just_parse_*(input)` for every `T`. Same input, same output. |
| No test deletions | No tests call the `just_parse_*` functions. All 18 existing tests are preserved unmodified. |
| Scope boundary | Only the `just_parse_*` wrapper functions in `src/parse.rs` and the `main.rs` call site are modified. No other refactoring is in scope. |
| Compiler-driven migration | After removing the six functions, the compiler identifies all call sites that need adaptation (only `main.rs:56`). |
| Hint comment resolved | The `подсказка: почему бы не заменить на один дженерик?` comment is removed because the technical debt it identifies is resolved. |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Making `Parsable` public exposes an internal trait to downstream consumers | Low | Low | `Parsable` is already effectively public -- the concrete types it is implemented on are all public. Formalizing it as `pub` enables the generic function. |
| The `Parser` trait may also need `pub` due to visibility rules on `Parsable::Parser` associated type bound | Medium | Low | Straightforward change. PRD explicitly acknowledges this as a known risk. Zero behavioral impact. The compiler will flag this if required. |
| Turbofish syntax (`just_parse::<T>(...)`) is slightly less ergonomic than named functions | Low | Low | Standard Rust practice. Type inference eliminates turbofish in many contexts. One function for all types outweighs the minor syntax cost. |
| Combinator types (`Map`, `List`, etc.) might unexpectedly need `pub` | Low | Low | Research Section 5.3 concludes they do NOT need `pub` because they are not part of the public function signature. The compiler will confirm. If unexpectedly needed, making them `pub` has no behavioral impact. |
| External code calling old `just_parse_*` names will break | Certain | Low | Only one known call site (`main.rs:56`), which is explicitly migrated in Task 8.4. |

---

## Deviations to Fix

None. The current codebase state matches the PRD's gap analysis exactly:

- Six wrapper functions exist at lines 937-962 with identical `<T as Parsable>::parser().parse(input)` bodies -- confirmed.
- `Parsable` trait is private (no `pub` keyword) at line 9 -- confirmed.
- `Parser` trait is private (no `pub` keyword) at line 3 -- confirmed.
- The hint comments `// просто обёртки` and `// подсказка: почему бы не заменить на один дженерик?` exist at lines 937-938 -- confirmed.
- The only external call site is `main.rs:56` calling `just_parse_anouncements()` -- confirmed.
- No tests call any `just_parse_*` or `just_user_*` function -- confirmed (grep returns zero hits in test code).
- All 18 existing tests pass.
- `cargo run -- example.log` succeeds.

No code deviates from requirements. No corrective tasks are needed.

---

## Implementation Tasks

### Task 8.1a: Make `Parsable` trait `pub`

**File:** `src/parse.rs`, line 9

Change `trait Parsable: Sized` to `pub trait Parsable: Sized`.

Attempt to compile. If the compiler emits E0445 requiring the `Parser` trait to also be public (because `Parsable::Parser: Parser<Dest = Self>` uses a private trait in a public interface), proceed to Task 8.1b. Otherwise, skip Task 8.1b.

### Task 8.1b: Make `Parser` trait `pub` (conditional)

**File:** `src/parse.rs`, line 3

Change `trait Parser` to `pub trait Parser`. This task is executed only if the compiler requires it after Task 8.1a. The PRD and research both anticipate this may be necessary.

### Task 8.2: Replace six wrapper functions with generic `just_parse<T: Parsable>()`

**File:** `src/parse.rs`, lines 937-962

Remove the following block entirely (lines 937-962):
- Line 937: `// просто обёртки`
- Line 938: `// подсказка: почему бы не заменить на один дженерик?`
- Lines 939-962: All six functions (`just_parse_asset_dsc`, `just_parse_backet`, `just_user_cash`, `just_user_backet`, `just_user_backets`, `just_parse_anouncements`) and their doc comments.

Replace with:
```rust
/// Generic wrapper for parsing any [Parsable] type.
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
    T::parser().parse(input)
}
```

### Task 8.3: Update `main.rs` call site

**File:** `src/main.rs`, line 56

Replace:
```rust
let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
```

With:
```rust
let announcements = analysis::parse::just_parse::<analysis::parse::Announcements>(parsing_demo).unwrap();
```

### Task 8.4: Verify

Run the acceptance criteria:

```bash
cargo test                # All 18 tests pass (no test cases deleted)
cargo run -- example.log  # Output identical to pre-refactoring
```

Additionally verify:

```bash
# Generic just_parse function exists
grep "pub fn just_parse<T: Parsable>" src/parse.rs
# Expected: one hit

# No old wrapper functions remain
grep "just_parse_asset_dsc\|just_parse_backet\|just_user_cash\|just_user_backet\|just_user_backets\|just_parse_anouncements" src/parse.rs
# Expected: zero hits

# Parsable trait is public
grep "pub trait Parsable" src/parse.rs
# Expected: one hit

# Hint comment removed
grep "подсказка: почему бы не заменить на один дженерик" src/parse.rs
# Expected: zero hits

# Comment removed
grep "просто обёртки" src/parse.rs
# Expected: zero hits

# main.rs updated
grep "just_parse::<" src/main.rs
# Expected: one hit

# No old function names in main.rs
grep "just_parse_anouncements" src/main.rs
# Expected: zero hits
```

---

## Open Questions

None. The phase specification is complete, the scope is well-defined, and the implementation path is clear. All six wrapper functions follow an identical pattern that maps directly to a single generic function. The only call site outside `parse.rs` is in `main.rs` and is straightforward to migrate. No architectural decision record is needed because there is only one viable approach -- a generic function with a `T: Parsable` bound -- and it is the obvious, canonical Rust solution.

# Tasklist: PH-8 -- Generic `just_parse<T>()`

**Status:** IMPLEMENT_STEP_OK
**Ticket:** PH-8
**PRD:** `docs/prd/PH-8.prd.md`
**Plan:** `docs/plan/PH-8.md`

---

## Context

Phase 8 collapses six nearly identical `just_parse_*` / `just_user_*` wrapper functions in `src/parse.rs` into a single generic `just_parse<T: Parsable>()` function. The `Parsable` trait (and possibly `Parser` trait) must be made public to satisfy Rust's visibility rules for the generic function's trait bound. The only external call site (`main.rs:56`) is migrated to use turbofish syntax. The hint comments identifying this technical debt are removed.

No deviations identified in the plan -- the codebase matches the PRD's "before" state exactly.

---

## Tasks

- [x] **8.1a Make `Parsable` trait `pub`**

  In `src/parse.rs`, change `trait Parsable: Sized` to `pub trait Parsable: Sized`. Attempt to compile. If the compiler emits E0445 requiring the `Parser` trait to also be public, proceed to task 8.1b.

  **Acceptance criteria:**
  1. `grep "pub trait Parsable" src/parse.rs` returns exactly one hit.
  2. If the compiler does NOT require `Parser` to be public, `cargo build` succeeds after this task alone.

- [x] **8.1b Make `Parser` trait `pub` (conditional)**

  In `src/parse.rs`, change `trait Parser` to `pub trait Parser`. This task is executed only if the compiler requires it after task 8.1a (E0445 on the associated type bound `Parsable::Parser: Parser<Dest = Self>`).

  **Acceptance criteria:**
  1. `cargo build` succeeds after making `Parsable` (and, if needed, `Parser`) public.
  2. If applied, `grep "pub trait Parser" src/parse.rs` returns exactly one hit.

- [x] **8.2 Replace six wrapper functions with generic `just_parse<T: Parsable>()`**

  In `src/parse.rs`, remove the entire block containing the six wrapper functions and their associated comments (lines 937-962):
  - Line 937: `// просто обёртки`
  - Line 938: `// подсказка: почему бы не заменить на один дженерик?`
  - `just_parse_asset_dsc()`, `just_parse_backet()`, `just_user_cash()`, `just_user_backet()`, `just_user_backets()`, `just_parse_anouncements()`

  Replace with a single generic function:
  ```rust
  /// Generic wrapper for parsing any [Parsable] type.
  pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
      T::parser().parse(input)
  }
  ```

  **Acceptance criteria:**
  1. `grep "pub fn just_parse<T: Parsable>" src/parse.rs` returns exactly one hit.
  2. Zero occurrences of `just_parse_asset_dsc`, `just_parse_backet`, `just_user_cash`, `just_user_backet`, `just_user_backets`, or `just_parse_anouncements` in `src/parse.rs`.
  3. Zero occurrences of `просто обёртки` in `src/parse.rs`.
  4. Zero occurrences of `подсказка: почему бы не заменить на один дженерик` in `src/parse.rs`.

- [x] **8.3 Update `main.rs` call site**

  In `src/main.rs`, replace the `just_parse_anouncements(...)` call with `just_parse::<analysis::parse::Announcements>(...)` using turbofish syntax, consistent with the file's existing fully-qualified-path style.

  **Acceptance criteria:**
  1. `grep "just_parse::<" src/main.rs` returns exactly one hit.
  2. Zero occurrences of `just_parse_anouncements` in `src/main.rs`.
  3. `cargo build` succeeds without errors.

- [x] **8.4 Final verification**

  Run acceptance checks to confirm the refactoring is complete and correct:
  - `cargo test` passes all tests (no test cases deleted).
  - `cargo run -- example.log` produces output identical to pre-refactoring.
  - Exactly one generic `just_parse` function exists.
  - Zero old wrapper functions remain.
  - `Parsable` trait is `pub`.
  - Hint comments are removed.
  - `main.rs` uses turbofish syntax.

  **Acceptance criteria:**
  1. `cargo test` passes all tests. `cargo run -- example.log` output is identical to pre-refactoring (success path unchanged).
  2. All PRD metrics are met: one generic `just_parse<T: Parsable>()` function, zero old wrapper functions, `Parsable` trait is `pub`, hint comment removed, `main.rs` call site updated, approximately 20 lines of boilerplate removed.

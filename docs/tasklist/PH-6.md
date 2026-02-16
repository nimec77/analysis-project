# Tasklist: PH-6 -- `match` instead of `if` chain

**Status:** IMPLEMENT_STEP_OK
**Ticket:** PH-6
**PRD:** `docs/prd/PH-6.prd.md`
**Plan:** `docs/plan/PH-6.md`

---

## Context

Phase 6 replaces the `if`/`else if`/`else` chain in `read_log()` with an exhaustive `match` expression on `ReadMode`. This eliminates the hint comment marking the debt, removes the unreachable `panic!` arm (which disappears naturally with exhaustive matching), and ensures the compiler verifies all variants are handled.

---

## Tasks

- [x] **6.1 Replace the `if`/`else if` chain with `match &mode` in `read_log()`**

  In `src/lib.rs`, remove lines 65-93 (the hint comment `// подсказка: лучше match`, the entire `if mode == ReadMode::All { ... } else if mode == ReadMode::Errors { ... } else if mode == ReadMode::Exchanges { ... } else { panic!(...) }` chain, and the panic hint comment `// подсказка: паниковать в библиотечном коде - нехорошо`). Replace with an exhaustive `match &mode` expression with three explicit arms (`ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges`), no wildcard arm.

  **Acceptance criteria:**
  - The `if`/`else if` chain is replaced with a `match &mode` expression containing three explicit arms and no wildcard/default arm (`_ =>`).
  - Each `match` arm produces the same boolean result as the corresponding `if` branch (filtering logic is identical).

- [x] **6.2 Remove the hint comment `// подсказка: лучше match`**

  The hint comment at line 65 of `src/lib.rs` marks the debt that Phase 6 resolves. It must be removed as part of the replacement.

  **Acceptance criteria:**
  - Zero occurrences of `подсказка: лучше match` in `src/lib.rs`.

- [x] **6.3 Remove the `panic!` arm and its hint comment**

  The `else { panic!("unknown mode {:?}", mode) }` arm and the comment `// подсказка: паниковать в библиотечном коде - нехорошо` are naturally eliminated by the exhaustive `match` (no default arm needed).

  **Acceptance criteria:**
  - Zero occurrences of `panic!("unknown mode` in `src/lib.rs`.
  - Zero occurrences of `подсказка: паниковать` in `src/lib.rs`.

- [x] **6.4 Preserve the Phase 9 hint comment**

  The hint `// подсказка: можно обойтись итераторами` (line 53) is Phase 9 scope and must remain untouched.

  **Acceptance criteria:**
  - Exactly one occurrence of `подсказка: можно обойтись итераторами` in `src/lib.rs`.

- [x] **6.5 Verify no API or behavior changes**

  The `ReadMode` enum retains `#[derive(Debug, PartialEq)]`. The `read_log()` function signature is unchanged. All call sites in `src/main.rs` and tests remain unchanged. No external dependencies added.

  **Acceptance criteria:**
  - `cargo build` compiles without errors.
  - `cargo test` passes all tests with no test deletions.
  - `cargo run -- example.log` produces output identical to pre-refactoring.
  - `ReadMode` enum still has `#[derive(Debug, PartialEq)]`.

# Tasklist: PH-5 -- `u8` constants -> `enum ReadMode`

**Status:** IMPLEMENT_STEP_OK
**Ticket:** PH-5 "Phase 5: `u8` constants -> `enum ReadMode`"
**PRD:** `docs/prd/PH-5.prd.md`
**Plan:** `docs/plan/PH-5.md`

---

## Context

Replace the three public `u8` mode constants (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) and the associated hint comment in `src/lib.rs` with a public `enum ReadMode` deriving `Debug` and `PartialEq`. Update the `read_log()` signature, all filtering comparisons, and every call site (`main.rs`, tests). Preserve the `if`/`else if` chain (Phase 6 scope), the `panic!` arm (Phase 7 scope), and all other hint comments.

No deviations identified in the plan -- the codebase matches the PRD's "before" state exactly.

---

## Tasks

- [x] **5.1 Define `enum ReadMode` and remove `u8` constants**
  - File: `src/lib.rs`
  - Remove the hint comment `// подсказка: лучше использовать enum и match` (line 5).
  - Remove the three Russian doc comments and the three `pub const` declarations (`READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES`) at lines 6-11.
  - Add in their place: `#[derive(Debug, PartialEq)] pub enum ReadMode { All, Errors, Exchanges }` with English doc comments on the enum and each variant.
  - **Acceptance criteria:**
    1. `src/lib.rs` contains `pub enum ReadMode` with variants `All`, `Errors`, `Exchanges`, deriving `Debug` and `PartialEq`.
    2. No `READ_MODE_ALL`, `READ_MODE_ERRORS`, or `READ_MODE_EXCHANGES` constants exist in `src/lib.rs`. The hint comment `// подсказка: лучше использовать enum и match` is removed.

- [x] **5.2a Update `read_log()` signature and filtering logic**
  - File: `src/lib.rs`
  - Change the `mode` parameter type from `u8` to `ReadMode` in the `read_log()` function signature.
  - Replace `mode == READ_MODE_ALL` with `mode == ReadMode::All`, `mode == READ_MODE_ERRORS` with `mode == ReadMode::Errors`, `mode == READ_MODE_EXCHANGES` with `mode == ReadMode::Exchanges` in the `if`/`else if` chain.
  - Update the `panic!` format string from `"unknown mode {}"` to `"unknown mode {:?}"`.
  - Preserve the `if`/`else if`/`else` chain structure (Phase 6), the `panic!` arm (Phase 7), and all other hint comments (`// подсказка: лучше match`, `// подсказка: паниковать в библиотечном коде - нехорошо`, `// подсказка: можно обойтись итераторами`).
  - **Acceptance criteria:**
    1. `read_log()` signature is `pub fn read_log(input: impl Read, mode: ReadMode, request_ids: Vec<u32>) -> Vec<LogLine>`.
    2. All three `if`/`else if` comparisons use `ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges` respectively. The `panic!` uses `{:?}` format. All other-phase hint comments are preserved.

- [x] **5.2b Adapt `main.rs` call site**
  - File: `src/main.rs`
  - Replace `analysis::READ_MODE_ALL` with `analysis::ReadMode::All` in the `read_log()` call.
  - **Acceptance criteria:**
    1. `src/main.rs` uses `analysis::ReadMode::All` instead of `analysis::READ_MODE_ALL`.
    2. `cargo build` succeeds for the binary crate (no compilation errors).

- [x] **5.3 Adapt test call sites**
  - File: `src/lib.rs` (test module)
  - Replace both `READ_MODE_ALL` references in `test_all` with `ReadMode::All`.
  - **Acceptance criteria:**
    1. All test calls use `ReadMode::All` instead of `READ_MODE_ALL`.
    2. `cargo test` passes -- all 16 tests pass with no test deletions.

- [x] **5.4 Final verification**
  - Run `cargo test` and `cargo run -- example.log` to confirm identical behavior.
  - Verify zero occurrences of `READ_MODE_ALL`, `READ_MODE_ERRORS`, `READ_MODE_EXCHANGES` in `src/`.
  - Verify zero `pub const ... u8` mode constants in `src/lib.rs`.
  - Verify the hint comment `// подсказка: лучше использовать enum и match` is removed.
  - Verify Phase 6, Phase 7, and Phase 9 hint comments are preserved.
  - **Acceptance criteria:**
    1. `cargo test` passes all tests. `cargo run -- example.log` output is identical to pre-refactoring.
    2. All PRD metrics are met: zero old constant references, enum defined, signature updated, hint removed, other hints preserved.

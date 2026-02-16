# ADR: PH-6 -- Match Ownership Strategy (`match &mode` vs. `match mode`)

**Status:** ACCEPTED
**Ticket:** PH-6 "Phase 6: `match` instead of `if` chain"
**Date:** 2026-02-16

---

## Context

The `read_log()` function takes `mode: ReadMode` by value (owned). The mode-filtering expression is inside a `for log in logs { ... }` loop, meaning the `match` expression executes on every iteration.

`ReadMode` is a simple fieldless enum with three variants (`All`, `Errors`, `Exchanges`). It derives `Debug` and `PartialEq` but does **not** derive `Copy` or `Clone`.

The current `if mode == ReadMode::All` comparisons work on every iteration because `PartialEq::eq` borrows `&self` and `&other` -- it does not consume `mode`. However, `match mode { ... }` would **move** `mode` on the first iteration, causing a compile error ("use of moved value: `mode`") on subsequent iterations.

The PRD's "After" example in Scenario 1 shows `match mode { ... }` (without `&`), but the PRD does not explicitly address the ownership concern or prescribe a specific approach.

---

## Options

### Option A: `match &mode { ... }` (match on reference)

Match on a borrowed reference to `mode`. Rust's match ergonomics (RFC 2005) allow the arm patterns to be written as `ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges` without explicit `&` prefixes, even when matching on `&ReadMode`.

**Pros:**
- Zero changes to the `ReadMode` enum definition.
- Minimal scope -- only the `if`/`else if` chain is changed.
- Idiomatic Rust pattern for matching a value used repeatedly.

**Cons:**
- The PRD's example shows `match mode`, not `match &mode`. This is a minor literal deviation, though the PRD does not explicitly require `match mode` over `match &mode`.

### Option B: Add `#[derive(Copy, Clone)]` to `ReadMode` and use `match mode { ... }`

Derive `Copy` and `Clone` on `ReadMode`, allowing the enum value to be implicitly copied on each loop iteration when matched by value.

**Pros:**
- Matches the PRD's example code literally (`match mode { ... }`).
- `Copy` is natural for simple fieldless enums and generally considered good practice.

**Cons:**
- Modifies the `ReadMode` enum definition, adding two new derives.
- Slightly exceeds the minimal scope of "replace the `if` chain with `match`" by also changing the enum's trait implementations.
- `Copy` on `ReadMode` is a public API change (downstream code could rely on `ReadMode` being non-`Copy`, e.g., for move semantics in function parameters).

---

## Decision

**Option A: `match &mode { ... }`**

This approach requires the fewest changes (only the `if`/`else if` chain is replaced), does not modify the `ReadMode` enum definition, and follows the conventions.md principle of "minimal changes per fix." The `&` in `match &mode` is a single-character addition that avoids any public API surface change.

While the PRD's example shows `match mode`, the PRD's stated goals are (1) replace the `if`/`else if` chain with `match`, (2) achieve compiler-verified exhaustiveness, and (3) preserve behavior. All three goals are met identically by both options. The PRD does not mandate `Copy` or forbid `&mode`.

---

## Consequences

- The `match` expression is `match &mode { ... }` with arm patterns `ReadMode::All`, `ReadMode::Errors`, `ReadMode::Exchanges` (no `&` prefix needed due to match ergonomics).
- `ReadMode` retains its existing `#[derive(Debug, PartialEq)]` with no additions.
- If a future phase wants to add `Copy` to `ReadMode`, it can do so independently. This decision does not prevent that.

# Phase 14: Naming improvements

**Goal:** Improve naming consistency across the parser module to better communicate intent and align with established conventions (e.g., nom naming).

## Tasks

- [ ] 14.1 Rename `All` struct → `Tuple` (matches nom's naming for sequential parsing returning a tuple)
- [ ] 14.2 Rename `all2()` → `tuple2()` and update all call sites
- [ ] 14.3 Rename `stdp` module → `primitives`
- [ ] 14.4 Rename `do_unquote()` → `unquote_escaped()`
- [ ] 14.5 Rename `do_unquote_non_escaped()` → `unquote_simple()`
- [ ] 14.6 Update all internal references and tests

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- None (standalone phase)

## Implementation Notes

Not changing: `A0/A1/A2` type params (standard tuple-impl pattern), `nz()` test helper, `AssetDsc.dsc` (matches domain key), arity suffixes (`alt2`, `permutation3`).

# Plan: Phase 3 -- Remove `unsafe` transmute

**Ticket:** Phase 3: Remove `unsafe` transmute
**Status:** PLAN_APPROVED
**PRD:** `docs/prd/phase-3.prd.md`
**Research:** `docs/research/phase-3.md`
**Phase spec:** `docs/phase/phase-3.md`

---

## 1. Summary

Phase 3 is a **verification-only phase**. The `unsafe { transmute(...) }` block that was the primary target was already removed as a side-effect of the Phase 2 refactoring (documented in `docs/phase-2-summary.md`). No source code changes are required. The phase consists of formal verification that no `unsafe` code remains, followed by project-tracking updates.

---

## 2. Components

This phase touches no source code components. The verification targets are:

| Component | File | Action |
|-----------|------|--------|
| `LogIterator::new()` | `src/lib.rs` | **Verify only.** Previously contained `unsafe { transmute }` to extend `RefMut` lifetime. Already removed in Phase 2. Current implementation uses direct ownership via `BufReader::with_capacity(4096, reader)`. |
| Parser module | `src/parse.rs` | **Verify only.** Confirm no `unsafe` or `transmute` calls. |
| CLI binary | `src/main.rs` | **Verify only.** Confirm no `unsafe` or `transmute` calls. |
| Project tracking | `docs/tasklist.md` | **Update.** Mark Phase 3 complete, advance current phase pointer. |

---

## 3. API Contract

**No API changes.** The public API (`read_log()`, `MyReader`, mode constants, `LogLine`, etc.) remains identical to its post-Phase-2 state. This phase makes zero source code modifications.

---

## 4. Data Flows

**Unchanged.** The data flow remains:

```
log file / byte stream
        |
    LogIterator  (reads lines, skips blanks -- owns Box<dyn MyReader> directly)
        |
    Parser       (parses each line into LogLine via LOG_LINE_PARSER)
        |
    read_log()   (filters by mode + request_ids)
        |
    Vec<LogLine>
```

The transmute was previously part of the `LogIterator::new()` construction step, extending a `RefMut` borrow to `'static`. That step no longer exists because `LogIterator` now takes direct ownership of the reader.

---

## 5. Implementation Tasks

### Task 1: Verify absence of `unsafe` code

**Action:** Run `grep -r "unsafe" src/` across all source files.

**Expected result:** Zero matches.

**Pre-verified in research:** Confirmed zero hits.

### Task 2: Verify absence of `transmute` calls

**Action:** Run `grep -r "transmute" src/` across all source files.

**Expected result:** Zero matches.

**Pre-verified in research:** Confirmed zero hits.

### Task 3: Verify hint comment removal

**Action:** Confirm the hint `// подсказка: unsafe избыточен, да и весь rc - тоже` is no longer present in any source file.

**Expected result:** Zero matches in `src/`. (The hint only appears in documentation files as historical references, which is correct.)

**Pre-verified in research:** Confirmed absent from all `src/` files. Removed during Phase 2.

### Task 4: Run acceptance criteria

**Action:**

```bash
cargo test && cargo run -- example.log
```

**Expected result:** All 16 tests pass. CLI output identical to pre-Phase-2 baseline. No tests deleted or modified.

### Task 5: Update `docs/tasklist.md`

**Action:** Three updates to `docs/tasklist.md`:

1. Change Phase 3 status in the progress table from `:white_circle:` to `:green_circle:`.
2. Check off task 3.1: change `- [ ]` to `- [x]` for "Replace the `unsafe { transmute(...) }` with safe code".
3. Advance "Current Phase" from `3` to `4`.

### Task 6: Create formal verification commit

**Action:** Commit the `docs/tasklist.md` update with a message noting this is a verification-only phase confirming the `unsafe` removal that occurred as a side-effect of Phase 2.

---

## 6. Non-Functional Requirements (NFRs)

| NFR | Requirement | Status |
|-----|-------------|--------|
| Zero external dependencies | No new crates in `Cargo.toml` | Met (no changes) |
| No behavior changes | Same input produces same output | Met (no code changes) |
| No test deletions | All existing tests preserved | Met (no code changes) |
| Safe Rust | Zero `unsafe` blocks in `src/` | Met (verified by grep) |
| Scope boundary | Only `unsafe`/`transmute` removal; no other debt | Met (verification only) |

---

## 7. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Work already completed in Phase 2 | **Certain** | Low | Phase 3 is a verification-and-close phase. No code changes needed. Formal verification (grep + test run) provides audit trail. |
| Hint comment still present in source | **None** (verified absent) | Low | Already removed during Phase 2. No action required. |
| `CLAUDE.md` contains stale reference to "unsafe transmute" | **Observed** | Low | Line 33 of `CLAUDE.md` mentions "unsafe transmute in `LogIterator::new`" as an example of hint-tagged technical debt. This is now stale but is **out of Phase 3 scope** per the PRD's "Files Affected" section. Can be addressed in a future housekeeping pass. |

---

## 8. Deviations to Fix

**None.** The current codebase state matches the PRD requirements exactly:

- The PRD anticipated this scenario as a "Certain" risk: "Work already completed in Phase 2 -- Phase 3 is effectively a no-op."
- The PRD's mitigation is: "Phase 3 becomes a verification-only phase. Confirm absence of `unsafe`, run tests, update tracking, and close the ticket."
- The research document confirmed all verification checks pass with zero deviations from requirements.

---

## 9. Open Questions

### Resolved: Should Phase 3 be marked as already complete?

**Resolution (from research):** Create a formal verification commit. Even though the `unsafe` code was already removed in Phase 2, Phase 3 should formally verify this (grep for `unsafe`/`transmute`, run tests), confirm no hint comments related to the `unsafe` transmute remain in source, and update `docs/tasklist.md`. This maintains a clean audit trail per phase.

### Observation: Stale reference in `CLAUDE.md`

`CLAUDE.md` line 33 mentions "unsafe transmute in `LogIterator::new`" and "unnecessary `Rc<RefCell>`" as examples of hint-tagged technical debt. Both are now resolved. Updating `CLAUDE.md` is outside Phase 3's defined scope but could be addressed in a future housekeeping pass or bundled with a later phase.

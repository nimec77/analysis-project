# Research: Phase 2 -- Remove `Rc<RefCell>`

**Ticket:** Phase 2: Remove `Rc<RefCell>`
**PRD:** `docs/prd/phase-2.prd.md`
**Phase spec:** `docs/phase/phase-2.md`

---

## 1. Existing Code Analysis

### 1.1 `RefMutWrapper` (lines 13-22 of `src/lib.rs`)

A newtype struct wrapping `std::cell::RefMut<'a, T>` that implements `std::io::Read` by delegating to the inner `RefMut`. This exists solely because `BufReader` requires `T: Read`, and `RefMut` itself does not implement `Read`. Once `Rc<RefCell>` is removed, there is no `RefMut` to wrap, so `RefMutWrapper` becomes unused.

```rust
#[derive(Debug)]
struct RefMutWrapper<'a, T>(std::cell::RefMut<'a, T>);
impl<'a, T> std::io::Read for RefMutWrapper<'a, T>
where
    T: std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}
```

**Verdict:** Will be removed entirely. No other usages exist in the codebase.

### 1.2 `MyReader` trait (lines 27-28 of `src/lib.rs`)

A supertrait combining `std::io::Read + std::fmt::Debug + 'static` to work around Rust's restriction on multiple non-auto traits in trait objects (`rustc E0225`).

```rust
pub trait MyReader: std::io::Read + std::fmt::Debug + 'static {}
impl<T: std::io::Read + std::fmt::Debug + 'static> MyReader for T {}
```

**Usages:**
- `src/lib.rs:34` -- `LogIterator.lines` field type references `Box<dyn MyReader>`
- `src/lib.rs:37` -- `LogIterator.reader_rc` field type
- `src/lib.rs:40` -- `LogIterator::new()` parameter
- `src/lib.rs:77` -- `read_log()` parameter
- `src/lib.rs:203,206` -- `test_all` variable types
- `src/main.rs:66` -- CLI variable type

**Verdict:** `MyReader` is `pub` and will still be needed after Phase 2. Even though `Rc<RefCell>` is removed, the `Box<dyn MyReader>` trait object pattern remains (the trait object is replaced with a generic `R: Read` in Phase 4, not Phase 2). The PRD task 2.3 says to remove `MyReader` only "if [it] become[s] unused." Since `Box<dyn MyReader>` will still be used in `LogIterator`, `read_log()`, `main.rs`, and tests, `MyReader` must be kept.

### 1.3 `LogIterator` struct (lines 32-63 of `src/lib.rs`)

Current definition:

```rust
struct LogIterator {
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
    reader_rc: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>,
}
```

The struct holds two fields:
1. `lines` -- an iterator chain: `Filter<Lines<BufReader<RefMutWrapper<'static, ...>>>>`. The `RefMutWrapper` borrows from the `RefCell` with a `'static` lifetime obtained via `unsafe { transmute }`.
2. `reader_rc` -- the `Rc<RefCell<Box<dyn MyReader>>>` kept alive so the `RefMut` inside `lines` does not dangle.

This is a self-referential struct pattern: `reader_rc` owns the data, and `lines` borrows from it. The field declaration order is critical -- `lines` is dropped first (releasing the `RefMut`), then `reader_rc` is dropped (releasing the `Rc`).

### 1.4 `LogIterator::new()` (lines 40-63 of `src/lib.rs`)

```rust
fn new(r: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>) -> Self {
    use std::io::BufRead;
    let the_borrow = r.borrow_mut();
    let the_borrow = unsafe { std::mem::transmute::<_, _>(the_borrow) };
    Self {
        lines: std::io::BufReader::with_capacity(4096, RefMutWrapper(the_borrow))
            .lines()
            .filter(|line_res| { ... }),
        reader_rc: r,
    }
}
```

The `transmute` on line 50 extends the lifetime of the `RefMut` from a local borrow to `'static`. This is the `unsafe` block that Phase 3 will address. However, in Phase 2 the mechanism changes fundamentally: once `Rc<RefCell>` is removed and ownership is direct, there is no `RefMut` at all, which means the `transmute` may naturally become unnecessary.

**IMPORTANT NOTE:** Per the PRD constraint (section "Scope boundary"): "This phase removes `Rc<RefCell>` only. The `unsafe { transmute }` may still be present (addressed in Phase 3)." However, in practice, once `Rc<RefCell>` is removed and `Box<dyn MyReader>` is passed directly into `BufReader`, the `RefMut` and the `transmute` both disappear because there is nothing to borrow mutably from a `RefCell` -- the owned `Box` is consumed by `BufReader`. This means the `unsafe` block will naturally be eliminated in Phase 2 as a side-effect, which is acceptable (the constraint says "may still be present," not "must still be present").

### 1.5 `read_log()` function (lines 76-129 of `src/lib.rs`)

```rust
pub fn read_log(
    input: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>,
    mode: u8,
    request_ids: Vec<u32>,
) -> Vec<LogLine> {
    let logs = LogIterator::new(input);
    ...
}
```

The signature must change to accept `Box<dyn MyReader>` instead of `Rc<RefCell<Box<dyn MyReader>>>`.

### 1.6 `test_all` (lines 202-216 of `src/lib.rs`)

```rust
let refcell1: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>> =
    std::rc::Rc::new(std::cell::RefCell::new(Box::new(SOURCE1.as_bytes())));
assert_eq!(read_log(refcell1.clone(), READ_MODE_ALL, vec![]).len(), 1);
let refcell: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>> =
    std::rc::Rc::new(std::cell::RefCell::new(Box::new(SOURCE.as_bytes())));
let all_parsed = read_log(refcell.clone(), READ_MODE_ALL, vec![]);
```

Must be simplified to pass `Box::new(...)` directly without `Rc::new(RefCell::new(...))`.

### 1.7 `main.rs` (lines 66-70 of `src/main.rs`)

```rust
let file: std::rc::Rc<std::cell::RefCell<Box<dyn analysis::MyReader>>> = std::rc::Rc::new(
    std::cell::RefCell::new(Box::new(std::fs::File::open(filename).unwrap())),
);
let logs = analysis::read_log(file.clone(), analysis::READ_MODE_ALL, vec![]);
```

Must be simplified to pass `Box::new(File::open(...).unwrap())` directly. The `.clone()` is also removed since ownership transfers.

---

## 2. Patterns Used

| Pattern | Where | Notes |
|---|---|---|
| Self-referential struct | `LogIterator` | `reader_rc` owns the data; `lines` borrows from it via `RefMut` with transmuted `'static` lifetime. Field ordering is critical. |
| Newtype wrapper | `RefMutWrapper` | Exists only to impl `Read` for `RefMut`. |
| Supertrait trait object | `MyReader` | Combines `Read + Debug + 'static` for use in `Box<dyn MyReader>`. |
| Lazy singleton | `LOG_LINE_PARSER` | In `parse.rs`. Not affected by this phase. |

---

## 3. Implementation Path

The key insight: `BufReader::new()` takes ownership of its `inner: R` argument (where `R: Read`). If `LogIterator` passes an owned `Box<dyn MyReader>` directly to `BufReader`, then `BufReader` owns the reader. There is no borrowing, no self-referential struct, no need for `RefMut`, no need for `transmute`.

### 3.1 New `LogIterator` struct

```rust
struct LogIterator {
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
```

The `reader_rc` field is removed entirely. The `lines` type changes from `BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>` to `BufReader<Box<dyn MyReader>>`.

### 3.2 New `LogIterator::new()`

```rust
fn new(reader: Box<dyn MyReader>) -> Self {
    use std::io::BufRead;
    Self {
        lines: std::io::BufReader::with_capacity(4096, reader)
            .lines()
            .filter(|line_res| {
                !line_res
                    .as_ref()
                    .ok()
                    .map(|line| line.trim().is_empty())
                    .unwrap_or(false)
            }),
    }
}
```

No `RefMut`, no `transmute`, no `RefMutWrapper`. The owned `Box<dyn MyReader>` is passed directly to `BufReader`.

### 3.3 New `read_log()` signature

```rust
pub fn read_log(
    input: Box<dyn MyReader>,
    mode: u8,
    request_ids: Vec<u32>,
) -> Vec<LogLine> {
    let logs = LogIterator::new(input);
    ...
}
```

### 3.4 Test adaptation

```rust
let reader1: Box<dyn MyReader> = Box::new(SOURCE1.as_bytes());
assert_eq!(read_log(reader1, READ_MODE_ALL, vec![]).len(), 1);
let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

### 3.5 `main.rs` adaptation

```rust
let file: Box<dyn analysis::MyReader> =
    Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

---

## 4. What Gets Removed

| Entity | File | Action |
|---|---|---|
| `RefMutWrapper` struct + `Read` impl | `src/lib.rs:13-22` | **Remove entirely** -- no longer needed without `RefCell` |
| `LogIterator.reader_rc` field | `src/lib.rs:37` | **Remove** -- no `Rc` to store |
| `unsafe { transmute }` block | `src/lib.rs:50` | **Removed as side-effect** -- no `RefMut` to transmute |
| `Rc<RefCell>` in `LogIterator::new()` parameter | `src/lib.rs:40` | **Changed to** `Box<dyn MyReader>` |
| `Rc<RefCell>` in `read_log()` parameter | `src/lib.rs:77` | **Changed to** `Box<dyn MyReader>` |
| `Rc<RefCell>` wrapping in `test_all` | `src/lib.rs:203-207` | **Simplified** to `Box::new(...)` |
| `Rc<RefCell>` wrapping in `main.rs` | `src/main.rs:66-68` | **Simplified** to `Box::new(...)` |
| Russian comments about `RefMut`/`Rc` | `src/lib.rs:42-48` | **Remove** -- no longer applicable |
| Comment on line 74 (`RefCell вообще не нужен`) | `src/lib.rs:74` | **Remove** -- debt resolved |
| Comment on line 12 (`Обёртка...`) | `src/lib.rs:12` | **Remove** -- `RefMutWrapper` gone |

## 5. What Gets Kept

| Entity | Reason |
|---|---|
| `MyReader` trait + blanket impl | Still used for `Box<dyn MyReader>` throughout. Removed in Phase 4 when generic `R: Read` replaces trait object. |
| `LogIterator` struct (modified) | Still needed; just has fewer fields. |
| `LogIterator` `Iterator` impl | Unchanged. |
| All test assertions | No test cases deleted per constraints. |
| Comment `// подсказка: вместо trait-объекта можно дженерик` (line 29) | Still relevant; addressed in Phase 4. |

---

## 6. Dependencies and Layers

```
main.rs  -->  lib.rs::read_log()  -->  LogIterator::new()  -->  BufReader
                                                             -->  Lines iterator
                                                             -->  Filter iterator
```

The change propagates from the inside out:
1. Change `LogIterator` fields and `new()` (core change)
2. Change `read_log()` signature (direct caller)
3. Change `test_all` (test caller)
4. Change `main.rs` (binary caller)

The PRD recommends a compiler-driven approach: change the `LogIterator` field first, then follow compiler errors outward.

---

## 7. Limitations and Risks

| Risk | Assessment |
|---|---|
| Self-referential struct issue | **Not a risk.** `BufReader` takes ownership of the `Box<dyn MyReader>`, so there is no self-referencing. The `lines` field contains the full ownership chain: `Filter<Lines<BufReader<Box<dyn MyReader>>>>`. |
| `unsafe` block removal as side-effect | **Low risk.** The PRD says `unsafe` "may still be present." In practice it vanishes because there is no `RefMut` to transmute. This is acceptable and simplifies the code further. Phase 3 can be marked as already resolved or reduced in scope. |
| `MyReader` usage in `main.rs` | **No risk.** `MyReader` remains `pub` and is still needed. Only the `Rc<RefCell>` wrapping around `Box<dyn MyReader>` is removed. |
| `.clone()` removal in call sites | **No risk.** `Rc::clone()` was used to pass the `Rc` to `LogIterator::new()` while retaining a copy in the outer scope. With direct ownership, `read_log()` consumes the `Box`, so `.clone()` calls are simply removed. The `Box` is moved, not cloned. |

---

## 8. Deviations from Requirements

None. The current codebase matches the PRD's description of the "before" state exactly. The implementation path described above follows the PRD scenarios precisely.

---

## 9. Resolved Questions

The PRD has no open questions. The scope is well-defined and the implementation path is mechanical.

---

## 10. New Technical Questions Discovered During Research

1. **Phase 3 overlap:** Removing `Rc<RefCell>` in Phase 2 also naturally eliminates the `unsafe { transmute }` block (Phase 3's target). After Phase 2, Phase 3 may have no remaining work. This should be communicated when Phase 3 planning begins.

---

## 11. Verification

Per the acceptance criteria:

```bash
cargo test              # All existing tests pass (no deletions)
cargo run -- example.log  # Output identical to before
```

Additionally verify:
- `grep -r "Rc\|RefCell" src/` returns no hits in `src/lib.rs` after the change
- `grep -r "RefMutWrapper" src/` returns no hits
- `grep -r "transmute" src/` returns no hits (side-effect of removing `Rc<RefCell>`)

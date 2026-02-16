# Plan: Phase 2 -- Remove `Rc<RefCell>`

**Ticket:** Phase 2: Remove `Rc<RefCell>`
**PRD:** `docs/prd/phase-2.prd.md`
**Research:** `docs/research/phase-2.md`
**Status:** PLAN_APPROVED

---

## Components

### 1. `RefMutWrapper` struct + `Read` impl (`src/lib.rs:12-22`)

A newtype wrapper around `std::cell::RefMut<'a, T>` that implements `std::io::Read` by delegating to the inner value. This struct exists solely to satisfy `BufReader`'s `T: Read` requirement when working through a `RefCell`.

**Action:** Remove entirely. Once `Rc<RefCell>` is gone, there is no `RefMut` to wrap and no need for this adapter.

### 2. `LogIterator` struct (`src/lib.rs:31-38`)

Currently a self-referential struct holding two fields:
- `lines`: a `Filter<Lines<BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>>>` iterator chain
- `reader_rc`: the `Rc<RefCell<Box<dyn MyReader>>>` kept alive to prevent the transmuted `RefMut` inside `lines` from dangling

**Action:** Remove the `reader_rc` field. Change the `lines` type parameter from `BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>` to `BufReader<Box<dyn MyReader>>`. The struct becomes a single-field wrapper around the iterator chain.

### 3. `LogIterator::new()` (`src/lib.rs:40-63`)

Currently accepts `Rc<RefCell<Box<dyn MyReader>>>`, borrows the `RefCell` mutably, transmutes the borrow lifetime to `'static`, wraps it in `RefMutWrapper`, and constructs a `BufReader`.

**Action:** Change the parameter to `Box<dyn MyReader>`. Pass the owned `Box` directly into `BufReader::with_capacity()`. Remove the `borrow_mut()` call, the `unsafe { transmute }` block, the `RefMutWrapper` wrapping, and the `reader_rc` field initialization. Remove all associated Russian comments about the borrow/Rc motivation (lines 42-48).

Note: The `unsafe { transmute }` is formally Phase 3's target, but the PRD constraint states it "may still be present" -- not "must still be present." Since there is no `RefMut` to transmute after the `Rc<RefCell>` removal, the `unsafe` block naturally disappears as a side-effect. This is acceptable.

### 4. `read_log()` function (`src/lib.rs:76-129`)

Currently accepts `Rc<RefCell<Box<dyn MyReader>>>` as the `input` parameter.

**Action:** Change the `input` parameter type to `Box<dyn MyReader>`. Pass it directly to `LogIterator::new()`. Remove the hint comment on line 74 (`// подсказка: RefCell вообще не нужен`).

### 5. `test_all` test (`src/lib.rs:202-216`)

Currently wraps byte-slice readers in `Rc::new(RefCell::new(Box::new(...)))` and passes them to `read_log()` with `.clone()`.

**Action:** Change variable types from `Rc<RefCell<Box<dyn MyReader>>>` to `Box<dyn MyReader>`. Remove `Rc::new(RefCell::new(...))` wrapping. Remove `.clone()` calls. No test assertions are deleted or changed.

### 6. `main.rs` CLI (`src/main.rs:66-70`)

Currently wraps the opened `File` in `Rc::new(RefCell::new(Box::new(...)))` and passes it to `read_log()` with `.clone()`.

**Action:** Change the variable type to `Box<dyn analysis::MyReader>`. Remove `Rc::new(RefCell::new(...))` wrapping. Remove `.clone()`. Pass the owned `Box` directly to `analysis::read_log()`.

### 7. `MyReader` trait + blanket impl (`src/lib.rs:24-28`)

A supertrait combining `Read + Debug + 'static`, used as the trait object bound throughout.

**Action:** Keep as-is. The trait remains in use for `Box<dyn MyReader>` in `LogIterator`, `read_log()`, `main.rs`, and tests. Removal is deferred to Phase 4 (generic `R: Read`). The hint comment on line 29 (`// подсказка: вместо trait-объекта можно дженерик`) is retained as it remains relevant for Phase 4.

---

## API Contract

### Before

```rust
pub fn read_log(
    input: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>,
    mode: u8,
    request_ids: Vec<u32>,
) -> Vec<LogLine>
```

### After

```rust
pub fn read_log(
    input: Box<dyn MyReader>,
    mode: u8,
    request_ids: Vec<u32>,
) -> Vec<LogLine>
```

The `MyReader` trait remains `pub`. The `read_log` function remains `pub`. The change is in the parameter type only -- consumers must pass an owned `Box<dyn MyReader>` instead of `Rc<RefCell<Box<dyn MyReader>>>`. Since the only external consumer is `main.rs` (within the same crate), this is a contained change.

---

## Data Flows

### Before

```
caller
  |  Rc<RefCell<Box<dyn MyReader>>>  (shared ownership, interior mutability)
  v
read_log()
  |  Rc<RefCell<Box<dyn MyReader>>>
  v
LogIterator::new()
  |  borrow_mut() -> RefMut
  |  transmute RefMut to 'static
  |  wrap in RefMutWrapper
  v
BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>
  |  .lines().filter(...)
  v
Filter<Lines<BufReader<RefMutWrapper<'static, ...>>>>
```

### After

```
caller
  |  Box<dyn MyReader>  (direct ownership, moved)
  v
read_log()
  |  Box<dyn MyReader>
  v
LogIterator::new()
  |  (ownership transferred directly)
  v
BufReader<Box<dyn MyReader>>
  |  .lines().filter(...)
  v
Filter<Lines<BufReader<Box<dyn MyReader>>>>
```

The key difference: ownership is linear. The `Box<dyn MyReader>` is moved into `LogIterator::new()`, then consumed by `BufReader::with_capacity()`. No shared ownership, no interior mutability, no unsafe lifetime extension.

---

## Non-Functional Requirements

| NFR | Requirement | How Satisfied |
|---|---|---|
| Zero external dependencies | No new crates in `Cargo.toml` | Only removes `std::rc::Rc` and `std::cell::RefCell` usage -- no crate changes |
| No behavior changes | Same input produces same output | Only ownership/type signatures change; filtering and parsing logic untouched |
| No test deletions | All test assertions preserved | Only type annotations and wrapping change in `test_all` |
| One commit | One issue category per commit | All changes in this phase are a single logical unit: "remove `Rc<RefCell>`" |
| Compiler-driven | Change signature, follow errors | Start with `LogIterator` fields, propagate outward to `read_log()`, tests, and `main.rs` |

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Self-referential struct issue after removing `reader_rc` | None | N/A | `BufReader` takes ownership of `Box<dyn MyReader>`. The `lines` field owns the entire chain: `Filter<Lines<BufReader<Box<dyn MyReader>>>>`. No borrowing occurs. |
| `unsafe` block removal as side-effect overlapping Phase 3 | Certain | Low (positive) | The PRD explicitly allows this: "may still be present." Phase 3 scope will be reduced accordingly. |
| `main.rs` compilation failure | Certain | Low | A known, mechanical change to the call site. Updated as part of this plan. |
| `MyReader` trait premature removal | Low | Medium | Research confirms `MyReader` is still used in 6+ locations after `Rc<RefCell>` removal. The plan explicitly retains it. |

---

## Deviations to Fix

None. The research document (section 8) confirms: "The current codebase matches the PRD's description of the 'before' state exactly." No existing code contradicts the requirements.

---

## Implementation Steps

The compiler-driven approach propagates changes from the innermost type outward.

### Step 1: Remove `RefMutWrapper` struct and its `Read` impl

**File:** `src/lib.rs`, lines 12-22

Remove the entire `RefMutWrapper` struct definition and its `impl Read` block. Also remove the introductory comment on line 12.

**Lines to remove:**
```rust
/// Обёртка, без которой не выполнено требование `std::io::BufReader<T: std::io::Read>`
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

### Step 2: Rewrite `LogIterator` struct definition

**File:** `src/lib.rs`, lines 31-38

Change the struct to a single field. Remove `reader_rc` and update the `lines` type parameter.

**Before:**
```rust
struct LogIterator {
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<RefMutWrapper<'static, Box<dyn MyReader>>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
    reader_rc: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>,
}
```

**After:**
```rust
struct LogIterator {
    lines: std::iter::Filter<
        std::io::Lines<std::io::BufReader<Box<dyn MyReader>>>,
        fn(&Result<String, std::io::Error>) -> bool,
    >,
}
```

### Step 3: Rewrite `LogIterator::new()`

**File:** `src/lib.rs`, lines 39-63

Change the parameter from `Rc<RefCell<Box<dyn MyReader>>>` to `Box<dyn MyReader>`. Remove `borrow_mut()`, `unsafe { transmute }`, `RefMutWrapper` wrapping, and the `reader_rc` field. Remove the Russian comments about the borrow motivation (lines 42-48).

**Before:**
```rust
impl LogIterator {
    fn new(r: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>) -> Self {
        use std::io::BufRead;
        // подсказка: unsafe избыточен, да и весь rc - тоже
        // примечание автора прототипа:
        // > Мотивация: хочу позаимствовать RefCell,
        // > но боюсь, что Rc протухнет - поэтому буду хранить и Rc и RefMut.
        // > Я знаю, что деструкторы полей структуры вызываются в
        // > порядке объявления в структуре - то есть сначала будет удалён
        // > мой RefMutWrapper, а уже потом и весь исходный reader_rc
        let the_borrow = r.borrow_mut();
        let the_borrow = unsafe { std::mem::transmute::<_, _>(the_borrow) };
        Self {
            lines: std::io::BufReader::with_capacity(4096, RefMutWrapper(the_borrow))
                .lines()
                .filter(|line_res| {
                    !line_res
                        .as_ref()
                        .ok()
                        .map(|line| line.trim().is_empty())
                        .unwrap_or(false)
                }),
            reader_rc: r,
        }
    }
}
```

**After:**
```rust
impl LogIterator {
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
}
```

### Step 4: Update `read_log()` signature and remove hint comment

**File:** `src/lib.rs`, lines 74-81

Remove the `// подсказка: RefCell вообще не нужен` hint comment (line 74). Change the `input` parameter type.

**Before:**
```rust
// подсказка: RefCell вообще не нужен
/// Принимает поток байт, отдаёт отфильтрованные и распарсенные логи
pub fn read_log(
    input: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>>,
    mode: u8,
    request_ids: Vec<u32>,
) -> Vec<LogLine> {
    let logs = LogIterator::new(input);
```

**After:**
```rust
/// Принимает поток байт, отдаёт отфильтрованные и распарсенные логи
pub fn read_log(
    input: Box<dyn MyReader>,
    mode: u8,
    request_ids: Vec<u32>,
) -> Vec<LogLine> {
    let logs = LogIterator::new(input);
```

### Step 5: Adapt `test_all` test

**File:** `src/lib.rs`, lines 202-208

Remove `Rc<RefCell>` wrapping and `.clone()` calls. Adapt variable types.

**Before:**
```rust
let refcell1: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>> =
    std::rc::Rc::new(std::cell::RefCell::new(Box::new(SOURCE1.as_bytes())));
assert_eq!(read_log(refcell1.clone(), READ_MODE_ALL, vec![]).len(), 1);
let refcell: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>> =
    std::rc::Rc::new(std::cell::RefCell::new(Box::new(SOURCE.as_bytes())));
let all_parsed = read_log(refcell.clone(), READ_MODE_ALL, vec![]);
```

**After:**
```rust
let reader1: Box<dyn MyReader> = Box::new(SOURCE1.as_bytes());
assert_eq!(read_log(reader1, READ_MODE_ALL, vec![]).len(), 1);
let reader: Box<dyn MyReader> = Box::new(SOURCE.as_bytes());
let all_parsed = read_log(reader, READ_MODE_ALL, vec![]);
```

### Step 6: Adapt `main.rs`

**File:** `src/main.rs`, lines 66-70

Remove `Rc<RefCell>` wrapping and `.clone()`.

**Before:**
```rust
let file: std::rc::Rc<std::cell::RefCell<Box<dyn analysis::MyReader>>> = std::rc::Rc::new(
    std::cell::RefCell::new(Box::new(std::fs::File::open(filename).unwrap())),
);

let logs = analysis::read_log(file.clone(), analysis::READ_MODE_ALL, vec![]);
```

**After:**
```rust
let file: Box<dyn analysis::MyReader> =
    Box::new(std::fs::File::open(filename).unwrap());
let logs = analysis::read_log(file, analysis::READ_MODE_ALL, vec![]);
```

### Step 7: Verify

Run the acceptance criteria:

```bash
cargo test              # All existing tests pass (no deletions)
cargo run -- example.log  # Output identical to before
```

Additional verification:
- `grep -r "Rc\|RefCell" src/lib.rs` returns no hits
- `grep -r "RefMutWrapper" src/` returns no hits
- `grep -r "transmute" src/` returns no hits

---

## Removed Hints (Technical Debt Resolved)

| Hint | Location | Resolution |
|---|---|---|
| `// подсказка: unsafe избыточен, да и весь rc - тоже` | `src/lib.rs:42` | `unsafe` and `Rc` both removed (side-effect of direct ownership) |
| `// подсказка: RefCell вообще не нужен` | `src/lib.rs:74` | `RefCell` removed entirely |

## Retained Hints (Still Relevant for Later Phases)

| Hint | Location | Target Phase |
|---|---|---|
| `// подсказка: вместо trait-объекта можно дженерик` | `src/lib.rs:29` | Phase 4 |
| `// подсказка: лучше использовать enum и match` | `src/lib.rs:4` | Phase 5 |
| `// подсказка: можно обойтись итераторами` | `src/lib.rs:83` | Phase 9 |
| `// подсказка: лучше match` | `src/lib.rs:95` | Phase 6 |
| `// подсказка: паниковать в библиотечном коде - нехорошо` | `src/lib.rs:121` | Phase 7 |

---

## Open Questions

None. The requirements are unambiguous, the research confirms no deviations, and the implementation path is mechanical.

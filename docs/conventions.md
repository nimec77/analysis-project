# Conventions

> Architecture, project structure, and refactoring order are described in [vision.md](vision.md).
> This file is a compact checklist of rules for writing and modifying code.

## Language & Style

- Existing Russian comments are left as-is. All new comments and documentation — in **English**.
- Types — `PascalCase`, functions and variables — `snake_case`, constants — `SCREAMING_SNAKE_CASE`.
- Indentation — 4 spaces, formatting — default `rustfmt`.

## Code Generation Rules

1. **Zero external dependencies.** No new crates in `Cargo.toml`.
2. **No behavior changes.** Same input — same output. Refactoring changes structure, not functionality.
3. **Follow the hints.** Every `// подсказка:` marker is a mandatory fix location. Do not skip any.
4. **Compiler drives.** Change a signature → follow compiler errors until it builds.
5. **`cargo test` after every logical step.** Tests are the safety net.
6. **Never delete tests.** Adapting types in tests is fine; deleting test cases is not.
7. **Respect dependency order.** Follow the phase table from [vision.md](vision.md) § 4 "Recommended order".
8. **One issue category = one commit.** Do not mix categories in a single commit.

## Idiomatic Rust

| Instead of | Use |
|---|---|
| `String` in parser signatures | `&str` with lifetimes |
| Manual loops with `push` | Iterators: `.filter()`, `.map()`, `.collect()` |
| `if mode == X ... else if ...` chains | `match` with exhaustive arms |
| `panic!` in library code | `Result` with error |
| `Box<dyn Trait>` (when type is known) | Generic `<R: Read>` |
| `Rc<RefCell<T>>` with a single owner | Direct ownership or `&mut` |
| `unsafe` transmute to extend lifetime | Remove the root cause (see `Rc<RefCell>`) |
| Duplicated `just_parse_*` functions | Single generic `just_parse<T: Parsable>()` |
| `OnceLock` singleton for parser | Direct parser construction (zero-sized types) |
| `u8` mode constants + runtime validation | `enum ReadMode` + `match` |
| Large enum variant `AuthData([u8; 1024])` | `Box<AuthData>` — eliminate stack bloat |
| `if value == 0 { return Err }` | `NonZeroU32::new(value).ok_or(...)` |

## What NOT to Do

- Do not add external crates.
- Do not change public API behavior.
- Do not delete test cases.
- Do not skip `// подсказка:` markers.
- Do not over-engineer beyond the task — minimal changes per fix.

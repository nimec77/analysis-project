# Phase 22: Parser fluent API (stretch)

**Goal:** Add fluent/chainable methods to the `Parser` trait so that `Parsable` implementations can be written in a more readable, builder-style syntax.

## Tasks

- [x] 22.1 Add `.map()` method to `Parser` trait as blanket extension
- [x] 22.2 Add `.preceded_by()` method
- [x] 22.3 Add `.strip_ws()` method
- [x] 22.4 Rewrite `Parsable` implementations using fluent style where it improves readability
- [x] 22.5 Example: `tag("Error").preceded_by(tag("System::")).map(|_| ...)`

## Acceptance Criteria

**Test:** `cargo test && cargo run -- example.log`

## Dependencies

- Phase 15 complete

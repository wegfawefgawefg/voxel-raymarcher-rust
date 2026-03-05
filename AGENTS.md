# AGENTS.md

## Scope

These instructions apply to this repository.

## Style Direction

- Prefer a pragmatic "C+-style" in Rust: explicit control flow, straightforward data movement, minimal magic.
- Keep Rust-idiomatic safety and ownership rules intact; do not force C-style patterns when they fight the language.
- Prefer simple, readable code over clever abstractions.

## File Size and Layout

- Target file size: roughly 300-500 lines when practical.
- If a file grows past ~500 lines, split by coherent responsibility (rendering, world, input, etc.), not by arbitrary type visibility.
- Small files are fine when a module is naturally small; do not pad files to hit a line count.

## Module Organization

- Keep module layout flat under `src/`.
- Avoid nested module trees unless there is a clear, strong need.
- Favor one-level `mod` declarations from `main.rs` (or `lib.rs` if introduced).

## Grouping Conventions

- Keep related structs/enums/functions together in the same file.
- Prefer this order inside files:
  1. imports and constants
  2. core types (`struct`/`enum`)
  3. primary public functions/methods
  4. internal helpers
- Keep hot-path logic close to its data definitions where it improves maintainability.

## Abstraction and API Rules

- Avoid deep trait hierarchies and unnecessary generics in core rendering paths.
- Prefer concrete types and explicit data layouts.
- Minimize hidden control flow and macro-heavy patterns.
- Avoid deeply nested control flow in functions; prefer early `continue`/`return` and helper extraction when nesting exceeds about 3 levels.

## Performance Mindset

- Treat data layout and cache behavior as first-class concerns.
- Prefer contiguous storage and predictable access patterns.
- Avoid avoidable allocations in frame/hot loops.
- Profile before and after major performance work (`perf`, `inferno-flamegraph`).

## Change Hygiene

- Preserve existing behavior unless a change is intentional and documented.
- Keep comments short and technical; explain non-obvious decisions.
- Run `cargo check` after code changes.

---
id: 0005-schema-json-skeleton
status: draft
scope: [src/schema.rs, src/main.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0001-mcpv-foundation]
---
# Task: schema json skeleton

## Plan

- [ ] Add a pure object-skeleton helper using defaults/examples/enum values, then simple property type placeholders; use {} as fallback.
- [ ] Bound recursion and handle nested objects/arrays, missing properties, unsupported composition/types, and references without network resolution or panic.
- [ ] Unit-test precedence, primitives, nested properties, arrays, empty/malformed schemas, unsupported references/composition, and recursion bounds.

## Context

Return valid object-root JSON. Skeletons are editable suggestions, not a validator or a promise of server acceptance. Keep the helper independent of UI/SDK state.

## Summary

Generate editable JSON argument skeletons with unit coverage.

## Acceptance

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `specdev check`

## Log

- 2026-10-13 created: schema json skeleton
- 2026-10-13 scope add src/schema.rs — MVP implementation slice
- 2026-10-13 scope add src/main.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0001-mcpv-foundation

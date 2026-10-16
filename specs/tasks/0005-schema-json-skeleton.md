---
id: 0005-schema-json-skeleton
status: approval
scope: [src/schema.rs, src/lib.rs]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
depends: [0001-mcpv-foundation]
---
# Task: schema json skeleton

## Plan

- [x] Expose a pure library helper object_skeleton(&serde_json::Value) -> serde_json::Value: prefer default, first examples value, then first enum value; always return an object or {}.
- [x] Otherwise include all declared properties, infer objects from properties, and use empty string/zero/false/empty array/null placeholders. Stop schema recursion at 16 levels; unresolved references, compositions, malformed schemas, and unsupported types yield null within properties or {} at root. Never resolve references or validate generated suggestions.
- [x] Unit-test precedence, primitives, nested properties, arrays, empty/malformed schemas, unsupported references/composition, and recursion bounds.

## Context

Return valid object-root JSON. Skeletons are editable suggestions, not a validator or a promise of server acceptance. Keep the helper independent of UI/SDK state.

## Summary

Generate editable JSON argument skeletons with unit coverage.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all-targets --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `specdev check`

## Review

Approved against the plan and two-file code scope. The public library helper is independent of SDK/UI state and leaves input unchanged. Nine schema tests cover precedence (including null/false/zero), nested required/optional properties, array/literal suggestions, malformed inputs, references/compositions/unions, root-object fallback, and depth bounds. All 45 ordinary project tests and quality gates pass; the intentional terminal panic probe remains ignored. Explicit values are copied without validation; arrays otherwise start empty, and optional properties are included. No UI behavior, dependencies, network calls, or unwrap/expect calls added. The existing future docs/demo.md scope warning remains.

## Log

- 2026-10-13 created: schema json skeleton
- 2026-10-13 scope add src/schema.rs — MVP implementation slice
- 2026-10-13 scope add src/main.rs — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends 0001-mcpv-foundation
- 2026-10-16 scope rm src/main.rs
- 2026-10-16 scope add src/lib.rs — Expose the pure schema helper for later editor integration without unused binary code
- 2026-10-16 advance draft → ready
- 2026-10-16 scope approved: src/schema.rs, src/lib.rs
- 2026-10-16 advance ready → in-progress/implement
- 2026-10-16 advance in-progress/implement → in-progress/verify
- 2026-10-16 advance in-progress/verify → in-progress/review
- 2026-10-16 advance in-progress/review → approval

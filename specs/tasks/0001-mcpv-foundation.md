---
id: 0001-mcpv-foundation
status: approval
scope: [Cargo.toml, Cargo.lock, rust-toolchain.toml, src/**, tmp/legacy-redb/**, README.md, .gitignore, notes.org, redb_tui.png]
created: 2026-10-13
source: User mcpv MVP brief; Rust 2024 and toolchain 1.99
---
# Task: mcpv foundation

## Plan

- [x] Archive old sources, screenshot, and notes in tmp/legacy-redb; snapshot the manifest/README before replacement and preserve unrelated settings/tooling.
- [x] Create mcpv 0.1.0, edition 2024, rust-version 1.99, toolchain 1.99.0 with rustfmt/clippy; add the approved client dependencies and track Cargo.lock.
- [x] Add main/cli/app/error modules, one required URL argument, standard help/version, and a typed feature-unavailable error on normal launch; update README and test CLI parsing/exit behavior.

## Context

Dependencies: ratatui 0.30.2, crossterm 0.29, clap 4 derive, tokio 1 macros/rt-multi-thread, serde_json 1, thiserror 2; rmcp =3.5.1 with default features disabled and client/transport-streamable-http-client-reqwest enabled. URL validation belongs to task 0002. Missing/extra arguments exit 2; help/version exit 0; normal launch reports pending connection/UI and exits 1. No network or terminal side effects.

The nine-file scope warning is justified by this approved project replacement; keep global quality limits unchanged. Preserve .tmuxp.yaml, skills, AGENTS.md, Justfile, and rustfmt settings.

## Summary

Replace the database project with the mcpv Rust foundation.

## Acceptance

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test --all-targets`
- [x] `specdev check`

## Review

Approved against the plan and declared scope. Three CLI tests pass; executable smoke checks confirm help/version 0, missing/extra arguments 2, and feature-unavailable launch 1 with a readable stderr message. cargo build --locked passes. Cargo metadata exposes only the new mcpv binary target. cmp against git HEAD verifies all eight archived files byte-for-byte. New source has no unwrap/expect, network calls, terminal operations, or file creation. specdev check has zero errors and five documented warnings: the approved replacement scope size and four future paths in other drafts. Unrelated .tmuxp.yaml and skill files remain untouched.

## Log

- 2026-10-13 created: mcpv foundation
- 2026-10-13 scope add Cargo.toml — MVP implementation slice
- 2026-10-13 scope add Cargo.lock — MVP implementation slice
- 2026-10-13 scope add rust-toolchain.toml — MVP implementation slice
- 2026-10-13 scope add src/** — MVP implementation slice
- 2026-10-13 scope add tmp/legacy-redb/** — MVP implementation slice
- 2026-10-13 scope add README.md — MVP implementation slice
- 2026-10-13 set source User mcpv MVP brief; Rust 2024 and toolchain 1.99
- 2026-10-13 set depends ""
- 2026-10-14 scope add .gitignore — Approved replacement archives obsolete artifacts and tracks the executable lockfile
- 2026-10-14 scope add notes.org — Approved replacement archives obsolete artifacts and tracks the executable lockfile
- 2026-10-14 scope add redb_tui.png — Approved replacement archives obsolete artifacts and tracks the executable lockfile
- 2026-10-14 advance draft → ready
- 2026-10-14 scope approved: Cargo.toml, Cargo.lock, rust-toolchain.toml, src/**, tmp/legacy-redb/**, README.md, .gitignore, notes.org, redb_tui.png
- 2026-10-14 advance ready → in-progress/implement
- 2026-10-14 baseline: .agents/skills/specdev/SKILL.md@9360e7cb2894, .agents/skills/specdev/references/examples.md@51bf9e0996b0, .agents/skills/specdev/specdev.meta@3330caadc168, .tmuxp.yaml@fe881ba45ed6, AGENTS.md@366e4f0189c2, specdev.toml@492c78bd02fd
- 2026-10-14 advance in-progress/implement → in-progress/verify
- 2026-10-14 advance in-progress/verify → in-progress/review
- 2026-10-14 advance in-progress/review → approval

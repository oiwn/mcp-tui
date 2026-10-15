# AGENTS.md

## File editing

Use apply_patch or another purpose-built file editing tool to edit files. Do not use Python scripts for file editing. Use specdev commands for task state, frontmatter, logs, and indexes.

<!-- BEGIN specdev -->
## specdev

This project uses **specdev** (specification-driven development). Load the
specdev skill when starting a session, continuing from specs, or picking up a
task. Always read `specs/overview.md` and `specs/ctx.md` before coding.
<!-- END specdev -->

## MCP server inspection

Use the repo-local mcpv skill at .agents/skills/mcpv/SKILL.md when inspecting, debugging, or retrieving information from MCP servers through the headless CLI.


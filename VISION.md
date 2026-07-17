# Vision

Build a simple CLI for visualizing the instruction files that configure coding agents and showing how those instructions are combined into the final context an agent receives.

Agent harnesses discover and compose files such as `CLAUDE.md`, `AGENTS.md`, and `CLAUDE.local.md`. Those files may come from global configuration, a project, nested directories, or local overrides. The resulting instruction set is often difficult to inspect, and each harness applies its own discovery and precedence rules.

The CLI should make that process visible. Given the current project or a target path, it should show:

- Which instruction files were discovered
- Where each file came from
- The order in which the files are applied
- Which instructions are global, project-specific, directory-specific, or local
- How overrides and precedence affect the final composed instructions
- The final instruction context produced for a selected harness

Harness-specific options could include:

```text
--claude
--opencode
--codex
```

Selecting a harness should apply that tool's real instruction-discovery and composition behavior, making it easy to compare how different harnesses interpret the same repository.

The initial implementation should remain small and focused: a fast, readable command-line tool, probably written in Rust. It should prioritize an accurate explanation of composition over editing or managing instruction files.

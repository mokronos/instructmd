# Harness instruction-loading references

One file per coding agent `instructmd` supports, documenting **exactly which
instruction/memory files each agent loads at startup, which it does not, and
why** — with links to the official documentation and source that each claim
rests on.

These notes are the specification `instructmd`'s resolver (`cli/src/lib.rs`) is
modelled against. They describe *initial startup resolution only*: lazy
subtree discovery, conditional/glob rules, imports, and context budgets are
noted where they exist but are not what the CLI simulates.

Research snapshot: **2026-07-19**. Where a claim rests on source that can change
without a doc update, it is marked **source-observed** and pinned to a commit.

| Agent | File | Native project file | Claude/AGENTS compatibility | Env switch that changes loading |
|---|---|---|---|---|
| OpenCode | [opencode.md](opencode.md) | `AGENTS.md` | reads `CLAUDE.md` fallback | `OPENCODE_DISABLE_CLAUDE_CODE`, `OPENCODE_DISABLE_CLAUDE_CODE_PROMPT`, `OPENCODE_DISABLE_PROJECT_CONFIG` |
| Claude Code | [claude-code.md](claude-code.md) | `CLAUDE.md` | does **not** read `AGENTS.md` natively | `CLAUDE_CODE_SAFE_MODE`, `CLAUDE_CODE_SIMPLE`, `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD` |
| Codex CLI | [codex.md](codex.md) | `AGENTS.md` / `AGENTS.override.md` | no `CLAUDE.md` unless configured | `CODEX_HOME` (no disable var; use `project_doc_max_bytes=0`) |
| pi | [pi.md](pi.md) | `AGENTS.md` | reads `CLAUDE.md` fallback | `PI_CODING_AGENT_DIR` (no disable var; use `--no-context-files`) |
| Gemini CLI | [gemini.md](gemini.md) | `GEMINI.md` | no `CLAUDE.md`/`AGENTS.md` unless configured | `GEMINI_SYSTEM_MD`, `GEMINI_CLI_TRUST_WORKSPACE` |
| Amp | [amp.md](amp.md) | `AGENTS.md` | reads `AGENT.md`/`CLAUDE.md` fallback | none |
| goose | [goose.md](goose.md) | `.goosehints` + `AGENTS.md` | via `CONTEXT_FILE_NAMES` | `CONTEXT_FILE_NAMES`, `GOOSE_PATH_ROOT` |
| Qwen Code | [qwen.md](qwen.md) | `QWEN.md` + `AGENTS.md` | via `context.fileName` | `QWEN_SYSTEM_MD` (no disable var; use `--safe-mode`) |

## Conventions in these files

- **Loaded at startup** — files pulled into context when the agent launches in a
  directory, in load order.
- **Loaded lazily** — files the agent only adds later, when it reads inside a
  subtree. `instructmd` does not simulate these.
- **Not loaded (and why)** — files that exist on disk but are deliberately
  skipped: candidate shadowing, deprecation, env/flag exclusion, or simply not
  recognised by that agent.
- **source-observed** — behavior confirmed in the implementation but not (or not
  fully) promised by the published docs; may drift between releases.

# Codex CLI

**Native project file:** `AGENTS.md` (and per-directory `AGENTS.override.md`) · **No `CLAUDE.md` unless configured**

Research snapshot: **2026-07-19**, `openai/codex` `main` (Rust `codex-rs`). The
loader was renamed `project_doc.rs` → `agents_md.rs` (April 2026, commit
`ab97c9aa`). Docs at `developers.openai.com/codex/*` now redirect to
`learn.chatgpt.com/docs/*`.

## Loaded at startup

1. **Global (user-level), from `$CODEX_HOME`** (default `~/.codex`): candidates
   tried in order `AGENTS.override.md`, then `AGENTS.md`. The **first non-empty**
   (after `trim()`) regular file wins; the other is ignored. **No byte cap**
   applies to the global file.
   ([codex-home/src/instructions/mod.rs](https://github.com/openai/codex/blob/main/codex-rs/codex-home/src/instructions/mod.rs))
   Docs: *"Codex uses only the first non-empty file at this level."*
   ([agents-md guide](https://developers.openai.com/codex/guides/agents-md))
2. **Project root detection**: walk up from cwd to the nearest ancestor holding
   any `project_root_markers` entry (default `[".git"]`). If no marker is found,
   **only the cwd** is scanned. Traversal never goes above the project root.
   ([config/src/project_root_markers.rs](https://github.com/openai/codex/blob/main/codex-rs/config/src/project_root_markers.rs))
3. **Per-directory scan, project root → cwd (inclusive)**: in each directory the
   candidate order is `AGENTS.override.md`, `AGENTS.md`, then each
   `project_doc_fallback_filenames` entry. **At most one file per directory**
   (first existing regular file wins). All found files concatenate root-first;
   whitespace-only files are skipped.
   ([core/src/agents_md.rs](https://github.com/openai/codex/blob/main/codex-rs/core/src/agents_md.rs))
4. **Assembly**: global instructions first, then project docs, joined at the
   user→project transition by `"\n\n--- project-doc ---\n\n"`.

## Not loaded (and why)

| File | Why not |
|---|---|
| `CLAUDE.md`, `codex.md`, `.codex.md`, `CODEX.md` | **Not recognised by default.** The legacy TS-era fallback names are gone from the Rust CLI. Add them via `project_doc_fallback_filenames` to load them. |
| `AGENTS.md` in a directory that also has `AGENTS.override.md` | One file per directory; `AGENTS.override.md` is tried first and wins in its own directory. |
| Any ancestor above the project root | Traversal stops at the first `project_root_markers` match; parents above it are never scanned. |
| **All** project docs | `project_doc_max_bytes = 0` short-circuits with no project docs (`if max_total == 0 { return Ok(None) }`). Also `project_root_markers = []` restricts the scan to cwd only. |
| Project-doc bytes beyond the budget | `project_doc_max_bytes` (default **32 KiB**) is a budget **across all project docs combined** (global file exempt). Read root→cwd; a file exceeding the remaining budget is **truncated** (with a warning), then later files are skipped once budget hits 0. |

## Environment variables

| Variable | Effect | Parsing |
|---|---|---|
| `CODEX_HOME` | Overrides the config/instructions home (default `~/.codex`) — where global `AGENTS.override.md`/`AGENTS.md` and `config.toml` live. | Set-and-non-empty; empty string ignored. Path must exist and be a directory or startup errors. ([utils/home-dir/src/lib.rs](https://github.com/openai/codex/blob/main/codex-rs/utils/home-dir/src/lib.rs)) |
| `CODEX_DISABLE_PROJECT_DOC` | **Does NOT exist in the Rust CLI** (zero code hits on `main`). It existed only in the **legacy TypeScript CLI** (2025), where `!== "1"` gated loading of the project doc (then `codex.md`) — not global instructions. | Legacy only: exact `"1"`. |

No other env var affects instruction loading in the current loader.

## Config keys (`config.toml`)

| Key | Default | Behavior |
|---|---|---|
| `project_doc_max_bytes` | `32768` (32 KiB) | Combined budget across project docs (global file exempt). `0` disables project docs entirely. |
| `project_doc_fallback_filenames` | `[]` | Extra per-directory candidate names, tried **after** `AGENTS.override.md`/`AGENTS.md`. Trimmed; empties/dupes dropped. |
| `project_root_markers` | `[".git"]` | Marker names for root detection. `[]` disables upward traversal (cwd only). Project-layer config is ignored for this key. |
| `model_instructions_file`, `instructions`, `developer_instructions` | — | Override *model*/developer instructions; **do not** change `AGENTS.md` discovery. The historical `experimental_instructions_file` no longer exists. |

## Flags

- **No `--no-project-doc` / `--project-doc` in the Rust CLI** (those were legacy
  TypeScript flags). Disable project docs via `codex -c project_doc_max_bytes=0`.
- `--cd/-C` changes the cwd anchor, hence discovery.

## Sources

- [AGENTS.md guide](https://developers.openai.com/codex/guides/agents-md) → [learn.chatgpt.com](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
- [`agents_md.rs`](https://github.com/openai/codex/blob/main/codex-rs/core/src/agents_md.rs)
- [`codex-home/src/instructions/mod.rs`](https://github.com/openai/codex/blob/main/codex-rs/codex-home/src/instructions/mod.rs)
- [`project_root_markers.rs`](https://github.com/openai/codex/blob/main/codex-rs/config/src/project_root_markers.rs)
- [`config_toml.rs`](https://github.com/openai/codex/blob/main/codex-rs/config/src/config_toml.rs)
- [`utils/home-dir/src/lib.rs`](https://github.com/openai/codex/blob/main/codex-rs/utils/home-dir/src/lib.rs)

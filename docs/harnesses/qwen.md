# Qwen Code

**Native project files:** `QWEN.md` **and** `AGENTS.md` (both, by default) · **local:** `.qwen/QWEN.local.md`

Research snapshot: **2026-07-19**, `QwenLM/qwen-code` `main` (a Gemini CLI fork).
Discovery is in `packages/core/src/utils/memoryDiscovery.ts`; filename constants
in `packages/core/src/memory/const.ts`. The default active list is
`[QWEN.md, AGENTS.md]` — Qwen Code looks for **both** by default. Docs confirm:
*"If your repository already has an `AGENTS.md` file for other AI tools, Qwen
reads that too."*

## Loaded at startup

1. **Global** — `~/.qwen/QWEN.md` (the global `~/.qwen` dir is checked for each
   active filename, so `~/.qwen/AGENTS.md` is also eligible).
2. **Upward walk** — from cwd upward, checking each directory for the active
   filenames; found files are `unshift`ed, so the final order reads
   **project-root → … → cwd**. The walk stops at the directory above the project
   root (`.git` boundary via `findProjectRoot`), falling back to the directory
   above home when there is no project root, and breaks on reaching `~/.qwen`.
   If cwd *is* home, only a direct check in home is done (no walk).
3. **Extension context files** — appended.
4. **`<projectRoot>/.qwen/QWEN.local.md`** — appended after hierarchical
   discovery, only when a real project root exists, implicit discovery is
   enabled, folder trust is granted, and the file is readable.

There is **no downward/subdirectory BFS scan** in current source (unlike older
Gemini CLI — no `bfsFileSearch`, no `memoryDiscoveryMaxDirs`). Imports use
`@path/to/file` relative to the containing file (`context.importFormat`, default
`'tree'`).

## Not loaded (and why)

| File | Why not |
|---|---|
| `CLAUDE.md` | Not a default context filename. Add it via `context.fileName` — but that key **replaces** the default list, so use `["QWEN.md","AGENTS.md","CLAUDE.md"]` to keep the defaults. |
| Workspace files / `QWEN.local.md` (untrusted folder) | The upward scan and `QWEN.local.md` are skipped when the folder is **untrusted** (`security.folderTrust`); global/home files still load. |
| Everything (customizations) | `--safe-mode` disables all context files, hooks, extensions, skills, and MCP servers. |
| `QWEN.local.md` / implicit walk | `--bare` skips the implicit startup auto-discovery gate that the upward scan and `QWEN.local.md` depend on. |
| Ancestors above the project root | The walk stops one directory above the `.git` project root (or above home). |
| Memory in include-directories | Loaded only with `context.loadFromIncludeDirectories: true` (default `false`). |

## Environment variables

| Variable | Effect |
|---|---|
| `QWEN_SYSTEM_MD` | Overrides the **base system prompt** (not `QWEN.md` context) with a markdown file: `true`/`1` → `.qwen/system.md`; `false`/`0` → disabled; any other value → a file path. Errors if missing. |
| `QWEN_WRITE_SYSTEM_MD` | Writes the generated base system prompt to a file. |
| `QWEN_CODE_SYSTEM_SETTINGS_PATH` / `QWEN_CODE_SYSTEM_DEFAULTS_PATH` | Relocate system settings/defaults files (indirectly affect context config). |
| — | **No env var disables `QWEN.md`/context loading** (nothing like `QWEN_MD=0`). Use `--safe-mode` or trust settings instead. |

## Settings keys (`settings.json`)

Locations (precedence low→high): `/etc/qwen-code/system-defaults.json` →
`~/.qwen/settings.json` → `.qwen/settings.json` → `/etc/qwen-code/settings.json`.

| Key | Behavior |
|---|---|
| `context.fileName` | String or array. **Replaces** the default list (falls back to `QWEN.md` only if all entries are blank). To add `CLAUDE.md` while keeping defaults, list all three. |
| `context.importFormat` | `'flat'` \| `'tree'` (default `'tree'`). |
| `context.includeDirectories` / `context.loadFromIncludeDirectories` | Extra workspace dirs; the latter (default `false`) controls whether memory loads from them. |
| `security.folderTrust.enabled` | Untrusted folders skip workspace context files. |
| — | **No dedicated "disable context loading" key** (and no `discoveryMaxDirs`). |

## CLI flags

- `--safe-mode` — disables all customizations incl. context files (the off-switch).
- `--bare` — skips implicit startup auto-discovery (gates the upward scan + `QWEN.local.md`).
- `--include-directories` (alias `--add-dir`) — adds workspace dirs (memory only with the setting above).
- `--extensions` / `--list-extensions` — control active extensions (hence extension context files).
- No `--memory` or context-filename flag.

## Uncertain / undocumented

- Relative order of extension context files vs `QWEN.local.md` in the final concatenation.
- Whether `--bare` still loads the global `~/.qwen/QWEN.md`.
- Docs surfaced keys `memory.enableManagedAutoMemory` / `memory.enableTeamMemory`
  that did not appear in the settings reference or source — treat as unverified.

## Sources

- [Memory guide](https://qwenlm.github.io/qwen-code-docs/en/users/features/memory/)
- [Settings/configuration](https://qwenlm.github.io/qwen-code-docs/en/users/configuration/settings/)
- [`memory/const.ts`](https://raw.githubusercontent.com/QwenLM/qwen-code/main/packages/core/src/memory/const.ts)
- [`memoryDiscovery.ts`](https://raw.githubusercontent.com/QwenLM/qwen-code/main/packages/core/src/utils/memoryDiscovery.ts)
- [`config.ts`](https://raw.githubusercontent.com/QwenLM/qwen-code/main/packages/cli/src/config/config.ts)
- [`prompts.ts` (system MD env vars)](https://raw.githubusercontent.com/QwenLM/qwen-code/main/packages/core/src/core/prompts.ts)

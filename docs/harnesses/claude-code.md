# Claude Code

**Native project file:** `CLAUDE.md` (and `CLAUDE.local.md`) · **Does not read `AGENTS.md` natively**

Research snapshot: **2026-07-19**. Primary docs:
[memory](https://code.claude.com/docs/en/memory),
[settings](https://code.claude.com/docs/en/settings),
[env vars](https://code.claude.com/docs/en/env-vars),
[CLI reference](https://code.claude.com/docs/en/cli-reference).

## Loaded at startup

The [memory location table](https://code.claude.com/docs/en/memory#choose-where-to-put-claude-md-files)
is explicitly *"in load order, from broadest scope to most specific"*:

1. **Managed / policy `CLAUDE.md`** — loaded first, cannot be excluded by user
   settings ([deploy org-wide](https://code.claude.com/docs/en/memory#deploy-organization-wide-claude-md)):
   - macOS: `/Library/Application Support/ClaudeCode/CLAUDE.md`
   - Linux & WSL: `/etc/claude-code/CLAUDE.md`
   - Windows: `C:\Program Files\ClaudeCode\CLAUDE.md`
   - Or an inline `claudeMd` string key in `managed-settings.json`.
2. **User instructions** — `~/.claude/CLAUDE.md`.
3. **User rules** — `~/.claude/rules/*.md`, loaded before project rules
   ([user-level rules](https://code.claude.com/docs/en/memory#user-level-rules)).
4. **Ancestor walk** — from the working directory **up to the filesystem root**,
   loading `CLAUDE.md` **and** `CLAUDE.local.md` in each directory. Concatenated
   (not overriding), ordered **filesystem root → cwd**; within a directory
   `CLAUDE.local.md` follows `CLAUDE.md`. These are *"loaded in full at launch"*
   ([how CLAUDE.md files load](https://code.claude.com/docs/en/memory#how-claude-md-files-load)).
5. **Project instructions** — `./CLAUDE.md` **or** `./.claude/CLAUDE.md`
   ([set up a project CLAUDE.md](https://code.claude.com/docs/en/memory#set-up-a-project-claude-md)).
6. **Project rules** — `.claude/rules/*.md`, discovered recursively; rules
   *without* `paths:` frontmatter load at launch with the same priority as
   `.claude/CLAUDE.md` ([organize rules](https://code.claude.com/docs/en/memory#set-up-rules)).
7. **Imports** — `@path/to/file` anywhere in a loaded file, expanded at launch,
   relative to the containing file, recursive to **4 hops**; backticked paths
   are not imported ([imports](https://code.claude.com/docs/en/memory#import-additional-files)).

`CLAUDE.local.md` is **still supported, not deprecated** — *"it loads alongside
`CLAUDE.md` and is treated the same way"* (same locations table).

## Loaded lazily (not simulated by instructmd)

- **Subdirectory `CLAUDE.md` / `CLAUDE.local.md` below cwd** — *"included when
  Claude reads files in those subdirectories"*, not at launch.
- **Path-scoped rules** (`paths:` frontmatter) — trigger when Claude reads a
  matching file.
- After `/compact`, only the project-root `CLAUDE.md` is re-injected; nested
  files reload on next relevant file read.

## Not loaded (and why)

| File | Why not |
|---|---|
| `AGENTS.md` | **Not read natively.** *"Claude Code reads `CLAUDE.md`, not `AGENTS.md`."* Bridge via `@AGENTS.md` import inside `CLAUDE.md`, or a symlink `ln -s AGENTS.md CLAUDE.md`. `/init` *reads* an existing `AGENTS.md` only to generate `CLAUDE.md`. ([AGENTS.md](https://code.claude.com/docs/en/memory#agents-md)) |
| Files matching `claudeMdExcludes` | The `claudeMdExcludes` setting (any layer; arrays merge) lists glob/absolute paths of `CLAUDE.md` files to skip, matched against absolute paths. Managed policy `CLAUDE.md` cannot be excluded this way. ([exclude specific files](https://code.claude.com/docs/en/memory#exclude-specific-claude-md-files)) |
| **All** `CLAUDE.md` (incl. managed) | `--safe-mode` / `CLAUDE_CODE_SAFE_MODE` disables all CLAUDE.md loading, *including managed*. |
| **All** `CLAUDE.md` (except managed) | `--bare` / `CLAUDE_CODE_SIMPLE` skips auto-discovery of hooks, skills, plugins, MCP, auto memory, and CLAUDE.md loading. |
| `CLAUDE.md` from `--add-dir` directories | By default *"additional directories do not load memory files"* unless `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`. |
| `.claude/rules/` (project) | Skipped if `project` is excluded from `--setting-sources`. |
| `CLAUDE.local.md` (additional dirs) | Skipped if `local` is excluded from `--setting-sources`. |

## Environment variables

| Variable | Effect |
|---|---|
| `CLAUDE_CODE_SAFE_MODE` (via `--safe-mode`) | Disables **all** CLAUDE.md including managed policy CLAUDE.md. Referenced in CLI reference + changelog; not on the env-vars page. |
| `CLAUDE_CODE_SIMPLE` (via `--bare`) | Bare mode skips auto memory and CLAUDE.md loading (plus hooks/skills/plugins/MCP). |
| `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD` | Set to `1` to load `CLAUDE.md`, `.claude/CLAUDE.md`, `.claude/rules/*.md`, and `CLAUDE.local.md` from `--add-dir` directories. Default: off. ([env vars](https://code.claude.com/docs/en/env-vars), [load from additional dirs](https://code.claude.com/docs/en/memory#load-from-additional-directories)) |
| `CLAUDE_CODE_DISABLE_AUTO_MEMORY` | Auto memory only, **not** CLAUDE.md. `=1` disables; `=0` forces it on even under `--bare`. |
| — | **No env var alters the ancestor walk.** The documented way to skip specific files is the `claudeMdExcludes` setting. |

## Flags / settings

- `--add-dir` grants file access only; CLAUDE.md from added dirs is not loaded
  unless the env var above is `1`.
- `--setting-sources user,project,local` — excluding `project` skips
  `.claude/rules/`; excluding `local` skips `CLAUDE.local.md` (for additional dirs).
- `claudeMdExcludes` — glob/absolute paths to skip (managed policy exempt).
- `claudeMd` — inline org CLAUDE.md content (managed settings only).
- The `InstructionsLoaded` hook *"fires when CLAUDE.md or `.claude/rules/*.md`
  files are loaded into context"* — a useful reference for what actually loaded.

## Uncertain / undocumented

- Whether excluding `user` from `--setting-sources` skips `~/.claude/CLAUDE.md`
  and `~/.claude/rules/` — not documented.
- The exact upper boundary of the ancestor walk — docs imply the filesystem root
  but state no explicit stop condition.
- Whether ancestor directories' `.claude/CLAUDE.md` / `.claude/rules/`
  (not just plain `CLAUDE.md`/`CLAUDE.local.md`) participate in the upward walk.
- `CLAUDE_CODE_SAFE_MODE` / `CLAUDE_CODE_SIMPLE` behavior when set manually
  (vs. via their flags) is not separately documented.

## Sources

- [Memory & instruction files](https://code.claude.com/docs/en/memory)
- [AGENTS.md compatibility](https://code.claude.com/docs/en/memory#agents-md)
- [Load from additional directories](https://code.claude.com/docs/en/memory#load-from-additional-directories)
- [Exclude specific CLAUDE.md files](https://code.claude.com/docs/en/memory#exclude-specific-claude-md-files)
- [Environment variables](https://code.claude.com/docs/en/env-vars)
- [Settings](https://code.claude.com/docs/en/settings)
- [CLI reference](https://code.claude.com/docs/en/cli-reference)

# Amp

**Native project file:** `AGENTS.md` · **Compatibility fallback:** `AGENT.md`, `CLAUDE.md`

Research snapshot: **2026-07-19**. Primary source: the
[Amp Owner's Manual](https://ampcode.com/manual) (AGENTS.md section), verified
verbatim. Amp is published by Amp Frontier Corporation (independent since
December 2025).

## Loaded at startup

The manual's rules verbatim:

> - `AGENTS.md` files in the current working directory (or editor workspace
>   roots) **and** parent directories (up to `$HOME`) are always included.
> - Subtree `AGENTS.md` files are included when the agent reads a file in the subtree.
> - System-wide guidance files, as well as both `$HOME/.config/amp/AGENTS.md` and
>   `$HOME/.config/AGENTS.md`, are always included if they exist.
> - If no `AGENTS.md` exists in a directory, but a file named `AGENT.md` (without
>   an S) or `CLAUDE.md` does exist, that file will be included.

Concrete inventory (always included if present):

1. **System-wide / organization-managed**:
   - Linux: `/etc/ampcode/AGENTS.md`
   - macOS: `/Library/Application Support/ampcode/AGENTS.md`
   - Windows: `%ProgramData%\ampcode\AGENTS.md`
2. **User-global** — **both** `$HOME/.config/amp/AGENTS.md` **and**
   `$HOME/.config/AGENTS.md`. Docs hardcode `$HOME/.config` (no `XDG_CONFIG_HOME`
   mention, even on Windows: `%USERPROFILE%\.config\amp\...`).
3. **Ancestor walk** — cwd (or each editor workspace root) and every parent
   **up to `$HOME`** (not filesystem root).
4. **Per-directory fallback** — `AGENTS.md` → else `AGENT.md` → or `CLAUDE.md`.

Inspect what loaded via `agents-md list` in the command palette. `@`-mentions
pull in extra files; a mentioned file with `globs:` frontmatter loads only once
Amp reads a matching file.

## Loaded lazily (not simulated by instructmd)

Subtree `AGENTS.md` below cwd is included only *"when the agent reads a file in
the subtree."*

## Not loaded (and why)

| File | Why not |
|---|---|
| `AGENT.md` / `CLAUDE.md` when an `AGENTS.md` exists in the same directory | Per-directory fallback: `AGENTS.md` is preferred; the others load only in its absence. |
| Ancestors above `$HOME` | The ancestor walk stops at `$HOME`, not the filesystem root. |
| `.cursorrules`, `.windsurfrules`, `.clinerules`, `.github/copilot-instructions.md` | The **original** (May 2025) `AGENT.md` announcement listed these root fallbacks; the **current** manual documents only `AGENT.md` and `CLAUDE.md`. Cursor migration is now `mv .cursorrules AGENTS.md` + `@.cursor/rules/*.mdc`. |

## Environment variables

**None affect instruction loading.** The documented vars (`AMP_API_KEY`,
`AMP_SKIP_UPDATE_CHECK`, `AMP_FORCE_BEL`, `AMP_TOOLBOX`, `AMP_SETTINGS_FILE`,
`AMP_LOG_LEVEL`, proxy/cert vars, `$EDITOR`) do not touch `AGENTS.md` discovery.
`XDG_CONFIG_HOME` is not mentioned anywhere in the manual.

## Settings / flags

- **No settings key alters or disables `AGENTS.md`/`CLAUDE.md` loading.** Every
  `amp.*` key was enumerated; there is no `amp.rules` and no CLAUDE.md toggle.
- Nearest analog (skills, **not** instructions): `amp.skills.disableClaudeCodeSkills`
  disables loading skills from `.claude/skills/` etc.
- **No CLI flag touches `AGENTS.md` discovery.** (`--settings-file` only relocates
  user settings.)

## Uncertain / undocumented

- `AGENT.md` vs `CLAUDE.md` precedence within one directory (AGENT.md listed
  first, so implied but not explicit).
- Whether `XDG_CONFIG_HOME` is honored for the global path (docs hardcode `$HOME/.config`).
- Relative order of `$HOME/.config/AGENTS.md` vs `$HOME/.config/amp/AGENTS.md`.
- Whether the `AGENT.md`/`CLAUDE.md` fallback applies to the global/system paths.

## Sources

- [Amp Owner's Manual — AGENTS.md](https://ampcode.com/manual#AGENTS.md)
- [Granular guidance (globs frontmatter)](https://ampcode.com/manual#granular-guidance)
- [News: AGENTS.md rename](https://ampcode.com/news/AGENTS.md)
- [News: multiple AGENT.md files](https://ampcode.com/news/multiple-AGENT.md-files)

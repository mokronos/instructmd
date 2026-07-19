# Gemini CLI

**Native project file:** `GEMINI.md` · **No `CLAUDE.md`/`AGENTS.md` unless configured**

Research snapshot: **2026-07-19**, docs + `google-gemini/gemini-cli` `main`.
Discovery is orchestrated by `MemoryContextManager.refresh()`
(`packages/core/src/context/memoryContextManager.ts`), concatenated per-tier by
`categorizeAndConcatenate()`.

## Loaded at startup

In tier order **global → extension → project (workspace) → user project memory**:

1. **Global** — `~/.gemini/GEMINI.md` (every configured filename variant checked
   in the global dir).
2. **Extension context files** — `GEMINI.md` from *active* extensions.
3. **Workspace/environment context** — only if the folder is **trusted**
   (untrusted → skipped). For each workspace directory (cwd +
   `context.includeDirectories`), an **upward walk** stopping at the first
   ancestor containing a boundary marker (default `['.git']`), else the
   workspace root — **not** home, **not** filesystem root. The global `~/.gemini`
   dir is skipped to avoid double-loading. Files ordered root-to-leaf, all
   filename variants grouped per directory, deduped by `dev:ino`.
4. **User project memory** — `~/.gemini/tmp/<project-hash>/memory/MEMORY.md`
   (private, from `/memory add` / Auto Memory).

Docs: [gemini-md](https://geminicli.com/docs/cli/gemini-md/),
[configuration](https://geminicli.com/docs/reference/configuration/).

## Loaded lazily (not simulated by instructmd)

Subdirectory `GEMINI.md` below cwd loads **just-in-time** via
`loadJitSubdirectoryMemory()` when a tool touches a path: *"the CLI automatically
scans for GEMINI.md files in that directory and its ancestors up to a trusted
root."*

> **Docs/source divergence.** The configuration docs still describe a **startup
> downward scan** of subdirectories below cwd (max 200, `discoveryMaxDirs`). Current
> `main` source has **no such scan** — it uses JIT loading instead, and
> `discoveryMaxDirs` has no consumer. A currently-released stable build may still
> match the docs. (source-observed)

## Not loaded (and why)

| File | Why not |
|---|---|
| `CLAUDE.md`, `AGENTS.md` | Not default context filenames. Add via `context.fileName` (array — all variants then load per directory). |
| Workspace `GEMINI.md` (all) | Skipped entirely when the folder is **untrusted**. Global + user project memory still load. Bypass with `GEMINI_CLI_TRUST_WORKSPACE=true`. |
| Ancestors above the boundary marker | The upward walk stops at the first `context.memoryBoundaryMarkers` match (default `.git`), else the workspace root. |
| Memory in `includeDirectories` (for `/memory reload`) | Loaded only when `context.loadMemoryFromIncludeDirectories: true`. |

## Environment variables

| Variable | Effect |
|---|---|
| `GEMINI_SYSTEM_MD` | Replaces the built-in **system prompt** with a markdown file: `true`/`1` → `./.gemini/system.md`, else a path. Separate from `GEMINI.md` context, but it *is* an instruction file loaded at startup. |
| `GEMINI_WRITE_SYSTEM_MD` | Writes the built-in system prompt to a file (output-only; no loading change). |
| `GEMINI_CLI_TRUST_WORKSPACE=true` | Bypasses the folder-trust check; trust gates whether workspace `GEMINI.md` files load. |
| `GEMINI_CLI_SYSTEM_SETTINGS_PATH` / `GEMINI_CLI_SYSTEM_DEFAULTS_PATH` | Relocate the system `settings.json` / `system-defaults.json` (which can set `context.*`). |
| `GEMINI_CLI_TRUSTED_FOLDERS_PATH` | Relocates `trustedFolders.json` (indirectly gates loading). |
| `GEMINI_CLI_HOME` | Documented as relocating user-level config (→ global `GEMINI.md`), but `storage.ts` on `main` hardcodes `~/.gemini` with no override — **unresolved conflict**. |
| — | **No env var disables `GEMINI.md` loading.** |

## Settings keys (`settings.json`)

Precedence (later wins): defaults → system defaults (`/etc/gemini-cli/system-defaults.json`)
→ user (`~/.gemini/settings.json`) → project (`.gemini/settings.json`) → system
override (`/etc/gemini-cli/settings.json`) → env → CLI args.

| Key | Default | Behavior |
|---|---|---|
| `context.fileName` | `GEMINI.md` | String or array. With an array, **all** matching variants load per directory (not first-match). |
| `context.includeDirectories` | `[]` | Extra workspace directories, each with its own upward walk. |
| `context.loadMemoryFromIncludeDirectories` | `false` | Whether `/memory reload` scans include-directories. |
| `context.memoryBoundaryMarkers` | `['.git']` | Names that stop the upward walk. |
| `context.discoveryMaxDirs` | `200` | Documented max dirs for the (now legacy) downward scan; **no consumer** in current discovery code. |
| `context.importFormat` | `undefined` | `'flat'` \| `'tree'` for `@`-imports. |
| `experimental.autoMemory` | off | Feeds the private project `MEMORY.md`. |
| — | — | **No setting disables context loading.** Practical equivalents: untrusted folder, boundary markers, or a nonexistent `fileName`. |

## CLI flags

- `--include-directories <dirs>` — adds workspace directories (each gets discovery).
- `-e, --extensions <names...>` / `gemini -e none` — selects active extensions.
- **No flag disables/filters `GEMINI.md` loading**; runtime control via
  `/memory show|refresh|add` and `/directory add`.

## Sources

- [GEMINI.md docs](https://geminicli.com/docs/cli/gemini-md/)
- [Configuration reference](https://geminicli.com/docs/reference/configuration/)
- [Memory imports](https://geminicli.com/docs/reference/memport/)
- [`memoryDiscovery.ts`](https://raw.githubusercontent.com/google-gemini/gemini-cli/main/packages/core/src/utils/memoryDiscovery.ts)
- [`memoryContextManager.ts`](https://raw.githubusercontent.com/google-gemini/gemini-cli/main/packages/core/src/context/memoryContextManager.ts)
- [`settingsSchema.ts`](https://raw.githubusercontent.com/google-gemini/gemini-cli/main/packages/cli/src/config/settingsSchema.ts)

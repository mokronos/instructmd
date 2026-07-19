# pi

**Native project file:** `AGENTS.md` · **Compatibility fallback:** `CLAUDE.md`

Research snapshot: **2026-07-19**, `earendil-works/pi` `main` (latest release
0.80.10, 2026-07-16). The CLI is `packages/coding-agent`; loading lives in
`src/core/resource-loader.ts` (`loadContextFileFromDir` / `loadProjectContextFiles`).

## Loaded at startup

Per-directory candidates, **first match wins** (one file max per directory):

```
AGENTS.md → AGENTS.MD → CLAUDE.md → CLAUDE.MD
```

Assembly order:

1. **Global** — the first candidate match in the agent dir (default
   `~/.pi/agent`, i.e. `~/.pi/agent/AGENTS.md` etc.), pushed first.
2. **Directory chain, root-first** — walk **upward from cwd to the filesystem
   root**; each hit is `unshift`ed, so the final order reads
   **filesystem root → … → parent → cwd** (cwd last, closest to the
   conversation). One file per directory; all matches concatenated.
3. **Dedup** — a `seenPaths` set skips a file already added (e.g. when the global
   agent dir lies on the cwd ancestor path).

README confirms: *"Pi loads `AGENTS.md` (or `CLAUDE.md`) at startup from
`~/.pi/agent/AGENTS.md` (global); parent directories (walking up from cwd);
current directory. All matching files are concatenated."*
([README](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/README.md))

## Not loaded (and why)

| File | Why not |
|---|---|
| `CLAUDE.md` / `CLAUDE.MD` when an `AGENTS.md`/`AGENTS.MD` exists in the same directory | First-match-wins per directory; `AGENTS.md` is tried first. |
| All context files | `--no-context-files` / `-nc` makes the loader return `[]` (both global and per-directory). |
| `.pi/SYSTEM.md` / `.pi/APPEND_SYSTEM.md` in an **untrusted** project | These *system-prompt* files are trust-gated (fall back to `~/.pi/agent/SYSTEM.md`). **`AGENTS.md`/`CLAUDE.md` are NOT trust-gated** in current source — see note below. |

> **Trust gating.** Commit 89a92207 (0.79.0, 2026-06-08) introduced project trust
> and its docs described parent-dir context files as trust-gated. Current `main`
> source has **no trust check** on `loadProjectContextFiles`, so for current pi:
> do **not** trust-gate `AGENTS.md`/`CLAUDE.md`; do trust-gate project
> `.pi/SYSTEM.md` / `.pi/APPEND_SYSTEM.md`. (source-observed; no explicit changelog
> entry announced the relaxation)

## Environment variables

| Variable | Effect |
|---|---|
| `PI_CODING_AGENT_DIR` | Overrides the agent config dir (default `~/.pi/agent`; tilde-expanded), moving where the **global** context file is looked up. (Rebranded builds use `<APP>_CODING_AGENT_DIR`.) ([config.ts](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/src/config.ts)) |
| — | **No env var disables context files.** `resource-loader.ts` reads no `process.env`; the only off-switch is the CLI flag. |

Other documented vars (`PI_CODING_AGENT_SESSION_DIR`, `PI_PACKAGE_DIR`,
`PI_OFFLINE`, `PI_SKIP_VERSION_CHECK`, `PI_TELEMETRY`, `PI_CACHE_RETENTION`) do
not affect context-file discovery.

## Flags / commands

| Flag/command | Effect |
|---|---|
| `--no-context-files` / `-nc` | *"Disable `AGENTS.md` and `CLAUDE.md` discovery"* — disables both global and per-directory files. |
| `--system-prompt` / `--append-system-prompt` | Replace/append the system prompt, overriding `SYSTEM.md` discovery. |
| `-a`/`--approve`, `-na`/`--no-approve`, `/trust` | Trust decisions — affect trust-gated resources (project `.pi/SYSTEM.md`, settings, extensions/skills), **not** `AGENTS.md`/`CLAUDE.md`. |
| `/reload` | Reloads keybindings, extensions, skills, prompts, themes, and context files. |

## Uncertain / undocumented

- Case-sensitivity: on case-insensitive filesystems the four candidates collapse;
  the `AGENTS.md → AGENTS.MD → CLAUDE.md → CLAUDE.MD` order only matters on
  case-sensitive filesystems (e.g. Linux).
- Extensions can rewrite the loaded set via an `agentsFilesOverride` hook, so
  extension-modified installs can deviate.

## Sources

- [README (context files)](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/README.md)
- [`resource-loader.ts`](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/src/core/resource-loader.ts)
- [`config.ts`](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/src/config.ts)
- [`cli/args.ts`](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/src/cli/args.ts)
- [docs/usage.md](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/docs/usage.md)

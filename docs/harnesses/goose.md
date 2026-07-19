# goose

**Native project files:** `.goosehints` **and** `AGENTS.md` (both, per directory)

Research snapshot: **2026-07-19**. Repo moved to `aaif-goose/goose` (Agentic AI
Foundation; `block/goose` mirrors it). Docs at
[goose-docs.ai](https://goose-docs.ai). Source pinned to `main` commit
[`8e78960`](https://github.com/aaif-goose/goose/blob/8e78960e535ab7f34630e7c5921a42f146cbc9f4/crates/goose/src/hints/load_hints.rs);
loading is now in the **core agent** (`crates/goose/src/hints/load_hints.rs`),
called unconditionally from `prompt_manager.rs` / `reply_parts.rs`.

Let `NAMES` = the context filename list (default `[".goosehints", "AGENTS.md"]`).

## Loaded at startup

1. **Global hints** (`### Global Hints`):
   - For **every** name in `NAMES`: `<config_dir>/<name>` — by default **both**
     `~/.config/goose/.goosehints` **and** `~/.config/goose/AGENTS.md`.
   - Additionally, **iff** `"AGENTS.md"` is in `NAMES`: `~/.agents/AGENTS.md`
     (the shared "agents home dir").
2. **Project hints** (`### Project Hints`): directory chain from **git root down
   to cwd** (git root processed first, cwd last). If there is **no git root,
   only cwd** is checked. In each directory, **every** name in `NAMES` is checked
   and **all** existing ones load — by default both `.goosehints` and `AGENTS.md`,
   in list order (`.goosehints` before `AGENTS.md`).

Empty files are skipped. `@import` references inline recursively (max depth 3,
confined to the git root / hint-file dir, gitignored targets skipped, per-file
cap 128 KiB).

## Loaded lazily (not simulated by instructmd)

A `SubdirectoryHintTracker` watches tool arguments and lazily loads hint files
from subdirectories below cwd the first time they are touched
(`### Subdirectory Hints (<dir>)`), staying active for the session.

## Not loaded (and why)

| File | Why not |
|---|---|
| Default names when `CONTEXT_FILE_NAMES` is set | The env var **replaces** the default list entirely (no merge). Setting it to `'[".cursorrules"]'` means `.goosehints`/`AGENTS.md` are no longer looked for anywhere (global or project). |
| `~/.agents/AGENTS.md` | Loaded only when `"AGENTS.md"` is in `NAMES`; drops out if the list is overridden without it. |
| Everything (effectively) | `CONTEXT_FILE_NAMES='[]'` yields an empty list → no hint files load (also skips `~/.agents/AGENTS.md`). There is **no dedicated disable var**; this is the only off-switch. (source-observed) |
| Ancestors above the git root | The project chain starts at the git root; parents above it are not scanned. |
| `@import` targets outside the repo, or gitignored | Rejected by the import boundary / gitignore filter. |

## Environment variables

| Variable | Effect | Parsing |
|---|---|---|
| `CONTEXT_FILE_NAMES` | **Replaces** the default `[".goosehints", "AGENTS.md"]` list — for **both** the global config dir and project directories. This is the switch that lets goose read e.g. `CLAUDE.md`. | Read as `Vec<String>` via config; env value is parsed **JSON-first**, so it must be a **JSON array of strings**, e.g. `export CONTEXT_FILE_NAMES='[".goosehints","CLAUDE.md"]'`. **Comma-separated does not work** (it fails the `Vec<String>` parse and silently falls back to defaults). Note: the env key is uppercased **with no `GOOSE_` prefix**. |
| `GOOSE_PATH_ROOT` | Relocates all goose dirs: config dir → `$GOOSE_PATH_ROOT/config`, agents home → `$GOOSE_PATH_ROOT/.agents`. Must be an **absolute** path or it is ignored. | absolute path |
| — | **No `GOOSE_DISABLE_HINTS` or equivalent exists.** `with_hints` is called unconditionally; use `CONTEXT_FILE_NAMES='[]'` to suppress. |

`CONTEXT_FILE_NAMES` can also be set in `~/.config/goose/config.yaml` (env var wins).

## Uncertain / undocumented

- **Order discrepancy**: docs state the default as `["AGENTS.md", ".goosehints"]`,
  but source constants are `[".goosehints", "AGENTS.md"]`. Source wins for
  within-directory concatenation order; treat docs as imprecise.
- Docs describe discovery direction ("working directory up to the repository
  root"); actual concatenation is root-first (`directories.reverse()`).

## Sources

- [Using goosehints](https://goose-docs.ai/docs/guides/context-engineering/using-goosehints)
- [Environment variables](https://goose-docs.ai/docs/guides/environment-variables)
- [`load_hints.rs`](https://github.com/aaif-goose/goose/blob/8e78960e535ab7f34630e7c5921a42f146cbc9f4/crates/goose/src/hints/load_hints.rs)
- [`config/base.rs` (env parsing)](https://github.com/aaif-goose/goose/blob/8e78960e535ab7f34630e7c5921a42f146cbc9f4/crates/goose/src/config/base.rs)
- [`config/paths.rs`](https://github.com/aaif-goose/goose/blob/8e78960e535ab7f34630e7c5921a42f146cbc9f4/crates/goose/src/config/paths.rs)
- [`import_files.rs`](https://github.com/aaif-goose/goose/blob/8e78960e535ab7f34630e7c5921a42f146cbc9f4/crates/goose/src/hints/import_files.rs)

# OpenCode

**Native project file:** `AGENTS.md` · **Compatibility fallback:** `CLAUDE.md`, then deprecated `CONTEXT.md`

Research snapshot: **2026-07-19**. Source pinned to `anomalyco/opencode` (formerly
`sst/opencode`), branch `dev`, commit
[`78587c1`](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/opencode/src/session/instruction.ts).
Instruction loading lives in `packages/opencode/src/session/instruction.ts`
(`Instruction.systemPaths()`).

## Loaded at startup

In load order:

1. **Global — exactly one file** (first existing wins, then the loop `break`s):
   1. `~/.config/opencode/AGENTS.md` — the global config dir is
      `$XDG_CONFIG_HOME/opencode`, relocated by `OPENCODE_CONFIG_DIR`.
      ([global.ts](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/core/src/global.ts))
   2. `~/.claude/CLAUDE.md` — **only if** the Claude Code prompt fallback is not
      disabled (see env vars).
      ([instruction.ts#L60-L68](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/opencode/src/session/instruction.ts#L60-L68))
2. **Project walk** — skipped entirely when `OPENCODE_DISABLE_PROJECT_CONFIG` is
   truthy. For each candidate filename, `findUp` walks from the working
   directory up to **and including the worktree root**
   ([fs-util.ts#L154-L166](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/core/src/fs-util.ts#L154-L166)).
   Candidate order:
   1. `AGENTS.md`
   2. `CLAUDE.md` — only if the Claude Code prompt fallback is not disabled
   3. `CONTEXT.md` — marked `// deprecated` in source

   **The first filename with any match wins**: all ancestor copies of that one
   filename load, then the loop `break`s and the remaining candidate filenames
   are skipped for the whole walk. (Source comment: *"The first project-level
   match wins so we don't stack AGENTS.md/CLAUDE.md from every ancestor."*)
3. **Configured `instructions`** — the `instructions: string[]` config array:
   local file paths, globs (`globUp` from cwd to worktree root), and
   `http(s)://` URLs. URLs are fetched with a 5-second timeout; failures yield
   an empty string silently.
   ([instruction.ts#L79-L103](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/opencode/src/session/instruction.ts))

Docs confirm the precedence and the `~/.claude/CLAUDE.md` fallback:
[opencode.ai/docs/rules](https://opencode.ai/docs/rules/),
[opencode.ai/docs/config](https://opencode.ai/docs/config/#precedence-order).

## Loaded lazily (not simulated by instructmd)

`Instruction.resolve()` dynamically attaches nearby `AGENTS.md` / `CLAUDE.md` /
`CONTEXT.md` when the agent **reads a file** in a subdirectory — walking from
that file's directory up to (exclusive) the project root, added once. This is a
runtime behavior, not startup resolution.

## Not loaded (and why)

| File | Why not |
|---|---|
| `CLAUDE.md` (global and project) | Excluded when `OPENCODE_DISABLE_CLAUDE_CODE` **or** `OPENCODE_DISABLE_CLAUDE_CODE_PROMPT` is set — both remove the same two fallbacks. |
| `CLAUDE.md` / `CONTEXT.md` when an `AGENTS.md` exists anywhere in the walk | First-filename-wins: once any `AGENTS.md` matches, the other candidate names are never tried. |
| `CONTEXT.md` | Deprecated; only reached if neither `AGENTS.md` nor `CLAUDE.md` matched anywhere in the walk. |
| Any project file | When `OPENCODE_DISABLE_PROJECT_CONFIG` is truthy, the whole project walk is skipped (only global + configured instructions remain). |
| Ancestor copies of a losing candidate filename | The walk stacks only the *winning* filename's copies; e.g. a parent `CLAUDE.md` never loads if a child `AGENTS.md` won. |

## Environment variables

| Variable | Effect on instruction loading | Parsing |
|---|---|---|
| `OPENCODE_DISABLE_CLAUDE_CODE` | Removes the `~/.claude/CLAUDE.md` global fallback **and** the `CLAUDE.md` project candidate. (Also disables Claude Code *skills*, which is unrelated to instruction markdown.) | `Config.boolean` — accepts `true/false`-style literals; default `false`. ([runtime-flags.ts#L23-L30](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/opencode/src/effect/runtime-flags.ts#L23-L30)) |
| `OPENCODE_DISABLE_CLAUDE_CODE_PROMPT` | **Identical effect on instructions** as the above (both OR-combine into one internal `disableClaudeCodePrompt` flag). Does *not* touch skills. | same |
| `OPENCODE_DISABLE_CLAUDE_CODE_SKILLS` | Skills only (`.claude/skills`). **No effect** on instruction markdown. | same |
| `OPENCODE_DISABLE_PROJECT_CONFIG` | Skips the entire project walk; relative `instructions` globs then resolve against the global config dir. | `truthy()`: value is `"true"` or `"1"`. ([flag.ts#L3-L6](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/core/src/flag/flag.ts#L3-L6)) |
| `OPENCODE_CONFIG` / `OPENCODE_CONFIG_CONTENT` | Custom config file path / inline JSON — changes which `instructions` array is seen. | raw string |
| `OPENCODE_CONFIG_DIR` | Relocates the global config dir, hence the global `AGENTS.md`. | raw string |
| `OPENCODE_TEST_HOME` | Overrides the home dir used for `~/.claude/CLAUDE.md` and `~/` expansion. | raw string |

> **Note on the two Claude switches.** The unsuffixed `OPENCODE_DISABLE_CLAUDE_CODE`
> and the `_PROMPT` variant have the **same** effect on instruction files (both drop
> the global `~/.claude/CLAUDE.md` *and* the `CLAUDE.md` project candidate). The only
> difference is that the unsuffixed one *additionally* disables Claude Code skills.

## Sources

- [Rules docs](https://opencode.ai/docs/rules/)
- [Config precedence docs](https://opencode.ai/docs/config/#precedence-order)
- [`instruction.ts`](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/opencode/src/session/instruction.ts)
- [`runtime-flags.ts`](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/opencode/src/effect/runtime-flags.ts#L23-L30)
- [`flag.ts`](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/core/src/flag/flag.ts)
- [`fs-util.ts`](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/core/src/fs-util.ts#L154-L166)
- [`global.ts`](https://github.com/anomalyco/opencode/blob/78587c141bbac2c60b33c277359ba635b3410750/packages/core/src/global.ts)

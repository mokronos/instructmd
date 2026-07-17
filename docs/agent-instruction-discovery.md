# How Coding Agents Load Instruction Files

Research snapshot: **2026-07-17**

This document compares how prominent coding agents discover and compose persistent instructions such as `AGENTS.md`, `CLAUDE.md`, and tool-specific rule files. It supports the CLI proposed in [`VISION.md`](../VISION.md): faithfully showing which files an agent would load, in what order, and why.

The companion [HTML atlas](agent-instruction-discovery.html) presents the same findings as a filterable visual comparison.

## Executive summary

There is no shared `AGENTS.md` standard beyond the filename. Implementations fall into four broad families:

1. **Ancestor chain plus lazy descendants:** Claude Code, OpenCode, Gemini CLI, Amp, goose, and parts of Copilot CLI initially load instructions above the working directory and may activate deeper files when a file is touched.
2. **Fixed ancestor chain at startup:** Codex, pi, and Qwen Code compose a root-to-working-directory chain but generally do not activate arbitrary descendants later. Codex stops at its project root; pi walks to the filesystem root.
3. **Conditional rule engines:** Cursor, Windsurf, Cline, Roo Code, and Continue combine always-on rules with rules selected by globs, model judgment, mode, or manual activation.
4. **Root or explicit context only:** Junie and Kiro use root/scope files without a documented nested hierarchy. Aider requires files to be explicitly supplied.

The differences that matter most to a visualizer are:

- **Boundary:** filesystem root, home directory, Git/worktree root, workspace root, or no traversal.
- **Selection:** one winning filename per directory versus every applicable file.
- **Timing:** startup only, task/request boundary, manual reload, or just-in-time activation after tool access.
- **Composition:** concatenation, same-directory replacement, metadata-based activation, or explicit precedence tiers.
- **Imports:** Claude, Gemini, Amp, Copilot CLI, Qwen, and goose expand file references, but with different syntax and safety limits.
- **Budgets:** Codex caps aggregate project instructions at 32 KiB by default; Windsurf caps individual native rules; most tools publish no instruction-specific hard limit.

Most “overrides” are not programmatic overrides. The harness usually concatenates prose in a defined order and relies on the model to reconcile conflicts. A visualizer should distinguish **file selection precedence** from **semantic instruction precedence**.

## Scope and method

The comparison covers widely used terminal agents, IDE agents, and close compatibility tools for which official behavior could be found:

- Claude Code, Codex CLI, OpenCode, pi, Gemini CLI, Amp, goose, and Qwen Code
- Cursor, GitHub Copilot coding agent, GitHub Copilot CLI, Windsurf/Devin Desktop, Cline, Roo Code, and Continue
- Junie, Kiro CLI, and Aider

Claims come from official product documentation and official repositories. User-facing documentation is treated as the contract. Details found only in official source are marked **source-observed** because they may change without notice. The set is broad rather than mathematically exhaustive: “popular” has no stable boundary, and new agents frequently adopt compatibility filenames.

## Comparison matrix

| Agent | Native project source | Initial boundary and order | Nested behavior | Same-directory behavior | Imports | Published limit |
|---|---|---|---|---|---|---|
| Claude Code | `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md`, `.claude/rules/` | Managed → user → filesystem root → CWD | Lazy on file read; path-scoped rules | Base and local can both apply | `@path`, 4 hops | No hard `CLAUDE.md` limit |
| Codex CLI | `AGENTS.override.md`, `AGENTS.md`, configured fallbacks | Global → project root → CWD | None below launch CWD | First non-empty candidate wins | None | 32 KiB project aggregate by default |
| OpenCode | `AGENTS.md`; `CLAUDE.md` fallback; configured paths/URLs | Global → worktree root → CWD → custom | Lazy on file read, source-observed | `AGENTS.md` suppresses fallback | Config-level files, globs, URLs | None documented |
| pi | `AGENTS.md`; `CLAUDE.md` fallback | Global → filesystem root → CWD | None | First candidate wins | None | None documented |
| Gemini CLI | `GEMINI.md`; configurable names | Global → Git/workspace boundary → CWD | JIT on tool access | All configured matches compose | `@path`, depth 5 | No file cap; 200-dir discovery default |
| Amp | `AGENTS.md`; `AGENT.md`, `CLAUDE.md` fallbacks | System/global → `$HOME` → CWD | Lazy subtree activation | First candidate wins | Paths and globs; conditional imports | None documented |
| goose | `AGENTS.md` and `.goosehints`; configurable replacement list | Global → Git root → CWD | Lazy subtree activation | Multiple configured names compose | `@relative/path`, depth 3 source-observed | Import parsing skips files over 128 KiB |
| Qwen Code | `QWEN.md`, `AGENTS.md`, `.qwen/QWEN.local.md` | Global → Git root → CWD → local | Ancestor hierarchy is source-observed; no general lazy subtree loading | Both default names compose | `@path`, depth 5 source-observed | None documented |
| Cursor | `.cursor/rules/**/*.mdc`, `AGENTS.md`, root `CLAUDE.md` | Applicable team → project → user rules | Nested and glob/model/manual activation | Same names do not replace; path is identity | `@file` in rules | Under 500 lines recommended |
| Copilot coding agent | `.github/copilot-instructions.md`, path rules, `AGENTS.md` | Applicable repository tiers; nearest `AGENTS.md` takes precedence | Path rules and nearest agent file | Not fully documented | None documented | None documented |
| Copilot CLI | Copilot files plus `AGENTS.md`, `CLAUDE.md`, `GEMINI.md` | Global + repo root/CWD/intermediate/touched path | Touched-path and `applyTo` activation | Applicable files combine; some deduplication | Recursive `@relative/path` in selected formats | None documented |
| Windsurf / Devin Desktop | `.devin/rules/*.md`, `AGENTS.md` | Global/system + workspace; Git parents discovered | Glob/model/manual; nested `AGENTS.md` by touched path | `.devin` preferred over legacy `.windsurf` | None documented | 6k global / 12k workspace chars per native rule |
| Cline | `.clinerules/`, root `AGENTS.md`, compatibility files | Global + primary workspace | `paths` activation; no reliable nested `AGENTS.md` contract | Workspace wins documented conflicts | None documented | None documented |
| Roo Code | `.roo/rules*`, root `AGENTS.md` | Explicit multi-tier prompt order | Recursive rule dirs; optional subfolder discovery | Rule dirs suppress matching legacy files | None documented | None documented |
| Continue | `.continue/rules/`, root compatibility file | Workspace/global rule assembly | Conditional rules; IDE also finds colocated `rules.md` | Rules concatenate | Config `uses` / `file://` | None documented |
| Junie | `.junie/AGENTS.md`, root `AGENTS.md`, legacy guidelines | Global + one project-root source | None documented | Ordered fallback, not merge | Migration import only | None documented |
| Kiro CLI | `.kiro/steering/*.md`, root `AGENTS.md` | Global + workspace | Resource globs, not scoped ancestry | All steering available; workspace wins conflicts | Agent `file://` resources | Context files capped at 75% of model window |
| Aider | Explicit `--read` / configured files | No automatic instruction discovery | None | Explicit list composes | Generic file/directory loading | Model context only |

## Exact discovery models

### Claude Code

Claude has the richest native `CLAUDE.md` hierarchy. It loads managed organization instructions, user instructions from `~/.claude/`, then walks from the filesystem root to the launch directory. `CLAUDE.md` and `CLAUDE.local.md` can both apply in one directory. Descendant files are deferred until Claude reads within their subtree; `.claude/rules` can similarly be always-on or activated by `paths` frontmatter.

`@path` imports resolve relative to the containing file and recurse up to four hops. `--add-dir` does not automatically import that directory’s instructions unless `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1` is set. After compaction, root project instructions are re-injected while nested instructions wait for relevant file access again.

Claude does not natively discover `AGENTS.md`; Anthropic recommends importing it from `CLAUDE.md` or using a symlink. Conflicting prose has no deterministic winner even though file order is defined.

Sources: [memory and instruction files](https://code.claude.com/docs/en/memory), [AGENTS.md compatibility](https://code.claude.com/docs/en/memory#agentsmd), [additional directories](https://code.claude.com/docs/en/memory#load-from-additional-directories).

### OpenAI Codex CLI

Codex first loads one non-empty global candidate from `$CODEX_HOME`: `AGENTS.override.md` before `AGENTS.md`. It finds a project root, normally using `.git`, and then loads at most one file per directory from project root to launch CWD. Candidate order is `AGENTS.override.md`, `AGENTS.md`, then `project_doc_fallback_filenames`. An override file replaces the normal file only in its own directory; files from different directories still concatenate.

Discovery happens once per run and does not continue into descendants below the launch directory. `project_root_markers` controls the boundary. `project_doc_max_bytes` defaults to 32 KiB across project files; earlier root files consume the budget before nearer files, which may be truncated or omitted. Global instructions are outside that budget.

Sources: [AGENTS.md guide](https://developers.openai.com/codex/agent-configuration/agents-md), [project root configuration](https://developers.openai.com/codex/config-file/config-advanced#project-root-detection), [official loader source](https://github.com/openai/codex/blob/main/codex-rs/core/src/agents_md.rs).

### OpenCode

OpenCode walks upward to the worktree boundary and selects `AGENTS.md`, falling back to `CLAUDE.md` and then the deprecated `CONTEXT.md`. Global precedence is `~/.config/opencode/AGENTS.md` over `~/.claude/CLAUDE.md`. Global, project, and configured `instructions` are combined; configured entries may be local files, globs, or HTTP URLs.

Current official source also performs Claude-like lazy discovery when a file is read, walking from that file toward the session root and adding nearby instructions once. This is **source-observed**, not yet fully described in the rules documentation. OpenCode does not expand `@file` inside instruction files.

Sources: [rules](https://opencode.ai/docs/rules/), [configuration precedence](https://opencode.ai/docs/config/#precedence-order), [official instruction loader](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/session/instruction.ts).

### pi

pi loads a global context file from `$PI_CODING_AGENT_DIR` (default `~/.pi/agent`) and then one context file per directory from filesystem root to CWD. Candidate order is `AGENTS.md`, `AGENTS.MD`, `CLAUDE.md`, `CLAUDE.MD`. Each file becomes a separately attributed instruction block.

There is no descendant-on-read loading or native import syntax. `/reload` rescans during a session, and `--no-context-files` disables discovery.

Sources: [context files and commands](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/README.md#context-files), [official resource loader](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/src/core/resource-loader.ts).

### Gemini CLI

Gemini loads `~/.gemini/GEMINI.md`, then workspace context from its configured boundary to the initial root. As tools read, list, edit, or write deeper paths, just-in-time discovery adds context files found on those paths. Context is concatenated with source delimiters rather than structurally overridden.

`context.fileName` accepts one name or an ordered list, so `AGENTS.md` compatibility can be configured. Imports recurse to a default depth of five, avoid cycles, and ignore references in code. `/memory reload` rescans; `/memory show` exposes active context. Useful knobs include `context.memoryBoundaryMarkers`, `context.includeDirectories`, and `context.loadMemoryFromIncludeDirectories`.

Sources: [GEMINI.md](https://geminicli.com/docs/cli/gemini-md/), [memory imports](https://geminicli.com/docs/reference/memport/), [context configuration](https://geminicli.com/docs/reference/configuration/#context), [official discovery source](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/utils/memoryDiscovery.ts).

### Amp

Amp loads one instruction file per directory from global/system locations and every directory from `$HOME` to CWD. The candidate order is `AGENTS.md`, `AGENT.md`, then `CLAUDE.md`. When Amp reads inside a subtree, that subtree’s instruction file becomes active.

Imports accept paths and globs. Imported files can carry `globs` frontmatter, making the import conditional on Amp reading a matching file. Amp defines ordering and scope but not a hard semantic conflict algorithm.

Sources: [Amp Owner’s Manual: AGENTS.md](https://ampcode.com/manual#AGENTS.md), [granular guidance](https://ampcode.com/manual#granular-guidance).

### goose

goose combines global hints with files from Git root to CWD, then keeps newly encountered subtree hints active for the session. Both `AGENTS.md` and `.goosehints` are defaults; `CONTEXT_FILE_NAMES` replaces the default list rather than extending it.

Whitespace-delimited `@relative/path` references inline content. Official source caps recursive import depth at three, rejects references outside the repository, and skips import parsing when the containing file exceeds 128 KiB. These limits are **source-observed**. A separate “message of instruction and modification” file can be reread every turn, but it is not part of repository hierarchy discovery.

Sources: [using goosehints](https://goose-docs.ai/docs/guides/context-engineering/using-goosehints), [persistent instructions](https://goose-docs.ai/docs/guides/context-engineering/using-persistent-instructions), [official loader](https://github.com/aaif-goose/goose/blob/main/crates/goose/src/hints/load_hints.rs).

### Qwen Code

The documented scopes are `~/.qwen/QWEN.md`, project `QWEN.md`, project `AGENTS.md`, and `.qwen/QWEN.local.md`, with local memory last. Current source additionally walks from Git root to CWD for `QWEN.md` and `AGENTS.md`; that ancestor behavior is **source-observed** rather than clearly promised by the guide.

Imports resolve relative paths. Current source limits recursion to five, blocks paths outside the project, and prevents cycles. `/refreshmemory` reloads instructions. `CLAUDE.md` is not a default fallback but can be selected through `context.fileName`.

Sources: [memory guide](https://qwenlm.github.io/qwen-code-docs/en/users/features/memory/), [official discovery source](https://github.com/QwenLM/qwen-code/blob/main/packages/core/src/utils/memoryDiscovery.ts), [official import source](https://github.com/QwenLM/qwen-code/blob/main/packages/core/src/utils/memoryImportProcessor.ts).

### Cursor

Cursor’s primary mechanism is `.cursor/rules/**/*.mdc`. Rules can be always-on, activated by matching `globs`, selected by the model from a description, or manually mentioned. Team, project, and user rules merge with documented conflict priority Team → Project → User, where the earlier tier wins. Same filenames at different paths remain distinct.

Cursor also supports root and nested `AGENTS.md`; a nested file applies to its subtree and more-specific guidance takes precedence. Root `CLAUDE.md` is supported. Rule bodies can reference files with `@filename`, though recursion and cycle semantics are not documented.

Sources: [rules reference](https://cursor.com/docs/context/rules), [AGENTS.md](https://cursor.com/docs/context/rules#agentsmd), [rules help](https://cursor.com/help/customization/rules).

### GitHub Copilot coding agent

The cloud coding agent supports repository-wide `.github/copilot-instructions.md`, path-specific `.github/instructions/**/*.instructions.md` selected by `applyTo`, and agent files. `AGENTS.md` may appear anywhere; GitHub says the nearest file takes precedence, but does not clearly state whether ancestors remain included. Root `CLAUDE.md` or `GEMINI.md` can be used as alternatives.

GitHub publishes precedence among instruction categories, but does not specify scan order, refresh caching, imports, or order among multiple matching path files. Organization instructions can also apply to the cloud agent; personal GitHub.com instructions do not.

Sources: [repository custom instructions](https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot), [instruction precedence](https://docs.github.com/en/copilot/concepts/prompting/response-customization#precedence-of-custom-instructions), [support matrix](https://docs.github.com/en/copilot/reference/custom-instructions-support).

### GitHub Copilot CLI

Copilot CLI has a broader, separately documented loader. It supports user instructions under `$COPILOT_HOME`, repository Copilot files, `AGENTS.md`, `CLAUDE.md`, `.claude/CLAUDE.md`, and `GEMINI.md`. Discovery covers repository root, CWD, intermediate directories, and directories on paths Copilot works with. `*.instructions.md` activates when `applyTo` matches.

Applicable files combine, with some identical copies deduplicated, but GitHub explicitly defines no general precedence order. Recursive `@relative/path` references are supported in repository-wide Copilot instructions, `AGENTS.md`, and `CLAUDE.md`, but not in `GEMINI.md` or `*.instructions.md`. Existing sessions require `/new`, exit/resume, or a new process to see edits.

Source: [Copilot CLI custom instructions](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-custom-instructions).

### Windsurf / Devin Desktop

Windsurf was renamed Devin Desktop in June 2026, while legacy paths remain supported. Native project rules live under `.devin/rules/*.md`, falling back to `.windsurf/rules/*.md` and `.windsurfrules`. Rules can be always-on, selected by the model, activated by glob, or manual. The workspace and subdirectories are scanned; Git parents up to the root are also considered.

Root `AGENTS.md` is always active. A subdirectory `AGENTS.md` becomes an inferred subtree glob rule and inherits parent instructions. Lowercase `agents.md` is also recognized. Native global rules are limited to 6,000 characters and workspace rules to 12,000 characters each; no `AGENTS.md` cap is published.

Sources: [memories and rules](https://docs.windsurf.com/windsurf/cascade/memories), [AGENTS.md](https://docs.windsurf.com/windsurf/cascade/agents-md), [rename changelog](https://windsurf.com/changelog).

### Cline

Cline combines global rules with project-root `.clinerules/` Markdown or text files. Rules can have `paths` globs and can be toggled. Root `.cursorrules`, `.windsurfrules`, and `AGENTS.md` are compatibility sources; `~/.agents/AGENTS.md` is a global cross-tool source. Workspace rules win documented conflicts with global rules.

Current docs do not guarantee arbitrary nested `AGENTS.md` ancestry. Conditional globs are based on prompt paths, open files, edits, and pending operations, which is a rule engine rather than filesystem inheritance. In multi-root workspaces, only the primary workspace supplies `.clinerules/`.

Sources: [Cline Rules](https://docs.cline.bot/customization/cline-rules), [multi-root behavior](https://docs.cline.bot/features/multiroot-workspace#cline-rules), [official rule paths](https://github.com/cline/cline/blob/main/sdk/packages/shared/src/storage/paths.ts).

### Roo Code

Roo has explicit global/project and generic/mode-specific tiers under `.roo/rules*`. Its documented assembly order includes prompt-tab instructions, mode rules, `.rooignore`, `AGENTS.md`, generic rules, and legacy fallbacks. Rule directories are recursively read and sorted case-insensitively by basename; a directory suppresses its corresponding legacy single-file fallback.

Root `AGENTS.md` is enabled by default, with `AGENT.md` fallback. Official source also loads `AGENTS.local.md` afterward. Optional subfolder discovery exists, but source indicates it is tied to child `.roo` directories rather than every arbitrary nested `AGENTS.md`; this is **source-observed**.

Sources: [custom instructions](https://docs.roocode.com/features/custom-instructions), [settings](https://docs.roocode.com/features/settings-management#roo-clineuseagentrules), [official prompt assembly](https://github.com/RooCodeInc/Roo-Code/blob/main/src/core/prompts/sections/custom-instructions.ts).

### Continue

Continue project rules live under `.continue/rules/`, load lexicographically, and can be always-on or conditional through `globs`, `regex`, and model retrieval. Config can import Hub rules or `file://` rules. Rules apply to Agent, Chat, and Edit, not autocomplete.

The IDE and CLI differ. The IDE selects one root compatibility file from `AGENTS.md`, `AGENT.md`, or `CLAUDE.md`, combines workspace and global rules, and **source-observably** finds colocated `rules.md` files scoped to their subtrees. The CLI checks a root compatibility file, project/global rules, and explicit `--rule` inputs, but does not use the IDE’s colocated discovery. Neither implements a general nested `AGENTS.md` ancestor chain.

Sources: [rules deep dive](https://docs.continue.dev/customize/deep-dives/rules), [CLI rules](https://docs.continue.dev/guides/cli#how-to-configure-rules), [official IDE loader](https://github.com/continuedev/continue/blob/main/core/config/markdown/loadMarkdownRules.ts), [official CLI assembly](https://github.com/continuedev/continue/blob/main/extensions/cli/src/systemMessage.ts).

### Junie

Junie checks the project root in fallback order: `.junie/AGENTS.md`, `AGENTS.md`, then legacy `.junie/guidelines.md` or `.junie/guidelines/`. It combines the selected project guidance with `~/.junie/AGENTS.md`; project instructions take precedence and identical content is deduplicated.

No ancestor or descendant hierarchy is documented. Instructions are read when a CLI task starts. Junie can offer to import another agent’s file into `.junie/AGENTS.md`, but that is migration rather than ongoing compatibility loading.

Sources: [guidelines and memory](https://junie.jetbrains.com/docs/guidelines-and-memory.html), [official guidelines repository](https://github.com/JetBrains/junie-guidelines).

### Kiro CLI

Amazon Q Developer CLI became Kiro CLI. Kiro combines global `~/.kiro/steering/` with workspace `.kiro/steering/`; workspace instructions win conflicts. Root `AGENTS.md` is always included. No directory-scoped hierarchy is documented.

Custom agents can add `file://` paths and globs as resources. Context files collectively may consume up to 75% of the active model’s context window, after which excess files are dropped. `/context show` reports usage. Steering has no documented textual import syntax.

Sources: [Kiro steering](https://kiro.dev/docs/cli/steering/), [custom-agent resources](https://kiro.dev/docs/cli/custom-agents/configuration-reference/#resources-field), [context management](https://kiro.dev/docs/cli/chat/context/), [Amazon Q transition](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html).

### Aider

Aider is the useful negative baseline: it has no meaningful automatic repository instruction hierarchy. A convention file such as `CONVENTIONS.md` must be loaded with `/read`, `--read`, or `.aider.conf.yml`. Multiple files and directories can be supplied as read-only context, but `AGENTS.md` has no special meaning.

Sources: [convention files](https://aider.chat/docs/usage/conventions.html), [configuration](https://aider.chat/docs/config.html), [`--read`](https://aider.chat/docs/config/options.html#read-file).

## Differences by dimension

### Boundaries are incompatible

- **Filesystem root:** Claude Code and pi can include instruction files above a repository.
- **Home directory:** Amp walks through `$HOME` and also has dedicated global/system locations.
- **Project or Git root:** Codex, OpenCode, Gemini, goose, and source-observed Qwen behavior stop at a project boundary by default.
- **Workspace root only:** Junie, Kiro, Aider, and several IDE rule systems do not promise ancestor traversal.

The visualizer must model boundary detection independently from filename matching. Running the same harness from `repo/packages/api` versus `repo` can produce a different chain.

### “Override” has three meanings

1. **Candidate replacement:** `AGENTS.override.md` suppresses `AGENTS.md` in the same Codex directory; OpenCode’s `AGENTS.md` suppresses its `CLAUDE.md` fallback.
2. **Tier precedence:** Cursor Team/Project/User and Cline workspace/global tiers claim a conflict winner.
3. **Prose ordering:** Claude, pi, Gemini, Amp, and others concatenate text; “more specific wins” is guidance to the model, not a deterministic merge operation.

These should be rendered differently. A file excluded by candidate selection is not the same as a loaded file whose prose may lose a conflict.

### Descendants change context over time

Claude, Gemini, Amp, goose, OpenCode source, Cursor, Windsurf, and Copilot can add instructions after startup based on paths the agent touches. Therefore there may be no single final context for a session. The CLI should support at least:

- **Initial context** for a launch directory.
- **Context after accessing a path**, potentially repeated for several paths.
- **Activation history** showing when each lazy or conditional rule entered context.

### Compatibility is asymmetric

- A repository with only `AGENTS.md` is invisible to Claude Code unless bridged.
- A repository with only `CLAUDE.md` works automatically in Claude, OpenCode fallback, pi fallback, Amp fallback, Cursor root, and Copilot CLI, but not by default in Codex, Gemini, Qwen, goose, Windsurf, Kiro, or Aider.
- Gemini, Qwen, goose, Codex, and OpenCode can be configured to recognize additional names, but their configuration semantics differ: replacement, ordered fallback, or additive explicit source.

## Implications for `instructmd`

The CLI should not implement a generic “find Markdown files” pass followed by harness-specific filenames. A faithful design needs a small evaluator per harness with shared primitives.

### Minimum domain model

Each discovered item should retain:

- `path` or remote URL
- `source_scope`: managed, system, global, organization, team, project, directory, local, or session
- `discovery_reason`: ancestor walk, exact location, fallback, glob, import, file access, model selection, or explicit configuration
- `boundary`: filesystem, home, worktree, Git, workspace, or none
- `selection_state`: selected, shadowed by same-directory candidate, excluded, empty, truncated, or over budget
- `activation`: startup, always, path/glob, model decision, manual, task, or request
- `order` in the final prompt, where known
- `evidence`: documented, source-observed, or unknown

### Evaluator phases

1. Resolve harness configuration and environment variables.
2. Detect the harness-specific boundary.
3. Enumerate global and initial project candidates.
4. Apply candidate fallback and exclusion rules.
5. Expand imports with harness-specific safety and depth rules.
6. Apply size or context budgets.
7. Simulate path accesses for lazy and conditional rules.
8. Render discovered files, excluded candidates, ordered prompt blocks, and unresolved precedence.

### Accuracy rules

- Never label later concatenated prose as a guaranteed override unless the vendor documents conflict precedence.
- Show source-observed behavior with a warning because it may drift from a released version.
- Pin evaluator behavior to a harness version; mutable `main`/`dev` source is not a stable specification.
- Preserve provenance through imports and deduplication.
- Treat cloud Copilot and Copilot CLI, and Continue IDE and CLI, as distinct harnesses.
- Include a trace mode explaining every stop, fallback, and exclusion decision.

## Open questions

Official documentation leaves several behaviors underspecified:

- Whether both root `CLAUDE.md` and `.claude/CLAUDE.md` load when both exist.
- Whether Copilot coding agent retains ancestor `AGENTS.md` files when the nearest one takes precedence.
- Same-tier ordering for Cursor, Windsurf, Cline, and matching Copilot path rules.
- Hot-reload behavior for several IDE agents.
- Stable guarantees for OpenCode lazy loading, Qwen ancestor traversal, Roo local/subfolder files, and Continue colocated rules.

An accurate implementation should encode these as unknowns or version-tested behavior, not silently invent precedence.

## Product notes

- Windsurf was renamed **Devin Desktop** in June 2026; legacy Windsurf paths remain supported.
- Amazon Q Developer CLI became **Kiro CLI**; the old open-source Q CLI is maintenance-only.
- Amp became independent from Sourcegraph in December 2025 and is now published by Amp Frontier Corporation.
- goose moved from Block to the Agentic AI Foundation at the Linux Foundation.

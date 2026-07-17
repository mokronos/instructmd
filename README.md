# instructmd

**[Visit the instructmd website](https://mokronos.github.io/instructmd/)**

`instructmd` is a Rust CLI for showing which Markdown instruction files a coding agent loads at startup, in what order, and why. It makes global, project, directory, local, and shadowed instruction files visible without changing your repository.

## Supported agents

The current resolver supports these startup discovery models:

- OpenCode
- Claude Code
- OpenAI Codex CLI
- pi
- Gemini CLI
- Amp
- goose
- Qwen Code

## Usage

The agent argument defaults to `opencode`; the target directory defaults to the current directory.

```bash
instructmd [OPTIONS] [AGENT]

# Inspect Codex instructions for a directory
instructmd codex --dir ./api

# Show paths and inclusion/exclusion explanations, without file contents
instructmd --verbose --no-content claude --dir .
```

Options:

| Option | Description |
| --- | --- |
| `-v`, `--verbose` | Explain included layers and existing files excluded during resolution. |
| `--dir <DIR>` | Directory to inspect (default: `.`). |
| `--no-content` | Do not print selected file contents. |
| `--no-color` | Disable ANSI color output. |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

## Install

```bash
cargo install instructmd
```

### Install from this checkout

Prerequisite: a current Rust toolchain with Cargo.

```bash
git clone https://github.com/mokronos/instructmd.git
cd instructmd
cargo install --path cli
```

Or run it directly from the checkout:

```bash
cargo run --manifest-path cli/Cargo.toml -- codex --dir .
```

## Development

The Rust CLI lives in `cli/`; the website is an Astro project at the repository root.

```bash
# Rust CLI
cargo fmt --manifest-path cli/Cargo.toml --check
cargo test --manifest-path cli/Cargo.toml
cargo run --manifest-path cli/Cargo.toml -- --help

# Astro website
npm ci
npm run dev
npm run check
npm run build
```

## Research and direction

- [Vision](VISION.md)
- [Agent instruction discovery research](docs/agent-instruction-discovery.md)
- [Prompt composition and inspection research](docs/prompt-composition-research.md)
- [Interactive instruction-discovery atlas](docs/agent-instruction-discovery.html)

## Current limitations

This is a startup instruction-file resolver, not a capture of an agent's complete model request or final session context. It currently does not simulate imports, configured instruction sources, URLs or globs, conditional rules, lazy descendant discovery, reloads, or context added by tools, skills, subagents, compaction, and provider-side processing. The Codex 32 KiB project-instruction cap is reported when exceeded but is not simulated. Agent behavior changes quickly; see the linked research for the evidence and scope behind each model.

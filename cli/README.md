# instructmd

Show the markdown instruction files a coding agent would load when it starts.

## Install

```sh
cargo install --path cli
```

This puts `instructmd` on your PATH (`~/.cargo/bin`), so you can run it from anywhere:

```sh
instructmd                          # opencode layering for the current directory
instructmd claude --no-content
instructmd codex --dir ./packages/api --no-color
```

During development you can run it without installing:

```sh
cargo run --manifest-path cli/Cargo.toml -- claude --no-content
```

## Usage

```
instructmd [AGENT] [--dir <PATH>] [--no-content] [--no-color]
```

| Option | Description |
|---|---|
| `AGENT` | Which coding agent's discovery rules to simulate. One of `opencode` (default), `claude`, `codex`, `pi`, `gemini`, `amp`, `goose`, `qwen`. |
| `--dir <PATH>` | Directory to resolve from, as if the agent were launched there. Defaults to the current directory. Affects boundary detection (git root, home, filesystem root) and the ancestor chain. |
| `--no-content` | Only print the layer list — scope, path, and discovery reason per file — without the file contents. Useful for a quick overview of large instruction files. |
| `--no-color` | Disable colored output. Colors are also disabled automatically when stdout is not a terminal (e.g. when piping). |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

Each layer (header and content) gets its own color from a palette of six distinct colors, cycling when there are more layers. The header line shows the layer number, scope (GLOBAL, PROJECT ROOT, DIRECTORY, LOCAL), file path, and the discovery reason. Files that exist on disk but lost same-directory candidate selection (e.g. a `CLAUDE.md` next to an `AGENTS.md` under opencode) are listed at the end as shadowed candidates.

The tool models initial startup resolution only, not lazy discovery, conditional rules, imports, or truncation.

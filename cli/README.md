# instructmd

Show the markdown instruction files a coding agent would load when it starts.

```sh
cargo run --manifest-path cli/Cargo.toml -- opencode --dir ./packages/api
cargo run --manifest-path cli/Cargo.toml -- claude --no-content
cargo run --manifest-path cli/Cargo.toml -- codex --dir . --no-color
```

`AGENT` defaults to `opencode`; supported values are `opencode`, `claude`, `codex`, `pi`, `gemini`, `amp`, `goose`, and `qwen`. The tool models initial startup resolution only, not lazy discovery, conditional rules, imports, or truncation.

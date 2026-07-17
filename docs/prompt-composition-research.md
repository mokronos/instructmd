# Coding Harness Prompt Composition and Inspection

Research snapshot: 2026-07-17. Coding harnesses change quickly, so the commands and source links below are version-sensitive.

## Short Answer

There usually is no single "final system message." Modern harnesses send a structured request containing some combination of:

- A system instruction or top-level `instructions` field
- Developer, user, assistant, and tool-result messages
- Tool definitions and schemas
- Provider options, output schemas, and safety settings
- Context added later by skills, hooks, compaction, or subagents

There are also three different things one might want to inspect:

1. **Loaded sources:** which `AGENTS.md`, `CLAUDE.md`, skills, agents, and configuration files were discovered.
2. **Client request:** the structured payload the harness assembled and passed to its provider SDK or HTTP client.
3. **Model-effective context:** the client request after provider-side routing, safety processing, normalization, and any hidden server instructions.

The first is often supported. The second is supported by only some harnesses, or requires a hook, telemetry, or a trusted gateway. The third is not observable from a local client.

## Comparison

| Harness | Best supported inspection | Sees complete client payload? | Main limitation |
| --- | --- | --- | --- |
| Codex CLI 0.144.5 | `codex debug prompt-input "..."` | No | Shows a fresh turn's model-visible `input`, but not top-level base `instructions`, tools, or the active session |
| OpenCode 1.18.3 | Plugin system/message/parameter hooks | No single hook does | Provider transforms and AI SDK serialization occur after the most useful hooks |
| Claude Code 2.1.193+ | Raw API body OpenTelemetry | Yes, at the Anthropic API request-object level | Still cannot show provider-side additions or transformations |
| Pi 0.80.10 | `before_provider_request` extension event | Yes, at the provider payload-object level | HTTP serialization and provider-side context remain later boundaries |
| Gemini CLI 0.51.0 | `/chat debug` or `/resume debug` | Nearly | Exports the model-level request, but not headers, auth, or every helper request |
| Aider 0.86.0 | `--show-prompts` and `--llm-history-file` | No | Output is captured before LiteLLM's provider-specific adaptation |

The most useful practical result is:

- Use a harness's source/context viewer when debugging discovery and precedence.
- Use Pi's provider hook, Claude Code's raw-body telemetry, or Gemini CLI's request export when debugging actual request assembly.
- Use a controlled logging gateway or TLS-intercepting proxy when byte-level outbound auditing is required and the harness has no final request hook.
- Never treat a client capture as proof of the provider's final internal model context.

## Codex CLI

Local version checked: `codex-cli 0.144.5`.

### Composition

Codex uses the OpenAI Responses-style structure. Its internal `Prompt` has separate `input`, `tools`, `parallel_tool_calls`, `base_instructions`, and `output_schema` fields; this is not one concatenated string ([source](https://github.com/openai/codex/blob/rust-v0.144.5/codex-rs/core/src/client_common.rs)).

The relevant composition is approximately:

1. Resolve base instructions from an explicit configuration override, resumed-session metadata, or the selected model's default instructions.
2. Discover global instructions from `$CODEX_HOME/AGENTS.override.md`, falling back to `$CODEX_HOME/AGENTS.md`.
3. Walk from the project root to the current directory. In each directory, use at most one of `AGENTS.override.md`, `AGENTS.md`, or a configured fallback filename.
4. Concatenate project instructions root-first, subject to `project_doc_max_bytes`, which defaults to 32 KiB.
5. Add environment/session context and the current conversation.
6. Initially advertise skill name, description, and path. Load the full `SKILL.md` only when the skill is used.
7. Supply tool definitions and output schemas in separate request fields.

Codex's official [AGENTS.md documentation](https://developers.openai.com/codex/agent-configuration/agents-md) documents discovery, ordering, and limits. Its [skills documentation](https://developers.openai.com/codex/build-skills) documents progressive disclosure.

An important role detail is that discovered `AGENTS.md` text is rendered as a model-visible **user-role context fragment**, not folded into the base system/developer instructions ([source](https://github.com/openai/codex/blob/rust-v0.144.5/codex-rs/core/src/context/user_instructions.rs)). Subagents have their own context and instructions; their entire context is not merged into the parent request.

### Inspection

The best built-in command in 0.144.5 is:

```bash
codex debug prompt-input "your prompt"
```

Its own help describes the result as the "model-visible prompt input list as JSON." The implementation creates a fresh ephemeral session and returns `prompt.input` ([source](https://github.com/openai/codex/blob/rust-v0.144.5/codex-rs/core/src/prompt_debug.rs)). Consequently it:

- Is useful for seeing developer/user context items, discovered `AGENTS.md` content, and a proposed user message.
- Does not dump an already-running session.
- Does not return the separate base `instructions`, tool schemas, output schema, or all provider options.
- Can differ from a later turn after skill loading, tool results, compaction, or other context changes.

Codex also stores rollout JSONL files under `$CODEX_HOME/sessions/YYYY/MM/DD/`. They contain session metadata and conversation events and can help reconstruct a request, but they are not guaranteed byte-for-byte request captures. Plaintext TUI logging can be enabled with `codex -c log_dir=./.codex-log`.

For the exact client HTTP request, route a custom provider/base URL through a controlled logging gateway. That can reveal the top-level `instructions`, `input`, `tools`, and options that `prompt-input` omits. The gateway must protect authorization headers and request bodies.

### Local Source Evidence

This machine also has a clean Codex checkout at `/home/mokronos/projects/codex`. Relevant files are:

- `codex-rs/core/src/project_doc.rs`: project instruction discovery and limits
- `codex-rs/core/src/context/user_instructions.rs`: `AGENTS.md` role and wrapper
- `codex-rs/core/src/client_common.rs`: structured prompt representation
- `codex-rs/core/src/client.rs`: Responses API request construction
- `codex-rs/core/src/prompt_debug.rs`: `debug prompt-input`

The checkout is not guaranteed to be the exact source commit used to build the installed 0.144.5 binary, so the release-tag links above are the stronger version-specific evidence.

## OpenCode

Local version checked: `1.18.3`.

### Composition

OpenCode's request preparation uses this high-level order:

1. Use the selected agent's custom prompt, if configured; otherwise use OpenCode's model-specific built-in prompt.
2. Add environment context.
3. Add discovered and configured instruction files.
4. Add MCP-provided instructions.
5. Advertise available skills.
6. Add structured-output instructions when applicable.
7. Add any per-message custom system text.
8. Prepend the resulting system content to converted conversation messages, except where a provider uses a separate instructions option.
9. Apply provider-specific message and option transforms, then let the Vercel AI SDK serialize the provider request.

A custom agent prompt therefore **replaces the model-specific built-in prompt**; environment and project context are still appended. The release [session assembly](https://github.com/anomalyco/opencode/blob/v1.18.3/packages/opencode/src/session/prompt.ts) and [request preparation](https://github.com/anomalyco/opencode/blob/v1.18.3/packages/opencode/src/session/llm/request.ts) show that ordering. For OpenAI OAuth, the joined system content is passed as provider `instructions`; for most other paths it becomes one or more system-role messages.

Instruction discovery is more nuanced than "load every file":

- Global priority is `~/.config/opencode/AGENTS.md`, then `~/.claude/CLAUDE.md` when Claude compatibility is enabled.
- Project filename priority is `AGENTS.md`, then `CLAUDE.md`, then deprecated `CONTEXT.md`.
- OpenCode takes the first filename family that has matches, while collecting matching ancestor files for that family.
- `instructions` in `opencode.json` can add absolute paths, relative paths, globs, home-relative paths, and HTTP(S) URLs.
- Reading a file can lazily attach a nested instruction file found nearer that file.

See the official [rules documentation](https://opencode.ai/docs/rules/) and release [instruction source](https://github.com/anomalyco/opencode/blob/v1.18.3/packages/opencode/src/session/instruction.ts).

Skills are progressive: the `skill` tool description advertises names and descriptions, while the full `SKILL.md` is loaded on demand ([skills documentation](https://opencode.ai/docs/skills/)). A subagent runs in a separate child session with its own agent, system construction, messages, and tools ([agents documentation](https://opencode.ai/docs/agents/)).

### Inspection

There is no `opencode debug prompt` equivalent in 1.18.3. Useful inventory commands include:

```bash
opencode debug config
opencode debug agent build
opencode debug skill
opencode debug paths
opencode --print-logs --log-level DEBUG
opencode export <session-id> --sanitize
```

These show resolved configuration, agents, available skills, paths, logs, and stored session data. They do not show the final provider request.

OpenCode plugins can inspect most application-level inputs through these hooks:

- `experimental.chat.system.transform`: assembled OpenCode system strings
- `experimental.chat.messages.transform`: internal conversation messages before final conversion
- `chat.params`: generation parameters and provider options
- `chat.headers`: application-added headers

The system hook is visible in the release [request preparation](https://github.com/anomalyco/opencode/blob/v1.18.3/packages/opencode/src/session/llm/request.ts). It is experimental and runs before later provider transforms and AI SDK serialization. There is no single normal plugin hook that receives the final canonical provider payload.

For an exact outbound request, use a controlled provider endpoint or trusted logging proxy, or instrument OpenCode after the AI SDK's provider adaptation. Session exports are not substitutes because transient system content and provider transforms may not be stored.

### Local Source Evidence

This machine has a clean OpenCode checkout at `/home/mokronos/projects/opencode`, slightly older than the installed binary. Relevant files are:

- `packages/opencode/src/session/instruction.ts`: instruction discovery
- `packages/opencode/src/session/system.ts`: built-in/model-specific prompts
- `packages/opencode/src/session/prompt.ts`: environment, instructions, MCP, and skill assembly
- `packages/opencode/src/session/llm/request.ts`: prepared request fields and plugin hooks
- `packages/opencode/src/session/llm.ts`: provider transforms and model call

## Claude Code

### Composition

Claude Code also sends structured Messages API requests rather than one final string. The API has a top-level `system` field, a `messages` sequence, and separate tools. Unlike OpenAI's public API, Anthropic's Messages API does not define a separate developer role.

Claude Code's startup/context model is approximately:

1. Load the hidden core system prompt and environment information.
2. Apply output-style and explicit system-prompt configuration.
3. Load auto-memory and `CLAUDE.md` context.
4. Advertise MCP tools and skill descriptions; load full skill content when invoked.
5. Add the user's prompt and subsequent conversation/tool turns.
6. Apply hook-provided context and later compaction when applicable.

Claude's own [context-window documentation](https://code.claude.com/docs/en/context-window) says the core system prompt is always loaded first and "You never see it." Its [memory documentation](https://code.claude.com/docs/en/memory) adds an important role distinction: `CLAUDE.md` content is delivered as a **user message after the system prompt**, not as part of the system prompt.

`CLAUDE.md` sources include managed, user, project, local, imported, rules, and lazily loaded nested files. The same documentation describes their order and the `/context` and `/memory` inspection commands. Skills advertise descriptions initially and load their full bodies when used ([skills documentation](https://code.claude.com/docs/en/skills)).

Each subagent has a separate context window, system prompt, tools, permissions, and task. A custom subagent file's Markdown body becomes that subagent's system prompt; it does not replace the parent request ([subagent documentation](https://code.claude.com/docs/en/sub-agents)).

### Inspection

For source/configuration inspection inside Claude Code, use:

```text
/context
/memory
/skills
/agents
/hooks
/mcp
/permissions
/status
```

`/context` is the best supported view of what loaded and how much context it occupies. `/memory` exposes instruction and auto-memory files. The `InstructionsLoaded` hook can record exactly which instruction files load and why. These do not print the hidden core system prompt or complete request.

Claude Code 2.1.193 and later has the strongest supported raw-request capture among the harnesses reviewed. Create a private directory and enable raw API body logging:

```bash
mkdir -m 700 /tmp/claude-api-bodies
CLAUDE_CODE_ENABLE_TELEMETRY=1 \
OTEL_LOGS_EXPORTER=console \
OTEL_LOG_RAW_API_BODIES=file:/tmp/claude-api-bodies \
claude
```

According to the official [monitoring documentation](https://code.claude.com/docs/en/monitoring-usage), `OTEL_LOG_RAW_API_BODIES=1` emits inline bodies truncated at 60 KB, while `file:<directory>` writes untruncated request and response bodies and emits a reference. The request JSON exposes the client-sent `system`, `messages`, and `tools` fields.

This capture is highly sensitive. It can contain prompts, source code, credentials returned by tools, and private instructions. Use a protected temporary directory, do not commit it, and delete it after inspection.

An enterprise LLM gateway configured through `ANTHROPIC_BASE_URL` can also audit outbound requests. Neither method reveals server-side additions or normalization after Anthropic receives the request.

## Pi

Version researched: `0.80.10`, from the current `earendil-works/pi` repository (the historical `badlogic/pi-mono` URLs redirect there).

### Composition

Pi builds a concrete system-prompt string containing:

1. Its built-in coding-agent identity and guidance, or a custom replacement prompt.
2. Active tool summaries and tool-specific guidelines.
3. Pi documentation paths.
4. Text from append-system-prompt configuration.
5. Project context files inside `<project_context>` and `<project_instructions>` blocks.
6. Available skill metadata when the read tool is active.
7. The current working directory.

The implementation is directly readable in [system-prompt.ts](https://github.com/earendil-works/pi/blob/v0.80.10/packages/coding-agent/src/core/system-prompt.ts). A custom prompt replaces the built-in body but Pi still appends configured additions, context files, skills, and the working directory.

Pi loads one supported context filename per directory, preferring `AGENTS.md`/`AGENTS.MD` and then `CLAUDE.md`/`CLAUDE.MD`, from global scope and the root-to-current-directory path. Skill metadata is present initially; explicit skill invocation expands the skill into the user-side prompt flow.

Pi does not have a built-in subagent framework comparable to Codex, OpenCode, or Claude Code. Extensions can create separate subordinate sessions, in which case each has its own prompt and history.

### Inspection

Pi offers inspection at several boundaries:

- `ctx.getSystemPrompt()`: Pi's current system-prompt string.
- `before_agent_start`: expanded user prompt, assembled system prompt, and structured prompt-construction options.
- `context`: messages immediately before each LLM call.
- `before_provider_headers`: outgoing headers after assembly.
- `before_provider_request`: the provider-specific payload after serialization logic and immediately before sending.

The last hook is the best direct answer to "what is Pi sending?" The official [extension documentation](https://github.com/earendil-works/pi/blob/v0.80.10/packages/coding-agent/docs/extensions.md) explicitly says it can inspect or replace the provider payload and that payload-level changes are not reflected by `ctx.getSystemPrompt()`.

A minimal inspection extension is:

```ts
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"

export default function (pi: ExtensionAPI) {
  pi.on("before_agent_start", (event) => {
    console.log("SYSTEM", event.systemPrompt)
    console.log("USER", event.prompt)
  })

  pi.on("before_provider_request", (event) => {
    console.log("PAYLOAD", JSON.stringify(event.payload, null, 2))
  })
}
```

Place it in `~/.pi/agent/extensions/request-debug.ts` or `.pi/extensions/request-debug.ts`. Later-loaded extensions can still alter the prompt or payload after an earlier handler, so load order matters. The payload can contain secrets and source code; do not leave permanent logging enabled.

## Gemini CLI

Version researched: stable `0.51.0`.

### Composition

Gemini CLI separates:

- `systemInstruction`: its dynamically built core prompt plus applicable global/private memory.
- An initial hidden user `session_context`: environment, workspace, project `GEMINI.md`, extension, and MCP context.
- `contents`: conversation, current prompt, tool results, expanded commands, and activated skills.
- `tools`: built-in, MCP, extension, skill-activation, and subagent declarations.

This role-level split is more precise than thinking of all `GEMINI.md` content as one system prompt. The 0.51.0 [prompt builder](https://github.com/google-gemini/gemini-cli/blob/v0.51.0/packages/core/src/prompts/promptProvider.ts), [session-context builder](https://github.com/google-gemini/gemini-cli/blob/v0.51.0/packages/core/src/utils/environmentContext.ts), and [request assembly](https://github.com/google-gemini/gemini-cli/blob/v0.51.0/packages/core/src/core/geminiChat.ts) expose those separate fields. Skills use progressive disclosure: initial metadata advertises them and activation returns the full `SKILL.md` body as tool content ([source](https://github.com/google-gemini/gemini-cli/blob/v0.51.0/packages/core/src/tools/activate-skill.ts)). Just-in-time nested `GEMINI.md` files can be attached when file tools access their directories.

The official [GEMINI.md documentation](https://geminicli.com/docs/cli/gemini-md/) covers global, workspace, and JIT discovery, imports, and configurable context filenames. `GEMINI_SYSTEM_MD` replaces the built-in core system prompt, while `GEMINI_WRITE_SYSTEM_MD` exports the generated prompt ([system-prompt documentation](https://geminicli.com/docs/cli/system-prompt/)).

### Inspection

Gemini CLI has unusually good built-in introspection:

```text
/memory list
/memory show
/skills list
/agents list
/extensions list
/tools desc
/mcp schema
/chat debug
```

`/memory show` displays concatenated hierarchical memory, but not the built-in prompt, current conversation, tool schemas, or all JIT/skill content. `GEMINI_WRITE_SYSTEM_MD=/path gemini` exports the generated `systemInstruction`, but not messages or tools.

`/chat debug` (also documented as `/resume debug`) exports the most recent API request as JSON. In stable 0.51.0 it includes the latest main-agent model-level contents, system instruction, tools, and generation/safety/tool configuration. It does not include URL, headers, authentication, literal HTTP bytes, or every helper/non-streaming request. The live [command reference](https://geminicli.com/docs/reference/commands/) inconsistently labels the `/resume` spelling as nightly-only, so check the installed version.

For exact transport inspection, API-key users can route `GOOGLE_GEMINI_BASE_URL` through a controlled endpoint, or use a trusted TLS-intercepting proxy. Google-login Code Assist uses a different wrapper and endpoint, so `/chat debug` is safer than attempting to redirect undocumented OAuth transport.

## Aider

Version researched: `0.86.0`.

Aider provides three useful inspection options:

```bash
aider --show-prompts
aider --llm-history-file .aider.llm.history
aider --verbose
```

The official [options reference](https://aider.chat/docs/config/options.html) describes `--show-prompts` as printing system prompts and exiting, and `--llm-history-file` as logging the LLM conversation. Aider's [message assembly](https://github.com/Aider-AI/aider/blob/v0.86.0/aider/coders/chat_chunks.py) includes system prompt, examples, read-only files, repository map, conversation history, editable files, current messages, and a final reminder.

These views occur before `litellm.completion()`. LiteLLM may adapt roles, tools, parameters, and provider payload shape afterward. Aider therefore exposes its logical prompt well but not the final provider-specific request. A controlled LiteLLM gateway or transport proxy is required for that boundary.

## Implications for an Instruction Visualizer

An instruction visualizer should not promise one universal "final system message." A more accurate output model is:

```text
discovered sources
  -> harness composition and role assignment
  -> conversation/context mutations
  -> provider request fields
  -> provider-side unknowns
```

For each harness, it should show:

- Source path, scope, discovery reason, and load time
- Precedence or concatenation order
- Whether content replaces or appends to built-ins
- The actual request role/field (`instructions`, `system`, `user`, `contents`, tool result, or tool schema)
- Whether loading is eager, lazy, skill-triggered, file-triggered, or subagent-only
- Truncation and token/byte limits
- Hooks or transforms that can still modify content after the displayed stage
- A confidence label such as `discovery reconstruction`, `client assembly`, or `captured outbound payload`

The tool can accurately emulate documented local discovery, but only a harness-native hook or request capture can prove what a particular running version sent. Even that cannot prove the provider's hidden final model context.

## Security Notes

- Prompt/request dumps commonly contain proprietary source, secrets in tool output, private memory files, and authorization metadata.
- Prefer sanitized source inventories before raw request capture.
- Store raw captures in a mode-0700 temporary directory and remove them after use.
- Never commit session rollouts, request bodies, telemetry exports, or LLM history files without reviewing and redacting them.
- A normal HTTP `CONNECT` proxy cannot read HTTPS bodies. Decryption requires a trusted local CA or a gateway that terminates TLS, which has substantial security implications.

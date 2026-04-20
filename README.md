# buoya-ai-cli

An AI-powered coding agent that runs in your terminal. Built in Rust for speed, safety, and reliability.

> **Status:** Early development — core architecture is in place, but the project is not yet feature-complete.

## What Is This

buoya-ai-cli is a terminal-based AI coding assistant — similar in spirit to [Claude Code](https://claude.ai/code), [OpenAI Codex CLI](https://github.com/openai/codex), or [Gemini CLI](https://github.com/google-gemini/gemini-cli). You describe a task in natural language, and the agent uses a set of file system tools to explore your codebase, read files, and (eventually) write and edit code on your behalf.

The goal is a fully autonomous coding partner that lives in your terminal: understands your project, reasons about your code, and takes action — all driven by a local or remote LLM.

## How It Works

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│  Your prompt │────▶│   LLM Agent      │────▶│  File Tools │
│  (terminal)  │◀────│  (Ollama / ...)  │     │  (read, ls) │
└─────────────┘     └──────────────────┘     └─────────────┘
```

1. **You type a task** — e.g. _"Explore this project and tell me what it does."_
2. **The agent reasons** — sends your prompt + system instructions to an LLM.
3. **The agent acts** — the LLM decides which tools to call (list directory, read a file, search for patterns, etc.) and invokes them.
4. **You get an answer** — the agent loops through tool calls and reasoning (up to 10 turns) and returns a final response.

## Current Capabilities

### File System Tools

| Tool               | What It Does                                                                                        |
| ------------------ | --------------------------------------------------------------------------------------------------- |
| **List Directory** | Lists files and subdirectories, with optional recursive traversal and file-type distribution stats. |
| **Find Files**     | Searches for files by name pattern (supports wildcards like `*.rs`).                                |
| **Get File Info**  | Returns metadata for a single file (size, extension, last modified).                                |
| **Read File**      | Reads file contents with line-range support and safe path resolution.                               |

### LLM Integration

- Currently supports **Ollama** as the LLM provider (run models locally).
- Configurable model, temperature, system prompt, and endpoint URL.
- Multi-turn agent loop — the LLM can call tools iteratively to gather information.

### Safety

- All file operations are scoped to the project root — path traversal attacks are blocked.
- File existence and permission checks before every operation.

## Project Structure

```
src/
├── main.rs                    # Entry point — loads config, builds agent, runs prompt
├── config/
│   ├── mod.rs                 # Configuration loader
│   └── config.toml            # LLM and agent settings
├── llm/
│   ├── mod.rs                 # LLM module root
│   ├── agent/
│   │   ├── mod.rs             # Agent builder and execution loop
│   │   └── providers.rs       # Provider-specific agent construction (Ollama)
│   └── tools/
│       ├── mod.rs             # Tool registry
│       ├── file_reader.rs     # Read file contents
│       ├── file_writer.rs     # (planned) Write files
│       ├── file_editer.rs     # (planned) Edit files
│       └── file_explorer/
│           ├── mod.rs         # Module root + shared core logic
│           ├── list_directory.rs
│           ├── find_files.rs
│           └── get_file_info.rs
└── types/
    └── mod.rs                 # FileInfo, DirectoryInfo types
```

## Configuration

Configuration lives in `src/config/config.toml`:

```toml
provider = "ollama"
api_key = ""
api_base_url = "http://localhost:11434"
model = "gemma4:e2b"
system_instruction = "You are a coding agent helper."
temperature = 0.2
think = false
```

| Field                | Description                                   |
| -------------------- | --------------------------------------------- |
| `provider`           | LLM provider (`ollama` — more planned)        |
| `api_key`            | API key (not needed for local Ollama)         |
| `api_base_url`       | LLM endpoint URL                              |
| `model`              | Model identifier to use                       |
| `system_instruction` | System prompt prepended to every conversation |
| `temperature`        | Sampling temperature (0.0 – 1.0)              |
| `think`              | Enable extended reasoning mode                |

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [Ollama](https://ollama.ai/) running locally (or another compatible endpoint)

### Build & Run

```bash
git clone https://github.com/ayoubbuoya/buoya-ai-cli.git
cd buoya-ai-cli
cargo run
```

The agent will explore the current project directory and describe what it finds.

## Roadmap

This project is under active development. Here's what's planned:

- [ ] **File Writer** — create new files from agent output
- [ ] **File Editor** — apply targeted edits to existing files (find-and-replace, line insertion)
- [ ] **Interactive REPL** — persistent conversation loop instead of single-shot prompts
- [ ] **Multi-provider support** — OpenAI, Anthropic, Google, and other LLM APIs
- [ ] **Shell tool** — run terminal commands with user approval
- [ ] **Git integration** — diff, commit, branch management through the agent
- [ ] **Code search** — grep/regex search across the project
- [ ] **Streaming output** — real-time token streaming for faster feedback
- [ ] **Context management** — smart file selection to stay within token limits
- [ ] **Approval workflow** — user confirmation before destructive file operations
- [ ] **Project awareness** — understand project type (Rust, Node, Python, etc.) and adapt behavior
- [ ] **Conversation history** — persist and resume sessions

## Tech Stack

| Component      | Technology                                       |
| -------------- | ------------------------------------------------ |
| Language       | Rust (edition 2024)                              |
| AI Framework   | [rig-core](https://github.com/0xPlaygrounds/rig) |
| Async Runtime  | Tokio                                            |
| Config         | TOML + serde                                     |
| File Traversal | walkdir                                          |

## Contributing

Contributions are welcome. The codebase is modular by design — adding a new tool or LLM provider is straightforward:

1. **New tool** — implement the `rig::tool::Tool` trait in `src/llm/tools/`, then register it in the agent builder.
2. **New provider** — add a variant to `ProviderAgent` in `src/llm/agent/providers.rs` and wire up the client.

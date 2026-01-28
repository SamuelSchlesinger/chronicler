# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Handoff Protocol

**At the end of each development session**, update `UnreadMessage.md` with a message for the next developer agent. Include:
- What you worked on
- Current state of the code
- Any known issues or incomplete work
- Suggested next steps

**At the start of each session**, read `UnreadMessage.md` to understand the current state, then clear it after reading.

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run a single test
cargo test test_name

# Run example agents (requires ANTHROPIC_API_KEY in .env)
cargo run --example simple_chat
cargo run --example tool_agent

# Run the D&D game (requires ANTHROPIC_API_KEY in .env)
cargo run -p dnd

# Run D&D in headless mode
cargo run -p dnd -- --headless --name "Thorin" --class fighter --race dwarf
```

## Workspace Structure

This workspace contains 6 crates:

| Crate | Path | Description |
|-------|------|-------------|
| `agentic` | `lib/` | Core framework with Agent, Tool, Memory, Safety traits |
| `claude` | `claude/` | Minimal Anthropic Claude API client |
| `dnd-macros` | `dnd-macros/` | Procedural macros for tool definitions |
| `dnd-core` | `dnd-core/` | D&D 5e game engine with AI Dungeon Master |
| `dnd` | `dnd/` | Terminal UI application for D&D |
| `agents` | `agents/` | Example agents and historical research |

## Core Library (`lib/src/`)

The `agentic` crate provides a trait-based framework for building AI agents:

| Module | Purpose |
|--------|---------|
| `agent.rs` | `Agent` trait - central abstraction for processing messages |
| `tool.rs` | `Tool` trait with `ToolRegistry` for executable functions |
| `memory.rs` | `EpisodicMemory`, `SemanticMemory`, `ProceduralMemory` traits |
| `safety.rs` | `SafetyValidator`, `Guardrail`, `ApprovalWorkflow` traits |
| `context.rs` | `ContextManager`, `Retriever`, `StatePersistence` traits |
| `llm/` | LLM providers - `AnthropicProvider` with streaming support |
| `id.rs` | Type-safe ID newtypes (AgentId, ToolId, MessageId, etc.) |
| `message.rs` | Message types with ContentBlock variants |
| `action.rs` | Action types for safety validation |
| `error.rs` | Error types using thiserror |

### Key Design Patterns

1. **Type-safe IDs**: All IDs are newtypes around UUID
2. **Async traits**: All major traits use `#[async_trait]`
3. **Builder pattern**: Configuration uses builder pattern
4. **Content blocks**: Messages use `Vec<ContentBlock>` for mixed content

## Claude API Client (`claude/src/`)

A minimal, focused Anthropic Claude API client:

```rust
use claude::{Claude, Request, Message};

let client = Claude::from_env()?;
let response = client.complete(
    Request::new(vec![Message::user("Hello")])
        .with_system("You are helpful.")
).await?;
```

Features:
- Non-streaming and streaming completions
- Tool use with automatic execution loop
- SSE parsing for streaming responses

## D&D Game Engine (`dnd-core/src/`)

The D&D 5e game engine provides:

| Module | Purpose |
|--------|---------|
| `session.rs` | `GameSession` - main public API |
| `rules.rs` | D&D 5e rules engine |
| `world.rs` | Game state, characters, locations |
| `dice.rs` | Dice notation parser (2d6+3, 4d6kh3, advantage) |
| `character_builder.rs` | Character creation |
| `persist.rs` | Save/load campaigns |
| `dm/` | AI Dungeon Master implementation |

### AI Dungeon Master (`dnd-core/src/dm/`)

```
dm/
├── agent.rs          # Main DM agent with tool execution
├── tools.rs          # D&D tools (dice, skill checks, etc.)
├── memory.rs         # Context management and summarization
├── prompts/          # System prompt templates (.txt files)
└── story_memory/     # Fact, entity, and relationship tracking
```

## D&D TUI (`dnd/src/`)

Vim-style terminal interface using ratatui:

| Module | Purpose |
|--------|---------|
| `main.rs` | Application entry point |
| `app.rs` | Application state, input modes |
| `events.rs` | Event handling |
| `character_creation.rs` | Character creation wizard |
| `ui/` | Rendering, layout, widgets |

### Input Modes

- **NORMAL**: Navigation (`j`/`k`), mode switching (`i`, `:`), hotkeys (`?` help)
- **INSERT**: Text input, `Esc` to return, `Enter` to send
- **COMMAND**: `:q` quit, `:w` save, `:e <file>` load

## Adding a New Tool

### Using the derive macro (recommended for D&D tools):

```rust
use dnd_macros::Tool;
use serde::Deserialize;

/// Roll dice using D&D notation
#[derive(Tool, Deserialize)]
#[tool(name = "roll_dice")]
struct RollDice {
    /// Dice notation like "2d6+3"
    notation: String,
    /// Optional purpose for the roll
    purpose: Option<String>,
}
```

### Using the trait directly:

```rust
#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "What it does" }
    fn input_schema(&self) -> &Value { /* JSON Schema */ }
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        // Implementation
    }
}
```

## Historical Research

The `agents/research/` directory contains design research reports used during initial framework development:
- Planning (HTN, GOAP, Tree-of-Thought)
- Memory systems (episodic, semantic, procedural)
- Safety and security patterns
- Privacy and compliance
- Multi-agent coordination
- Industry survey (Anthropic MCP, OpenAI, Google, LangChain)

See `ARCHITECTURE.md` for the current design documentation.

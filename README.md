# Agentic

A Rust framework for building AI agents, featuring a complete D&D 5e game with an AI Dungeon Master.

## Overview

This workspace contains:

| Crate | Description |
|-------|-------------|
| **`agentic`** (lib/) | Core framework with Agent, Tool, Memory, and Safety traits |
| **`claude`** | Minimal Anthropic Claude API client with streaming and tool use |
| **`dnd-core`** | D&D 5e game engine with AI Dungeon Master |
| **`dnd`** | Terminal UI for playing D&D |
| **`dnd-macros`** | Procedural macros for tool definitions |
| **`agents`** | Example agents and research materials |

## Quick Start

### Playing D&D

```bash
# Set your Anthropic API key
export ANTHROPIC_API_KEY=your_key_here
# Or create a .env file with: ANTHROPIC_API_KEY=your_key_here

# Run the TUI game
cargo run -p dnd

# Or run in headless mode
cargo run -p dnd -- --headless --name "Thorin" --class fighter --race dwarf
```

### Using the Framework

```rust
use agentic::prelude::*;

// Define a custom tool
struct MyTool;

#[async_trait::async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Does something useful" }
    fn input_schema(&self) -> &serde_json::Value {
        &serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            }
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput::text("Done!"))
    }
}
```

### Using the Claude Client

```rust
use claude::{Claude, Request, Message};

#[tokio::main]
async fn main() -> Result<(), claude::Error> {
    let client = Claude::from_env()?;

    let response = client.complete(
        Request::new(vec![Message::user("Hello!")])
            .with_system("You are a helpful assistant.")
            .with_max_tokens(1024)
    ).await?;

    println!("{}", response.text());
    Ok(())
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      dnd (TUI binary)                        │
│  Vim-style terminal interface with ratatui                   │
└─────────────────────────────┬───────────────────────────────┘
                              │ uses
┌─────────────────────────────▼───────────────────────────────┐
│                    dnd-core (library)                        │
│  GameSession, RulesEngine, AI Dungeon Master, Persistence    │
└─────────────────────────────┬───────────────────────────────┘
                              │ uses
┌─────────────────────────────▼───────────────────────────────┐
│                     claude (library)                         │
│  Anthropic API client: completions, streaming, tool use      │
└─────────────────────────────────────────────────────────────┘
```

The `agentic` core library provides generic traits that can be used independently:
- **Agent** - Central abstraction for message processing
- **Tool** - Executable functions with JSON Schema inputs
- **Memory** - Episodic, semantic, and procedural memory systems
- **Safety** - Guardrails, validators, and approval workflows

## D&D Game Features

- **Full D&D 5e mechanics**: Dice rolling, skill checks, combat, conditions
- **AI Dungeon Master**: Powered by Claude with specialized subagents
- **Vim-style TUI**: Normal, Insert, and Command modes
- **Campaign persistence**: Save and load your adventures
- **Character creation**: All PHB races, classes, and backgrounds

### TUI Controls

| Mode | Key | Action |
|------|-----|--------|
| Normal | `i` | Enter insert mode |
| Normal | `:` | Enter command mode |
| Normal | `?` | Toggle help |
| Normal | `j`/`k` | Scroll narrative |
| Insert | `Enter` | Send message |
| Insert | `Esc` | Return to normal mode |
| Command | `:q` | Quit |
| Command | `:w` | Save game |
| Command | `:e <file>` | Load game |

## Development

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example simple_chat
cargo run --example tool_agent
```

## Project Structure

```
agentic/
├── lib/                 # Core agentic framework
│   └── src/
│       ├── agent.rs     # Agent trait
│       ├── tool.rs      # Tool trait and registry
│       ├── memory.rs    # Memory systems
│       ├── safety.rs    # Safety validators
│       ├── context.rs   # Context management
│       └── llm/         # LLM providers
├── claude/              # Anthropic API client
├── dnd-macros/          # Proc macros for tools
├── dnd-core/            # D&D game engine
│   └── src/
│       ├── session.rs   # GameSession API
│       ├── rules.rs     # D&D 5e rules engine
│       ├── world.rs     # Game state
│       ├── dice.rs      # Dice notation parser
│       └── dm/          # AI Dungeon Master
├── dnd/                 # TUI application
└── agents/              # Examples and research
    ├── examples/
    └── research/        # Design research reports
```

## License

MIT

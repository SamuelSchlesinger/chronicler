# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

At the end of each session, if you learned something important about the codebase that isn't documented here, add it.

## Licensing Compliance

This project uses D&D content under the **SRD 5.2 (Creative Commons Attribution 4.0)** license. When adding or modifying D&D-related content:

**Only use content from the SRD 5.2:**
- Reference: https://dnd.wizards.com/resources/systems-reference-document
- The SRD includes: 9 races, 12 classes, basic spells, monsters, and core mechanics

**Do NOT include copyrighted content from:**
- Player's Handbook (beyond SRD content)
- Monster Manual (beyond SRD creatures)
- Other D&D sourcebooks (Xanathar's, Tasha's, etc.)
- Setting-specific content (Forgotten Realms lore, named NPCs, etc.)

**Safe to use:**
- All races: Human, Elf, Dwarf, Halfling, Half-Orc, Half-Elf, Tiefling, Gnome, Dragonborn
- All 12 base classes with one subclass each (per SRD)
- Spells, monsters, and magic items listed in the SRD
- Core game mechanics (ability scores, skills, combat rules, etc.)
- Original content you create

**When in doubt:** Check the SRD document directly or use generic/original content instead.

## Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run a single test
cargo test test_name

# Run Claude API examples (requires ANTHROPIC_API_KEY in .env)
cargo run -p claude --example simple_chat
cargo run -p claude --example tool_use

# Run the game (requires ANTHROPIC_API_KEY in .env)
cargo run -p chronicle
```

## Pre-Commit Requirements

**Before committing any changes, ALL of these must pass:**

```bash
cargo fmt --all          # Format code
cargo clippy --workspace # Lint check (must have no warnings)
cargo test --workspace   # All tests must pass
```

**Do not commit if any of these fail.** Fix issues before committing.

## Workspace Structure

This workspace contains 4 crates:

| Crate | Path | Description |
|-------|------|-------------|
| `claude` | `claude/` | Minimal Anthropic Claude API client |
| `chronicle-macros` | `chronicle-macros/` | Procedural macros for tool definitions |
| `chronicle-core` | `chronicle-core/` | Tabletop RPG game engine compatible with D&D 5e, with AI Dungeon Master |
| `chronicle` | `chronicle-bevy/` | Bevy GUI application |

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
- Tool use with automatic execution loop (`complete_with_tools`)
- SSE parsing for streaming responses

## Game Engine (`chronicle-core/src/`)

The tabletop RPG game engine (compatible with D&D 5e) provides:

| Module | Purpose |
|--------|---------|
| `session.rs` | `GameSession` - main public API |
| `rules.rs` | D&D 5e rules engine |
| `world.rs` | Game state, characters, locations |
| `dice.rs` | Dice notation parser (2d6+3, 4d6kh3, advantage) |
| `character_builder.rs` | Character creation |
| `persist.rs` | Save/load campaigns |
| `dm/` | AI Dungeon Master implementation |

### AI Dungeon Master (`chronicle-core/src/dm/`)

```
dm/
├── agent.rs          # Main DM agent with tool execution
├── tools.rs          # RPG tools (dice, skill checks, etc.)
├── memory.rs         # Context management and summarization
├── prompts/          # System prompt templates (.txt files)
└── story_memory/     # Fact, entity, and relationship tracking
```

## Adding a New Tool

### Using the derive macro (recommended):

```rust
use chronicle_macros::Tool;
use serde::Deserialize;

/// Roll dice using standard notation
#[derive(Tool, Deserialize)]
#[tool(name = "roll_dice")]
struct RollDice {
    /// Dice notation like "2d6+3"
    notation: String,
    /// Optional purpose for the roll
    purpose: Option<String>,
}
```

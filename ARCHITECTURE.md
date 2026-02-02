# Architecture

This document describes the architecture of the chronicle-ai workspace, focused on AI-driven tabletop RPG gameplay (compatible with D&D 5e) with long campaigns, multi-interface support, and programmatic testability.

> For historical context on the original framework design, see `docs/archive/FRAMEWORK_DESIGN.md`.

## Design Constraints

| Constraint | Decision |
|------------|----------|
| Primary goal | Tabletop RPG Dungeon Master game |
| State mutation | AI returns structured intents → Rules engine validates/applies |
| LLM provider | Anthropic primary, trait hooks for future providers |
| Context management | Critical—campaigns can run for hours across sessions |
| Subagents | Yes—Combat, NPC, Rules agents with focused contexts |
| Persistence | Full campaign save/load for multi-session play |
| Interfaces | Multi-interface: TUI now, web/Discord later |
| Core API | Rust library API (programmatic access for AI-assisted development) |
| Testing | Manual testing sufficient; seedable RNG not required |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      chronicle (binary crate)                    │
│  TUI (ratatui) ──┐                                               │
│  CLI (optional) ─┼─► Interface adapters                          │
│  HTTP (future) ──┘                                               │
└───────────────────────────────┬─────────────────────────────────┘
                                │ uses
┌───────────────────────────────▼─────────────────────────────────┐
│                   chronicle-core (library crate)                 │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ GameSession  │  │  DungeonMaster│  │   RulesEngine        │   │
│  │ (public API) │──│  (AI agent)   │──│   (validates/applies)│   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│         │                 │                     │                │
│         ▼                 ▼                     ▼                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │  GameWorld   │  │  Subagents   │  │   Intent/Effect      │   │
│  │  (state)     │  │  (Combat,NPC,│  │   (command pattern)  │   │
│  │              │  │   Rules)     │  │                      │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│         │                 │                                      │
│         ▼                 ▼                                      │
│  ┌──────────────┐  ┌──────────────┐                             │
│  │  Persistence │  │   Memory     │                             │
│  │  (serde/fs)  │  │  (context    │                             │
│  │              │  │   management)│                             │
│  └──────────────┘  └──────────────┘                             │
└───────────────────────────────┬─────────────────────────────────┘
                                │ uses
┌───────────────────────────────▼─────────────────────────────────┐
│                       claude (library crate)                     │
│  Anthropic API client with streaming, tools, message protocol   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Crate Structure (Simplified)

```
chronicle-ai/
├── claude/              # Anthropic API client (focused, minimal)
│   └── src/
│       ├── lib.rs       # Client, Message, Tool, Stream
│       └── ...
├── chronicle-core/      # Game logic + AI DM (the heart)
│   └── src/
│       ├── lib.rs       # GameSession (public API)
│       ├── world.rs     # GameWorld, Character, Location, etc.
│       ├── rules.rs     # RulesEngine, Intent, Effect
│       ├── dm/          # AI Dungeon Master
│       │   ├── mod.rs
│       │   ├── agent.rs # Main DM agent
│       │   ├── subagents.rs  # Combat, NPC, Rules specialists
│       │   ├── memory.rs     # Context management, summarization
│       │   └── tools.rs      # DM tools (dice, checks, etc.)
│       ├── dice.rs      # Dice notation parser
│       └── persist.rs   # Save/load campaigns
├── chronicle/           # Binary with TUI
│   └── src/
│       ├── main.rs
│       ├── tui/         # ratatui rendering
│       └── ...
└── Cargo.toml           # Workspace
```

**Key changes from current structure:**
- Generic agent framework → `claude` (focused Anthropic client, no generic Agent trait)
- `agents/src/dnd/` → `chronicle-core` (promoted to its own crate)
- Remove unused abstractions (SafetyValidator, generic Memory traits, etc.)

---

## Core Design: Intent/Effect Pattern

The AI doesn't mutate state directly. Instead:

```rust
// AI returns structured intents
enum Intent {
    Attack { attacker: CharacterId, target: CharacterId, weapon: String },
    CastSpell { caster: CharacterId, spell: String, targets: Vec<CharacterId> },
    SkillCheck { character: CharacterId, skill: Skill, dc: u8 },
    Damage { target: CharacterId, amount: u32, damage_type: DamageType },
    Heal { target: CharacterId, amount: u32 },
    Move { character: CharacterId, destination: LocationId },
    Rest { rest_type: RestType },
    // ... more intents
}

// Rules engine validates and produces effects
enum Effect {
    HpChanged { target: CharacterId, old: i32, new: i32, reason: String },
    ConditionApplied { target: CharacterId, condition: Condition },
    ConditionRemoved { target: CharacterId, condition: Condition },
    DiceRolled { notation: String, result: DiceResult },
    ItemAdded { character: CharacterId, item: Item },
    ItemRemoved { character: CharacterId, item: Item },
    LocationChanged { character: CharacterId, from: LocationId, to: LocationId },
    CombatStarted { combatants: Vec<CharacterId> },
    CombatEnded { outcome: CombatOutcome },
    // ... more effects
}

// The flow
impl RulesEngine {
    fn resolve(&self, world: &GameWorld, intent: Intent) -> Resolution {
        match intent {
            Intent::Attack { attacker, target, weapon } => {
                // 1. Validate: Does attacker have weapon? Is target valid?
                // 2. Roll attack: d20 + modifiers vs AC
                // 3. If hit, roll damage
                // 4. Return effects
                Resolution {
                    effects: vec![
                        Effect::DiceRolled { ... },
                        Effect::HpChanged { ... },
                    ],
                    narrative_context: "...", // For AI to narrate
                }
            }
            // ... other intents
        }
    }
}
```

**Benefits:**
- AI generates creative narrative; rules engine ensures consistency
- Effects can be logged, replayed, undone
- Easy to test: verify rules engine in isolation
- AI mistakes don't corrupt game state

---

## Public API: GameSession

```rust
/// The main entry point for all interfaces (TUI, web, programmatic).
pub struct GameSession {
    world: GameWorld,
    dm: DungeonMaster,
    rules: RulesEngine,
    config: SessionConfig,
}

impl GameSession {
    /// Create a new game with a fresh character.
    pub async fn new(config: SessionConfig) -> Result<Self>;

    /// Load a saved campaign.
    pub async fn load(path: impl AsRef<Path>) -> Result<Self>;

    /// Save the current campaign.
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()>;

    /// Process a player action (the main interaction point).
    /// Returns the DM's response including narrative and any state changes.
    pub async fn player_action(&mut self, input: &str) -> Result<Response>;

    /// Get current game state (for UI rendering).
    pub fn world(&self) -> &GameWorld;

    /// Get recent narrative history.
    pub fn narrative(&self) -> &[NarrativeEntry];

    /// Check if in combat.
    pub fn in_combat(&self) -> bool;

    /// Get current combat state if in combat.
    pub fn combat(&self) -> Option<&CombatState>;
}

/// Response from a player action.
pub struct Response {
    /// The narrative text from the DM.
    pub narrative: String,

    /// Effects that were applied to the game world.
    pub effects: Vec<Effect>,

    /// Dice rolls that occurred.
    pub dice_rolls: Vec<DiceResult>,

    /// Whether the game is waiting for a specific input.
    pub awaiting: Option<AwaitingInput>,
}

/// What the game is waiting for (if anything).
pub enum AwaitingInput {
    /// Normal free-form input.
    FreeForm,
    /// Choice from a list (e.g., "Do you open the door or search the room?").
    Choice(Vec<String>),
    /// Confirmation (e.g., "Are you sure you want to attack the king?").
    Confirmation(String),
}
```

**Usage from TUI:**
```rust
let mut session = GameSession::new(config).await?;
loop {
    let input = tui.get_input();
    let response = session.player_action(&input).await?;
    tui.render(&session.world(), &response);
}
```

**Usage from Claude Code (programmatic testing):**
```rust
let mut session = GameSession::new(test_config()).await?;
let response = session.player_action("I attack the goblin").await?;
assert!(response.effects.iter().any(|e| matches!(e, Effect::DiceRolled { .. })));
assert!(session.world().get_npc("goblin").unwrap().hp < 10);
```

---

## Subagent Architecture

For long campaigns, context efficiency matters. Subagents handle specialized domains:

```rust
pub struct DungeonMaster {
    client: Claude,
    memory: DmMemory,
    router: SubagentRouter,
}

pub struct SubagentRouter {
    combat: CombatAgent,
    npc: NpcAgent,
    rules: RulesAgent,
    exploration: ExplorationAgent,
}

impl SubagentRouter {
    /// Classify input and route to appropriate subagent(s).
    fn route(&self, input: &str, context: &GameWorld) -> Vec<SubagentType> {
        // Smart routing based on:
        // - Current game state (in combat? talking to NPC?)
        // - Keywords in input
        // - Recent context
    }
}
```

Each subagent has:
- **Focused system prompt** (smaller than full DM prompt)
- **Relevant context slice** (only combat state for CombatAgent, only NPC data for NpcAgent)
- **Specialized tools** (CombatAgent gets attack tools, NpcAgent gets dialogue tools)

**Context Budget Example:**
- Full DM prompt: ~4000 tokens
- CombatAgent prompt: ~1000 tokens (just combat rules)
- NpcAgent prompt: ~800 tokens (just NPC personality + dialogue)

---

## Memory & Context Management

For long campaigns, we need aggressive context management:

```rust
pub struct DmMemory {
    /// Recent conversation (sliding window).
    recent: VecDeque<Message>,

    /// Compressed summaries of older conversations.
    summaries: Vec<SessionSummary>,

    /// Key facts extracted from play (NPCs met, quests, decisions).
    facts: FactStore,

    /// Current context budget.
    budget: TokenBudget,
}

impl DmMemory {
    /// Build context for the next LLM call, staying within budget.
    fn build_context(&self, world: &GameWorld, budget: usize) -> Vec<Message> {
        let mut context = Vec::new();
        let mut tokens = 0;

        // 1. Always include critical facts
        tokens += self.add_critical_facts(&mut context);

        // 2. Add recent messages (most recent first)
        for msg in self.recent.iter().rev() {
            if tokens + msg.token_count() > budget { break; }
            context.push(msg.clone());
            tokens += msg.token_count();
        }

        // 3. If space, add relevant summaries
        if tokens < budget {
            tokens += self.add_relevant_summaries(&mut context, budget - tokens);
        }

        context.reverse(); // Chronological order
        context
    }

    /// Summarize and compress old messages when approaching limits.
    async fn compress(&mut self, client: &Claude) -> Result<()> {
        // Use LLM to summarize older messages into compact facts
    }
}
```

---

## Claude Client (Simplified)

Instead of a generic `LlmProvider` trait, a focused Anthropic client:

```rust
pub struct Claude {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl Claude {
    pub fn new(api_key: impl Into<String>) -> Self;
    pub fn from_env() -> Result<Self>;

    /// Simple completion.
    pub async fn complete(&self, request: Request) -> Result<Response>;

    /// Streaming completion.
    pub async fn stream(&self, request: Request) -> Result<impl Stream<Item = Event>>;

    /// Complete with tool use loop (handles tool calls automatically).
    pub async fn complete_with_tools<F>(
        &self,
        request: Request,
        executor: F,
    ) -> Result<Response>
    where
        F: Fn(&str, Value) -> Result<Value>;
}

pub struct Request {
    pub system: String,
    pub messages: Vec<Message>,
    pub tools: Vec<Tool>,
    pub max_tokens: usize,
}

pub struct Tool {
    pub name: String,
    pub description: String,
    pub schema: Value,  // JSON Schema
}

// Macro for easy tool definition
macro_rules! tool {
    ($name:literal, $desc:literal, { $($field:ident : $type:ty),* $(,)? }) => {
        Tool {
            name: $name.into(),
            description: $desc.into(),
            schema: json!({
                "type": "object",
                "properties": {
                    $( stringify!($field): <$type as JsonSchema>::schema() ),*
                },
                "required": [ $( stringify!($field) ),* ]
            })
        }
    };
}
```

---

## What We're Removing

| Current | Why Remove |
|---------|------------|
| `Agent` trait | Chronicle has one agent type (DM); generic trait adds boilerplate with no benefit |
| `SafetyValidator`, `Guardrail` | Chronicle doesn't need safety validation pipelines |
| Generic `Memory` traits | Specialized DmMemory for chronicle needs; generic traits unused |
| `ContextManager` trait | Direct implementation in DmMemory |
| `Context` vs `ConversationContext` duality | Single `DmMemory` type |
| `ToolRegistry` ceremony | Direct tool list; macro for definitions |
| `AgentId`, `SessionId`, etc. newtypes | Useful, but chronicle only needs `CharacterId`, `LocationId`, etc. |
| Dual `process()` / `process_action()` | Single `player_action()` API |

---

## Migration Path

### Phase 1: Core Restructure
1. Create `claude/` crate with focused Anthropic client
2. Create `chronicle-core/` crate with `GameSession` API
3. Implement Intent/Effect pattern with `RulesEngine`
4. Port existing game mechanics (dice, character, combat)

### Phase 2: AI Integration
5. Implement `DungeonMaster` agent using new `claude` crate
6. Wire up tool execution through `RulesEngine`
7. Add subagent routing
8. Implement `DmMemory` with context management

### Phase 3: Persistence & Polish
9. Add campaign save/load
10. Port TUI to use new `GameSession` API
11. Test programmatically (you and I iterate)
12. Add streaming narrative display

---

## Resolved Design Decisions

### Tool Schema Definition: Derive Macro

```rust
// Proc-macro crate: chronicle-macros
#[derive(Tool)]
#[tool(name = "roll_dice", description = "Roll dice using standard notation")]
struct RollDice {
    /// The dice notation (e.g., "2d6+3", "1d20 advantage")
    notation: String,
    /// Optional reason for the roll
    reason: Option<String>,
}

// Expands to Tool trait impl with JSON schema generated from struct fields
```

### Subagent Memory: Hybrid Approach

```rust
pub struct DmMemory {
    /// Shared across all subagents - key facts, NPC relationships, quest state
    shared_facts: FactStore,

    /// Shared summaries of past sessions
    shared_summaries: Vec<SessionSummary>,

    /// Per-subagent recent conversation (isolated)
    agent_contexts: HashMap<SubagentType, VecDeque<Message>>,
}

impl DmMemory {
    fn context_for(&self, agent: SubagentType, budget: usize) -> Vec<Message> {
        // 1. Always include relevant shared facts
        // 2. Add agent-specific recent messages
        // 3. Fill remaining budget with relevant summaries
    }
}
```

### Combat Pacing: Observable Outcomes

NPCs resolve turns automatically, but the player steps through **observable outcomes**:

```rust
pub struct CombatRound {
    /// What the player did
    player_action: Intent,
    player_effects: Vec<Effect>,

    /// What the player observed (NPC actions they could see/hear)
    observable_outcomes: Vec<ObservableOutcome>,
}

pub struct ObservableOutcome {
    /// Who acted
    actor: CharacterId,
    /// What the player perceived
    narrative: String,
    /// Mechanical effects (for UI display)
    effects: Vec<Effect>,
}

// Example flow:
// 1. Player: "I attack the goblin"
// 2. Rules engine resolves player attack
// 3. Rules engine resolves all NPC turns
// 4. DM narrates: "You slash the goblin for 8 damage! The goblin
//    retaliates with a rusty dagger but you parry. Behind you,
//    the hobgoblin charges and strikes your ally for 12 damage."
// 5. Player reads at their own pace, then takes next action
```

This gives cinematic pacing—player acts, sees consequences unfold, acts again.

---

## World Building System

The world building system enables the AI to create and manage a persistent, evolving game world through specialized tools and memory systems.

### World Building Flow

```
┌──────────────┐
│ Player Input │
└──────┬───────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                    RELEVANCE CHECKER (Fast Model)                        │
│  • Checks registered consequences against player action                  │
│  • Surfaces relevant NPCs/locations not explicitly mentioned             │
│  • Retrieves key facts from story memory                                 │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                      SYSTEM PROMPT COMPOSITION                           │
│  dm_base.txt + story_memory.txt + world_building.txt + ...               │
│  + Campaign context + Character info + Triggered consequences            │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                      CLAUDE API (Tool Loop)                              │
│           Generates narrative + tool calls (streaming)                   │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        TOOL PARSING LAYER                                │
│              Tool Call JSON  →  Intent (typed struct)                    │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          RULES ENGINE                                    │
│             Intent  →  Resolution (Narrative + Effects)                  │
└──────────────────────────────────────┬───────────────────────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              ▼                        ▼                        ▼
┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│     GAME WORLD      │   │    STORY MEMORY     │   │   TOOL RESULT       │
│  npcs, locations,   │   │  facts, relations,  │   │  (back to Claude    │
│  quests, log        │   │  consequences       │   │   for next turn)    │
└─────────────────────┘   └─────────────────────┘   └─────────────────────┘
```

### World Building Tools

The DM has access to 17 specialized world building tools:

| Domain | Tools | Purpose |
|--------|-------|---------|
| **NPC Management** | `create_npc`, `update_npc`, `move_npc`, `remove_npc` | Introduce and manage non-player characters with personality, disposition, and knowledge |
| **Location Management** | `create_location`, `connect_locations`, `update_location` | Build navigable world with nested locations and travel connections |
| **Story Memory** | `remember_fact`, `register_consequence` | Record important facts and set up triggered future events |
| **Quest Management** | `create_quest`, `add_quest_objective`, `complete_objective`, `complete_quest`, `fail_quest` | Track missions with objectives and rewards |
| **Time & Events** | `schedule_event`, `check_schedule` | Time-based story triggers |
| **State** | `assert_state` | Declarative state changes |

### Story Memory Architecture

```rust
pub struct StoryMemory {
    entities: HashMap<EntityId, Entity>,     // NPCs, locations, items, etc.
    name_index: HashMap<String, EntityId>,   // Case-insensitive lookup
    facts: Vec<StoryFact>,                   // Important narrative facts
    relationships: Vec<Relationship>,         // Entity connections
    consequences: Vec<Consequence>,           // Triggered events
    knowledge: Vec<KnowledgeEntry>,          // Asymmetric info (who knows what)
    scheduled_events: Vec<ScheduledEvent>,   // Time-based triggers
}
```

**Entity Types:** `Npc`, `Location`, `Item`, `Quest`, `Organization`, `Event`, `Creature`

**Fact Categories:** `appearance`, `personality`, `event`, `relationship`, `backstory`, `motivation`, `capability`, `location`, `possession`, `status`, `secret`

### Importance Decay

Facts and consequences decay over time to manage context window:

- **Facts:** 2% decay per turn (1% for stable categories like `location`)
- **Consequences:** 1% decay per turn
- **Entities:** Reset when mentioned, decay when idle

Only the top 30 most important facts are included in context. This naturally surfaces recent and high-importance information while older details fade.

### Consequence System

Consequences enable reactive storytelling:

```rust
// DM registers a consequence
register_consequence(
    trigger: "player returns to the village",
    description: "The villagers have organized a celebration in the hero's honor",
    severity: "moderate",
    related_entities: ["Millbrook Village", "Elder Thom"],
    importance: 0.8
)

// Later, when player says "I head back to the village"
// Relevance checker detects match → consequence triggers → added to context
```

This lets the DM plant story seeds that bloom naturally based on player actions.

### NPC Creation Example

Complete data flow for `create_npc`:

1. **Claude generates tool call:**
   ```json
   { "name": "create_npc", "input": {
       "name": "Mira the Innkeeper",
       "description": "A stout dwarf with a braided red beard",
       "personality": "Gruff but secretly kind-hearted",
       "disposition": "neutral",
       "location": "The Rusty Tankard",
       "known_information": ["knows about bandits on the north road"]
   }}
   ```

2. **Tool parsing** creates `Intent::CreateNpc { ... }`

3. **Rules engine** resolves to `Effect::NpcCreated { name, location }`

4. **Effect applied** to `GameWorld.npcs` HashMap

5. **Tool result** returned to Claude: "NPC Mira the Innkeeper enters the world"

6. **Claude continues** narrative or calls more tools

---

## Token Efficiency Notes

The redesign prioritizes smaller code:
- **Fewer abstractions** = less code to understand/generate
- **Specialized types** (DmMemory vs generic Memory) = no unused fields
- **Macros for boilerplate** (tool definitions) = less repetitive code
- **Intent/Effect enums** = exhaustive, compiler-checked, no stringly-typed mistakes
- **Single API surface** (GameSession) = one thing to learn

Estimated LOC comparison:
| Component | Current | Redesign |
|-----------|---------|----------|
| LLM client | ~600 | ~300 |
| Agent/DM | ~400 | ~250 |
| Tools | ~470 | ~200 (with macros) |
| Game state | ~800 | ~600 |
| TUI | ~1000 | ~800 |
| **Total** | **~3300** | **~2150** |

~35% reduction while adding features (persistence, subagents, proper tool execution).

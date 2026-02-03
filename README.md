# chronicler

*An AI Dungeon Master that remembers.*

---

> *"I see you've found the Shadowfell Codex. Most visitors are drawn to the prettier volumes."*
>
> *A knowing smile crosses her weathered features. "That particular tome has been the subject of much... debate among our scholars."*
>
> — [The Wizard's Bargain](docs/transcripts/wizards_bargain.md)

---

A solo tabletop RPG powered by Claude. You play; the AI runs the world — narrating scenes, voicing NPCs, rolling dice, and tracking the consequences of your choices across sessions.

Compatible with D&D 5e. Runs locally. [Bring your own API key.](https://console.anthropic.com/)

![Chronicle AI Screenshot](screenshot.png)

## Quick Start

```bash
git clone https://github.com/SamuelSchlesinger/chronicler.git
cd chronicler
export ANTHROPIC_API_KEY=your_key_here
cargo run -p chronicler
```

## What Makes It Different

**The AI plays by the rules.** Skill checks, saving throws, combat, conditions, death saves — mechanically resolved, not handwaved. The AI expresses intent; a rules engine validates the effects.

**Your choices persist.** Spare the bandit? He might return. Offend the merchant? She remembers. A persistent story memory with importance decay ensures consequences resurface naturally — even across sessions.

**It acts, not asks.** No "which enemy do you attack?" The DM reads the situation and moves the story forward. Consequence triggers and state inference run in the background, keeping the world consistent without breaking flow.

## See It In Action

| | |
|---|---|
| [The Goblin Ambush](docs/transcripts/goblin_ambush.md) | A dwarf fighter springs a trap. Death saves ensue. |
| [The Wizard's Bargain](docs/transcripts/wizards_bargain.md) | An elf wizard seeks forbidden knowledge in ancient archives. |

## Agentic Innovations

This isn't just a game — it's an experiment in **storytelling agent design**. The AI DM demonstrates several techniques for building agents that maintain long-term coherence while staying creative.

### Multi-Model Architecture

Different models for different jobs:

| Task | Model | Why |
|------|-------|-----|
| Narrative generation | Sonnet | Creative, expressive, handles complex roleplay |
| Relevance checking | Haiku | Fast, cheap — runs every turn to check story triggers |
| State inference | Haiku | Detects implied changes the main model didn't record |

This keeps costs low while maintaining quality where it matters.

### Intent/Effect Separation

The AI doesn't mutate game state directly. It expresses **intents** ("attack the goblin"), and a rules engine produces validated **effects** ("8 damage, goblin HP reduced to 3").

```
AI: "I swing my axe at the goblin!"
    ↓
Intent: Attack { target: "goblin", weapon: "greataxe" }
    ↓
Rules Engine validates, rolls dice, applies modifiers
    ↓
Effect: Damage { target: "goblin", amount: 8, type: "slashing" }
    ↓
World state updates atomically
```

This prevents the AI from "hallucinating" impossible game states while preserving narrative freedom.

### Persistent Story Memory

LLMs forget. After ~20 exchanges, early details fall out of context. The DM maintains a **structured story memory** that persists across sessions:

```rust
remember_fact(
  subject: "Mira",
  category: "secret",
  fact: "She's the one who poisoned the well, but blames the herbalist",
  importance: 0.9,
  related_entities: ["Riverside", "Old Thomas"]
)
```

Facts decay 2% per turn — recent events and dramatic moments stay relevant; minor details fade. When you mention "Mira" 50 turns later, her secrets resurface automatically.

### Consequence Seeds

The DM can plant **deferred narrative triggers** that bloom when conditions are met:

```rust
register_consequence(
  trigger: "player mentions the missing children to a local",
  consequence: "The blacksmith overhears and becomes hostile — his son is among the missing",
  severity: "major"
)
```

A fast model (Haiku) checks every player action against pending consequences using semantic matching — not keywords. This enables:

- **Revenge plots** — spare the bandit, he returns
- **Reputation cascades** — help the village, merchants offer discounts
- **Ticking clocks** — ignore the cultists too long, the ritual completes

### Post-Narrative State Inference

The DM writes "She smiles warmly and thanks you for saving her shop." But did it call `update_npc(disposition="friendly")`? Often not.

After each response, a secondary model analyzes the narrative and infers state changes with confidence scores. High-confidence changes (>0.8) are applied automatically:

```
Narrative: "Captain Voss storms out, muttering about incompetent adventurers"
Inferred: location="outside", disposition="hostile" (confidence: 0.92)
Applied: ✓
```

This closes the "narrative-state gap" that plagues most AI game masters.

### Proactive World Design

When entering a new location, the DM doesn't improvise from nothing. It designs the environment *before* describing it:

- **NPCs** with goals, secrets, and routines
- **Relationships** between characters (rivals, lovers, debts owed)
- **Scheduled events** that happen with or without the player
- **Consequence seeds** for player interactions

The tavern feels alive because it *is* modeled — the nervous woman by the door has a reason to watch it.

## Architecture

```
chronicler         Bevy + egui desktop app
    |
chronicler-core    Game engine, rules, AI DM, persistence
    |
claude             Minimal Anthropic API client
```

[How the AI Dungeon Master Works](docs/HOW_IT_WORKS.md) — detailed technical deep-dive.

## Development

```bash
cargo build --workspace
cargo test --workspace
```

Requires Rust and an [Anthropic API key](https://console.anthropic.com/). Runs on macOS, Linux, and Windows.

## License

[CC BY-NC 4.0](LICENSE) — free for non-commercial use.

D&D mechanics from [SRD 5.2](https://dnd.wizards.com/resources/systems-reference-document) (CC BY 4.0, Wizards of the Coast).

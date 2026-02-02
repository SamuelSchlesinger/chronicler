# chronicle-ai

*An AI Dungeon Master that remembers.*

---

> *The woman's smile widens, revealing teeth that seem just a bit too sharp.*
>
> *"Another seeker of forbidden knowledge, I presume? How... interesting."*
>
> — [The Wizard's Bargain](docs/transcripts/wizards_bargain.md)

---

A solo tabletop RPG powered by Claude. You play; the AI runs the world — narrating scenes, voicing NPCs, rolling dice, and tracking the consequences of your choices across sessions.

Compatible with D&D 5e. Runs locally. [Bring your own API key.](https://console.anthropic.com/)

## Quick Start

```bash
git clone https://github.com/SamuelSchlesinger/chronicle-ai.git
cd chronicle-ai
export ANTHROPIC_API_KEY=your_key_here
cargo run -p chronicle
```

## What Makes It Different

**The AI plays by the rules.** Skill checks, saving throws, combat, conditions, death saves — mechanically resolved, not handwaved.

**Your choices persist.** Spare the bandit? He might return. Offend the merchant? She remembers. The AI maintains a story memory that resurfaces naturally.

**It acts, not asks.** No "which enemy do you attack?" The DM reads the situation and moves the story forward.

## See It In Action

| | |
|---|---|
| [The Goblin Ambush](docs/transcripts/goblin_ambush.md) | A fighter springs a trap. Death saves ensue. |
| [Tavern Trouble](docs/transcripts/tavern_trouble.md) | A bard's song uncovers a deadly mystery. |
| [The Wizard's Bargain](docs/transcripts/wizards_bargain.md) | An elf seeks forbidden knowledge. |
| [Into the Crypt](docs/transcripts/into_the_crypt.md) | A cleric faces the restless dead. |
| [The Heist](docs/transcripts/the_heist.md) | A rogue infiltrates a merchant lord's manor. |
| [Blood and Thunder](docs/transcripts/blood_and_thunder.md) | A barbarian faces death in the arena. |

## Under the Hood

```
chronicle          Bevy + egui desktop app
    |
chronicle-core     Game engine, rules, AI DM, persistence
    |
claude             Minimal Anthropic API client
```

The AI doesn't mutate game state directly. It calls tools — `skill_check`, `apply_damage`, `start_combat`, `remember_fact` — and a rules engine validates and applies the effects. The AI handles narrative; the engine ensures mechanical consistency.

[How the AI Dungeon Master Works](docs/HOW_IT_WORKS.md) — prompt design, tool system, story memory.

## Development

```bash
cargo build --workspace
cargo test --workspace
```

Requires Rust and an [Anthropic API key](https://console.anthropic.com/). Runs on macOS, Linux, and Windows.

## License

[CC BY-NC 4.0](LICENSE) — free for non-commercial use.

D&D mechanics from [SRD 5.2](https://dnd.wizards.com/resources/systems-reference-document) (CC BY 4.0, Wizards of the Coast).

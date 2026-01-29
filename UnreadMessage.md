# Developer Handoff

## What was done

Continued implementing D&D 5e mechanics and UI overlays from the previous session.

### New Features Added This Session

#### 1. Death Save Rolls (D&D 5e Rules)
- Added `Intent::DeathSave { character_id }` for making death saving throws
- Added `Effect::DeathSaveSuccess { target_id, roll, total_successes }`
- Added `Effect::Stabilized { target_id }` (3 successes = stabilized)
- Implemented `resolve_death_save()` in `dnd-core/src/rules.rs` with full D&D 5e rules:
  - d20 roll, 10+ is success
  - Natural 20: regain 1 HP and consciousness, reset death saves
  - Natural 1: counts as 2 failures
  - 3 successes: stabilized (unconscious but stable)
  - 3 failures: death
- Added `death_save` DM tool in `dnd-core/src/dm/tools.rs`
- Added UI effect handlers in `dnd/src/effects.rs`

#### 2. Concentration Checks (D&D 5e Rules)
- Added `Intent::ConcentrationCheck { character_id, damage_taken, spell_name }`
- Added `Effect::ConcentrationBroken { character_id, spell_name, damage_taken, roll, dc }`
- Added `Effect::ConcentrationMaintained { character_id, spell_name, roll, dc }`
- Implemented `resolve_concentration_check()` in `dnd-core/src/rules.rs`:
  - Constitution saving throw
  - DC = max(10, damage_taken / 2) per D&D 5e rules
  - Success maintains concentration, failure breaks it
- Added `concentration_check` DM tool
- Added UI effect handlers

#### 3. Quest Log Overlay (Shift+Q)
- Added `Overlay::QuestLog` variant
- Added `quests: Vec<Quest>` to `WorldUpdate` struct in `dnd/src/ai_worker.rs`
- Implemented `render_quest_log_overlay()` in `dnd/src/ui/render.rs`:
  - Shows Active quests with objectives (completed/incomplete markers)
  - Shows Completed quests (green)
  - Shows Failed/Abandoned quests (red)
  - Shows "No quests yet" message if empty
- Added `toggle_quest_log()` method to `App`
- Added Shift+Q keybinding in `dnd/src/events.rs`
- Updated hotkey bar to show "Q:quest" shortcut
- Updated help overlay to document Shift+Q

## Current State

- All 90 tests pass in dnd-core
- Build compiles cleanly (only pre-existing dead code warnings in dnd/)
- Death saves, concentration checks, and quest log overlay are fully implemented

## Files Modified This Session

| File | Changes |
|------|---------|
| `dnd-core/src/rules.rs` | Added DeathSave/ConcentrationCheck intents, new effects, resolution methods |
| `dnd-core/src/dm/tools.rs` | Added death_save and concentration_check tools |
| `dnd/src/effects.rs` | Added UI handlers for new effects (DeathSaveSuccess, Stabilized, ConcentrationBroken, ConcentrationMaintained) |
| `dnd/src/ai_worker.rs` | Added quests field to WorldUpdate |
| `dnd/src/ui/render.rs` | Added QuestLog overlay, render_quest_log_overlay(), updated help text |
| `dnd/src/app.rs` | Added toggle_quest_log() method |
| `dnd/src/events.rs` | Added Shift+Q keybinding, Q to close quest log |
| `dnd/src/ui/widgets/status_bar.rs` | Added Q:quest to hotkey bar |

## Known Limitations

- No weight/encumbrance enforcement yet
- Attunement for magical items is deferred
- No dual wielding support
- Critical hits on unconscious targets don't auto-crit
- Journal overlay (Shift+J) is still a placeholder

## Next Steps

### Core Mechanics
1. Trigger concentration checks automatically when a concentrating character takes damage
2. Trigger death save rolls automatically on player's turn when at 0 HP
3. Add weight/encumbrance enforcement

### UI Enhancements
1. Implement Journal overlay (Shift+J) - notes and game events log
2. Narrative widget scroll estimation uses byte length - low priority fix

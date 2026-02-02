//! Gameplay tool parsing - converts gameplay polish tool calls into game Intents.

use crate::rules::Intent;
use crate::world::{Ability, GameWorld};
use serde_json::Value;

/// Parse gameplay-related tool calls into Intents.
pub fn parse_gameplay_tool(name: &str, input: &Value, _world: &GameWorld) -> Option<Intent> {
    match name {
        "modify_ability_score" => {
            let ability_str = input.get("ability")?.as_str()?;
            let ability = parse_ability(ability_str)?;
            let modifier = input.get("modifier")?.as_i64()? as i8;
            let source = input.get("source")?.as_str()?.to_string();
            let duration = input
                .get("duration")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(Intent::ModifyAbilityScore {
                ability,
                modifier,
                source,
                duration,
            })
        }

        "advance_time" => {
            let minutes = input.get("minutes")?.as_u64()? as u32;
            Some(Intent::AdvanceTime { minutes })
        }

        "restore_spell_slot" => {
            let slot_level = input.get("slot_level")?.as_u64()? as u8;
            let source = input.get("source")?.as_str()?.to_string();

            // Validate slot level
            if !(1..=9).contains(&slot_level) {
                return None;
            }

            Some(Intent::RestoreSpellSlot { slot_level, source })
        }

        _ => None,
    }
}

/// Parse ability name string into Ability enum.
fn parse_ability(name: &str) -> Option<Ability> {
    match name.to_lowercase().as_str() {
        "strength" | "str" => Some(Ability::Strength),
        "dexterity" | "dex" => Some(Ability::Dexterity),
        "constitution" | "con" => Some(Ability::Constitution),
        "intelligence" | "int" => Some(Ability::Intelligence),
        "wisdom" | "wis" => Some(Ability::Wisdom),
        "charisma" | "cha" => Some(Ability::Charisma),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Character, CharacterClass, ClassLevel, GameWorld};
    use serde_json::json;

    fn create_test_world() -> GameWorld {
        let mut character = Character::new("Test Hero");
        character.classes.push(ClassLevel {
            class: CharacterClass::Wizard,
            level: 5,
            subclass: None,
        });
        GameWorld::new("Test Campaign", character)
    }

    #[test]
    fn test_modify_ability_score() {
        let world = create_test_world();
        let input = json!({
            "ability": "strength",
            "modifier": 2,
            "source": "Belt of Giant Strength",
            "duration": "while worn"
        });

        let intent = parse_gameplay_tool("modify_ability_score", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::ModifyAbilityScore {
            ability,
            modifier,
            source,
            duration,
        }) = intent
        {
            assert_eq!(ability, Ability::Strength);
            assert_eq!(modifier, 2);
            assert_eq!(source, "Belt of Giant Strength");
            assert_eq!(duration, Some("while worn".to_string()));
        } else {
            panic!("Expected ModifyAbilityScore intent");
        }
    }

    #[test]
    fn test_advance_time() {
        let world = create_test_world();
        let input = json!({
            "minutes": 60
        });

        let intent = parse_gameplay_tool("advance_time", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::AdvanceTime { minutes }) = intent {
            assert_eq!(minutes, 60);
        } else {
            panic!("Expected AdvanceTime intent");
        }
    }

    #[test]
    fn test_restore_spell_slot() {
        let world = create_test_world();
        let input = json!({
            "slot_level": 3,
            "source": "Pearl of Power"
        });

        let intent = parse_gameplay_tool("restore_spell_slot", &input, &world);
        assert!(intent.is_some());

        if let Some(Intent::RestoreSpellSlot { slot_level, source }) = intent {
            assert_eq!(slot_level, 3);
            assert_eq!(source, "Pearl of Power");
        } else {
            panic!("Expected RestoreSpellSlot intent");
        }
    }

    #[test]
    fn test_restore_spell_slot_invalid_level() {
        let world = create_test_world();

        // Level 0 is invalid
        let input = json!({
            "slot_level": 0,
            "source": "Invalid"
        });
        assert!(parse_gameplay_tool("restore_spell_slot", &input, &world).is_none());

        // Level 10 is invalid
        let input = json!({
            "slot_level": 10,
            "source": "Invalid"
        });
        assert!(parse_gameplay_tool("restore_spell_slot", &input, &world).is_none());
    }

    #[test]
    fn test_parse_ability() {
        assert_eq!(parse_ability("strength"), Some(Ability::Strength));
        assert_eq!(parse_ability("STR"), Some(Ability::Strength));
        assert_eq!(parse_ability("dexterity"), Some(Ability::Dexterity));
        assert_eq!(parse_ability("DEX"), Some(Ability::Dexterity));
        assert_eq!(parse_ability("constitution"), Some(Ability::Constitution));
        assert_eq!(parse_ability("CON"), Some(Ability::Constitution));
        assert_eq!(parse_ability("intelligence"), Some(Ability::Intelligence));
        assert_eq!(parse_ability("INT"), Some(Ability::Intelligence));
        assert_eq!(parse_ability("wisdom"), Some(Ability::Wisdom));
        assert_eq!(parse_ability("WIS"), Some(Ability::Wisdom));
        assert_eq!(parse_ability("charisma"), Some(Ability::Charisma));
        assert_eq!(parse_ability("CHA"), Some(Ability::Charisma));
        assert_eq!(parse_ability("invalid"), None);
    }
}

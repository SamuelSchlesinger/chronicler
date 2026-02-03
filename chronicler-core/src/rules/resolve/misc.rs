//! Miscellaneous resolution methods (experience, features, facts, consequences, ability scores).

use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{Ability, CharacterId, GameWorld};

impl RulesEngine {
    pub(crate) fn resolve_gain_experience(&self, world: &GameWorld, amount: u32) -> Resolution {
        let new_total = world.player_character.experience + amount;
        let current_level = world.player_character.level;

        // XP thresholds for levels 1-20
        let xp_thresholds = [
            0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000,
            140000, 165000, 195000, 225000, 265000, 305000, 355000,
        ];

        let new_level = xp_thresholds
            .iter()
            .rposition(|&threshold| new_total >= threshold)
            .map(|idx| (idx + 1) as u8)
            .unwrap_or(1);

        let mut resolution = Resolution::new(format!(
            "Gained {amount} experience points (Total: {new_total})"
        ));

        resolution = resolution.with_effect(Effect::ExperienceGained { amount, new_total });

        if new_level > current_level {
            resolution = resolution.with_effect(Effect::LevelUp { new_level });
        }

        resolution
    }

    pub(crate) fn resolve_use_feature(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        feature_name: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if let Some(feature) = character.features.iter().find(|f| f.name == feature_name) {
            if let Some(ref uses) = feature.uses {
                if uses.current > 0 {
                    Resolution::new(format!(
                        "{} uses {} ({} uses remaining)",
                        character.name,
                        feature_name,
                        uses.current - 1
                    ))
                    .with_effect(Effect::FeatureUsed {
                        feature_name: feature_name.to_string(),
                        uses_remaining: uses.current - 1,
                    })
                } else {
                    Resolution::new(format!(
                        "{} has no uses of {} remaining",
                        character.name, feature_name
                    ))
                }
            } else {
                Resolution::new(format!("{} uses {}", character.name, feature_name))
            }
        } else {
            Resolution::new(format!(
                "{} does not have the feature {}",
                character.name, feature_name
            ))
        }
    }

    pub(crate) fn resolve_remember_fact(
        &self,
        subject_name: &str,
        subject_type: &str,
        fact: &str,
        category: &str,
        related_entities: &[String],
        importance: f32,
    ) -> Resolution {
        // The actual storage is handled by the DM agent, not the rules engine.
        // We return a confirmation message and an effect that signals what to store.
        let related_str = if related_entities.is_empty() {
            String::new()
        } else {
            format!(" (related: {})", related_entities.join(", "))
        };

        Resolution::new(format!(
            "Noted: {subject_name} ({subject_type}) - {fact}{related_str}"
        ))
        .with_effect(Effect::FactRemembered {
            subject_name: subject_name.to_string(),
            subject_type: subject_type.to_string(),
            fact: fact.to_string(),
            category: category.to_string(),
            related_entities: related_entities.to_vec(),
            importance,
        })
    }

    pub(crate) fn resolve_change_location(
        &self,
        world: &GameWorld,
        new_location: &str,
        _location_type: Option<String>,
        _description: Option<String>,
    ) -> Resolution {
        let previous_location = world.current_location.name.clone();

        Resolution::new(format!(
            "You travel from {previous_location} to {new_location}."
        ))
        .with_effect(Effect::LocationChanged {
            previous_location,
            new_location: new_location.to_string(),
        })
    }

    pub(crate) fn resolve_register_consequence(
        &self,
        trigger_description: &str,
        consequence_description: &str,
        severity: &str,
        _related_entities: &[String],
        importance: f32,
        expires_in_turns: Option<u32>,
    ) -> Resolution {
        // Generate a unique ID for this consequence
        let consequence_id = uuid::Uuid::new_v4().to_string();

        let severity_display = match severity.to_lowercase().as_str() {
            "minor" => "minor",
            "moderate" => "moderate",
            "major" => "major",
            "critical" => "critical",
            _ => "moderate",
        };

        let expiry_note = match expires_in_turns {
            Some(turns) => format!(" (expires in {turns} turns)"),
            None => String::new(),
        };

        Resolution::new(format!(
            "Consequence registered: If {trigger_description}, then {consequence_description} ({severity_display} severity, importance {importance:.1}){expiry_note}"
        ))
        .with_effect(Effect::ConsequenceRegistered {
            consequence_id,
            trigger_description: trigger_description.to_string(),
            consequence_description: consequence_description.to_string(),
            severity: severity_display.to_string(),
        })
    }

    pub(crate) fn resolve_modify_ability_score(
        &self,
        ability: Ability,
        modifier: i8,
        source: &str,
        duration: Option<&str>,
    ) -> Resolution {
        let modifier_text = if modifier >= 0 {
            format!("+{}", modifier)
        } else {
            format!("{}", modifier)
        };
        let duration_text = duration
            .map(|d| format!(" for {}", d))
            .unwrap_or_else(|| " permanently".to_string());

        Resolution::new(format!(
            "{} modified by {}{} from {}",
            ability.name(),
            modifier_text,
            duration_text,
            source
        ))
        .with_effect(Effect::AbilityScoreModified {
            ability,
            modifier,
            source: source.to_string(),
        })
    }
}

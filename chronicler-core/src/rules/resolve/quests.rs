//! Quest management resolution methods.

use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;

impl RulesEngine {
    pub(crate) fn resolve_create_quest(
        &self,
        name: &str,
        description: &str,
        giver: Option<&str>,
        objectives: &[(String, bool)],
        rewards: &[String],
    ) -> Resolution {
        Resolution::new(format!(
            "Quest Started: \"{}\"{}",
            name,
            giver.map(|g| format!(" (from {})", g)).unwrap_or_default()
        ))
        .with_effect(Effect::QuestCreated {
            name: name.to_string(),
            description: description.to_string(),
            giver: giver.map(|s| s.to_string()),
            objectives: objectives.to_vec(),
            rewards: rewards.to_vec(),
        })
    }

    pub(crate) fn resolve_add_quest_objective(
        &self,
        quest_name: &str,
        objective: &str,
        optional: bool,
    ) -> Resolution {
        Resolution::new(format!(
            "New objective for \"{}\": {}{}",
            quest_name,
            objective,
            if optional { " (optional)" } else { "" }
        ))
        .with_effect(Effect::QuestObjectiveAdded {
            quest_name: quest_name.to_string(),
            objective: objective.to_string(),
            optional,
        })
    }

    pub(crate) fn resolve_complete_objective(
        &self,
        quest_name: &str,
        objective_description: &str,
    ) -> Resolution {
        Resolution::new(format!(
            "Objective completed for \"{}\": {}",
            quest_name, objective_description
        ))
        .with_effect(Effect::QuestObjectiveCompleted {
            quest_name: quest_name.to_string(),
            objective_description: objective_description.to_string(),
        })
    }

    pub(crate) fn resolve_complete_quest(
        &self,
        quest_name: &str,
        completion_note: Option<&str>,
    ) -> Resolution {
        Resolution::new(format!(
            "Quest Completed: \"{}\"{}",
            quest_name,
            completion_note
                .map(|n| format!(" - {}", n))
                .unwrap_or_default()
        ))
        .with_effect(Effect::QuestCompleted {
            quest_name: quest_name.to_string(),
            completion_note: completion_note.map(|s| s.to_string()),
        })
    }

    pub(crate) fn resolve_fail_quest(&self, quest_name: &str, failure_reason: &str) -> Resolution {
        Resolution::new(format!(
            "Quest Failed: \"{}\" - {}",
            quest_name, failure_reason
        ))
        .with_effect(Effect::QuestFailed {
            quest_name: quest_name.to_string(),
            failure_reason: failure_reason.to_string(),
        })
    }

    pub(crate) fn resolve_update_quest(
        &self,
        quest_name: &str,
        new_description: Option<&str>,
        add_rewards: &[String],
    ) -> Resolution {
        let mut parts = vec![format!("Quest \"{}\" updated", quest_name)];
        if new_description.is_some() {
            parts.push("description changed".to_string());
        }
        if !add_rewards.is_empty() {
            parts.push(format!("rewards added: {}", add_rewards.join(", ")));
        }
        Resolution::new(parts.join("; ")).with_effect(Effect::QuestUpdated {
            quest_name: quest_name.to_string(),
            new_description: new_description.map(|s| s.to_string()),
            add_rewards: add_rewards.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::types::Effect;

    // ========== Quest Creation Tests ==========

    #[test]
    fn test_create_quest() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_create_quest(
            "The Lost Artifact",
            "Find the ancient relic hidden in the dungeon",
            Some("Elder Mage"),
            &[
                ("Enter the dungeon".to_string(), false),
                ("Defeat the guardian".to_string(), false),
            ],
            &["500 gold".to_string(), "Magical weapon".to_string()],
        );

        assert!(resolution.narrative.contains("Quest Started"));
        assert!(resolution.narrative.contains("The Lost Artifact"));
        assert!(resolution.narrative.contains("Elder Mage"));
        assert!(resolution.effects.iter().any(|e| {
            matches!(e, Effect::QuestCreated {
                name,
                giver: Some(g),
                objectives,
                rewards,
                ..
            } if name == "The Lost Artifact"
                && g == "Elder Mage"
                && objectives.len() == 2
                && rewards.len() == 2)
        }));
    }

    #[test]
    fn test_create_quest_no_giver() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_create_quest(
            "Personal Quest",
            "Pursue your own goals",
            None,
            &[("Complete the objective".to_string(), false)],
            &[],
        );

        assert!(resolution.narrative.contains("Quest Started"));
        assert!(resolution.narrative.contains("Personal Quest"));
        // Should not mention a giver
        assert!(!resolution.narrative.contains("from"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::QuestCreated { giver: None, .. })));
    }

    #[test]
    fn test_create_quest_with_optional_objectives() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_create_quest(
            "Side Quest",
            "Help the villagers",
            Some("Village Elder"),
            &[
                ("Main objective".to_string(), false),
                ("Optional: Extra reward task".to_string(), true),
            ],
            &["100 gold".to_string()],
        );

        assert!(resolution.effects.iter().any(|e| {
            matches!(e, Effect::QuestCreated { objectives, .. }
                if objectives.len() == 2
                && !objectives[0].1  // First is not optional
                && objectives[1].1) // Second is optional
        }));
    }

    // ========== Quest Objective Tests ==========

    #[test]
    fn test_add_quest_objective() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_add_quest_objective(
            "The Lost Artifact",
            "Find the secret entrance",
            false,
        );

        assert!(resolution.narrative.contains("New objective"));
        assert!(resolution.narrative.contains("The Lost Artifact"));
        assert!(resolution.narrative.contains("Find the secret entrance"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::QuestObjectiveAdded { quest_name, optional: false, .. } if quest_name == "The Lost Artifact")
        ));
    }

    #[test]
    fn test_add_quest_objective_optional() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_add_quest_objective("Side Quest", "Bonus task", true);

        assert!(resolution.narrative.contains("(optional)"));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::QuestObjectiveAdded { optional: true, .. })));
    }

    #[test]
    fn test_complete_objective() {
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_complete_objective("The Lost Artifact", "Enter the dungeon");

        assert!(resolution.narrative.contains("Objective completed"));
        assert!(resolution.narrative.contains("The Lost Artifact"));
        assert!(resolution.narrative.contains("Enter the dungeon"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::QuestObjectiveCompleted { quest_name, objective_description }
                if quest_name == "The Lost Artifact" && objective_description == "Enter the dungeon")
        ));
    }

    // ========== Quest Completion Tests ==========

    #[test]
    fn test_complete_quest() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_complete_quest(
            "The Lost Artifact",
            Some("Returned the artifact to the Elder Mage"),
        );

        assert!(resolution.narrative.contains("Quest Completed"));
        assert!(resolution.narrative.contains("The Lost Artifact"));
        assert!(resolution.narrative.contains("Returned the artifact"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::QuestCompleted { quest_name, completion_note: Some(_) }
                if quest_name == "The Lost Artifact")
        ));
    }

    #[test]
    fn test_complete_quest_no_note() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_complete_quest("Simple Quest", None);

        assert!(resolution.narrative.contains("Quest Completed"));
        assert!(resolution.narrative.contains("Simple Quest"));
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::QuestCompleted {
                completion_note: None,
                ..
            }
        )));
    }

    #[test]
    fn test_fail_quest() {
        let engine = RulesEngine::new();

        let resolution =
            engine.resolve_fail_quest("The Lost Artifact", "The villain destroyed the artifact");

        assert!(resolution.narrative.contains("Quest Failed"));
        assert!(resolution.narrative.contains("The Lost Artifact"));
        assert!(resolution
            .narrative
            .contains("villain destroyed the artifact"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::QuestFailed { quest_name, failure_reason }
                if quest_name == "The Lost Artifact" && failure_reason.contains("villain"))
        ));
    }

    // ========== Quest Update Tests ==========

    #[test]
    fn test_update_quest_description() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_update_quest(
            "The Lost Artifact",
            Some("Updated description with new information"),
            &[],
        );

        assert!(resolution
            .narrative
            .contains("Quest \"The Lost Artifact\" updated"));
        assert!(resolution.narrative.contains("description changed"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::QuestUpdated { new_description: Some(_), add_rewards, .. }
                if add_rewards.is_empty())
        ));
    }

    #[test]
    fn test_update_quest_rewards() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_update_quest(
            "The Lost Artifact",
            None,
            &["Bonus reward".to_string(), "Extra gold".to_string()],
        );

        assert!(resolution.narrative.contains("rewards added"));
        assert!(resolution.narrative.contains("Bonus reward"));
        assert!(resolution.narrative.contains("Extra gold"));
        assert!(resolution.effects.iter().any(
            |e| matches!(e, Effect::QuestUpdated { new_description: None, add_rewards, .. }
                if add_rewards.len() == 2)
        ));
    }

    #[test]
    fn test_update_quest_description_and_rewards() {
        let engine = RulesEngine::new();

        let resolution = engine.resolve_update_quest(
            "The Lost Artifact",
            Some("New description"),
            &["New reward".to_string()],
        );

        assert!(resolution.narrative.contains("description changed"));
        assert!(resolution.narrative.contains("rewards added"));
    }
}

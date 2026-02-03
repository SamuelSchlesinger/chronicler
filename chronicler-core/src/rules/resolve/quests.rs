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

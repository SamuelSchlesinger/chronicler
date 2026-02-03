//! Quest and objective tracking for D&D campaigns.
//!
//! This module provides structures for managing quests, their objectives,
//! and completion status.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A quest or objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<String>,
    pub giver: Option<String>,
}

impl Quest {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            status: QuestStatus::Active,
            objectives: Vec::new(),
            rewards: Vec::new(),
            giver: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.completed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Active,
    Completed,
    Failed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub completed: bool,
    pub optional: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Quest Tests ==========

    #[test]
    fn test_quest_new() {
        let quest = Quest::new("Save the Village", "Rescue the villagers from the bandits");

        assert_eq!(quest.name, "Save the Village");
        assert_eq!(quest.description, "Rescue the villagers from the bandits");
        assert_eq!(quest.status, QuestStatus::Active);
        assert!(quest.objectives.is_empty());
        assert!(quest.rewards.is_empty());
        assert!(quest.giver.is_none());
    }

    #[test]
    fn test_quest_has_unique_id() {
        let quest1 = Quest::new("Quest 1", "Description 1");
        let quest2 = Quest::new("Quest 2", "Description 2");

        assert_ne!(quest1.id, quest2.id);
    }

    #[test]
    fn test_quest_is_complete_no_objectives() {
        let quest = Quest::new("Empty Quest", "A quest with no objectives");

        // A quest with no objectives is NOT complete
        assert!(!quest.is_complete());
    }

    #[test]
    fn test_quest_is_complete_single_objective() {
        let mut quest = Quest::new("Simple Quest", "One objective");
        quest.objectives.push(QuestObjective {
            description: "Kill the dragon".to_string(),
            completed: false,
            optional: false,
        });

        assert!(!quest.is_complete());

        quest.objectives[0].completed = true;
        assert!(quest.is_complete());
    }

    #[test]
    fn test_quest_is_complete_multiple_objectives() {
        let mut quest = Quest::new("Multi-part Quest", "Several things to do");
        quest.objectives.push(QuestObjective {
            description: "Find the sword".to_string(),
            completed: true,
            optional: false,
        });
        quest.objectives.push(QuestObjective {
            description: "Slay the beast".to_string(),
            completed: false,
            optional: false,
        });
        quest.objectives.push(QuestObjective {
            description: "Return to town".to_string(),
            completed: false,
            optional: false,
        });

        assert!(!quest.is_complete());

        // Complete all objectives
        quest.objectives[1].completed = true;
        quest.objectives[2].completed = true;
        assert!(quest.is_complete());
    }

    #[test]
    fn test_quest_is_complete_with_optional() {
        let mut quest = Quest::new("Quest with Optional", "Main and optional objectives");
        quest.objectives.push(QuestObjective {
            description: "Main objective".to_string(),
            completed: true,
            optional: false,
        });
        quest.objectives.push(QuestObjective {
            description: "Optional bonus".to_string(),
            completed: false,
            optional: true,
        });

        // Note: current is_complete() checks ALL objectives
        // This might be a design decision - testing current behavior
        assert!(!quest.is_complete());

        quest.objectives[1].completed = true;
        assert!(quest.is_complete());
    }

    #[test]
    fn test_quest_status_values() {
        assert_eq!(QuestStatus::Active, QuestStatus::Active);
        assert_ne!(QuestStatus::Active, QuestStatus::Completed);
        assert_ne!(QuestStatus::Completed, QuestStatus::Failed);
        assert_ne!(QuestStatus::Failed, QuestStatus::Abandoned);
    }

    #[test]
    fn test_quest_with_giver() {
        let mut quest = Quest::new("Rescue Mission", "Save the princess");
        quest.giver = Some("King Roland".to_string());

        assert_eq!(quest.giver, Some("King Roland".to_string()));
    }

    #[test]
    fn test_quest_with_rewards() {
        let mut quest = Quest::new("Bounty Hunt", "Eliminate the bandit leader");
        quest.rewards.push("500 gold".to_string());
        quest.rewards.push("+1 Longsword".to_string());
        quest.rewards.push("Noble's favor".to_string());

        assert_eq!(quest.rewards.len(), 3);
        assert!(quest.rewards.contains(&"500 gold".to_string()));
    }

    // ========== QuestObjective Tests ==========

    #[test]
    fn test_quest_objective_fields() {
        let obj = QuestObjective {
            description: "Defeat the boss".to_string(),
            completed: false,
            optional: false,
        };

        assert_eq!(obj.description, "Defeat the boss");
        assert!(!obj.completed);
        assert!(!obj.optional);
    }

    #[test]
    fn test_quest_objective_optional() {
        let obj = QuestObjective {
            description: "Find all collectibles".to_string(),
            completed: false,
            optional: true,
        };

        assert!(obj.optional);
    }

    // ========== QuestStatus Tests ==========

    #[test]
    fn test_quest_status_copy() {
        let status = QuestStatus::Active;
        let copied = status; // Copy trait
        assert_eq!(status, copied);
    }

    #[test]
    fn test_quest_all_statuses() {
        let statuses = [
            QuestStatus::Active,
            QuestStatus::Completed,
            QuestStatus::Failed,
            QuestStatus::Abandoned,
        ];

        // Ensure all statuses are distinct
        for (i, s1) in statuses.iter().enumerate() {
            for (j, s2) in statuses.iter().enumerate() {
                if i == j {
                    assert_eq!(s1, s2);
                } else {
                    assert_ne!(s1, s2);
                }
            }
        }
    }
}

//! Knowledge tracking for entity information asymmetry.
//!
//! This module tracks what each entity knows, when they learned it,
//! and the verification status of that knowledge (true, false, rumor, etc.).

use super::entity::EntityId;
use super::fact::FactId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for a knowledge entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KnowledgeId(Uuid);

impl KnowledgeId {
    /// Create a new unique knowledge ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for KnowledgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for KnowledgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The verification status of a piece of knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Known to be true (verified by DM/story)
    True,
    /// Known to be false (deliberate lie or misinformation)
    False,
    /// Partially true (contains both accurate and inaccurate information)
    PartiallyTrue,
    /// Unknown verification status (rumor, hearsay)
    Unknown,
    /// Was true but is now outdated
    Outdated,
}

impl VerificationStatus {
    /// Parse from string (case-insensitive).
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "true" | "verified" | "fact" => Self::True,
            "false" | "lie" | "misinformation" => Self::False,
            "partial" | "partially_true" | "partially true" => Self::PartiallyTrue,
            "outdated" | "stale" => Self::Outdated,
            _ => Self::Unknown,
        }
    }

    /// Get the name of this verification status.
    pub fn name(&self) -> &'static str {
        match self {
            Self::True => "verified",
            Self::False => "false",
            Self::PartiallyTrue => "partially true",
            Self::Unknown => "unverified",
            Self::Outdated => "outdated",
        }
    }
}

/// A record of an entity knowing a piece of information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Unique identifier for this knowledge entry.
    pub id: KnowledgeId,

    /// The entity that knows this information.
    pub knowing_entity: EntityId,

    /// The content of the knowledge (what they know).
    pub content: String,

    /// Optional reference to a story fact this knowledge is about.
    pub fact_id: Option<FactId>,

    /// The turn when this knowledge was acquired.
    pub learned_at_turn: u32,

    /// Who or what the entity learned this from.
    pub learned_from: Option<KnowledgeSource>,

    /// The verification status of this knowledge.
    pub verification_status: VerificationStatus,

    /// Whether this knowledge is still current (not superseded).
    pub is_current: bool,

    /// Optional context about how this knowledge was shared.
    pub context: Option<String>,
}

impl KnowledgeEntry {
    /// Create a new knowledge entry.
    pub fn new(
        knowing_entity: EntityId,
        content: impl Into<String>,
        verification_status: VerificationStatus,
        current_turn: u32,
    ) -> Self {
        Self {
            id: KnowledgeId::new(),
            knowing_entity,
            content: content.into(),
            fact_id: None,
            learned_at_turn: current_turn,
            learned_from: None,
            verification_status,
            is_current: true,
            context: None,
        }
    }

    /// Set the source of this knowledge.
    pub fn with_source(mut self, source: KnowledgeSource) -> Self {
        self.learned_from = Some(source);
        self
    }

    /// Link this knowledge to a story fact.
    pub fn with_fact(mut self, fact_id: FactId) -> Self {
        self.fact_id = Some(fact_id);
        self
    }

    /// Add context about how this knowledge was shared.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Mark this knowledge as superseded.
    pub fn supersede(&mut self) {
        self.is_current = false;
    }

    /// Update the verification status.
    pub fn update_verification(&mut self, status: VerificationStatus) {
        self.verification_status = status;
    }
}

/// The source of knowledge - where/who the entity learned from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Learned from another entity.
    Entity(EntityId),
    /// Learned from direct observation.
    Observation,
    /// Learned from a written source (book, scroll, etc.).
    WrittenSource(String),
    /// Learned from the player character.
    Player,
    /// The entity always knew this (background knowledge).
    Background,
    /// Unknown or unspecified source.
    Unknown,
}

impl KnowledgeSource {
    /// Parse from string, with optional entity ID.
    pub fn from_str(s: &str, entity_id: Option<EntityId>) -> Self {
        match s.to_lowercase().as_str() {
            "observation" | "observed" | "witnessed" => Self::Observation,
            "player" | "the player" | "pc" => Self::Player,
            "background" | "always knew" => Self::Background,
            "unknown" | "" => Self::Unknown,
            _ => {
                if let Some(id) = entity_id {
                    Self::Entity(id)
                } else {
                    Self::WrittenSource(s.to_string())
                }
            }
        }
    }

    /// Get a description of this source.
    pub fn description(&self) -> String {
        match self {
            Self::Entity(_) => "from another entity".to_string(),
            Self::Observation => "from direct observation".to_string(),
            Self::WrittenSource(s) => format!("from written source: {}", s),
            Self::Player => "from the player".to_string(),
            Self::Background => "background knowledge".to_string(),
            Self::Unknown => "unknown source".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_entry_creation() {
        let entity_id = EntityId::new();
        let entry = KnowledgeEntry::new(
            entity_id,
            "The treasure is hidden in the cave",
            VerificationStatus::True,
            5,
        );

        assert_eq!(entry.knowing_entity, entity_id);
        assert!(entry.content.contains("treasure"));
        assert!(entry.is_current);
        assert_eq!(entry.verification_status, VerificationStatus::True);
    }

    #[test]
    fn test_knowledge_with_source() {
        let entity_id = EntityId::new();
        let source_id = EntityId::new();

        let entry = KnowledgeEntry::new(
            entity_id,
            "The king is planning a war",
            VerificationStatus::Unknown,
            10,
        )
        .with_source(KnowledgeSource::Entity(source_id))
        .with_context("Overheard at the tavern");

        assert!(matches!(
            entry.learned_from,
            Some(KnowledgeSource::Entity(_))
        ));
        assert!(entry.context.is_some());
    }

    #[test]
    fn test_verification_status_parsing() {
        assert_eq!(VerificationStatus::parse("true"), VerificationStatus::True);
        assert_eq!(VerificationStatus::parse("lie"), VerificationStatus::False);
        assert_eq!(
            VerificationStatus::parse("rumor"),
            VerificationStatus::Unknown
        );
        assert_eq!(
            VerificationStatus::parse("partial"),
            VerificationStatus::PartiallyTrue
        );
    }

    #[test]
    fn test_supersede_knowledge() {
        let entity_id = EntityId::new();
        let mut entry =
            KnowledgeEntry::new(entity_id, "Old information", VerificationStatus::True, 1);

        assert!(entry.is_current);
        entry.supersede();
        assert!(!entry.is_current);
    }
}

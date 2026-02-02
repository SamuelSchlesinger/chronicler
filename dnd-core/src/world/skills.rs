//! D&D 5e skills and proficiency levels.
//!
//! This module contains the skill definitions and proficiency tracking
//! for the D&D 5e rules system.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::Ability;

/// D&D 5e skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Athletics,
    Acrobatics,
    SleightOfHand,
    Stealth,
    Arcana,
    History,
    Investigation,
    Nature,
    Religion,
    AnimalHandling,
    Insight,
    Medicine,
    Perception,
    Survival,
    Deception,
    Intimidation,
    Performance,
    Persuasion,
}

impl Skill {
    pub fn ability(&self) -> Ability {
        match self {
            Skill::Athletics => Ability::Strength,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Ability::Dexterity,
            Skill::Arcana
            | Skill::History
            | Skill::Investigation
            | Skill::Nature
            | Skill::Religion => Ability::Intelligence,
            Skill::AnimalHandling
            | Skill::Insight
            | Skill::Medicine
            | Skill::Perception
            | Skill::Survival => Ability::Wisdom,
            Skill::Deception | Skill::Intimidation | Skill::Performance | Skill::Persuasion => {
                Ability::Charisma
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Skill::Athletics => "Athletics",
            Skill::Acrobatics => "Acrobatics",
            Skill::SleightOfHand => "Sleight of Hand",
            Skill::Stealth => "Stealth",
            Skill::Arcana => "Arcana",
            Skill::History => "History",
            Skill::Investigation => "Investigation",
            Skill::Nature => "Nature",
            Skill::Religion => "Religion",
            Skill::AnimalHandling => "Animal Handling",
            Skill::Insight => "Insight",
            Skill::Medicine => "Medicine",
            Skill::Perception => "Perception",
            Skill::Survival => "Survival",
            Skill::Deception => "Deception",
            Skill::Intimidation => "Intimidation",
            Skill::Performance => "Performance",
            Skill::Persuasion => "Persuasion",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Skill::Athletics => "Covers climbing, jumping, or swimming. Used for scaling cliffs, avoiding hazards, or struggling against currents.",
            Skill::Acrobatics => "Staying on your feet in tricky situations like balancing on tightropes, running across ice, or performing acrobatic stunts.",
            Skill::SleightOfHand => "Acts of legerdemain such as planting something on someone, concealing objects, or picking pockets.",
            Skill::Stealth => "Concealing yourself from enemies, slinking past guards, slipping away unnoticed, or sneaking up on someone.",
            Skill::Arcana => "Recall lore about spells, magic items, eldritch symbols, magical traditions, and planes of existence.",
            Skill::History => "Recall lore about historical events, legendary people, ancient kingdoms, past disputes, and lost civilizations.",
            Skill::Investigation => "Looking for clues and making deductions. Deduce hidden locations, determine weapon types, or find structural weaknesses.",
            Skill::Nature => "Recall lore about terrain, plants, animals, weather, and natural cycles.",
            Skill::Religion => "Recall lore about deities, rites, prayers, religious hierarchies, holy symbols, and secret cults.",
            Skill::AnimalHandling => "Calm domesticated animals, keep mounts from being spooked, intuit animal intentions, or control mounts during risky maneuvers.",
            Skill::Insight => "Discern true intentions of a creature, detect lies, or predict someone's next move through body language and speech patterns.",
            Skill::Medicine => "Stabilize a dying companion or diagnose an illness. Covers practical medical knowledge rather than magical healing.",
            Skill::Perception => "Spot, hear, or detect the presence of something. Measures general awareness and keenness of senses.",
            Skill::Survival => "Follow tracks, hunt game, guide groups through wastelands, identify nearby creatures, predict weather, or avoid natural hazards.",
            Skill::Deception => "Convincingly hide the truth through ambiguity, lies, or maintaining a disguise.",
            Skill::Intimidation => "Influence someone through overt threats, hostile actions, or physical violence.",
            Skill::Performance => "Delight an audience with music, dance, acting, storytelling, or other entertainment.",
            Skill::Persuasion => "Influence someone through tact, social graces, or good nature. Used for cordial requests or proper etiquette.",
        }
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Proficiency level for skills/tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProficiencyLevel {
    #[default]
    None,
    Half,
    Proficient,
    Expertise,
}

impl ProficiencyLevel {
    pub fn bonus(&self, proficiency_bonus: i8) -> i8 {
        match self {
            ProficiencyLevel::None => 0,
            ProficiencyLevel::Half => proficiency_bonus / 2,
            ProficiencyLevel::Proficient => proficiency_bonus,
            ProficiencyLevel::Expertise => proficiency_bonus * 2,
        }
    }
}

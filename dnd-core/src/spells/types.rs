//! Type definitions for spells and spellcasting mechanics.

use crate::rules::DamageType;
use crate::world::Ability;
use serde::{Deserialize, Serialize};

/// Schools of magic in D&D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}

impl SpellSchool {
    pub fn name(&self) -> &'static str {
        match self {
            SpellSchool::Abjuration => "Abjuration",
            SpellSchool::Conjuration => "Conjuration",
            SpellSchool::Divination => "Divination",
            SpellSchool::Enchantment => "Enchantment",
            SpellSchool::Evocation => "Evocation",
            SpellSchool::Illusion => "Illusion",
            SpellSchool::Necromancy => "Necromancy",
            SpellSchool::Transmutation => "Transmutation",
        }
    }
}

/// How long it takes to cast a spell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastingTime {
    Action,
    BonusAction,
    Reaction(String), // Trigger condition
    Minutes(u32),
    Hours(u32),
}

impl CastingTime {
    pub fn description(&self) -> String {
        match self {
            CastingTime::Action => "1 action".to_string(),
            CastingTime::BonusAction => "1 bonus action".to_string(),
            CastingTime::Reaction(trigger) => format!("1 reaction, {}", trigger),
            CastingTime::Minutes(m) => format!("{} minute{}", m, if *m == 1 { "" } else { "s" }),
            CastingTime::Hours(h) => format!("{} hour{}", h, if *h == 1 { "" } else { "s" }),
        }
    }
}

/// Range of a spell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellRange {
    Self_,
    Touch,
    Feet(u32),
    Miles(u32),
    Sight,
    Unlimited,
    SelfRadius(u32),    // Self with radius (e.g., Thunderwave)
    SelfCone(u32),      // Self with cone (e.g., Burning Hands)
    SelfLine(u32, u32), // Self with line (length, width)
}

impl SpellRange {
    pub fn description(&self) -> String {
        match self {
            SpellRange::Self_ => "Self".to_string(),
            SpellRange::Touch => "Touch".to_string(),
            SpellRange::Feet(f) => format!("{} feet", f),
            SpellRange::Miles(m) => format!("{} mile{}", m, if *m == 1 { "" } else { "s" }),
            SpellRange::Sight => "Sight".to_string(),
            SpellRange::Unlimited => "Unlimited".to_string(),
            SpellRange::SelfRadius(r) => format!("Self ({}-foot radius)", r),
            SpellRange::SelfCone(c) => format!("Self ({}-foot cone)", c),
            SpellRange::SelfLine(l, w) => format!("Self ({}-foot line, {} feet wide)", l, w),
        }
    }
}

/// Spell components required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Components {
    pub verbal: bool,
    pub somatic: bool,
    pub material: Option<String>,
}

impl Components {
    pub fn v() -> Self {
        Self {
            verbal: true,
            somatic: false,
            material: None,
        }
    }

    pub fn vs() -> Self {
        Self {
            verbal: true,
            somatic: true,
            material: None,
        }
    }

    pub fn vsm(material: &str) -> Self {
        Self {
            verbal: true,
            somatic: true,
            material: Some(material.to_string()),
        }
    }

    pub fn s() -> Self {
        Self {
            verbal: false,
            somatic: true,
            material: None,
        }
    }

    pub fn sm(material: &str) -> Self {
        Self {
            verbal: false,
            somatic: true,
            material: Some(material.to_string()),
        }
    }

    pub fn description(&self) -> String {
        let mut parts = Vec::new();
        if self.verbal {
            parts.push("V");
        }
        if self.somatic {
            parts.push("S");
        }
        if self.material.is_some() {
            parts.push("M");
        }
        let base = parts.join(", ");
        if let Some(ref mat) = self.material {
            format!("{} ({})", base, mat)
        } else {
            base
        }
    }
}

/// Duration of a spell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellDuration {
    Instantaneous,
    Rounds(u32),
    Minutes(u32),
    Hours(u32),
    Days(u32),
    UntilDispelled,
    Special,
}

impl SpellDuration {
    pub fn description(&self) -> String {
        match self {
            SpellDuration::Instantaneous => "Instantaneous".to_string(),
            SpellDuration::Rounds(r) => format!("{} round{}", r, if *r == 1 { "" } else { "s" }),
            SpellDuration::Minutes(m) => format!("{} minute{}", m, if *m == 1 { "" } else { "s" }),
            SpellDuration::Hours(h) => format!("{} hour{}", h, if *h == 1 { "" } else { "s" }),
            SpellDuration::Days(d) => format!("{} day{}", d, if *d == 1 { "" } else { "s" }),
            SpellDuration::UntilDispelled => "Until dispelled".to_string(),
            SpellDuration::Special => "Special".to_string(),
        }
    }
}

/// Type of spell attack (if any).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellAttackType {
    Melee,
    Ranged,
}

/// Area of effect shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AreaOfEffect {
    None,
    Sphere(u32),        // Radius in feet
    Cube(u32),          // Side length in feet
    Cone(u32),          // Length in feet
    Line(u32, u32),     // Length, width in feet
    Cylinder(u32, u32), // Radius, height in feet
}

/// How spell damage scales with level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageScaling {
    /// Cantrip scaling: increases at levels 5, 11, 17
    CantripScaling,
    /// Scales with upcast level (e.g., +1d6 per level above base)
    PerSlotLevel { extra_dice: String },
    /// No scaling
    None,
}

/// Complete spell definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    pub name: String,
    pub level: u8, // 0 for cantrips
    pub school: SpellSchool,
    pub casting_time: CastingTime,
    pub range: SpellRange,
    pub components: Components,
    pub duration: SpellDuration,
    pub concentration: bool,
    pub ritual: bool,
    pub description: String,

    // Combat mechanics
    pub damage_dice: Option<String>,
    pub damage_type: Option<DamageType>,
    pub damage_scaling: DamageScaling,
    pub healing_dice: Option<String>,
    pub save_type: Option<Ability>,
    pub save_effect: Option<String>, // What happens on save (e.g., "half damage")
    pub attack_type: Option<SpellAttackType>,
    pub area_of_effect: AreaOfEffect,

    // Class lists (simplified - which classes can learn this spell)
    pub classes: Vec<SpellClass>,
}

/// Classes that can learn spells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellClass {
    Bard,
    Cleric,
    Druid,
    Paladin,
    Ranger,
    Sorcerer,
    Warlock,
    Wizard,
}

impl SpellData {
    /// Check if this is a cantrip.
    pub fn is_cantrip(&self) -> bool {
        self.level == 0
    }

    /// Get the number of damage dice based on caster level (for cantrips).
    pub fn cantrip_dice_count(&self, caster_level: u8) -> u8 {
        match caster_level {
            1..=4 => 1,
            5..=10 => 2,
            11..=16 => 3,
            _ => 4,
        }
    }

    /// Calculate damage dice for a given caster level and slot level.
    pub fn effective_damage_dice(&self, caster_level: u8, slot_level: u8) -> Option<String> {
        let base_dice = self.damage_dice.as_ref()?;

        match &self.damage_scaling {
            DamageScaling::CantripScaling => {
                // Parse base dice (e.g., "1d10") and multiply
                let count = self.cantrip_dice_count(caster_level);
                if let Some(d_pos) = base_dice.find('d') {
                    let die_type = &base_dice[d_pos..];
                    Some(format!("{}{}", count, die_type))
                } else {
                    Some(base_dice.clone())
                }
            }
            DamageScaling::PerSlotLevel { extra_dice } => {
                if slot_level > self.level {
                    let extra_levels = slot_level - self.level;
                    // Parse extra dice (e.g., "1d6") and multiply
                    if let Some(d_pos) = extra_dice.find('d') {
                        let num: u8 = extra_dice[..d_pos].parse().unwrap_or(1);
                        let die_type = &extra_dice[d_pos..];
                        let total_extra = num * extra_levels;
                        Some(format!("{} + {}{}", base_dice, total_extra, die_type))
                    } else {
                        Some(base_dice.clone())
                    }
                } else {
                    Some(base_dice.clone())
                }
            }
            DamageScaling::None => Some(base_dice.clone()),
        }
    }
}

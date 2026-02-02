//! World state snapshot for UI rendering.

use dnd_core::world::{
    AbilityScores, CombatState, Condition, DeathSaves, GameMode, GameTime, HitPoints, Item, Quest,
    Skill,
};
use dnd_core::GameSession;
use std::collections::HashMap;

/// World state snapshot for UI rendering.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorldUpdate {
    /// Player hit points.
    pub player_hp: HitPoints,
    /// Current combat state if any.
    pub combat: Option<CombatState>,
    /// Current game mode.
    pub mode: GameMode,
    /// Current game time.
    pub game_time: GameTime,
    /// Player name.
    pub player_name: String,
    /// Player class.
    pub player_class: Option<String>,
    /// Player level.
    pub player_level: u8,
    /// Player AC.
    pub player_ac: u8,
    /// Player initiative modifier.
    pub player_initiative: i8,
    /// Player speed.
    pub player_speed: u32,
    /// Current location name.
    pub current_location: String,
    /// Campaign name.
    pub campaign_name: String,
    /// Active conditions affecting the player.
    pub conditions: Vec<Condition>,
    /// Death save progress (when at 0 HP).
    pub death_saves: DeathSaves,
    /// Player's gold pieces.
    pub gold: i32,
    /// Player's silver pieces.
    pub silver: i32,
    /// Equipped weapon name (if any).
    pub equipped_weapon: Option<String>,
    /// Equipped armor name (if any).
    pub equipped_armor: Option<String>,
    /// Inventory items.
    pub inventory_items: Vec<Item>,
    /// Ability scores.
    pub ability_scores: AbilityScores,
    /// Skill proficiencies (skill -> proficiency level string).
    pub skill_proficiencies: HashMap<Skill, String>,
    /// Proficiency bonus.
    pub proficiency_bonus: i8,
    /// Active and completed quests.
    pub quests: Vec<Quest>,
    /// Spell slots (level 1-9): (available, total) for each level
    pub spell_slots: Vec<(u8, u8)>,
    /// Known/prepared spells
    pub known_spells: Vec<String>,
    /// Known cantrips
    pub cantrips: Vec<String>,
    /// Spellcasting ability (if any)
    pub spellcasting_ability: Option<String>,
    /// Spell save DC (if spellcaster)
    pub spell_save_dc: Option<u8>,
    /// Spell attack bonus (if spellcaster)
    pub spell_attack_bonus: Option<i8>,
}

impl Default for WorldUpdate {
    fn default() -> Self {
        Self {
            player_hp: HitPoints::new(10),
            combat: None,
            mode: GameMode::Exploration,
            game_time: GameTime::default(),
            player_name: "???".to_string(),
            player_class: None,
            player_level: 1,
            player_ac: 10,
            player_initiative: 0,
            player_speed: 30,
            current_location: "Unknown".to_string(),
            campaign_name: "New Campaign".to_string(),
            conditions: Vec::new(),
            death_saves: DeathSaves::default(),
            gold: 0,
            silver: 0,
            equipped_weapon: None,
            equipped_armor: None,
            inventory_items: Vec::new(),
            ability_scores: AbilityScores::default(),
            skill_proficiencies: HashMap::new(),
            proficiency_bonus: 2,
            quests: Vec::new(),
            spell_slots: Vec::new(),
            known_spells: Vec::new(),
            cantrips: Vec::new(),
            spellcasting_ability: None,
            spell_save_dc: None,
            spell_attack_bonus: None,
        }
    }
}

impl WorldUpdate {
    /// Create a WorldUpdate snapshot from a GameSession.
    pub fn from_session(session: &GameSession) -> Self {
        let world = session.world();
        let character = &world.player_character;
        Self {
            player_hp: character.hit_points.clone(),
            combat: world.combat.clone(),
            mode: world.mode,
            game_time: world.game_time.clone(),
            player_name: character.name.clone(),
            player_class: character
                .classes
                .first()
                .map(|c| c.class.name().to_string()),
            player_level: character.level,
            player_ac: character.current_ac(),
            player_initiative: character.initiative_modifier(),
            player_speed: character.speed.walk,
            current_location: world.current_location.name.clone(),
            campaign_name: world.campaign_name.clone(),
            conditions: character.conditions.iter().map(|c| c.condition).collect(),
            death_saves: character.death_saves.clone(),
            gold: character.inventory.gold,
            silver: character.inventory.silver,
            equipped_weapon: character
                .equipment
                .main_hand
                .as_ref()
                .map(|w| w.base.name.clone()),
            equipped_armor: character
                .equipment
                .armor
                .as_ref()
                .map(|a| a.base.name.clone()),
            inventory_items: character.inventory.items.clone(),
            ability_scores: character.ability_scores.clone(),
            skill_proficiencies: character
                .skill_proficiencies
                .iter()
                .map(|(skill, level)| (*skill, format!("{level:?}")))
                .collect(),
            proficiency_bonus: character.proficiency_bonus(),
            quests: world.quests.clone(),
            spell_slots: character
                .spellcasting
                .as_ref()
                .map(|sc| {
                    sc.spell_slots
                        .slots
                        .iter()
                        .map(|slot| (slot.available(), slot.total))
                        .collect()
                })
                .unwrap_or_default(),
            known_spells: character
                .spellcasting
                .as_ref()
                .map(|sc| sc.spells_known.clone())
                .unwrap_or_default(),
            cantrips: character
                .spellcasting
                .as_ref()
                .map(|sc| sc.cantrips_known.clone())
                .unwrap_or_default(),
            spellcasting_ability: character
                .spellcasting
                .as_ref()
                .map(|sc| sc.ability.name().to_string()),
            spell_save_dc: character.spellcasting.as_ref().map(|sc| {
                let mod_ = character.ability_scores.modifier(sc.ability);
                (8 + mod_ + character.proficiency_bonus()) as u8
            }),
            spell_attack_bonus: character.spellcasting.as_ref().map(|sc| {
                let mod_ = character.ability_scores.modifier(sc.ability);
                mod_ + character.proficiency_bonus()
            }),
        }
    }
}

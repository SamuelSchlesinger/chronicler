//! D&D 5e Rules Engine with Intent/Effect system.
//!
//! This module implements the core game mechanic pipeline:
//! 1. AI suggests an Intent (what the player/NPC wants to do)
//! 2. RulesEngine resolves the Intent using D&D 5e rules
//! 3. Effects are produced that describe state changes
//! 4. Effects are applied to the GameWorld
//!
//! This separation ensures deterministic, testable game mechanics
//! independent of AI decision-making.

use crate::dice::{self, Advantage, ComponentResult, DiceExpression, DieType, RollResult};
use crate::world::{
    Ability, CharacterClass, CharacterId, Combatant, Condition, GameWorld, Item, ItemType, Skill,
    SlotInfo, SpellSlots, SpellcastingData,
};
use serde::{Deserialize, Serialize};

/// Roll dice with a fallback expression. If both fail, returns a minimal result.
///
/// This avoids nested unwraps which could panic in edge cases.
fn roll_with_fallback(notation: &str, fallback: &str) -> RollResult {
    dice::roll(notation)
        .or_else(|_| dice::roll(fallback))
        .unwrap_or_else(|_| {
            // Create a minimal fallback result (1d4 = 1)
            let expr = DiceExpression {
                components: vec![],
                modifier: 1,
                original: fallback.to_string(),
            };
            RollResult {
                expression: expr,
                component_results: vec![ComponentResult {
                    die_type: DieType::D4,
                    rolls: vec![1],
                    kept: vec![1],
                    subtotal: 1,
                }],
                modifier: 0,
                total: 1,
                natural_20: false,
                natural_1: false,
            }
        })
}

/// Calculate the number of d6s for Sneak Attack based on Rogue level.
/// Sneak Attack scales: 1d6 at level 1, +1d6 every odd level.
fn sneak_attack_dice(rogue_level: u8) -> u8 {
    // 1d6 at 1, 2d6 at 3, 3d6 at 5, etc.
    rogue_level.div_ceil(2)
}

/// An intent represents what a character wants to do.
/// The AI generates intents, the RulesEngine resolves them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// Attack a target with a weapon
    Attack {
        attacker_id: CharacterId,
        target_id: CharacterId,
        weapon_name: String,
        advantage: Advantage,
    },

    /// Cast a spell
    CastSpell {
        caster_id: CharacterId,
        spell_name: String,
        targets: Vec<CharacterId>,
        spell_level: u8,
        /// Target names (for when we don't have CharacterIds)
        target_names: Vec<String>,
    },

    /// Make a skill check
    SkillCheck {
        character_id: CharacterId,
        skill: Skill,
        dc: i32,
        advantage: Advantage,
        description: String,
    },

    /// Make an ability check (raw ability, not skill)
    AbilityCheck {
        character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        description: String,
    },

    /// Make a saving throw
    SavingThrow {
        character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        source: String,
    },

    /// Deal damage to a target
    Damage {
        target_id: CharacterId,
        amount: i32,
        damage_type: DamageType,
        source: String,
    },

    /// Heal a target
    Heal {
        target_id: CharacterId,
        amount: i32,
        source: String,
    },

    /// Apply a condition to a target
    ApplyCondition {
        target_id: CharacterId,
        condition: Condition,
        source: String,
        duration_rounds: Option<u32>,
    },

    /// Remove a condition from a target
    RemoveCondition {
        target_id: CharacterId,
        condition: Condition,
    },

    /// Move to a different location or position
    Move {
        character_id: CharacterId,
        destination: String,
        distance_feet: u32,
    },

    /// Take a short rest
    ShortRest,

    /// Take a long rest
    LongRest,

    /// Start combat
    StartCombat { combatants: Vec<CombatantInit> },

    /// End combat
    EndCombat,

    /// Advance to next turn in combat
    NextTurn,

    /// Roll initiative for a character
    RollInitiative {
        character_id: CharacterId,
        name: String,
        modifier: i8,
        is_player: bool,
    },

    /// Generic dice roll (not tied to a specific mechanic)
    RollDice { notation: String, purpose: String },

    /// Advance game time
    AdvanceTime { minutes: u32 },

    /// Add experience points
    GainExperience { amount: u32 },

    /// Use a class feature
    UseFeature {
        character_id: CharacterId,
        feature_name: String,
    },

    /// Remember a story fact (for narrative consistency)
    RememberFact {
        subject_name: String,
        subject_type: String,
        fact: String,
        category: String,
        related_entities: Vec<String>,
        importance: f32,
    },

    // Inventory management
    /// Add an item to the player's inventory
    AddItem {
        item_name: String,
        quantity: u32,
        item_type: Option<String>,
        description: Option<String>,
        magical: bool,
        weight: Option<f32>,
        value_gp: Option<f32>,
    },

    /// Remove an item from the player's inventory
    RemoveItem { item_name: String, quantity: u32 },

    /// Equip an item from inventory
    EquipItem { item_name: String },

    /// Unequip an item from a slot
    UnequipItem { slot: String },

    /// Use a consumable item
    UseItem {
        item_name: String,
        target_id: Option<CharacterId>,
    },

    /// Adjust the player's gold
    AdjustGold { amount: i32, reason: String },

    /// Adjust the player's silver
    AdjustSilver { amount: i32, reason: String },

    /// Make a death saving throw (when at 0 HP)
    DeathSave { character_id: CharacterId },

    /// Make a concentration check (when taking damage while concentrating)
    ConcentrationCheck {
        character_id: CharacterId,
        damage_taken: i32,
        spell_name: String,
    },

    /// Change the current location
    ChangeLocation {
        new_location: String,
        location_type: Option<String>,
        description: Option<String>,
    },

    /// Register a consequence for future triggering
    RegisterConsequence {
        /// Natural language description of when this triggers
        trigger_description: String,
        /// Natural language description of what happens when triggered
        consequence_description: String,
        /// Severity level: minor, moderate, major, critical
        severity: String,
        /// Names of related entities
        related_entities: Vec<String>,
        /// Importance score (0.0 to 1.0)
        importance: f32,
        /// Number of turns until this expires (None = never expires)
        expires_in_turns: Option<u32>,
    },

    // ========================================================================
    // Class Feature Intents
    // ========================================================================
    /// Barbarian enters a rage
    UseRage { character_id: CharacterId },

    /// Barbarian ends their rage
    EndRage {
        character_id: CharacterId,
        reason: String,
    },

    /// Monk spends ki points
    UseKi {
        character_id: CharacterId,
        points: u8,
        ability: String,
    },

    /// Paladin uses Lay on Hands
    UseLayOnHands {
        character_id: CharacterId,
        target_name: String,
        hp_amount: u32,
        cure_disease: bool,
        neutralize_poison: bool,
    },

    /// Paladin uses Divine Smite
    UseDivineSmite {
        character_id: CharacterId,
        spell_slot_level: u8,
        target_is_undead_or_fiend: bool,
    },

    /// Druid transforms into a beast via Wild Shape
    UseWildShape {
        character_id: CharacterId,
        beast_form: String,
        beast_hp: i32,
        beast_ac: Option<u8>,
    },

    /// Druid reverts from Wild Shape
    EndWildShape {
        character_id: CharacterId,
        reason: String,
        excess_damage: i32,
    },

    /// Cleric/Paladin uses Channel Divinity
    UseChannelDivinity {
        character_id: CharacterId,
        option: String,
        targets: Vec<String>,
    },

    /// Bard grants Bardic Inspiration
    UseBardicInspiration {
        character_id: CharacterId,
        target_name: String,
        die_size: String,
    },

    /// Fighter uses Action Surge
    UseActionSurge {
        character_id: CharacterId,
        action_taken: String,
    },

    /// Fighter uses Second Wind
    UseSecondWind { character_id: CharacterId },

    /// Sorcerer uses Sorcery Points for Metamagic
    UseSorceryPoints {
        character_id: CharacterId,
        points: u8,
        metamagic: String,
        spell_name: Option<String>,
        slot_level: Option<u8>,
    },

    // ========================================================================
    // Quest Management Intents
    // ========================================================================
    /// Create a new quest
    CreateQuest {
        name: String,
        description: String,
        giver: Option<String>,
        /// Objectives as (description, is_optional) pairs
        objectives: Vec<(String, bool)>,
        rewards: Vec<String>,
    },

    /// Add an objective to an existing quest
    AddQuestObjective {
        quest_name: String,
        objective: String,
        optional: bool,
    },

    /// Complete a specific objective in a quest
    CompleteObjective {
        quest_name: String,
        /// Partial match allowed for objective description
        objective_description: String,
    },

    /// Mark a quest as completed
    CompleteQuest {
        quest_name: String,
        completion_note: Option<String>,
    },

    /// Mark a quest as failed
    FailQuest {
        quest_name: String,
        failure_reason: String,
    },

    /// Update quest details
    UpdateQuest {
        quest_name: String,
        new_description: Option<String>,
        add_rewards: Vec<String>,
    },

    // ========================================================================
    // World Building Intents
    // ========================================================================
    /// Create a new NPC in the world
    CreateNpc {
        name: String,
        description: String,
        personality: String,
        occupation: Option<String>,
        disposition: String,
        location: Option<String>,
        known_information: Vec<String>,
    },

    /// Update an existing NPC's attributes
    UpdateNpc {
        npc_name: String,
        disposition: Option<String>,
        add_information: Vec<String>,
        new_description: Option<String>,
        new_personality: Option<String>,
    },

    /// Move an NPC to a new location
    MoveNpc {
        npc_name: String,
        destination: String,
        reason: Option<String>,
    },

    /// Remove an NPC from the world
    RemoveNpc {
        npc_name: String,
        reason: String,
        permanent: bool,
    },

    /// Create a new location in the world
    CreateLocation {
        name: String,
        location_type: String,
        description: String,
        parent_location: Option<String>,
        items: Vec<String>,
        npcs_present: Vec<String>,
    },

    /// Connect two locations with a path/route
    ConnectLocations {
        from_location: String,
        to_location: String,
        direction: Option<String>,
        travel_time_minutes: Option<u32>,
        bidirectional: bool,
    },

    /// Update an existing location's attributes
    UpdateLocation {
        location_name: String,
        new_description: Option<String>,
        add_items: Vec<String>,
        remove_items: Vec<String>,
        add_npcs: Vec<String>,
        remove_npcs: Vec<String>,
    },

    /// Modify an ability score temporarily or permanently
    ModifyAbilityScore {
        ability: Ability,
        modifier: i8,
        source: String,
        duration: Option<String>,
    },

    /// Restore a spell slot
    RestoreSpellSlot { slot_level: u8, source: String },

    // ========================================================================
    // State Assertion Intents (declarative state changes)
    // ========================================================================
    /// Assert a state change for an entity (simpler alternative to specific update tools)
    AssertState {
        entity_name: String,
        state_type: StateType,
        new_value: String,
        reason: String,
        target_entity: Option<String>,
    },

    // ========================================================================
    // Knowledge Tracking Intents
    // ========================================================================
    /// Share knowledge with an entity
    ShareKnowledge {
        knowing_entity: String,
        content: String,
        source: String,
        verification: String,
        context: Option<String>,
    },

    // ========================================================================
    // Scheduled Event Intents
    // ========================================================================
    /// Schedule a future event
    ScheduleEvent {
        description: String,
        /// Minutes from now (for relative timing)
        minutes: Option<u32>,
        /// Hours from now (for relative timing)
        hours: Option<u32>,
        /// Absolute timing: day of month
        day: Option<u8>,
        /// Absolute timing: month
        month: Option<u8>,
        /// Absolute timing: year
        year: Option<i32>,
        /// Hour of day (0-23)
        hour: Option<u8>,
        /// For daily events: hour
        daily_hour: Option<u8>,
        /// For daily events: minute
        daily_minute: Option<u8>,
        /// Location where event occurs
        location: Option<String>,
        /// Entities involved
        involved_entities: Vec<String>,
        /// Visibility to player
        visibility: String,
        /// Whether event repeats
        repeating: bool,
    },

    /// Cancel a scheduled event
    CancelEvent {
        event_description: String,
        reason: String,
    },
}

/// Types of state that can be asserted/changed for entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    /// Attitude toward the player (hostile/unfriendly/neutral/friendly/helpful)
    Disposition,
    /// Physical location
    Location,
    /// Alive/dead/injured/missing/etc.
    Status,
    /// What the entity knows
    Knowledge,
    /// Relationship to another entity
    Relationship,
}

impl StateType {
    /// Parse from string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "disposition" => Some(StateType::Disposition),
            "location" => Some(StateType::Location),
            "status" => Some(StateType::Status),
            "knowledge" => Some(StateType::Knowledge),
            "relationship" => Some(StateType::Relationship),
            _ => None,
        }
    }

    /// Get the name of this state type.
    pub fn name(&self) -> &'static str {
        match self {
            StateType::Disposition => "disposition",
            StateType::Location => "location",
            StateType::Status => "status",
            StateType::Knowledge => "knowledge",
            StateType::Relationship => "relationship",
        }
    }
}

/// Initial combatant data for starting combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatantInit {
    pub id: CharacterId,
    pub name: String,
    pub is_player: bool,
    pub is_ally: bool,
    pub current_hp: i32,
    pub max_hp: i32,
    pub armor_class: u8,
    /// Initiative modifier (DEX mod for most creatures)
    pub initiative_modifier: i8,
}

/// Common D&D damage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Slashing,
    Piercing,
    Bludgeoning,
    Fire,
    Cold,
    Lightning,
    Thunder,
    Acid,
    Poison,
    Necrotic,
    Radiant,
    Force,
    Psychic,
}

impl DamageType {
    pub fn name(&self) -> &'static str {
        match self {
            DamageType::Slashing => "slashing",
            DamageType::Piercing => "piercing",
            DamageType::Bludgeoning => "bludgeoning",
            DamageType::Fire => "fire",
            DamageType::Cold => "cold",
            DamageType::Lightning => "lightning",
            DamageType::Thunder => "thunder",
            DamageType::Acid => "acid",
            DamageType::Poison => "poison",
            DamageType::Necrotic => "necrotic",
            DamageType::Radiant => "radiant",
            DamageType::Force => "force",
            DamageType::Psychic => "psychic",
        }
    }
}

/// The result of resolving an intent.
#[derive(Debug, Clone)]
pub struct Resolution {
    pub effects: Vec<Effect>,
    pub narrative: String,
}

impl Resolution {
    pub fn new(narrative: impl Into<String>) -> Self {
        Self {
            effects: Vec::new(),
            narrative: narrative.into(),
        }
    }

    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn with_effects(mut self, effects: impl IntoIterator<Item = Effect>) -> Self {
        self.effects.extend(effects);
        self
    }
}

/// Effects are the result of resolving an intent.
/// They describe concrete state changes to apply to the GameWorld.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    /// A dice roll occurred
    DiceRolled { roll: RollResult, purpose: String },

    /// Sneak Attack was used (for tracking once-per-turn usage)
    SneakAttackUsed {
        character_id: CharacterId,
        damage_dice: u8,
    },

    /// HP changed (damage or healing)
    HpChanged {
        target_id: CharacterId,
        amount: i32,
        new_current: i32,
        new_max: i32,
        dropped_to_zero: bool,
    },

    /// A condition was applied
    ConditionApplied {
        target_id: CharacterId,
        condition: Condition,
        source: String,
        duration_rounds: Option<u32>,
    },

    /// A condition was removed
    ConditionRemoved {
        target_id: CharacterId,
        condition: Condition,
    },

    /// Combat started
    CombatStarted,

    /// Combat ended
    CombatEnded,

    /// Turn advanced in combat
    TurnAdvanced {
        round: u32,
        current_combatant: String,
    },

    /// Initiative rolled
    InitiativeRolled {
        character_id: CharacterId,
        name: String,
        roll: i32,
        total: i32,
    },

    /// Combatant added to initiative order
    CombatantAdded {
        id: CharacterId,
        name: String,
        initiative: i32,
        is_ally: bool,
        current_hp: i32,
        max_hp: i32,
        armor_class: u8,
    },

    /// Time advanced
    TimeAdvanced { minutes: u32 },

    /// Experience gained
    ExperienceGained { amount: u32, new_total: u32 },

    /// Level up occurred
    LevelUp { new_level: u8 },

    /// Feature use consumed
    FeatureUsed {
        feature_name: String,
        uses_remaining: u8,
    },

    /// Spell slot consumed
    SpellSlotUsed { level: u8, remaining: u8 },

    /// Rest completed
    RestCompleted { rest_type: RestType },

    /// A check succeeded
    CheckSucceeded {
        check_type: String,
        roll: i32,
        dc: i32,
    },

    /// A check failed
    CheckFailed {
        check_type: String,
        roll: i32,
        dc: i32,
    },

    /// Attack hit
    AttackHit {
        attacker_name: String,
        target_name: String,
        attack_roll: i32,
        target_ac: u8,
        is_critical: bool,
    },

    /// Attack missed
    AttackMissed {
        attacker_name: String,
        target_name: String,
        attack_roll: i32,
        target_ac: u8,
    },

    /// A story fact was recorded for memory
    FactRemembered {
        subject_name: String,
        subject_type: String,
        fact: String,
        category: String,
        related_entities: Vec<String>,
        importance: f32,
    },

    // Inventory effects
    /// An item was added to inventory
    ItemAdded {
        item_name: String,
        quantity: u32,
        new_total: u32,
    },

    /// An item was removed from inventory
    ItemRemoved {
        item_name: String,
        quantity: u32,
        remaining: u32,
    },

    /// An item was equipped
    ItemEquipped { item_name: String, slot: String },

    /// An item was unequipped
    ItemUnequipped { item_name: String, slot: String },

    /// An item was used (consumable)
    ItemUsed { item_name: String, result: String },

    /// Gold was added or removed
    GoldChanged {
        amount: i32,
        new_total: i32,
        reason: String,
    },

    /// Silver was added or removed
    SilverChanged {
        amount: i32,
        new_total: i32,
        reason: String,
    },

    /// AC was recalculated due to equipment change
    AcChanged { new_ac: u8, source: String },

    /// Death save failure (damage while at 0 HP)
    DeathSaveFailure {
        target_id: CharacterId,
        failures: u8,
        total_failures: u8,
        source: String,
    },

    /// Death saves were reset (healed from 0 HP)
    DeathSavesReset { target_id: CharacterId },

    /// Character died (3 death save failures or massive damage)
    CharacterDied {
        target_id: CharacterId,
        cause: String,
    },

    /// Death save success (from rolling)
    DeathSaveSuccess {
        target_id: CharacterId,
        roll: i32,
        total_successes: u8,
    },

    /// Character stabilized (3 death save successes)
    Stabilized { target_id: CharacterId },

    /// Concentration was broken
    ConcentrationBroken {
        character_id: CharacterId,
        spell_name: String,
        damage_taken: i32,
        roll: i32,
        dc: i32,
    },

    /// Concentration was maintained
    ConcentrationMaintained {
        character_id: CharacterId,
        spell_name: String,
        roll: i32,
        dc: i32,
    },

    /// Location changed
    LocationChanged {
        previous_location: String,
        new_location: String,
    },

    /// A consequence was registered for future triggering
    ConsequenceRegistered {
        /// Unique identifier (as string for serialization)
        consequence_id: String,
        trigger_description: String,
        consequence_description: String,
        severity: String,
    },

    /// A consequence was triggered
    ConsequenceTriggered {
        /// Unique identifier (as string for serialization)
        consequence_id: String,
        consequence_description: String,
    },

    /// A class-specific resource was used (ki, rage, sorcery points, etc.)
    ClassResourceUsed {
        character_name: String,
        resource_name: String,
        description: String,
    },

    /// Barbarian rage started
    RageStarted {
        character_id: CharacterId,
        damage_bonus: i8,
    },

    /// Barbarian rage ended
    RageEnded {
        character_id: CharacterId,
        reason: String,
    },

    // ========================================================================
    // Quest Effects
    // ========================================================================
    /// A new quest was created
    QuestCreated {
        name: String,
        description: String,
        giver: Option<String>,
        objectives: Vec<(String, bool)>,
        rewards: Vec<String>,
    },

    /// A quest objective was added
    QuestObjectiveAdded {
        quest_name: String,
        objective: String,
        optional: bool,
    },

    /// A quest objective was completed
    QuestObjectiveCompleted {
        quest_name: String,
        objective_description: String,
    },

    /// A quest was completed
    QuestCompleted {
        quest_name: String,
        completion_note: Option<String>,
    },

    /// A quest was failed
    QuestFailed {
        quest_name: String,
        failure_reason: String,
    },

    /// A quest was updated
    QuestUpdated {
        quest_name: String,
        new_description: Option<String>,
        add_rewards: Vec<String>,
    },

    // ========================================================================
    // World Building Effects
    // ========================================================================
    /// An NPC was created
    NpcCreated {
        name: String,
        location: Option<String>,
    },

    /// An NPC was updated
    NpcUpdated { npc_name: String, changes: String },

    /// An NPC was moved to a new location
    NpcMoved {
        npc_name: String,
        from_location: Option<String>,
        to_location: String,
    },

    /// An NPC was removed from the world
    NpcRemoved { npc_name: String, reason: String },

    /// A location was created
    LocationCreated { name: String, location_type: String },

    /// Two locations were connected
    LocationsConnected {
        from: String,
        to: String,
        direction: Option<String>,
    },

    /// A location was updated
    LocationUpdated {
        location_name: String,
        changes: String,
    },

    /// An ability score was modified
    AbilityScoreModified {
        ability: Ability,
        modifier: i8,
        source: String,
    },

    /// A spell slot was restored
    SpellSlotRestored { level: u8, new_remaining: u8 },

    // ========================================================================
    // State Assertion Effects
    // ========================================================================
    /// An entity's state was asserted/changed
    StateAsserted {
        entity_name: String,
        state_type: StateType,
        old_value: Option<String>,
        new_value: String,
        reason: String,
        target_entity: Option<String>,
    },

    // ========================================================================
    // Knowledge Tracking Effects
    // ========================================================================
    /// Knowledge was shared with an entity
    KnowledgeShared {
        knowing_entity: String,
        content: String,
        source: String,
        verification: String,
        context: Option<String>,
    },

    // ========================================================================
    // Scheduled Event Effects
    // ========================================================================
    /// An event was scheduled
    EventScheduled {
        description: String,
        trigger_description: String,
        location: Option<String>,
        visibility: String,
    },

    /// A scheduled event was cancelled
    EventCancelled { description: String, reason: String },

    /// A scheduled event was triggered (for notification)
    EventTriggered {
        description: String,
        location: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RestType {
    Short,
    Long,
}

/// The rules engine resolves intents into effects using D&D 5e rules.
pub struct RulesEngine;

impl RulesEngine {
    pub fn new() -> Self {
        Self
    }

    /// Resolve an intent and produce effects.
    pub fn resolve(&self, world: &GameWorld, intent: Intent) -> Resolution {
        match intent {
            Intent::Attack {
                attacker_id,
                target_id,
                weapon_name,
                advantage,
            } => self.resolve_attack(world, attacker_id, target_id, &weapon_name, advantage),
            Intent::CastSpell {
                caster_id,
                spell_name,
                targets: _,
                spell_level,
                target_names,
            } => self.resolve_cast_spell(world, caster_id, &spell_name, spell_level, &target_names),
            Intent::SkillCheck {
                character_id,
                skill,
                dc,
                advantage,
                description,
            } => self.resolve_skill_check(world, character_id, skill, dc, advantage, &description),
            Intent::AbilityCheck {
                character_id,
                ability,
                dc,
                advantage,
                description,
            } => self.resolve_ability_check(
                world,
                character_id,
                ability,
                dc,
                advantage,
                &description,
            ),
            Intent::SavingThrow {
                character_id,
                ability,
                dc,
                advantage,
                source,
            } => self.resolve_saving_throw(world, character_id, ability, dc, advantage, &source),
            Intent::Damage {
                target_id,
                amount,
                damage_type,
                source,
            } => self.resolve_damage(world, target_id, amount, damage_type, &source),
            Intent::Heal {
                target_id,
                amount,
                source,
            } => self.resolve_heal(world, target_id, amount, &source),
            Intent::ApplyCondition {
                target_id,
                condition,
                source,
                duration_rounds,
            } => {
                self.resolve_apply_condition(world, target_id, condition, &source, duration_rounds)
            }
            Intent::RemoveCondition {
                target_id,
                condition,
            } => self.resolve_remove_condition(world, target_id, condition),
            Intent::ShortRest => self.resolve_short_rest(world),
            Intent::LongRest => self.resolve_long_rest(world),
            Intent::StartCombat { combatants } => self.resolve_start_combat(world, combatants),
            Intent::EndCombat => self.resolve_end_combat(world),
            Intent::NextTurn => self.resolve_next_turn(world),
            Intent::RollInitiative {
                character_id,
                name,
                modifier,
                is_player,
            } => self.resolve_roll_initiative(character_id, &name, modifier, is_player),
            Intent::RollDice { notation, purpose } => self.resolve_roll_dice(&notation, &purpose),
            Intent::AdvanceTime { minutes } => self.resolve_advance_time(minutes),
            Intent::GainExperience { amount } => self.resolve_gain_experience(world, amount),
            Intent::UseFeature {
                character_id,
                feature_name,
            } => self.resolve_use_feature(world, character_id, &feature_name),
            Intent::RememberFact {
                subject_name,
                subject_type,
                fact,
                category,
                related_entities,
                importance,
            } => self.resolve_remember_fact(
                &subject_name,
                &subject_type,
                &fact,
                &category,
                &related_entities,
                importance,
            ),
            // Inventory intents
            Intent::AddItem {
                item_name,
                quantity,
                item_type,
                description,
                magical,
                weight,
                value_gp,
            } => self.resolve_add_item(
                world,
                &item_name,
                quantity,
                item_type.as_deref(),
                description.as_deref(),
                magical,
                weight,
                value_gp,
            ),
            Intent::RemoveItem {
                item_name,
                quantity,
            } => self.resolve_remove_item(world, &item_name, quantity),
            Intent::EquipItem { item_name } => self.resolve_equip_item(world, &item_name),
            Intent::UnequipItem { slot } => self.resolve_unequip_item(world, &slot),
            Intent::UseItem {
                item_name,
                target_id,
            } => self.resolve_use_item(world, &item_name, target_id),
            Intent::AdjustGold { amount, reason } => {
                self.resolve_adjust_gold(world, amount, &reason)
            }
            Intent::AdjustSilver { amount, reason } => {
                self.resolve_adjust_silver(world, amount, &reason)
            }
            Intent::DeathSave { character_id } => self.resolve_death_save(world, character_id),
            Intent::ConcentrationCheck {
                character_id,
                damage_taken,
                spell_name,
            } => self.resolve_concentration_check(world, character_id, damage_taken, &spell_name),
            Intent::ChangeLocation {
                new_location,
                location_type,
                description,
            } => self.resolve_change_location(world, &new_location, location_type, description),
            Intent::RegisterConsequence {
                trigger_description,
                consequence_description,
                severity,
                related_entities,
                importance,
                expires_in_turns,
            } => self.resolve_register_consequence(
                &trigger_description,
                &consequence_description,
                &severity,
                &related_entities,
                importance,
                expires_in_turns,
            ),

            // Class feature intents
            Intent::UseRage { character_id } => self.resolve_use_rage(world, character_id),
            Intent::EndRage {
                character_id,
                reason,
            } => self.resolve_end_rage(world, character_id, &reason),
            Intent::UseKi {
                character_id,
                points,
                ability,
            } => self.resolve_use_ki(world, character_id, points, &ability),
            Intent::UseLayOnHands {
                character_id,
                target_name,
                hp_amount,
                cure_disease,
                neutralize_poison,
            } => self.resolve_use_lay_on_hands(
                world,
                character_id,
                &target_name,
                hp_amount,
                cure_disease,
                neutralize_poison,
            ),
            Intent::UseDivineSmite {
                character_id,
                spell_slot_level,
                target_is_undead_or_fiend,
            } => self.resolve_use_divine_smite(
                world,
                character_id,
                spell_slot_level,
                target_is_undead_or_fiend,
            ),
            Intent::UseWildShape {
                character_id,
                beast_form,
                beast_hp,
                beast_ac,
            } => self.resolve_use_wild_shape(world, character_id, &beast_form, beast_hp, beast_ac),
            Intent::EndWildShape {
                character_id,
                reason,
                excess_damage,
            } => self.resolve_end_wild_shape(world, character_id, &reason, excess_damage),
            Intent::UseChannelDivinity {
                character_id,
                option,
                targets,
            } => self.resolve_use_channel_divinity(world, character_id, &option, &targets),
            Intent::UseBardicInspiration {
                character_id,
                target_name,
                die_size,
            } => self.resolve_use_bardic_inspiration(world, character_id, &target_name, &die_size),
            Intent::UseActionSurge {
                character_id,
                action_taken,
            } => self.resolve_use_action_surge(world, character_id, &action_taken),
            Intent::UseSecondWind { character_id } => {
                self.resolve_use_second_wind(world, character_id)
            }
            Intent::UseSorceryPoints {
                character_id,
                points,
                metamagic,
                spell_name,
                slot_level,
            } => self.resolve_use_sorcery_points(
                world,
                character_id,
                points,
                &metamagic,
                spell_name.as_deref(),
                slot_level,
            ),

            // Quest management
            Intent::CreateQuest {
                name,
                description,
                giver,
                objectives,
                rewards,
            } => self.resolve_create_quest(
                &name,
                &description,
                giver.as_deref(),
                &objectives,
                &rewards,
            ),
            Intent::AddQuestObjective {
                quest_name,
                objective,
                optional,
            } => self.resolve_add_quest_objective(&quest_name, &objective, optional),
            Intent::CompleteObjective {
                quest_name,
                objective_description,
            } => self.resolve_complete_objective(&quest_name, &objective_description),
            Intent::CompleteQuest {
                quest_name,
                completion_note,
            } => self.resolve_complete_quest(&quest_name, completion_note.as_deref()),
            Intent::FailQuest {
                quest_name,
                failure_reason,
            } => self.resolve_fail_quest(&quest_name, &failure_reason),
            Intent::UpdateQuest {
                quest_name,
                new_description,
                add_rewards,
            } => self.resolve_update_quest(&quest_name, new_description.as_deref(), &add_rewards),

            // World Building intents
            Intent::CreateNpc {
                name,
                description,
                personality,
                occupation,
                disposition,
                location,
                known_information,
            } => self.resolve_create_npc(
                world,
                &name,
                &description,
                &personality,
                occupation.as_deref(),
                &disposition,
                location.as_deref(),
                &known_information,
            ),
            Intent::UpdateNpc {
                npc_name,
                disposition,
                add_information,
                new_description,
                new_personality,
            } => self.resolve_update_npc(
                world,
                &npc_name,
                disposition.as_deref(),
                &add_information,
                new_description.as_deref(),
                new_personality.as_deref(),
            ),
            Intent::MoveNpc {
                npc_name,
                destination,
                reason,
            } => self.resolve_move_npc(world, &npc_name, &destination, reason.as_deref()),
            Intent::RemoveNpc {
                npc_name,
                reason,
                permanent,
            } => self.resolve_remove_npc(&npc_name, &reason, permanent),
            Intent::CreateLocation {
                name,
                location_type,
                description,
                parent_location,
                items,
                npcs_present,
            } => self.resolve_create_location(
                &name,
                &location_type,
                &description,
                parent_location.as_deref(),
                &items,
                &npcs_present,
            ),
            Intent::ConnectLocations {
                from_location,
                to_location,
                direction,
                travel_time_minutes,
                bidirectional,
            } => self.resolve_connect_locations(
                &from_location,
                &to_location,
                direction.as_deref(),
                travel_time_minutes,
                bidirectional,
            ),
            Intent::UpdateLocation {
                location_name,
                new_description,
                add_items,
                remove_items,
                add_npcs,
                remove_npcs,
            } => self.resolve_update_location(
                world,
                &location_name,
                new_description.as_deref(),
                &add_items,
                &remove_items,
                &add_npcs,
                &remove_npcs,
            ),
            Intent::ModifyAbilityScore {
                ability,
                modifier,
                source,
                duration,
            } => self.resolve_modify_ability_score(ability, modifier, &source, duration.as_deref()),
            Intent::RestoreSpellSlot { slot_level, source } => {
                self.resolve_restore_spell_slot(world, slot_level, &source)
            }

            // State assertion
            Intent::AssertState {
                entity_name,
                state_type,
                new_value,
                reason,
                target_entity,
            } => self.resolve_assert_state(
                world,
                &entity_name,
                state_type,
                &new_value,
                &reason,
                target_entity.as_deref(),
            ),

            // Knowledge tracking
            Intent::ShareKnowledge {
                knowing_entity,
                content,
                source,
                verification,
                context,
            } => self.resolve_share_knowledge(
                &knowing_entity,
                &content,
                &source,
                &verification,
                context.as_deref(),
            ),

            // Scheduled events
            Intent::ScheduleEvent {
                description,
                minutes,
                hours,
                day,
                month,
                year,
                hour,
                daily_hour,
                daily_minute,
                location,
                involved_entities: _,
                visibility,
                repeating,
            } => self.resolve_schedule_event(
                world,
                &description,
                minutes,
                hours,
                day,
                month,
                year,
                hour,
                daily_hour,
                daily_minute,
                location.as_deref(),
                &visibility,
                repeating,
            ),

            Intent::CancelEvent {
                event_description,
                reason,
            } => self.resolve_cancel_event(&event_description, &reason),

            #[allow(unreachable_patterns)]
            _ => Resolution::new("Intent not yet implemented"),
        }
    }

    fn resolve_attack(
        &self,
        world: &GameWorld,
        _attacker_id: CharacterId,
        target_id: CharacterId,
        weapon_name: &str,
        advantage: Advantage,
    ) -> Resolution {
        let attacker = &world.player_character;

        // Unconscious characters cannot attack
        if attacker.has_condition(Condition::Unconscious) {
            return Resolution::new(format!(
                "{} is unconscious and cannot attack!",
                attacker.name
            ));
        }

        // Get target AC from combat state, or use player AC if targeting self
        let target_ac = if target_id == world.player_character.id {
            world.player_character.current_ac()
        } else if let Some(ref combat) = world.combat {
            combat
                .combatants
                .iter()
                .find(|c| c.id == target_id)
                .map(|c| c.armor_class)
                .unwrap_or(10) // Fallback for unknown targets
        } else {
            10 // Default AC outside combat
        };

        // Look up weapon from database or equipped weapon
        let weapon = crate::items::get_weapon(weapon_name);
        let equipped_weapon = attacker.equipment.main_hand.as_ref();

        // Determine the weapon properties
        let (damage_dice, is_finesse, is_ranged) = if let Some(w) = &weapon {
            (w.damage_dice.clone(), w.is_finesse(), w.is_ranged())
        } else if let Some(w) = equipped_weapon {
            (w.damage_dice.clone(), w.is_finesse(), w.is_ranged())
        } else {
            // Default to unarmed strike
            ("1".to_string(), false, false)
        };

        // Determine which ability modifier to use
        // Ranged: DEX only
        // Finesse: higher of STR or DEX
        // Melee: STR only
        let str_mod = attacker.ability_scores.modifier(Ability::Strength);
        let dex_mod = attacker.ability_scores.modifier(Ability::Dexterity);

        // Track if this is a strength-based melee attack (for rage bonus)
        let is_strength_melee = if is_ranged {
            false
        } else if is_finesse {
            str_mod >= dex_mod // Using STR for finesse weapon
        } else {
            true
        };

        let ability_mod = if is_ranged {
            dex_mod
        } else if is_finesse {
            str_mod.max(dex_mod)
        } else {
            str_mod
        };

        let attack_mod = ability_mod + attacker.proficiency_bonus();
        let attack_expr = DiceExpression::parse(&format!("1d20+{attack_mod}")).unwrap();
        let attack_roll = attack_expr.roll_with_advantage(advantage);

        let mut resolution = Resolution::new(format!(
            "{} attacks with {} (roll: {} vs AC {})",
            attacker.name, weapon_name, attack_roll.total, target_ac
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: attack_roll.clone(),
            purpose: format!("Attack with {weapon_name}"),
        });

        // Natural 1 always misses, natural 20 always hits (and crits)
        let hits = !attack_roll.is_fumble()
            && (attack_roll.total >= target_ac as i32 || attack_roll.is_critical());

        if hits {
            resolution = resolution.with_effect(Effect::AttackHit {
                attacker_name: attacker.name.clone(),
                target_name: "target".to_string(),
                attack_roll: attack_roll.total,
                target_ac,
                is_critical: attack_roll.is_critical(),
            });

            // Roll damage with ability modifier and rage bonus (if applicable)
            let rage_bonus = if is_strength_melee && attacker.class_resources.rage_active {
                attacker.class_resources.rage_damage_bonus as i32
            } else {
                0
            };
            let total_mod = ability_mod as i32 + rage_bonus;

            let damage_expr = if attack_roll.is_critical() {
                // Critical hit: double the number of dice
                // Parse "XdY" and produce "2XdY"
                let doubled_dice = if let Some(d_pos) = damage_dice.find('d') {
                    let num_dice: i32 = damage_dice[..d_pos].parse().unwrap_or(1);
                    let die_type = &damage_dice[d_pos..];
                    format!("{}{}", num_dice * 2, die_type)
                } else {
                    // Not a dice expression, just double the flat value
                    let flat: i32 = damage_dice.parse().unwrap_or(1);
                    format!("{}", flat * 2)
                };
                format!("{doubled_dice}+{total_mod}")
            } else {
                format!("{damage_dice}+{total_mod}")
            };
            let damage_roll = roll_with_fallback(&damage_expr, "1d4");
            resolution = resolution.with_effect(Effect::DiceRolled {
                roll: damage_roll.clone(),
                purpose: "Damage".to_string(),
            });

            // Check for Sneak Attack (Rogue feature)
            let rogue_level = attacker
                .classes
                .iter()
                .find(|c| c.class == CharacterClass::Rogue)
                .map(|c| c.level)
                .unwrap_or(0);

            if rogue_level > 0 && (is_finesse || is_ranged) {
                // Check if sneak attack conditions are met:
                // 1. Has advantage, OR
                // 2. An ally is engaged with the target (in melee range)
                let has_advantage = matches!(advantage, Advantage::Advantage);

                // Check for ally adjacent to target (any non-player ally in combat)
                let has_ally_adjacent = if let Some(ref combat) = world.combat {
                    combat
                        .combatants
                        .iter()
                        .any(|c| c.is_ally && !c.is_player && c.current_hp > 0 && c.id != target_id)
                } else {
                    false
                };

                // Check if sneak attack hasn't been used this turn
                let sneak_attack_available = if let Some(ref combat) = world.combat {
                    !combat.sneak_attack_used.contains(&attacker.id)
                } else {
                    true // Outside combat, allow it
                };

                if sneak_attack_available && (has_advantage || has_ally_adjacent) {
                    let sneak_dice = sneak_attack_dice(rogue_level);
                    let sneak_expr = if attack_roll.is_critical() {
                        format!("{}d6", sneak_dice * 2) // Double dice on crit
                    } else {
                        format!("{}d6", sneak_dice)
                    };
                    let sneak_roll = roll_with_fallback(&sneak_expr, "1d6");
                    resolution = resolution.with_effect(Effect::DiceRolled {
                        roll: sneak_roll.clone(),
                        purpose: "Sneak Attack".to_string(),
                    });
                    resolution = resolution.with_effect(Effect::SneakAttackUsed {
                        character_id: attacker.id,
                        damage_dice: sneak_dice,
                    });
                }
            }
        } else {
            resolution = resolution.with_effect(Effect::AttackMissed {
                attacker_name: attacker.name.clone(),
                target_name: "target".to_string(),
                attack_roll: attack_roll.total,
                target_ac,
            });
        }

        resolution
    }

    fn resolve_cast_spell(
        &self,
        world: &GameWorld,
        _caster_id: CharacterId,
        spell_name: &str,
        slot_level: u8,
        target_names: &[String],
    ) -> Resolution {
        use crate::spells::{get_spell, SpellAttackType};

        let caster = &world.player_character;

        // Look up the spell
        let spell = match get_spell(spell_name) {
            Some(s) => s,
            None => {
                return Resolution::new(format!(
                    "Unknown spell: '{}'. The spell is not in the database.",
                    spell_name
                ));
            }
        };

        // Determine the effective slot level
        let effective_slot = if spell.level == 0 {
            0 // Cantrips don't use slots
        } else if slot_level == 0 {
            spell.level // Use base spell level if not specified
        } else if slot_level < spell.level {
            return Resolution::new(format!(
                "Cannot cast {} using a level {} slot - requires at least level {}.",
                spell.name, slot_level, spell.level
            ));
        } else {
            slot_level
        };

        // Check and consume spell slot (if not a cantrip)
        if spell.level > 0 {
            if let Some(ref spellcasting) = caster.spellcasting {
                let slot_idx = (effective_slot - 1) as usize;
                if slot_idx >= 9 {
                    return Resolution::new("Invalid spell slot level.");
                }
                let available = spellcasting.spell_slots.slots[slot_idx].available();
                if available == 0 {
                    return Resolution::new(format!(
                        "{} has no level {} spell slots remaining!",
                        caster.name, effective_slot
                    ));
                }
            } else {
                return Resolution::new(format!(
                    "{} doesn't have spellcasting ability!",
                    caster.name
                ));
            }
        }

        // Get spellcasting ability modifier
        let spell_mod = caster
            .spellcasting
            .as_ref()
            .map(|sc| caster.ability_scores.modifier(sc.ability))
            .unwrap_or(0);
        let spell_attack_bonus = spell_mod + caster.proficiency_bonus();
        // Minimum DC of 8 as a sanity floor (though in practice, no valid build would go lower)
        let spell_save_dc = (8 + spell_mod + caster.proficiency_bonus()).max(8);

        // Build the resolution
        let mut resolution = Resolution::new(String::new());
        let mut narrative_parts = Vec::new();

        // Casting announcement
        let slot_text = if spell.level == 0 {
            String::new()
        } else if effective_slot > spell.level {
            format!(" (upcast at level {})", effective_slot)
        } else {
            format!(" (level {} slot)", effective_slot)
        };
        narrative_parts.push(format!(
            "{} casts {}{}!",
            caster.name, spell.name, slot_text
        ));

        // Handle concentration
        if spell.concentration {
            narrative_parts.push("(Concentration)".to_string());
        }

        // Determine damage dice (accounting for cantrip scaling and upcasting)
        let caster_level = caster.level;
        let damage_dice = spell.effective_damage_dice(caster_level, effective_slot);

        // Handle spell attack (if applicable)
        if let Some(attack_type) = &spell.attack_type {
            let attack_type_name = match attack_type {
                SpellAttackType::Melee => "melee",
                SpellAttackType::Ranged => "ranged",
            };

            // Roll spell attack
            let attack_roll = roll_with_fallback(&format!("1d20+{}", spell_attack_bonus), "1d20");

            resolution = resolution.with_effect(Effect::DiceRolled {
                roll: attack_roll.clone(),
                purpose: format!("{} spell attack", attack_type_name),
            });

            let target_name = target_names.first().map(|s| s.as_str()).unwrap_or("target");

            // Look up target AC from combat state by name
            let target_ac = if let Some(ref combat) = world.combat {
                combat
                    .combatants
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(target_name))
                    .map(|c| c.armor_class)
                    .unwrap_or(10)
            } else {
                10 // Default AC outside combat
            };

            narrative_parts.push(format!(
                "Makes a {} spell attack against {}: {} vs AC {}.",
                attack_type_name, target_name, attack_roll.total, target_ac
            ));

            let hits = !attack_roll.is_fumble()
                && (attack_roll.total >= target_ac as i32 || attack_roll.is_critical());

            if hits {
                narrative_parts.push("Hit!".to_string());
                resolution = resolution.with_effect(Effect::AttackHit {
                    attacker_name: caster.name.clone(),
                    target_name: target_name.to_string(),
                    attack_roll: attack_roll.total,
                    target_ac,
                    is_critical: attack_roll.is_critical(),
                });

                // Roll damage
                if let Some(ref dice_str) = damage_dice {
                    let damage_formula = if attack_roll.is_critical() {
                        // Double dice on crit
                        if let Some(d_pos) = dice_str.find('d') {
                            let num: i32 = dice_str[..d_pos].parse().unwrap_or(1);
                            format!("{}d{}", num * 2, &dice_str[d_pos + 1..])
                        } else {
                            dice_str.clone()
                        }
                    } else {
                        dice_str.clone()
                    };

                    if let Ok(damage_roll) = dice::roll(&damage_formula) {
                        let damage_type_name =
                            spell.damage_type.map(|dt| dt.name()).unwrap_or("magical");

                        narrative_parts.push(format!(
                            "Deals {} {} damage.",
                            damage_roll.total, damage_type_name
                        ));

                        resolution = resolution.with_effect(Effect::DiceRolled {
                            roll: damage_roll,
                            purpose: format!("{} damage", spell.name),
                        });
                    }
                }
            } else {
                narrative_parts.push("Miss!".to_string());
                resolution = resolution.with_effect(Effect::AttackMissed {
                    attacker_name: caster.name.clone(),
                    target_name: target_name.to_string(),
                    attack_roll: attack_roll.total,
                    target_ac,
                });
            }
        }
        // Handle saving throw spells
        else if let Some(save_ability) = spell.save_type {
            let save_effect = spell.save_effect.as_deref().unwrap_or("negates effect");

            narrative_parts.push(format!(
                "Targets must make a DC {} {} saving throw ({} on success).",
                spell_save_dc,
                save_ability.name(),
                save_effect
            ));

            // Roll damage (before save resolution)
            if let Some(ref dice_str) = damage_dice {
                if let Ok(damage_roll) = dice::roll(dice_str) {
                    let damage_type_name =
                        spell.damage_type.map(|dt| dt.name()).unwrap_or("magical");

                    narrative_parts.push(format!(
                        "On a failed save: {} {} damage.",
                        damage_roll.total, damage_type_name
                    ));

                    resolution = resolution.with_effect(Effect::DiceRolled {
                        roll: damage_roll,
                        purpose: format!("{} damage", spell.name),
                    });
                }
            }
        }
        // Handle healing spells
        else if let Some(ref healing_dice) = spell.healing_dice {
            let healing_formula = format!("{}+{}", healing_dice, spell_mod);
            if let Ok(healing_roll) = dice::roll(&healing_formula) {
                let target_name = target_names.first().map(|s| s.as_str()).unwrap_or("target");
                narrative_parts.push(format!(
                    "{} heals {} for {} HP.",
                    caster.name, target_name, healing_roll.total
                ));

                resolution = resolution.with_effect(Effect::DiceRolled {
                    roll: healing_roll,
                    purpose: format!("{} healing", spell.name),
                });
            }
        }
        // Utility spells (no attack/save/healing)
        else {
            narrative_parts.push(spell.description.clone());
        }

        // Add spell slot consumption effect (for leveled spells)
        if spell.level > 0 {
            resolution = resolution.with_effect(Effect::SpellSlotUsed {
                level: effective_slot,
                remaining: 0, // Will be calculated by effect application
            });
        }

        resolution.narrative = narrative_parts.join(" ");
        resolution
    }

    fn resolve_skill_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        skill: Skill,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity checks
        if character.has_condition(Condition::Unconscious) {
            let ability = skill.ability();
            if matches!(ability, Ability::Strength | Ability::Dexterity) {
                return Resolution::new(format!(
                    "{} is unconscious and automatically fails the {} check!",
                    character.name,
                    skill.name()
                ))
                .with_effect(Effect::CheckFailed {
                    check_type: skill.name().to_string(),
                    roll: 0,
                    dc,
                });
            }
        }

        let modifier = character.skill_modifier(skill);

        // Check for armor-imposed stealth disadvantage
        let effective_advantage = if skill == Skill::Stealth {
            if let Some(ref armor) = character.equipment.armor {
                if armor.stealth_disadvantage {
                    // Armor imposes disadvantage on Stealth
                    advantage.combine(Advantage::Disadvantage)
                } else {
                    advantage
                }
            } else {
                advantage
            }
        } else {
            advantage
        };

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(effective_advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        // Note if stealth disadvantage was applied
        let disadvantage_note = if skill == Skill::Stealth
            && effective_advantage != advantage
            && matches!(effective_advantage, Advantage::Disadvantage)
        {
            " [armor disadvantage]"
        } else {
            ""
        };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {}){}",
            character.name,
            result_str,
            skill.name(),
            roll.total,
            dc,
            disadvantage_note
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", skill.name(), description),
        });

        if success {
            resolution = resolution.with_effect(Effect::CheckSucceeded {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        } else {
            resolution = resolution.with_effect(Effect::CheckFailed {
                check_type: skill.name().to_string(),
                roll: roll.total,
                dc,
            });
        }

        resolution
    }

    fn resolve_ability_check(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        description: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity checks
        if character.has_condition(Condition::Unconscious)
            && matches!(ability, Ability::Strength | Ability::Dexterity)
        {
            return Resolution::new(format!(
                "{} is unconscious and automatically fails the {} check!",
                character.name,
                ability.abbreviation()
            ))
            .with_effect(Effect::CheckFailed {
                check_type: format!("{} check", ability.abbreviation()),
                roll: 0,
                dc,
            });
        }

        let modifier = character.ability_scores.modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} ({} check: {} vs DC {})",
            character.name,
            result_str,
            ability.abbreviation(),
            roll.total,
            dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} check - {}", ability.abbreviation(), description),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: ability.abbreviation().to_string(),
                roll: roll.total,
                dc,
            })
        }
    }

    fn resolve_saving_throw(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: Advantage,
        source: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters automatically fail Strength and Dexterity saving throws
        if character.has_condition(Condition::Unconscious)
            && matches!(ability, Ability::Strength | Ability::Dexterity)
        {
            return Resolution::new(format!(
                "{} is unconscious and automatically fails the {} saving throw!",
                character.name,
                ability.abbreviation()
            ))
            .with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: 0,
                dc,
            });
        }

        let modifier = character.saving_throw_modifier(ability);

        let expr = DiceExpression::parse(&format!("1d20+{modifier}")).unwrap();
        let roll = expr.roll_with_advantage(advantage);

        let success = roll.total >= dc;
        let result_str = if success { "succeeds" } else { "fails" };

        let mut resolution = Resolution::new(format!(
            "{} {} on {} saving throw ({} vs DC {})",
            character.name,
            result_str,
            ability.abbreviation(),
            roll.total,
            dc
        ));

        resolution = resolution.with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: format!("{} save vs {}", ability.abbreviation(), source),
        });

        if success {
            resolution.with_effect(Effect::CheckSucceeded {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        } else {
            resolution.with_effect(Effect::CheckFailed {
                check_type: format!("{} save", ability.abbreviation()),
                roll: roll.total,
                dc,
            })
        }
    }

    fn resolve_damage(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        amount: i32,
        damage_type: DamageType,
        source: &str,
    ) -> Resolution {
        let target = &world.player_character;

        // Special handling for damage while already at 0 HP
        if target.hit_points.current <= 0 {
            // Massive damage while at 0 HP = instant death
            if amount >= target.hit_points.maximum {
                return Resolution::new(format!(
                    "{} takes {} {} damage from {} while unconscious - INSTANT DEATH! (Damage {} >= max HP {})",
                    target.name, amount, damage_type.name(), source, amount, target.hit_points.maximum
                ))
                .with_effect(Effect::CharacterDied {
                    target_id,
                    cause: format!("Massive damage while unconscious from {source}"),
                });
            }

            // Damage while at 0 HP causes death save failures
            // (Critical hits cause 2 failures, but we don't know if this was a crit here)
            let new_failures = target.death_saves.failures + 1;
            let died = new_failures >= 3;

            if died {
                return Resolution::new(format!(
                    "{} takes {} {} damage from {} while unconscious - death save failure! Total failures: 3 - {} DIES!",
                    target.name, amount, damage_type.name(), source, target.name
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id,
                    failures: 1,
                    total_failures: new_failures,
                    source: source.to_string(),
                })
                .with_effect(Effect::CharacterDied {
                    target_id,
                    cause: "Failed 3 death saving throws".to_string(),
                });
            }

            return Resolution::new(format!(
                "{} takes {} {} damage from {} while unconscious - death save failure! (Failures: {}/3)",
                target.name, amount, damage_type.name(), source, new_failures
            ))
            .with_effect(Effect::DeathSaveFailure {
                target_id,
                failures: 1,
                total_failures: new_failures,
                source: source.to_string(),
            });
        }

        let mut hp = target.hit_points.clone();
        let result = hp.take_damage(amount);

        // Check for massive damage (instant death)
        // If damage reduces you to 0 HP AND remaining damage >= max HP, instant death
        let overflow_damage = if result.dropped_to_zero {
            amount - (target.hit_points.current + target.hit_points.temporary)
        } else {
            0
        };
        let instant_death = result.dropped_to_zero && overflow_damage >= hp.maximum;

        // Build narrative with HP status so DM knows the character's state
        let hp_status = if instant_death {
            format!(
                " (INSTANT DEATH! Massive damage ({} overflow) exceeds max HP of {})",
                overflow_damage, hp.maximum
            )
        } else if result.dropped_to_zero {
            format!(
                " (HP: 0/{} - UNCONSCIOUS! Character falls and begins making death saving throws)",
                hp.maximum
            )
        } else if hp.current <= hp.maximum / 4 {
            format!(" (HP: {}/{} - critically wounded)", hp.current, hp.maximum)
        } else if hp.current <= hp.maximum / 2 {
            format!(" (HP: {}/{} - bloodied)", hp.current, hp.maximum)
        } else {
            format!(" (HP: {}/{})", hp.current, hp.maximum)
        };

        let mut resolution = Resolution::new(format!(
            "{} takes {} {} damage from {}{}",
            target.name,
            amount,
            damage_type.name(),
            source,
            hp_status
        ));

        resolution = resolution.with_effect(Effect::HpChanged {
            target_id,
            amount: -amount,
            new_current: hp.current,
            new_max: hp.maximum,
            dropped_to_zero: result.dropped_to_zero,
        });

        if instant_death {
            resolution = resolution.with_effect(Effect::CharacterDied {
                target_id,
                cause: format!("Massive damage from {source}"),
            });
        }

        resolution
    }

    fn resolve_heal(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        amount: i32,
        source: &str,
    ) -> Resolution {
        let target = &world.player_character;
        let mut hp = target.hit_points.clone();
        let was_unconscious = hp.current <= 0;
        let healed = hp.heal(amount);

        // Build narrative with HP status
        let hp_status = if was_unconscious && hp.current > 0 {
            format!(
                " (HP: {}/{} - regains consciousness!)",
                hp.current, hp.maximum
            )
        } else if hp.current == hp.maximum {
            format!(" (HP: {}/{} - fully healed)", hp.current, hp.maximum)
        } else {
            format!(" (HP: {}/{})", hp.current, hp.maximum)
        };

        let resolution = Resolution::new(format!(
            "{} heals {} hit points from {}{}",
            target.name, healed, source, hp_status
        ));

        resolution.with_effect(Effect::HpChanged {
            target_id,
            amount: healed,
            new_current: hp.current,
            new_max: hp.maximum,
            dropped_to_zero: false,
        })
    }

    fn resolve_apply_condition(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        condition: Condition,
        source: &str,
        duration_rounds: Option<u32>,
    ) -> Resolution {
        let target = &world.player_character;

        let duration_text = duration_rounds
            .map(|d| format!(" for {} rounds", d))
            .unwrap_or_default();

        let resolution = Resolution::new(format!(
            "{} is now {} ({}){}",
            target.name,
            condition.name(),
            source,
            duration_text
        ));

        resolution.with_effect(Effect::ConditionApplied {
            target_id,
            condition,
            source: source.to_string(),
            duration_rounds,
        })
    }

    fn resolve_remove_condition(
        &self,
        world: &GameWorld,
        target_id: CharacterId,
        condition: Condition,
    ) -> Resolution {
        let target = &world.player_character;

        let resolution =
            Resolution::new(format!("{} is no longer {}", target.name, condition.name()));

        resolution.with_effect(Effect::ConditionRemoved {
            target_id,
            condition,
        })
    }

    fn resolve_short_rest(&self, world: &GameWorld) -> Resolution {
        // Can't rest during combat
        if world.combat.is_some() {
            return Resolution::new("Cannot take a short rest while in combat!");
        }

        Resolution::new("The party takes a short rest, spending 1 hour resting.")
            .with_effect(Effect::TimeAdvanced { minutes: 60 })
            .with_effect(Effect::RestCompleted {
                rest_type: RestType::Short,
            })
    }

    fn resolve_long_rest(&self, world: &GameWorld) -> Resolution {
        // Can't rest during combat
        if world.combat.is_some() {
            return Resolution::new("Cannot take a long rest while in combat!");
        }

        Resolution::new("The party takes a long rest, spending 8 hours resting.")
            .with_effect(Effect::TimeAdvanced { minutes: 480 })
            .with_effect(Effect::RestCompleted {
                rest_type: RestType::Long,
            })
    }

    fn resolve_start_combat(
        &self,
        world: &GameWorld,
        combatants: Vec<CombatantInit>,
    ) -> Resolution {
        let mut resolution = Resolution::new("Combat begins! Roll for initiative.")
            .with_effect(Effect::CombatStarted);

        // Roll initiative for each combatant
        for init in combatants {
            let modifier = if init.is_player {
                world.player_character.initiative_modifier()
            } else {
                init.initiative_modifier
            };

            let roll = dice::roll("1d20").unwrap();
            let total = roll.total + modifier as i32;

            resolution = resolution.with_effect(Effect::InitiativeRolled {
                character_id: init.id,
                name: init.name.clone(),
                roll: roll.total,
                total,
            });

            resolution = resolution.with_effect(Effect::CombatantAdded {
                id: init.id,
                name: init.name,
                initiative: total,
                is_ally: init.is_ally,
                current_hp: init.current_hp,
                max_hp: init.max_hp,
                armor_class: init.armor_class,
            });
        }

        resolution
    }

    fn resolve_end_combat(&self, _world: &GameWorld) -> Resolution {
        Resolution::new("Combat ends.").with_effect(Effect::CombatEnded)
    }

    fn resolve_next_turn(&self, world: &GameWorld) -> Resolution {
        if let Some(ref combat) = world.combat {
            let mut combat_clone = combat.clone();
            combat_clone.next_turn();

            let current = combat_clone
                .current_combatant()
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            Resolution::new(format!(
                "Next turn: {} (Round {})",
                current, combat_clone.round
            ))
            .with_effect(Effect::TurnAdvanced {
                round: combat_clone.round,
                current_combatant: current,
            })
        } else {
            Resolution::new("No combat in progress")
        }
    }

    fn resolve_roll_initiative(
        &self,
        character_id: CharacterId,
        name: &str,
        modifier: i8,
        _is_player: bool,
    ) -> Resolution {
        let roll = dice::roll("1d20").unwrap();
        let total = roll.total + modifier as i32;

        Resolution::new(format!(
            "{} rolls initiative: {} + {} = {}",
            name, roll.total, modifier, total
        ))
        .with_effect(Effect::DiceRolled {
            roll: roll.clone(),
            purpose: "Initiative".to_string(),
        })
        .with_effect(Effect::InitiativeRolled {
            character_id,
            name: name.to_string(),
            roll: roll.total,
            total,
        })
    }

    fn resolve_roll_dice(&self, notation: &str, purpose: &str) -> Resolution {
        match dice::roll(notation) {
            Ok(roll) => Resolution::new(format!("Rolling {notation} for {purpose}: {roll}"))
                .with_effect(Effect::DiceRolled {
                    roll,
                    purpose: purpose.to_string(),
                }),
            Err(e) => Resolution::new(format!("Failed to roll {notation}: {e}")),
        }
    }

    fn resolve_advance_time(&self, minutes: u32) -> Resolution {
        let hours = minutes / 60;
        let mins = minutes % 60;

        let time_str = if hours > 0 && mins > 0 {
            format!("{hours} hours and {mins} minutes")
        } else if hours > 0 {
            format!("{hours} hours")
        } else {
            format!("{mins} minutes")
        };

        Resolution::new(format!("{time_str} pass.")).with_effect(Effect::TimeAdvanced { minutes })
    }

    fn resolve_gain_experience(&self, world: &GameWorld, amount: u32) -> Resolution {
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

    fn resolve_use_feature(
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

    fn resolve_remember_fact(
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

    // Inventory resolution methods

    #[allow(clippy::too_many_arguments)]
    fn resolve_add_item(
        &self,
        world: &GameWorld,
        item_name: &str,
        quantity: u32,
        _item_type: Option<&str>,
        _description: Option<&str>,
        _magical: bool,
        _weight: Option<f32>,
        _value_gp: Option<f32>,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if item already exists
        let existing_qty = character
            .inventory
            .find_item(item_name)
            .map(|i| i.quantity)
            .unwrap_or(0);
        let new_total = existing_qty + quantity;

        // Note: item_type, description, magical, weight, value_gp are passed through
        // but the actual item creation happens in apply_effect or could be enhanced
        // to look up standard items from the items database.

        let qty_str = if quantity > 1 {
            format!("{quantity} x ")
        } else {
            String::new()
        };

        Resolution::new(format!(
            "{} receives {}{} (now has {} total)",
            character.name, qty_str, item_name, new_total
        ))
        .with_effect(Effect::ItemAdded {
            item_name: item_name.to_string(),
            quantity,
            new_total,
        })
    }

    fn resolve_remove_item(&self, world: &GameWorld, item_name: &str, quantity: u32) -> Resolution {
        let character = &world.player_character;

        if let Some(item) = character.inventory.find_item(item_name) {
            if item.quantity >= quantity {
                let remaining = item.quantity - quantity;
                let qty_str = if quantity > 1 {
                    format!("{quantity} x ")
                } else {
                    String::new()
                };
                Resolution::new(format!(
                    "{} loses {}{} ({} remaining)",
                    character.name, qty_str, item_name, remaining
                ))
                .with_effect(Effect::ItemRemoved {
                    item_name: item_name.to_string(),
                    quantity,
                    remaining,
                })
            } else {
                Resolution::new(format!(
                    "{} doesn't have enough {} (has {}, needs {})",
                    character.name, item_name, item.quantity, quantity
                ))
            }
        } else {
            Resolution::new(format!("{} doesn't have any {}", character.name, item_name))
        }
    }

    fn resolve_equip_item(&self, world: &GameWorld, item_name: &str) -> Resolution {
        let character = &world.player_character;

        if let Some(item) = character.inventory.find_item(item_name) {
            let slot = match item.item_type {
                ItemType::Weapon => "main_hand",
                ItemType::Armor => "armor",
                ItemType::Shield => "shield",
                _ => {
                    return Resolution::new(format!(
                        "{item_name} cannot be equipped (not a weapon, armor, or shield)"
                    ));
                }
            };

            // Check for two-handed weapon + shield conflict
            if slot == "shield" {
                if let Some(ref weapon) = character.equipment.main_hand {
                    if weapon.is_two_handed() {
                        return Resolution::new(format!(
                            "Cannot equip {} - {} requires two hands",
                            item_name, weapon.base.name
                        ));
                    }
                }
            }

            // Check for shield + two-handed weapon conflict
            if slot == "main_hand" {
                if let Some(db_weapon) = crate::items::get_weapon(item_name) {
                    if db_weapon.is_two_handed() && character.equipment.shield.is_some() {
                        return Resolution::new(format!(
                            "Cannot equip {item_name} - it requires two hands but a shield is equipped. Unequip the shield first."
                        ));
                    }
                }
            }

            // Check strength requirement for heavy armor
            if slot == "armor" {
                if let Some(db_armor) = crate::items::get_armor(item_name) {
                    if let Some(str_req) = db_armor.strength_requirement {
                        let char_str = character.ability_scores.strength;
                        if char_str < str_req {
                            return Resolution::new(format!(
                                "{} equips {} but doesn't meet the Strength {} requirement (has {}). Movement speed reduced by 10 feet.",
                                character.name, item_name, str_req, char_str
                            ))
                            .with_effect(Effect::ItemEquipped {
                                item_name: item_name.to_string(),
                                slot: slot.to_string(),
                            });
                        }
                    }
                }
            }

            Resolution::new(format!(
                "{} equips {} in {} slot",
                character.name, item_name, slot
            ))
            .with_effect(Effect::ItemEquipped {
                item_name: item_name.to_string(),
                slot: slot.to_string(),
            })
        } else {
            Resolution::new(format!(
                "{} doesn't have {} in their inventory",
                character.name, item_name
            ))
        }
    }

    fn resolve_unequip_item(&self, world: &GameWorld, slot: &str) -> Resolution {
        let character = &world.player_character;

        let item_name = match slot.to_lowercase().as_str() {
            "armor" => character
                .equipment
                .armor
                .as_ref()
                .map(|a| a.base.name.clone()),
            "shield" => character.equipment.shield.as_ref().map(|s| s.name.clone()),
            "main_hand" | "weapon" => character
                .equipment
                .main_hand
                .as_ref()
                .map(|w| w.base.name.clone()),
            "off_hand" => character
                .equipment
                .off_hand
                .as_ref()
                .map(|i| i.name.clone()),
            _ => {
                return Resolution::new(format!(
                    "Unknown equipment slot: {slot}. Valid slots: armor, shield, main_hand, off_hand"
                ));
            }
        };

        if let Some(name) = item_name {
            Resolution::new(format!("{} unequips {}", character.name, name)).with_effect(
                Effect::ItemUnequipped {
                    item_name: name,
                    slot: slot.to_string(),
                },
            )
        } else {
            Resolution::new(format!("Nothing equipped in {slot} slot"))
        }
    }

    fn resolve_use_item(
        &self,
        world: &GameWorld,
        item_name: &str,
        _target_id: Option<CharacterId>,
    ) -> Resolution {
        let character = &world.player_character;

        // Unconscious characters cannot use items themselves
        if character.has_condition(Condition::Unconscious) {
            return Resolution::new(format!(
                "{} is unconscious and cannot use items!",
                character.name
            ));
        }

        if let Some(item) = character.inventory.find_item(item_name) {
            // Check if it's a consumable type
            match item.item_type {
                ItemType::Potion => {
                    // Look up proper healing amount from database, fall back to basic potion
                    let (dice_expr, bonus) =
                        if let Some(potion) = crate::items::get_potion(item_name) {
                            match potion.effect {
                                crate::world::ConsumableEffect::Healing { ref dice, bonus } => {
                                    (dice.clone(), bonus)
                                }
                                _ => ("2d4".to_string(), 2),
                            }
                        } else {
                            ("2d4".to_string(), 2) // Default healing potion
                        };

                    let heal_expr = if bonus != 0 {
                        format!("{dice_expr}+{bonus}")
                    } else {
                        dice_expr
                    };
                    let heal_roll = roll_with_fallback(&heal_expr, "1d4");

                    Resolution::new(format!(
                        "{} drinks {} and heals for {} HP",
                        character.name, item_name, heal_roll.total
                    ))
                    .with_effect(Effect::ItemUsed {
                        item_name: item_name.to_string(),
                        result: format!("Healed {} HP", heal_roll.total),
                    })
                    .with_effect(Effect::HpChanged {
                        target_id: character.id,
                        amount: heal_roll.total,
                        new_current: (character.hit_points.current + heal_roll.total)
                            .min(character.hit_points.maximum),
                        new_max: character.hit_points.maximum,
                        dropped_to_zero: false,
                    })
                    .with_effect(Effect::ItemRemoved {
                        item_name: item_name.to_string(),
                        quantity: 1,
                        remaining: item.quantity.saturating_sub(1),
                    })
                }
                ItemType::Scroll => Resolution::new(format!(
                    "{} reads {} and it crumbles to dust",
                    character.name, item_name
                ))
                .with_effect(Effect::ItemUsed {
                    item_name: item_name.to_string(),
                    result: "Scroll consumed".to_string(),
                })
                .with_effect(Effect::ItemRemoved {
                    item_name: item_name.to_string(),
                    quantity: 1,
                    remaining: item.quantity.saturating_sub(1),
                }),
                _ => Resolution::new(format!("{item_name} is not a consumable item")),
            }
        } else {
            Resolution::new(format!(
                "{} doesn't have {} in their inventory",
                character.name, item_name
            ))
        }
    }

    fn resolve_adjust_gold(&self, world: &GameWorld, amount: i32, reason: &str) -> Resolution {
        let character = &world.player_character;
        let new_total = character.inventory.gold + amount;

        if new_total < 0 {
            Resolution::new(format!(
                "{} doesn't have enough gold (has {} gp, needs {} gp)",
                character.name, character.inventory.gold, -amount
            ))
        } else {
            let action = if amount >= 0 { "gains" } else { "spends" };
            Resolution::new(format!(
                "{} {} {} gp {} (now has {} gp)",
                character.name,
                action,
                amount.abs(),
                reason,
                new_total
            ))
            .with_effect(Effect::GoldChanged {
                amount,
                new_total,
                reason: reason.to_string(),
            })
        }
    }

    fn resolve_adjust_silver(&self, world: &GameWorld, amount: i32, reason: &str) -> Resolution {
        let character = &world.player_character;
        let new_total = character.inventory.silver + amount;

        if new_total < 0 {
            Resolution::new(format!(
                "{} doesn't have enough silver (has {} sp, needs {} sp)",
                character.name, character.inventory.silver, -amount
            ))
        } else {
            let action = if amount >= 0 { "gains" } else { "spends" };
            Resolution::new(format!(
                "{} {} {} sp {} (now has {} sp)",
                character.name,
                action,
                amount.abs(),
                reason,
                new_total
            ))
            .with_effect(Effect::SilverChanged {
                amount,
                new_total,
                reason: reason.to_string(),
            })
        }
    }

    /// Resolve a death saving throw (D&D 5e rules)
    /// - Roll d20 (no modifiers by default)
    /// - 10+ = success, <10 = failure
    /// - Natural 20 = regain 1 HP and become conscious
    /// - Natural 1 = counts as 2 failures
    /// - 3 successes = stable
    /// - 3 failures = death
    fn resolve_death_save(&self, world: &GameWorld, character_id: CharacterId) -> Resolution {
        let character = &world.player_character;

        // Must be at 0 HP to make death saves
        if character.hit_points.current > 0 {
            return Resolution::new(format!(
                "{} is not dying and doesn't need to make a death save.",
                character.name
            ));
        }

        // Roll d20
        let roll = dice::roll("1d20").unwrap();
        let roll_value = roll.total;

        // Check for natural 20 - regain 1 HP
        if roll.is_critical() {
            return Resolution::new(format!(
                "{} rolls a NATURAL 20 on their death save! They regain 1 HP and become conscious!",
                character.name
            ))
            .with_effect(Effect::DeathSavesReset {
                target_id: character_id,
            })
            .with_effect(Effect::HpChanged {
                target_id: character_id,
                amount: 1,
                new_current: 1,
                new_max: character.hit_points.maximum,
                dropped_to_zero: false,
            })
            .with_effect(Effect::ConditionRemoved {
                target_id: character_id,
                condition: Condition::Unconscious,
            });
        }

        // Check for natural 1 - counts as 2 failures
        if roll.is_fumble() {
            let new_failures = character.death_saves.failures + 2;
            if new_failures >= 3 {
                return Resolution::new(format!(
                    "{} rolls a NATURAL 1 on their death save! Two failures! {} has died!",
                    character.name, character.name
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 2,
                    total_failures: new_failures.min(3),
                    source: "Natural 1 on death save".to_string(),
                })
                .with_effect(Effect::CharacterDied {
                    target_id: character_id,
                    cause: "Failed death saves".to_string(),
                });
            } else {
                return Resolution::new(format!(
                    "{} rolls a NATURAL 1 on their death save! That counts as TWO failures! ({}/3)",
                    character.name, new_failures
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 2,
                    total_failures: new_failures,
                    source: "Natural 1 on death save".to_string(),
                });
            }
        }

        // Normal roll - 10+ is success, <10 is failure
        if roll_value >= 10 {
            let new_successes = character.death_saves.successes + 1;
            if new_successes >= 3 {
                Resolution::new(format!(
                    "{} rolls {} on their death save - SUCCESS! With 3 successes, {} is now STABLE!",
                    character.name, roll_value, character.name
                ))
                .with_effect(Effect::DeathSaveSuccess {
                    target_id: character_id,
                    roll: roll_value,
                    total_successes: 3,
                })
                .with_effect(Effect::Stabilized { target_id: character_id })
            } else {
                Resolution::new(format!(
                    "{} rolls {} on their death save - SUCCESS! ({}/3 successes)",
                    character.name, roll_value, new_successes
                ))
                .with_effect(Effect::DeathSaveSuccess {
                    target_id: character_id,
                    roll: roll_value,
                    total_successes: new_successes,
                })
            }
        } else {
            let new_failures = character.death_saves.failures + 1;
            if new_failures >= 3 {
                Resolution::new(format!(
                    "{} rolls {} on their death save - FAILURE! With 3 failures, {} has DIED!",
                    character.name, roll_value, character.name
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 1,
                    total_failures: 3,
                    source: "Death save".to_string(),
                })
                .with_effect(Effect::CharacterDied {
                    target_id: character_id,
                    cause: "Failed death saves".to_string(),
                })
            } else {
                Resolution::new(format!(
                    "{} rolls {} on their death save - FAILURE! ({}/3 failures)",
                    character.name, roll_value, new_failures
                ))
                .with_effect(Effect::DeathSaveFailure {
                    target_id: character_id,
                    failures: 1,
                    total_failures: new_failures,
                    source: "Death save".to_string(),
                })
            }
        }
    }

    /// Resolve a concentration check (D&D 5e rules)
    /// - CON saving throw
    /// - DC = max(10, damage / 2)
    fn resolve_concentration_check(
        &self,
        world: &GameWorld,
        character_id: CharacterId,
        damage_taken: i32,
        spell_name: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Calculate DC: max(10, damage / 2)
        let dc = (damage_taken / 2).max(10);

        // Get CON modifier
        let con_mod = character.ability_scores.modifier(Ability::Constitution);
        let proficiency = character.proficiency_bonus();

        // Check if proficient in CON saves (some classes like Sorcerer, Wizard with War Caster)
        // For now, assume base CON save without proficiency unless they have the save proficiency
        let save_mod = if character
            .saving_throw_proficiencies
            .contains(&Ability::Constitution)
        {
            con_mod + proficiency
        } else {
            con_mod
        };

        // Roll the save
        let roll = dice::roll(&format!("1d20+{save_mod}")).unwrap();
        let roll_total = roll.total;

        if roll_total >= dc {
            Resolution::new(format!(
                "{} makes a DC {} Constitution save to maintain concentration on {}. Rolls {} - SUCCESS! Concentration maintained.",
                character.name, dc, spell_name, roll_total
            ))
            .with_effect(Effect::ConcentrationMaintained {
                character_id,
                spell_name: spell_name.to_string(),
                roll: roll_total,
                dc,
            })
        } else {
            Resolution::new(format!(
                "{} makes a DC {} Constitution save to maintain concentration on {}. Rolls {} - FAILED! Concentration is broken!",
                character.name, dc, spell_name, roll_total
            ))
            .with_effect(Effect::ConcentrationBroken {
                character_id,
                spell_name: spell_name.to_string(),
                damage_taken,
                roll: roll_total,
                dc,
            })
        }
    }

    fn resolve_change_location(
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

    fn resolve_register_consequence(
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

    // ========================================================================
    // Class Feature Resolution Functions
    // ========================================================================

    fn resolve_use_rage(&self, world: &GameWorld, _character_id: CharacterId) -> Resolution {
        let character = &world.player_character;

        // Check if already raging
        if world.player_character.class_resources.rage_active {
            return Resolution::new(format!("{} is already raging!", character.name));
        }

        // Check for rage uses remaining
        let rage_feature = character.features.iter().find(|f| f.name == "Rage");
        if let Some(feature) = rage_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no rage uses remaining! (Recovers on long rest)",
                        character.name
                    ));
                }
            }
        }

        // Determine rage damage bonus based on level
        let barbarian_level = character
            .classes
            .iter()
            .find(|c| c.class == crate::world::CharacterClass::Barbarian)
            .map(|c| c.level)
            .unwrap_or(1);

        let rage_damage = match barbarian_level {
            1..=8 => 2,
            9..=15 => 3,
            _ => 4,
        };

        Resolution::new(format!(
            "{} enters a RAGE! Gains: advantage on STR checks/saves, +{} rage damage to melee attacks, resistance to bludgeoning/piercing/slashing damage. Cannot cast spells or concentrate while raging.",
            character.name, rage_damage
        ))
        .with_effect(Effect::RageStarted {
            character_id: world.player_character.id,
            damage_bonus: rage_damage,
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Rage".to_string(),
            description: format!("Entered rage (1 minute, +{rage_damage} damage)"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Rage".to_string(),
            uses_remaining: 0,
        })
    }

    fn resolve_end_rage(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        reason: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if !world.player_character.class_resources.rage_active {
            return Resolution::new(format!("{} is not currently raging.", character.name));
        }

        let reason_text = match reason {
            "duration_expired" => "Rage ended (1 minute duration expired).",
            "unconscious" => "Rage ended (knocked unconscious).",
            "no_combat_action" => "Rage ended (turn ended without attacking or taking damage).",
            "voluntary" => "Rage ended voluntarily.",
            _ => "Rage ended.",
        };

        Resolution::new(format!("{}'s rage ends. {}", character.name, reason_text))
            .with_effect(Effect::RageEnded {
                character_id: world.player_character.id,
                reason: reason_text.to_string(),
            })
            .with_effect(Effect::ClassResourceUsed {
                character_name: character.name.clone(),
                resource_name: "Rage".to_string(),
                description: reason_text.to_string(),
            })
    }

    fn resolve_use_ki(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        points: u8,
        ability: &str,
    ) -> Resolution {
        let character = &world.player_character;
        let resources = &world.player_character.class_resources;

        if resources.ki_points < points {
            return Resolution::new(format!(
                "{} doesn't have enough ki points! Has {} but needs {}.",
                character.name, resources.ki_points, points
            ));
        }

        let ability_description = match ability {
            "flurry_of_blows" => "Flurry of Blows: Make two unarmed strikes as a bonus action.",
            "patient_defense" => "Patient Defense: Take the Dodge action as a bonus action.",
            "step_of_the_wind" => "Step of the Wind: Disengage or Dash as a bonus action, jump distance doubled.",
            "stunning_strike" => "Stunning Strike: Target must make a CON save or be Stunned until the end of your next turn.",
            _ => ability,
        };

        Resolution::new(format!(
            "{} spends {} ki point{}. {}",
            character.name,
            points,
            if points == 1 { "" } else { "s" },
            ability_description
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Ki Points".to_string(),
            description: format!("Spent {points} ki for {ability}"),
        })
    }

    fn resolve_use_lay_on_hands(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        target_name: &str,
        hp_amount: u32,
        cure_disease: bool,
        neutralize_poison: bool,
    ) -> Resolution {
        let character = &world.player_character;
        let pool = world.player_character.class_resources.lay_on_hands_pool;

        let total_cost =
            hp_amount + if cure_disease { 5 } else { 0 } + if neutralize_poison { 5 } else { 0 };

        if pool < total_cost {
            return Resolution::new(format!(
                "{} doesn't have enough in their Lay on Hands pool! Has {} HP but needs {}.",
                character.name, pool, total_cost
            ));
        }

        let mut effects_text = Vec::new();
        if hp_amount > 0 {
            effects_text.push(format!("restores {hp_amount} HP"));
        }
        if cure_disease {
            effects_text.push("cures one disease".to_string());
        }
        if neutralize_poison {
            effects_text.push("neutralizes one poison".to_string());
        }

        Resolution::new(format!(
            "{} uses Lay on Hands on {}: {}. ({} HP remaining in pool)",
            character.name,
            target_name,
            effects_text.join(", "),
            pool - total_cost
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Lay on Hands".to_string(),
            description: format!("Used {total_cost} points on {target_name}"),
        })
    }

    fn resolve_use_divine_smite(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        spell_slot_level: u8,
        target_is_undead_or_fiend: bool,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if they have spell slots available
        if let Some(ref spellcasting) = character.spellcasting {
            let slot_idx = spell_slot_level.saturating_sub(1) as usize;
            if slot_idx < 9 {
                let slot = &spellcasting.spell_slots.slots[slot_idx];
                if slot.available() == 0 {
                    return Resolution::new(format!(
                        "{} has no level {} spell slots remaining!",
                        character.name, spell_slot_level
                    ));
                }
            }
        }

        // Calculate damage dice
        // Base: 2d8, +1d8 per slot level above 1st, max 5d8
        // Extra 1d8 vs undead/fiends
        let base_dice = 2 + (spell_slot_level.saturating_sub(1)).min(3);
        let total_dice = if target_is_undead_or_fiend {
            (base_dice + 1).min(6)
        } else {
            base_dice.min(5)
        };

        let damage_roll = roll_with_fallback(&format!("{total_dice}d8"), "2d8");

        let extra_text = if target_is_undead_or_fiend {
            " (extra damage vs undead/fiend)"
        } else {
            ""
        };

        Resolution::new(format!(
            "{} channels divine power into their strike! Divine Smite deals {}d8 = {} radiant damage{}. (Level {} slot expended)",
            character.name, total_dice, damage_roll.total, extra_text, spell_slot_level
        ))
        .with_effect(Effect::DiceRolled {
            roll: damage_roll,
            purpose: "Divine Smite damage".to_string(),
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Divine Smite".to_string(),
            description: format!("Used level {spell_slot_level} slot for smite"),
        })
    }

    fn resolve_use_wild_shape(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        beast_form: &str,
        beast_hp: i32,
        _beast_ac: Option<u8>,
    ) -> Resolution {
        let character = &world.player_character;

        // Check if already in Wild Shape
        if world
            .player_character
            .class_resources
            .wild_shape_form
            .is_some()
        {
            return Resolution::new(format!("{} is already in Wild Shape form!", character.name));
        }

        // Find Wild Shape feature uses
        let wild_shape_feature = character.features.iter().find(|f| f.name == "Wild Shape");
        if let Some(feature) = wild_shape_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Wild Shape uses remaining! (Recovers on short/long rest)",
                        character.name
                    ));
                }
            }
        }

        // Calculate duration based on Druid level
        let druid_level = character
            .classes
            .iter()
            .find(|c| c.class == crate::world::CharacterClass::Druid)
            .map(|c| c.level)
            .unwrap_or(2);
        let duration_hours = druid_level / 2;

        Resolution::new(format!(
            "{} transforms into a {}! Beast form has {} HP. Duration: {} hour{}. Mental stats, proficiencies, and features retained. Cannot cast spells but can maintain concentration.",
            character.name, beast_form, beast_hp, duration_hours,
            if duration_hours == 1 { "" } else { "s" }
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Wild Shape".to_string(),
            description: format!("Transformed into {beast_form} ({beast_hp} HP)"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Wild Shape".to_string(),
            uses_remaining: 0,
        })
    }

    fn resolve_end_wild_shape(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        reason: &str,
        excess_damage: i32,
    ) -> Resolution {
        let character = &world.player_character;

        if world
            .player_character
            .class_resources
            .wild_shape_form
            .is_none()
        {
            return Resolution::new(format!(
                "{} is not currently in Wild Shape form.",
                character.name
            ));
        }

        let reason_text = match reason {
            "duration_expired" => "Wild Shape ended (duration expired).",
            "hp_zero" => {
                if excess_damage > 0 {
                    &format!(
                        "Wild Shape ended (beast HP dropped to 0). {} excess damage carries over to normal form!",
                        excess_damage
                    )
                } else {
                    "Wild Shape ended (beast HP dropped to 0)."
                }
            }
            "voluntary" => "Wild Shape ended voluntarily as a bonus action.",
            "incapacitated" => "Wild Shape ended (druid became incapacitated).",
            _ => "Wild Shape ended.",
        };

        let mut resolution = Resolution::new(format!(
            "{} reverts to their normal form. {}",
            character.name, reason_text
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Wild Shape".to_string(),
            description: reason_text.to_string(),
        });

        // Apply excess damage if any
        if excess_damage > 0 {
            resolution = resolution.with_effect(Effect::HpChanged {
                target_id: world.player_character.id,
                amount: -excess_damage,
                new_current: (character.hit_points.current - excess_damage).max(0),
                new_max: character.hit_points.maximum,
                dropped_to_zero: character.hit_points.current - excess_damage <= 0,
            });
        }

        resolution
    }

    fn resolve_use_channel_divinity(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        option: &str,
        targets: &[String],
    ) -> Resolution {
        let character = &world.player_character;

        // Check for Channel Divinity uses
        let cd_feature = character
            .features
            .iter()
            .find(|f| f.name == "Channel Divinity");
        if let Some(feature) = cd_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Channel Divinity uses remaining! (Recovers on short/long rest)",
                        character.name
                    ));
                }
            }
        }

        let option_description = match option.to_lowercase().as_str() {
            "turn undead" => {
                "Turn Undead: Each undead within 30 feet must make a WIS save. On failure, they must spend their turns moving away and cannot take reactions for 1 minute."
            }
            "divine spark" => {
                "Divine Spark: Either deal 1d8 radiant damage to one creature within 30 feet (DEX save for half), or restore 1d8 HP to one creature within 30 feet."
            }
            "sacred weapon" => {
                "Sacred Weapon: Your weapon becomes magical for 1 minute, +CHA to attack rolls, and sheds bright light."
            }
            _ => option,
        };

        let targets_text = if targets.is_empty() {
            String::new()
        } else {
            format!(" Targets: {}.", targets.join(", "))
        };

        Resolution::new(format!(
            "{} uses Channel Divinity: {}.{}",
            character.name, option_description, targets_text
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Channel Divinity".to_string(),
            description: option.to_string(),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Channel Divinity".to_string(),
            uses_remaining: 0,
        })
    }

    fn resolve_use_bardic_inspiration(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        target_name: &str,
        die_size: &str,
    ) -> Resolution {
        let character = &world.player_character;

        // Check for Bardic Inspiration uses
        let bi_feature = character
            .features
            .iter()
            .find(|f| f.name == "Bardic Inspiration");
        if let Some(feature) = bi_feature {
            if let Some(ref uses) = feature.uses {
                if uses.current == 0 {
                    return Resolution::new(format!(
                        "{} has no Bardic Inspiration uses remaining! (Recovers on long rest, or short rest at level 5+)",
                        character.name
                    ));
                }
            }
        }

        Resolution::new(format!(
            "{} inspires {} with a rousing performance! {} gains a {} Bardic Inspiration die they can add to one ability check, attack roll, or saving throw within the next 10 minutes.",
            character.name, target_name, target_name, die_size
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Bardic Inspiration".to_string(),
            description: format!("Inspired {target_name} with a {die_size}"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Bardic Inspiration".to_string(),
            uses_remaining: 0,
        })
    }

    fn resolve_use_action_surge(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        action_taken: &str,
    ) -> Resolution {
        let character = &world.player_character;

        if world.player_character.class_resources.action_surge_used {
            return Resolution::new(format!(
                "{} has already used Action Surge! (Recovers on short/long rest)",
                character.name
            ));
        }

        Resolution::new(format!(
            "{} surges with renewed vigor! Takes an additional action this turn: {}",
            character.name, action_taken
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Action Surge".to_string(),
            description: action_taken.to_string(),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Action Surge".to_string(),
            uses_remaining: 0,
        })
    }

    fn resolve_use_second_wind(&self, world: &GameWorld, _character_id: CharacterId) -> Resolution {
        let character = &world.player_character;

        if world.player_character.class_resources.second_wind_used {
            return Resolution::new(format!(
                "{} has already used Second Wind! (Recovers on short/long rest)",
                character.name
            ));
        }

        // Calculate healing: 1d10 + fighter level
        let fighter_level = character
            .classes
            .iter()
            .find(|c| c.class == crate::world::CharacterClass::Fighter)
            .map(|c| c.level)
            .unwrap_or(1);

        let healing_roll = roll_with_fallback(&format!("1d10+{fighter_level}"), "1d10+1");
        let healing = healing_roll.total;

        let new_hp = (character.hit_points.current + healing).min(character.hit_points.maximum);

        Resolution::new(format!(
            "{} catches their breath with Second Wind! Regains 1d10+{} = {} HP. (Now at {}/{})",
            character.name, fighter_level, healing, new_hp, character.hit_points.maximum
        ))
        .with_effect(Effect::DiceRolled {
            roll: healing_roll,
            purpose: "Second Wind healing".to_string(),
        })
        .with_effect(Effect::HpChanged {
            target_id: world.player_character.id,
            amount: healing,
            new_current: new_hp,
            new_max: character.hit_points.maximum,
            dropped_to_zero: false,
        })
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Second Wind".to_string(),
            description: format!("Healed {healing} HP"),
        })
        .with_effect(Effect::FeatureUsed {
            feature_name: "Second Wind".to_string(),
            uses_remaining: 0,
        })
    }

    fn resolve_use_sorcery_points(
        &self,
        world: &GameWorld,
        _character_id: CharacterId,
        points: u8,
        metamagic: &str,
        spell_name: Option<&str>,
        slot_level: Option<u8>,
    ) -> Resolution {
        let character = &world.player_character;
        let resources = &world.player_character.class_resources;

        // Handle slot conversion separately
        if metamagic == "convert_to_slot" {
            if let Some(level) = slot_level {
                let cost = level; // Costs spell level points to create a slot
                if resources.sorcery_points < cost {
                    return Resolution::new(format!(
                        "{} doesn't have enough sorcery points! Has {} but needs {} to create a level {} slot.",
                        character.name, resources.sorcery_points, cost, level
                    ));
                }
                return Resolution::new(format!(
                    "{} converts {} sorcery points into a level {} spell slot.",
                    character.name, cost, level
                ))
                .with_effect(Effect::ClassResourceUsed {
                    character_name: character.name.clone(),
                    resource_name: "Sorcery Points".to_string(),
                    description: format!("Created level {level} spell slot"),
                });
            }
        }

        if metamagic == "convert_from_slot" {
            if let Some(level) = slot_level {
                return Resolution::new(format!(
                    "{} converts a level {} spell slot into {} sorcery points.",
                    character.name, level, level
                ))
                .with_effect(Effect::ClassResourceUsed {
                    character_name: character.name.clone(),
                    resource_name: "Sorcery Points".to_string(),
                    description: format!("Gained {level} points from slot"),
                });
            }
        }

        // Regular Metamagic usage
        if resources.sorcery_points < points {
            return Resolution::new(format!(
                "{} doesn't have enough sorcery points! Has {} but needs {}.",
                character.name, resources.sorcery_points, points
            ));
        }

        let metamagic_description = match metamagic.to_lowercase().as_str() {
            "careful" => "Careful Spell: Protect allies from your spell's area effect.",
            "distant" => "Distant Spell: Double the spell's range (or 30 ft if touch).",
            "empowered" => "Empowered Spell: Reroll up to CHA mod damage dice.",
            "extended" => "Extended Spell: Double the spell's duration (max 24 hours).",
            "heightened" => "Heightened Spell: Target has disadvantage on first save.",
            "quickened" => "Quickened Spell: Cast as a bonus action instead of an action.",
            "subtle" => "Subtle Spell: Cast without verbal or somatic components.",
            "twinned" => "Twinned Spell: Target a second creature with a single-target spell.",
            _ => metamagic,
        };

        let spell_text = spell_name.map_or(String::new(), |s| format!(" on {}", s));

        Resolution::new(format!(
            "{} uses {}{} ({} sorcery point{}).",
            character.name,
            metamagic_description,
            spell_text,
            points,
            if points == 1 { "" } else { "s" }
        ))
        .with_effect(Effect::ClassResourceUsed {
            character_name: character.name.clone(),
            resource_name: "Sorcery Points".to_string(),
            description: format!("Used {points} for {metamagic}"),
        })
    }

    // ========================================================================
    // Quest Resolution Methods
    // ========================================================================

    fn resolve_create_quest(
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

    fn resolve_add_quest_objective(
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

    fn resolve_complete_objective(
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

    fn resolve_complete_quest(
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

    fn resolve_fail_quest(&self, quest_name: &str, failure_reason: &str) -> Resolution {
        Resolution::new(format!(
            "Quest Failed: \"{}\" - {}",
            quest_name, failure_reason
        ))
        .with_effect(Effect::QuestFailed {
            quest_name: quest_name.to_string(),
            failure_reason: failure_reason.to_string(),
        })
    }

    fn resolve_update_quest(
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

    // ========================================================================
    // World Building Resolution Methods
    // ========================================================================

    #[allow(clippy::too_many_arguments)]
    fn resolve_create_npc(
        &self,
        world: &GameWorld,
        name: &str,
        _description: &str,
        _personality: &str,
        occupation: Option<&str>,
        disposition: &str,
        location: Option<&str>,
        _known_information: &[String],
    ) -> Resolution {
        // Check if an NPC with this name already exists (case-insensitive)
        let existing_npc = world
            .npcs
            .values()
            .find(|n| n.name.eq_ignore_ascii_case(name));

        if let Some(existing) = existing_npc {
            // NPC already exists - return an error with guidance
            return Resolution::new(format!(
                "DUPLICATE NPC ERROR: An NPC named '{}' already exists (disposition: {:?}). \
                Use 'update_npc' instead to modify their disposition, add information, or change their description. \
                Do NOT call create_npc again for this character.",
                existing.name,
                existing.disposition
            ));
        }

        let location_text = location.map(|l| format!(" at {}", l)).unwrap_or_default();
        let occupation_text = occupation.map(|o| format!(" ({})", o)).unwrap_or_default();

        Resolution::new(format!(
            "NPC {} ({}){}{}  enters the world",
            name, disposition, occupation_text, location_text
        ))
        .with_effect(Effect::NpcCreated {
            name: name.to_string(),
            location: location.map(|s| s.to_string()),
        })
    }

    fn resolve_update_npc(
        &self,
        world: &GameWorld,
        npc_name: &str,
        disposition: Option<&str>,
        add_information: &[String],
        new_description: Option<&str>,
        new_personality: Option<&str>,
    ) -> Resolution {
        // Check if NPC exists
        let npc_exists = world
            .npcs
            .values()
            .any(|n| n.name.eq_ignore_ascii_case(npc_name));

        if !npc_exists {
            return Resolution::new(format!("NPC '{}' not found in the world", npc_name));
        }

        let mut changes = Vec::new();
        if disposition.is_some() {
            changes.push("disposition changed");
        }
        if !add_information.is_empty() {
            changes.push("new information learned");
        }
        if new_description.is_some() {
            changes.push("description updated");
        }
        if new_personality.is_some() {
            changes.push("personality updated");
        }

        let changes_text = if changes.is_empty() {
            "no changes".to_string()
        } else {
            changes.join(", ")
        };

        Resolution::new(format!("NPC {} updated: {}", npc_name, changes_text)).with_effect(
            Effect::NpcUpdated {
                npc_name: npc_name.to_string(),
                changes: changes_text,
            },
        )
    }

    fn resolve_move_npc(
        &self,
        world: &GameWorld,
        npc_name: &str,
        destination: &str,
        reason: Option<&str>,
    ) -> Resolution {
        // Find the NPC's current location
        let from_location = world
            .npcs
            .values()
            .find(|n| n.name.eq_ignore_ascii_case(npc_name))
            .and_then(|n| n.location_id)
            .and_then(|loc_id| world.known_locations.get(&loc_id))
            .map(|loc| loc.name.clone());

        let reason_text = reason.map(|r| format!(" ({})", r)).unwrap_or_default();

        Resolution::new(format!(
            "NPC {} moves to {}{}",
            npc_name, destination, reason_text
        ))
        .with_effect(Effect::NpcMoved {
            npc_name: npc_name.to_string(),
            from_location,
            to_location: destination.to_string(),
        })
    }

    fn resolve_remove_npc(&self, npc_name: &str, reason: &str, permanent: bool) -> Resolution {
        let permanence = if permanent {
            "permanently"
        } else {
            "temporarily"
        };

        Resolution::new(format!(
            "NPC {} {} removed: {}",
            npc_name, permanence, reason
        ))
        .with_effect(Effect::NpcRemoved {
            npc_name: npc_name.to_string(),
            reason: reason.to_string(),
        })
    }

    fn resolve_create_location(
        &self,
        name: &str,
        location_type: &str,
        description: &str,
        parent_location: Option<&str>,
        items: &[String],
        npcs_present: &[String],
    ) -> Resolution {
        let parent_text = parent_location
            .map(|p| format!(" in {}", p))
            .unwrap_or_default();
        let items_text = if items.is_empty() {
            String::new()
        } else {
            format!(" with items: {}", items.join(", "))
        };
        let npcs_text = if npcs_present.is_empty() {
            String::new()
        } else {
            format!(" featuring NPCs: {}", npcs_present.join(", "))
        };

        Resolution::new(format!(
            "New location created: {} ({}){}{}{} - {}",
            name, location_type, parent_text, items_text, npcs_text, description
        ))
        .with_effect(Effect::LocationCreated {
            name: name.to_string(),
            location_type: location_type.to_string(),
        })
    }

    fn resolve_connect_locations(
        &self,
        from_location: &str,
        to_location: &str,
        direction: Option<&str>,
        travel_time_minutes: Option<u32>,
        bidirectional: bool,
    ) -> Resolution {
        let direction_text = direction
            .map(|d| format!(" ({} direction)", d))
            .unwrap_or_default();
        let travel_text = travel_time_minutes
            .map(|t| format!(", {} minutes travel time", t))
            .unwrap_or_default();
        let bidirectional_text = if bidirectional {
            " (bidirectional)"
        } else {
            " (one-way)"
        };

        Resolution::new(format!(
            "Locations connected: {} to {}{}{}{}",
            from_location, to_location, direction_text, travel_text, bidirectional_text
        ))
        .with_effect(Effect::LocationsConnected {
            from: from_location.to_string(),
            to: to_location.to_string(),
            direction: direction.map(|s| s.to_string()),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_update_location(
        &self,
        world: &GameWorld,
        location_name: &str,
        new_description: Option<&str>,
        add_items: &[String],
        remove_items: &[String],
        add_npcs: &[String],
        remove_npcs: &[String],
    ) -> Resolution {
        // Check if location exists
        let location_exists = world
            .known_locations
            .values()
            .any(|l| l.name.eq_ignore_ascii_case(location_name))
            || world
                .current_location
                .name
                .eq_ignore_ascii_case(location_name);

        if !location_exists {
            return Resolution::new(format!(
                "Location '{}' not found in the world",
                location_name
            ));
        }

        let mut changes: Vec<String> = Vec::new();
        if new_description.is_some() {
            changes.push("description updated".to_string());
        }
        if !add_items.is_empty() {
            changes.push(format!("added items: {}", add_items.join(", ")));
        }
        if !remove_items.is_empty() {
            changes.push(format!("removed items: {}", remove_items.join(", ")));
        }
        if !add_npcs.is_empty() {
            changes.push(format!("NPCs arrived: {}", add_npcs.join(", ")));
        }
        if !remove_npcs.is_empty() {
            changes.push(format!("NPCs left: {}", remove_npcs.join(", ")));
        }

        let changes_text = if changes.is_empty() {
            "no changes".to_string()
        } else {
            changes.join("; ")
        };

        Resolution::new(format!(
            "Location {} updated: {}",
            location_name, changes_text
        ))
        .with_effect(Effect::LocationUpdated {
            location_name: location_name.to_string(),
            changes: changes_text,
        })
    }

    fn resolve_modify_ability_score(
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

    fn resolve_restore_spell_slot(
        &self,
        world: &GameWorld,
        slot_level: u8,
        source: &str,
    ) -> Resolution {
        if slot_level == 0 || slot_level > 9 {
            return Resolution::new(format!(
                "Invalid spell slot level: {}. Must be between 1 and 9.",
                slot_level
            ));
        }

        let new_remaining = world
            .player_character
            .spellcasting
            .as_ref()
            .map(|sc| {
                let slot_idx = (slot_level - 1) as usize;
                sc.spell_slots.slots[slot_idx].available() + 1
            })
            .unwrap_or(0);

        Resolution::new(format!(
            "Level {} spell slot restored by {}",
            slot_level, source
        ))
        .with_effect(Effect::SpellSlotRestored {
            level: slot_level,
            new_remaining,
        })
    }

    fn resolve_assert_state(
        &self,
        world: &GameWorld,
        entity_name: &str,
        state_type: StateType,
        new_value: &str,
        reason: &str,
        target_entity: Option<&str>,
    ) -> Resolution {
        // Try to find the NPC to get old value
        let old_value = world
            .npcs
            .values()
            .find(|npc| npc.name.eq_ignore_ascii_case(entity_name))
            .and_then(|npc| {
                match state_type {
                    StateType::Disposition => {
                        let disp_str = match npc.disposition {
                            crate::world::Disposition::Hostile => "hostile",
                            crate::world::Disposition::Unfriendly => "unfriendly",
                            crate::world::Disposition::Neutral => "neutral",
                            crate::world::Disposition::Friendly => "friendly",
                            crate::world::Disposition::Helpful => "helpful",
                        };
                        Some(disp_str.to_string())
                    }
                    StateType::Location => npc
                        .location_id
                        .and_then(|id| world.known_locations.get(&id))
                        .map(|loc| loc.name.clone()),
                    StateType::Status => None, // NPC struct doesn't have a status field
                    _ => None,
                }
            });

        let narrative = match state_type {
            StateType::Disposition => {
                format!(
                    "{}'s disposition is now {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Location => {
                format!(
                    "{} is now at {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Status => {
                format!(
                    "{}'s status is now {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Knowledge => {
                format!(
                    "{} now knows: {} (reason: {})",
                    entity_name, new_value, reason
                )
            }
            StateType::Relationship => {
                if let Some(target) = target_entity {
                    format!(
                        "{}'s relationship with {} is now {} (reason: {})",
                        entity_name, target, new_value, reason
                    )
                } else {
                    format!(
                        "{}'s relationship status: {} (reason: {})",
                        entity_name, new_value, reason
                    )
                }
            }
        };

        Resolution::new(narrative).with_effect(Effect::StateAsserted {
            entity_name: entity_name.to_string(),
            state_type,
            old_value,
            new_value: new_value.to_string(),
            reason: reason.to_string(),
            target_entity: target_entity.map(|s| s.to_string()),
        })
    }

    fn resolve_share_knowledge(
        &self,
        knowing_entity: &str,
        content: &str,
        source: &str,
        verification: &str,
        context: Option<&str>,
    ) -> Resolution {
        let narrative = if let Some(ctx) = context {
            format!(
                "{} now knows: \"{}\" (from: {}, {}) [{}]",
                knowing_entity, content, source, verification, ctx
            )
        } else {
            format!(
                "{} now knows: \"{}\" (from: {}, {})",
                knowing_entity, content, source, verification
            )
        };

        Resolution::new(narrative).with_effect(Effect::KnowledgeShared {
            knowing_entity: knowing_entity.to_string(),
            content: content.to_string(),
            source: source.to_string(),
            verification: verification.to_string(),
            context: context.map(|s| s.to_string()),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_schedule_event(
        &self,
        _world: &GameWorld,
        description: &str,
        minutes: Option<u32>,
        hours: Option<u32>,
        day: Option<u8>,
        month: Option<u8>,
        year: Option<i32>,
        hour: Option<u8>,
        daily_hour: Option<u8>,
        daily_minute: Option<u8>,
        location: Option<&str>,
        visibility: &str,
        repeating: bool,
    ) -> Resolution {
        // Build the trigger description
        let trigger_description = if let (Some(dh), Some(dm)) = (daily_hour, daily_minute) {
            if repeating {
                format!("daily at {:02}:{:02}", dh, dm)
            } else {
                format!("at {:02}:{:02}", dh, dm)
            }
        } else if let (Some(d), Some(m), Some(y)) = (day, month, year) {
            if let Some(h) = hour {
                format!("on {}/{}/{} at {:02}:00", m, d, y, h)
            } else {
                format!("on {}/{}/{}", m, d, y)
            }
        } else if minutes.is_some() || hours.is_some() {
            let total_minutes = minutes.unwrap_or(0) + hours.unwrap_or(0) * 60;
            if total_minutes < 60 {
                format!("in {} minutes", total_minutes)
            } else if total_minutes < 1440 {
                let h = total_minutes / 60;
                let m = total_minutes % 60;
                if m > 0 {
                    format!("in {} hours and {} minutes", h, m)
                } else {
                    format!("in {} hours", h)
                }
            } else {
                let days = total_minutes / 1440;
                format!("in {} days", days)
            }
        } else {
            "at an unspecified time".to_string()
        };

        let loc_desc = location.map(|l| format!(" at {}", l)).unwrap_or_default();

        let vis_desc = match visibility {
            "private" | "secret" => " (private)",
            "hinted" => " (hinted)",
            _ => "",
        };

        let narrative = format!(
            "Scheduled: \"{}\" {}{}{}",
            description, trigger_description, loc_desc, vis_desc
        );

        Resolution::new(narrative).with_effect(Effect::EventScheduled {
            description: description.to_string(),
            trigger_description,
            location: location.map(|s| s.to_string()),
            visibility: visibility.to_string(),
        })
    }

    fn resolve_cancel_event(&self, event_description: &str, reason: &str) -> Resolution {
        let narrative = format!("Event cancelled: \"{}\" - {}", event_description, reason);

        Resolution::new(narrative).with_effect(Effect::EventCancelled {
            description: event_description.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply effects to the game world.
pub fn apply_effects(world: &mut GameWorld, effects: &[Effect]) {
    for effect in effects {
        apply_effect(world, effect);
    }
}

/// Apply a single effect to the game world.
pub fn apply_effect(world: &mut GameWorld, effect: &Effect) {
    match effect {
        Effect::HpChanged {
            amount,
            dropped_to_zero,
            ..
        } => {
            let was_unconscious = world.player_character.hit_points.current <= 0;

            if *amount < 0 {
                world.player_character.hit_points.take_damage(-*amount);
            } else {
                world.player_character.hit_points.heal(*amount);
            }

            // Add Unconscious condition if dropped to 0 (only if not already unconscious)
            if *dropped_to_zero {
                world
                    .player_character
                    .add_condition(Condition::Unconscious, "Dropped to 0 HP");
            }

            // Remove Unconscious condition and reset death saves if healed above 0
            if was_unconscious && world.player_character.hit_points.current > 0 {
                world
                    .player_character
                    .conditions
                    .retain(|c| c.condition != Condition::Unconscious);
                // Reset death saves when regaining consciousness
                world.player_character.death_saves.reset();
            }

            // Sync HP to combat state if in combat
            if let Some(ref mut combat) = world.combat {
                let player_id = world.player_character.id;
                combat.update_combatant_hp(player_id, world.player_character.hit_points.current);
            }
        }
        Effect::ConditionApplied {
            condition,
            source,
            duration_rounds,
            ..
        } => {
            world.player_character.add_condition_with_duration(
                *condition,
                source.clone(),
                *duration_rounds,
            );
        }
        Effect::ConditionRemoved { condition, .. } => {
            world
                .player_character
                .conditions
                .retain(|c| c.condition != *condition);
        }
        Effect::CombatStarted => {
            world.start_combat();
        }
        Effect::CombatEnded => {
            world.end_combat();
        }
        Effect::CombatantAdded {
            id,
            name,
            initiative,
            is_ally,
            current_hp,
            max_hp,
            armor_class,
        } => {
            if let Some(ref mut combat) = world.combat {
                combat.add_combatant(Combatant {
                    id: *id,
                    name: name.clone(),
                    initiative: *initiative,
                    is_player: *id == world.player_character.id,
                    is_ally: *is_ally,
                    current_hp: *current_hp,
                    max_hp: *max_hp,
                    armor_class: *armor_class,
                });
            }
        }
        Effect::TurnAdvanced { .. } => {
            if let Some(ref mut combat) = world.combat {
                combat.next_turn();
            }

            // Decrement condition durations and remove expired conditions
            world.player_character.conditions.retain_mut(|c| {
                if let Some(ref mut duration) = c.duration_rounds {
                    if *duration > 0 {
                        *duration -= 1;
                    }
                    *duration > 0 // Keep only if duration remaining
                } else {
                    true // Keep permanent conditions
                }
            });
        }
        Effect::TimeAdvanced { minutes } => {
            world.game_time.advance_minutes(*minutes);
        }
        Effect::RestCompleted { rest_type } => match rest_type {
            RestType::Short => world.short_rest(),
            RestType::Long => world.long_rest(),
        },
        Effect::ExperienceGained { amount, .. } => {
            world.player_character.experience += amount;
        }
        Effect::LevelUp { new_level } => {
            let character = &mut world.player_character;
            let old_level = character.level;
            character.level = *new_level;

            // Get the primary class for level-up calculations
            if let Some(class_level) = character.classes.first_mut() {
                let class = class_level.class;
                let hit_die = class.hit_die();

                // Update class level
                class_level.level = *new_level;

                // Add HP: roll hit die + CON modifier (use average for consistency)
                // Average is (max/2 + 1), e.g., d8 = 5, d10 = 6, d12 = 7
                let hit_die_average = (hit_die.sides() / 2 + 1) as i32;
                let con_mod = character.ability_scores.modifier(Ability::Constitution) as i32;
                let hp_gained = (hit_die_average + con_mod).max(1);
                character.hit_points.maximum += hp_gained;
                character.hit_points.current += hp_gained;

                // Add a hit die
                character.hit_dice.add(hit_die, 1);

                // Update spell slots for spellcasters
                if class.spellcasting_ability().is_some() {
                    let new_slots = class.spell_slots_at_level(*new_level);

                    if let Some(ref mut spellcasting) = character.spellcasting {
                        for (i, &total) in new_slots.iter().enumerate() {
                            let old_total = spellcasting.spell_slots.slots[i].total;
                            spellcasting.spell_slots.slots[i].total = total;
                            // Restore any new slots gained
                            if total > old_total {
                                let gained = total - old_total;
                                spellcasting.spell_slots.slots[i].used =
                                    spellcasting.spell_slots.slots[i]
                                        .used
                                        .saturating_sub(gained);
                            }
                        }
                    } else if new_slots.iter().any(|&s| s > 0) {
                        // Class just gained spellcasting (e.g., Paladin/Ranger at level 2)
                        character.spellcasting = Some(SpellcastingData {
                            ability: class.spellcasting_ability().unwrap(),
                            spells_known: Vec::new(),
                            spells_prepared: Vec::new(),
                            cantrips_known: Vec::new(),
                            spell_slots: SpellSlots {
                                slots: std::array::from_fn(|i| SlotInfo {
                                    total: new_slots[i],
                                    used: 0,
                                }),
                            },
                        });
                    }

                    // Track spell learning capacity changes for narrative purposes
                    // (actual spell selection happens via DM tools or character choices)
                    let old_cantrips = class.cantrips_known_at_level(old_level);
                    let new_cantrips = class.cantrips_known_at_level(*new_level);
                    let _cantrips_gained = new_cantrips.saturating_sub(old_cantrips);

                    // For "known" casters, track new spell capacity
                    if let Some(old_known) = class.spells_known_at_level(old_level) {
                        if let Some(new_known) = class.spells_known_at_level(*new_level) {
                            let _spells_gained = new_known.saturating_sub(old_known);
                            // Player can now learn more spells up to new_known total
                        }
                    }

                    // For Wizard, track spellbook expansion
                    let _wizard_spells_added = class.wizard_spellbook_spells_at_level(*new_level);
                }

                // Update class resources based on class and level
                match class {
                    CharacterClass::Monk => {
                        // Ki points = Monk level
                        character.class_resources.max_ki_points = *new_level;
                        character.class_resources.ki_points = *new_level;
                    }
                    CharacterClass::Sorcerer => {
                        // Sorcery points = Sorcerer level (gained at level 2)
                        if *new_level >= 2 {
                            character.class_resources.max_sorcery_points = *new_level;
                            // Give the new points
                            let gained = *new_level - old_level;
                            character.class_resources.sorcery_points =
                                (character.class_resources.sorcery_points + gained)
                                    .min(character.class_resources.max_sorcery_points);
                        }
                    }
                    CharacterClass::Paladin => {
                        // Lay on Hands pool = 5 × Paladin level
                        character.class_resources.lay_on_hands_max = (*new_level as u32) * 5;
                        // Restore to full on level up
                        character.class_resources.lay_on_hands_pool =
                            character.class_resources.lay_on_hands_max;
                    }
                    CharacterClass::Barbarian => {
                        // Rage uses increase at certain levels
                        let rage_uses = match *new_level {
                            1..=2 => 2,
                            3..=5 => 3,
                            6..=11 => 4,
                            12..=16 => 5,
                            17..=19 => 6,
                            20 => u8::MAX, // Unlimited at 20
                            _ => 2,
                        };
                        // Update rage feature uses
                        if let Some(rage_feature) =
                            character.features.iter_mut().find(|f| f.name == "Rage")
                        {
                            if let Some(ref mut uses) = rage_feature.uses {
                                uses.maximum = rage_uses;
                                uses.current = rage_uses;
                            }
                        }
                        // Rage damage bonus increases at levels 9 and 16
                        character.class_resources.rage_damage_bonus = match *new_level {
                            1..=8 => 2,
                            9..=15 => 3,
                            16..=20 => 4,
                            _ => 2,
                        };
                    }
                    _ => {}
                }
            }
        }
        Effect::FeatureUsed {
            feature_name,
            uses_remaining,
        } => {
            if let Some(feature) = world
                .player_character
                .features
                .iter_mut()
                .find(|f| f.name == *feature_name)
            {
                if let Some(ref mut uses) = feature.uses {
                    uses.current = *uses_remaining;
                }
            }
        }
        Effect::SpellSlotUsed { level, .. } => {
            if let Some(ref mut spellcasting) = world.player_character.spellcasting {
                spellcasting.spell_slots.use_slot(*level);
            }
        }
        // Effects that don't modify state (informational)
        Effect::DiceRolled { .. } => {}
        Effect::CheckSucceeded { .. } => {}
        Effect::CheckFailed { .. } => {}
        Effect::AttackHit { .. } => {}
        Effect::AttackMissed { .. } => {}
        Effect::InitiativeRolled { .. } => {}
        Effect::SneakAttackUsed { character_id, .. } => {
            // Mark that this character has used their sneak attack this turn
            if let Some(ref mut combat) = world.combat {
                combat.sneak_attack_used.insert(*character_id);
            }
        }
        // FactRemembered is handled by the DM agent's memory system, not world state
        Effect::FactRemembered { .. } => {}

        // Inventory effects
        Effect::ItemAdded {
            item_name,
            quantity,
            ..
        } => {
            // Try to look up item from standard database first
            let item = if let Some(standard_item) = crate::items::find_item(item_name) {
                let mut item = standard_item.as_item();
                item.quantity = *quantity;
                item
            } else {
                // Fall back to generic item
                Item {
                    name: item_name.clone(),
                    quantity: *quantity,
                    weight: 0.0,
                    value_gp: 0.0,
                    description: None,
                    item_type: ItemType::Other,
                    magical: false,
                }
            };
            world.player_character.inventory.add_item(item);
        }
        Effect::ItemRemoved {
            item_name,
            quantity,
            ..
        } => {
            world
                .player_character
                .inventory
                .remove_item(item_name, *quantity);
        }
        Effect::ItemEquipped { item_name, slot } => {
            // Look up item from database for proper stats, fall back to defaults
            match slot.as_str() {
                "armor" => {
                    if world
                        .player_character
                        .inventory
                        .find_item(item_name)
                        .is_some()
                    {
                        // Try to get proper armor stats from database
                        let armor = if let Some(db_armor) = crate::items::get_armor(item_name) {
                            db_armor
                        } else {
                            // Fall back to medium armor defaults
                            crate::world::ArmorItem::new(
                                item_name.clone(),
                                crate::world::ArmorType::Medium,
                                14,
                            )
                        };
                        world.player_character.equipment.armor = Some(armor);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                "shield" => {
                    if let Some(item) = world.player_character.inventory.find_item(item_name) {
                        let shield_item = item.clone();
                        world.player_character.equipment.shield = Some(shield_item);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                "main_hand" | "weapon" => {
                    if world
                        .player_character
                        .inventory
                        .find_item(item_name)
                        .is_some()
                    {
                        // Try to get proper weapon stats from database
                        let weapon = if let Some(db_weapon) = crate::items::get_weapon(item_name) {
                            db_weapon
                        } else {
                            // Fall back to generic 1d8 slashing
                            crate::world::WeaponItem::new(
                                item_name.clone(),
                                "1d8",
                                crate::world::WeaponDamageType::Slashing,
                            )
                        };
                        world.player_character.equipment.main_hand = Some(weapon);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                "off_hand" => {
                    if let Some(item) = world.player_character.inventory.find_item(item_name) {
                        let off_hand_item = item.clone();
                        world.player_character.equipment.off_hand = Some(off_hand_item);
                        world.player_character.inventory.remove_item(item_name, 1);
                    }
                }
                _ => {}
            }
        }
        Effect::ItemUnequipped { slot, .. } => match slot.as_str() {
            "armor" => {
                if let Some(armor) = world.player_character.equipment.armor.take() {
                    world.player_character.inventory.add_item(armor.base);
                }
            }
            "shield" => {
                if let Some(shield) = world.player_character.equipment.shield.take() {
                    world.player_character.inventory.add_item(shield);
                }
            }
            "main_hand" | "weapon" => {
                if let Some(weapon) = world.player_character.equipment.main_hand.take() {
                    world.player_character.inventory.add_item(weapon.base);
                }
            }
            "off_hand" => {
                if let Some(item) = world.player_character.equipment.off_hand.take() {
                    world.player_character.inventory.add_item(item);
                }
            }
            _ => {}
        },
        // ItemUsed is informational - the actual effects (healing, etc.) are separate effects
        Effect::ItemUsed { .. } => {}
        Effect::GoldChanged { new_total, .. } => {
            world.player_character.inventory.gold = *new_total;
        }
        Effect::SilverChanged { new_total, .. } => {
            world.player_character.inventory.silver = *new_total;
        }
        // AcChanged is informational - AC is recalculated from equipment
        Effect::AcChanged { .. } => {}

        Effect::DeathSaveFailure { failures, .. } => {
            for _ in 0..*failures {
                world.player_character.death_saves.add_failure();
            }
        }

        Effect::DeathSavesReset { .. } => {
            world.player_character.death_saves.reset();
        }

        Effect::CharacterDied { .. } => {
            // Character death is tracked via the effect itself
            // The UI/game can check for this effect and handle appropriately
            // For now, we don't modify world state further (could add a `dead: bool` flag)
        }

        Effect::DeathSaveSuccess {
            total_successes, ..
        } => {
            world.player_character.death_saves.successes = *total_successes;
        }

        Effect::Stabilized { .. } => {
            // Character is stable - still unconscious but no longer making death saves
            world.player_character.death_saves.reset();
            // Note: Character remains Unconscious until healed
        }

        Effect::ConcentrationBroken { .. } => {
            // Concentration tracking would be handled here if we had it
            // For now, this is informational for the UI/narrative
        }

        Effect::ConcentrationMaintained { .. } => {
            // Informational - concentration continues
        }
        Effect::LocationChanged { new_location, .. } => {
            world.current_location.name = new_location.clone();
        }
        Effect::ConsequenceRegistered { .. } => {
            // Consequence storage is handled by the DM agent in story_memory
            // This effect is informational for the rules layer
        }
        Effect::ConsequenceTriggered { .. } => {
            // Consequence triggering is handled by the relevance checker
            // This effect is informational for the UI/narrative
        }
        Effect::ClassResourceUsed { .. } => {
            // Class resource usage is tracked in ClassResources
            // The actual state changes are handled by the DM based on the effect
            // This effect is informational for the narrative/UI
        }
        Effect::RageStarted { damage_bonus, .. } => {
            world.player_character.class_resources.rage_active = true;
            world.player_character.class_resources.rage_damage_bonus = *damage_bonus;
            world.player_character.class_resources.rage_rounds_remaining = Some(10);
            // 1 minute = 10 rounds
        }
        Effect::RageEnded { .. } => {
            world.player_character.class_resources.rage_active = false;
            world.player_character.class_resources.rage_damage_bonus = 0;
            world.player_character.class_resources.rage_rounds_remaining = None;
        }

        // Quest effects
        Effect::QuestCreated {
            name,
            description,
            giver,
            objectives,
            rewards,
        } => {
            use crate::world::{Quest, QuestObjective};
            let mut quest = Quest::new(name.clone(), description.clone());
            quest.giver = giver.clone();
            quest.objectives = objectives
                .iter()
                .map(|(desc, optional)| QuestObjective {
                    description: desc.clone(),
                    completed: false,
                    optional: *optional,
                })
                .collect();
            quest.rewards = rewards.clone();
            world.quests.push(quest);
        }

        Effect::QuestObjectiveAdded {
            quest_name,
            objective,
            optional,
        } => {
            use crate::world::QuestObjective;
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                quest.objectives.push(QuestObjective {
                    description: objective.clone(),
                    completed: false,
                    optional: *optional,
                });
            }
        }

        Effect::QuestObjectiveCompleted {
            quest_name,
            objective_description,
        } => {
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                // Find objective by partial match
                if let Some(obj) = quest.objectives.iter_mut().find(|o| {
                    o.description
                        .to_lowercase()
                        .contains(&objective_description.to_lowercase())
                }) {
                    obj.completed = true;
                }
            }
        }

        Effect::QuestCompleted { quest_name, .. } => {
            use crate::world::QuestStatus;
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                quest.status = QuestStatus::Completed;
                // Mark all non-optional objectives as complete
                for obj in &mut quest.objectives {
                    if !obj.optional {
                        obj.completed = true;
                    }
                }
            }
        }

        Effect::QuestFailed { quest_name, .. } => {
            use crate::world::QuestStatus;
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                quest.status = QuestStatus::Failed;
            }
        }

        Effect::QuestUpdated {
            quest_name,
            new_description,
            add_rewards,
        } => {
            if let Some(quest) = world.quests.iter_mut().find(|q| q.name == *quest_name) {
                if let Some(desc) = new_description {
                    quest.description = desc.clone();
                }
                quest.rewards.extend(add_rewards.iter().cloned());
            }
        }

        // World Building effects
        Effect::NpcCreated { name, location } => {
            use crate::world::NPC;
            let mut npc = NPC::new(name.clone());

            // Set location if provided
            if let Some(loc_name) = location {
                // Find the location by name
                if let Some(loc) = world
                    .known_locations
                    .values()
                    .find(|l| l.name.eq_ignore_ascii_case(loc_name))
                {
                    npc.location_id = Some(loc.id);
                }
            }

            world.npcs.insert(npc.id, npc);
        }

        Effect::NpcUpdated { npc_name, .. } => {
            // The actual updates are passed through the Intent
            // This effect is informational for the narrative/UI
            // The real state changes would need the full update data
            // which is handled by the DM agent passing updated NPC data
            let _ = npc_name; // Suppress unused warning
        }

        Effect::NpcMoved {
            npc_name,
            to_location,
            ..
        } => {
            // Find NPC by name and update their location
            if let Some(npc) = world
                .npcs
                .values_mut()
                .find(|n| n.name.eq_ignore_ascii_case(npc_name))
            {
                // Find the destination location
                if let Some(loc) = world
                    .known_locations
                    .values()
                    .find(|l| l.name.eq_ignore_ascii_case(to_location))
                {
                    npc.location_id = Some(loc.id);
                } else {
                    // Location not found, clear location_id
                    npc.location_id = None;
                }
            }
        }

        Effect::NpcRemoved { npc_name, .. } => {
            // Remove NPC from the world
            let npc_id = world
                .npcs
                .values()
                .find(|n| n.name.eq_ignore_ascii_case(npc_name))
                .map(|n| n.id);

            if let Some(id) = npc_id {
                world.npcs.remove(&id);
            }
        }

        Effect::LocationCreated {
            name,
            location_type,
        } => {
            use crate::world::{Location, LocationType};

            let loc_type = match location_type.to_lowercase().as_str() {
                "wilderness" => LocationType::Wilderness,
                "town" => LocationType::Town,
                "city" => LocationType::City,
                "dungeon" => LocationType::Dungeon,
                "building" => LocationType::Building,
                "room" => LocationType::Room,
                "road" => LocationType::Road,
                "cave" => LocationType::Cave,
                _ => LocationType::Other,
            };

            let location = Location::new(name.clone(), loc_type);
            world.known_locations.insert(location.id, location);
        }

        Effect::LocationsConnected {
            from,
            to,
            direction,
        } => {
            use crate::world::LocationConnection;

            // Find the source and destination locations
            let from_id = world
                .known_locations
                .values()
                .find(|l| l.name.eq_ignore_ascii_case(from))
                .map(|l| l.id);

            let to_loc = world
                .known_locations
                .values()
                .find(|l| l.name.eq_ignore_ascii_case(to));

            if let (Some(from_id), Some(to_loc)) = (from_id, to_loc) {
                let connection = LocationConnection {
                    destination_id: to_loc.id,
                    destination_name: to_loc.name.clone(),
                    direction: direction.clone(),
                    travel_time_minutes: 0,
                };

                if let Some(from_loc) = world.known_locations.get_mut(&from_id) {
                    from_loc.connections.push(connection);
                }
            }
        }

        Effect::LocationUpdated { location_name, .. } => {
            // The actual updates are passed through the Intent
            // This effect is informational for the narrative/UI
            let _ = location_name; // Suppress unused warning
        }

        Effect::AbilityScoreModified {
            ability, modifier, ..
        } => {
            // Apply ability score modifier
            let score = match ability {
                Ability::Strength => &mut world.player_character.ability_scores.strength,
                Ability::Dexterity => &mut world.player_character.ability_scores.dexterity,
                Ability::Constitution => &mut world.player_character.ability_scores.constitution,
                Ability::Intelligence => &mut world.player_character.ability_scores.intelligence,
                Ability::Wisdom => &mut world.player_character.ability_scores.wisdom,
                Ability::Charisma => &mut world.player_character.ability_scores.charisma,
            };
            *score = (*score as i16 + *modifier as i16).clamp(1, 30) as u8;
        }

        Effect::SpellSlotRestored { level, .. } => {
            if let Some(ref mut spellcasting) = world.player_character.spellcasting {
                if *level >= 1 && *level <= 9 {
                    let slot_idx = (*level - 1) as usize;
                    if spellcasting.spell_slots.slots[slot_idx].used > 0 {
                        spellcasting.spell_slots.slots[slot_idx].used -= 1;
                    }
                }
            }
        }
        Effect::StateAsserted {
            entity_name,
            state_type,
            new_value,
            ..
        } => {
            // Find and update the NPC
            if let Some(npc) = world
                .npcs
                .values_mut()
                .find(|npc| npc.name.eq_ignore_ascii_case(entity_name))
            {
                match state_type {
                    StateType::Disposition => {
                        // Parse disposition string to enum
                        let new_disp = match new_value.to_lowercase().as_str() {
                            "hostile" => crate::world::Disposition::Hostile,
                            "unfriendly" => crate::world::Disposition::Unfriendly,
                            "neutral" => crate::world::Disposition::Neutral,
                            "friendly" => crate::world::Disposition::Friendly,
                            "helpful" => crate::world::Disposition::Helpful,
                            _ => return, // Invalid disposition, skip
                        };
                        npc.disposition = new_disp;
                    }
                    StateType::Location => {
                        // Find or create the location
                        // For now, just store as known information since location_id requires a proper Location
                        if !npc
                            .known_information
                            .iter()
                            .any(|i| i.contains(&format!("at {}", new_value)))
                        {
                            npc.known_information
                                .push(format!("Currently at {}", new_value));
                        }
                    }
                    StateType::Status => {
                        // NPC struct doesn't have a status field, store as known information
                        if !npc.known_information.iter().any(|i| i.contains("Status:")) {
                            npc.known_information.retain(|i| !i.starts_with("Status:"));
                        }
                        npc.known_information.push(format!("Status: {}", new_value));
                    }
                    StateType::Knowledge => {
                        // Add to known_information if not already present
                        if !npc.known_information.contains(new_value) {
                            npc.known_information.push(new_value.clone());
                        }
                    }
                    StateType::Relationship => {
                        // Relationships are tracked in story memory, not NPC struct directly
                        // The effect is recorded for story memory to process
                    }
                }
            }
        }
        Effect::KnowledgeShared {
            knowing_entity,
            content,
            ..
        } => {
            // Store in NPC known_information if the entity is an NPC
            if let Some(npc) = world
                .npcs
                .values_mut()
                .find(|npc| npc.name.eq_ignore_ascii_case(knowing_entity))
            {
                if !npc.known_information.contains(content) {
                    npc.known_information.push(content.clone());
                }
            }
            // Note: Full knowledge tracking (verification, source) is handled
            // in story_memory by the DM agent
        }

        // Scheduled event effects - these are primarily managed by StoryMemory,
        // not GameWorld. The actual scheduling is done by the DM agent.
        Effect::EventScheduled { .. }
        | Effect::EventCancelled { .. }
        | Effect::EventTriggered { .. } => {
            // No GameWorld state changes needed - StoryMemory handles these
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::create_sample_fighter;

    #[test]
    fn test_skill_check() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::SkillCheck {
            character_id: world.player_character.id,
            skill: Skill::Athletics,
            dc: 15,
            advantage: Advantage::Normal,
            description: "Climbing a cliff".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(!resolution.effects.is_empty());
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_damage() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 10,
            damage_type: DamageType::Slashing,
            source: "Goblin".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { amount, .. } if *amount == -10)));
    }

    #[test]
    fn test_heal() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 10;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Heal {
            target_id: world.player_character.id,
            amount: 5,
            source: "Healing Potion".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::HpChanged { amount, .. } if *amount == 5)));
    }

    #[test]
    fn test_apply_damage_effect() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        let initial_hp = world.player_character.hit_points.current;

        let effect = Effect::HpChanged {
            target_id: world.player_character.id,
            amount: -10,
            new_current: initial_hp - 10,
            new_max: world.player_character.hit_points.maximum,
            dropped_to_zero: false,
        };

        apply_effect(&mut world, &effect);
        assert_eq!(world.player_character.hit_points.current, initial_hp - 10);
    }

    #[test]
    fn test_healing_removes_unconscious() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let mut world = GameWorld::new("Test", character);

        // Verify character is unconscious
        assert!(world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == Condition::Unconscious));

        // Apply healing effect
        let effect = Effect::HpChanged {
            target_id: world.player_character.id,
            amount: 5,
            new_current: 5,
            new_max: world.player_character.hit_points.maximum,
            dropped_to_zero: false,
        };
        apply_effect(&mut world, &effect);

        // Verify unconscious is removed
        assert!(!world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == Condition::Unconscious));
        assert_eq!(world.player_character.hit_points.current, 5);
    }

    #[test]
    fn test_massive_damage_detection() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 10;
        character.hit_points.maximum = 30;
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Damage that would cause instant death (10 current + 30 max = need 40+ damage)
        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 50,
            damage_type: DamageType::Slashing,
            source: "Dragon".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        // Should mention instant death in the narrative
        assert!(resolution.narrative.contains("INSTANT DEATH"));
    }

    #[test]
    fn test_start_combat() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let intent = Intent::StartCombat {
            combatants: vec![CombatantInit {
                id: character.id,
                name: "Roland".to_string(),
                is_player: true,
                is_ally: true,
                current_hp: character.hit_points.current,
                max_hp: character.hit_points.maximum,
                armor_class: character.current_ac(),
                initiative_modifier: character.initiative_modifier(),
            }],
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CombatStarted)));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::InitiativeRolled { .. })));
    }

    #[test]
    fn test_roll_dice() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::RollDice {
            notation: "2d6+3".to_string(),
            purpose: "Damage".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_damage_narrative_includes_hp_status() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 5,
            damage_type: DamageType::Slashing,
            source: "Goblin".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        // Narrative should include HP status
        assert!(
            resolution.narrative.contains("HP:"),
            "Damage narrative should include HP status: {}",
            resolution.narrative
        );
    }

    #[test]
    fn test_damage_narrative_shows_unconscious() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 5; // Low HP
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        let intent = Intent::Damage {
            target_id: world.player_character.id,
            amount: 10, // More than current HP
            damage_type: DamageType::Slashing,
            source: "Goblin".to_string(),
        };

        let resolution = engine.resolve(&world, intent);
        // Narrative should indicate unconscious
        assert!(
            resolution.narrative.contains("UNCONSCIOUS"),
            "Lethal damage narrative should indicate UNCONSCIOUS: {}",
            resolution.narrative
        );
        // Effect should have dropped_to_zero
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::HpChanged {
                dropped_to_zero: true,
                ..
            }
        )));
    }

    #[test]
    fn test_short_rest_blocked_during_combat() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Start combat (just need combat state to exist)
        world.combat = Some(crate::world::CombatState::new());

        let resolution = engine.resolve(&world, Intent::ShortRest);
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("Cannot take a short rest"));
    }

    #[test]
    fn test_long_rest_blocked_during_combat() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // Start combat (just need combat state to exist)
        world.combat = Some(crate::world::CombatState::new());

        let resolution = engine.resolve(&world, Intent::LongRest);
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("Cannot take a long rest"));
    }

    #[test]
    fn test_rest_allowed_outside_combat() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test", character);
        let engine = RulesEngine::new();

        // No combat active
        assert!(world.combat.is_none());

        let short_rest = engine.resolve(&world, Intent::ShortRest);
        assert!(!short_rest.effects.is_empty());
        assert!(short_rest.effects.iter().any(|e| matches!(
            e,
            Effect::RestCompleted {
                rest_type: RestType::Short
            }
        )));

        let long_rest = engine.resolve(&world, Intent::LongRest);
        assert!(!long_rest.effects.is_empty());
        assert!(long_rest.effects.iter().any(|e| matches!(
            e,
            Effect::RestCompleted {
                rest_type: RestType::Long
            }
        )));
    }

    #[test]
    fn test_unconscious_cannot_attack() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let resolution = engine.resolve(
            &world,
            Intent::Attack {
                attacker_id: character.id,
                target_id: CharacterId::new(),
                weapon_name: "Longsword".to_string(),
                advantage: Advantage::Normal,
            },
        );

        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("unconscious"));
        assert!(resolution.narrative.contains("cannot attack"));
    }

    #[test]
    fn test_unconscious_auto_fails_str_dex_checks() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Athletics is a Strength skill - should auto-fail
        let athletics_check = engine.resolve(
            &world,
            Intent::SkillCheck {
                character_id: character.id,
                skill: Skill::Athletics,
                dc: 10,
                advantage: Advantage::Normal,
                description: "Climbing".to_string(),
            },
        );
        assert!(athletics_check.narrative.contains("unconscious"));
        assert!(athletics_check.narrative.contains("automatically fails"));

        // Acrobatics is a Dexterity skill - should auto-fail
        let acrobatics_check = engine.resolve(
            &world,
            Intent::SkillCheck {
                character_id: character.id,
                skill: Skill::Acrobatics,
                dc: 10,
                advantage: Advantage::Normal,
                description: "Tumbling".to_string(),
            },
        );
        assert!(acrobatics_check.narrative.contains("unconscious"));
        assert!(acrobatics_check.narrative.contains("automatically fails"));

        // Perception is a Wisdom skill - should NOT auto-fail
        let perception_check = engine.resolve(
            &world,
            Intent::SkillCheck {
                character_id: character.id,
                skill: Skill::Perception,
                dc: 10,
                advantage: Advantage::Normal,
                description: "Noticing".to_string(),
            },
        );
        // Should actually roll (won't auto-fail since it's Wisdom-based)
        assert!(perception_check
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_unconscious_auto_fails_str_dex_saves() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Dexterity save - should auto-fail
        let dex_save = engine.resolve(
            &world,
            Intent::SavingThrow {
                character_id: character.id,
                ability: Ability::Dexterity,
                dc: 15,
                advantage: Advantage::Normal,
                source: "Fireball".to_string(),
            },
        );
        assert!(dex_save.narrative.contains("unconscious"));
        assert!(dex_save.narrative.contains("automatically fails"));

        // Constitution save - should NOT auto-fail
        let con_save = engine.resolve(
            &world,
            Intent::SavingThrow {
                character_id: character.id,
                ability: Ability::Constitution,
                dc: 15,
                advantage: Advantage::Normal,
                source: "Poison".to_string(),
            },
        );
        // Should actually roll
        assert!(con_save
            .effects
            .iter()
            .any(|e| matches!(e, Effect::DiceRolled { .. })));
    }

    #[test]
    fn test_damage_at_zero_hp_causes_death_save_failure() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        let resolution = engine.resolve(
            &world,
            Intent::Damage {
                target_id: character.id,
                amount: 5,
                damage_type: DamageType::Slashing,
                source: "Goblin".to_string(),
            },
        );

        // Should have death save failure effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::DeathSaveFailure {
                failures: 1,
                total_failures: 1,
                ..
            }
        )));
        assert!(resolution.narrative.contains("death save failure"));
    }

    #[test]
    fn test_massive_damage_at_zero_hp_causes_instant_death() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character.hit_points.maximum = 30;
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Damage >= max HP while at 0 HP = instant death
        let resolution = engine.resolve(
            &world,
            Intent::Damage {
                target_id: character.id,
                amount: 30,
                damage_type: DamageType::Slashing,
                source: "Dragon".to_string(),
            },
        );

        // Should have character died effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CharacterDied { .. })));
        assert!(resolution.narrative.contains("INSTANT DEATH"));
    }

    #[test]
    fn test_healing_resets_death_saves() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character.death_saves.failures = 2; // 2 failures already
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let mut world = GameWorld::new("Test", character);

        // Apply healing effect
        let effect = Effect::HpChanged {
            target_id: world.player_character.id,
            amount: 5,
            new_current: 5,
            new_max: world.player_character.hit_points.maximum,
            dropped_to_zero: false,
        };
        apply_effect(&mut world, &effect);

        // Death saves should be reset
        assert_eq!(world.player_character.death_saves.failures, 0);
        assert_eq!(world.player_character.death_saves.successes, 0);
        // Unconscious should be removed
        assert!(!world
            .player_character
            .conditions
            .iter()
            .any(|c| c.condition == Condition::Unconscious));
    }

    #[test]
    fn test_three_death_save_failures_causes_death() {
        let mut character = create_sample_fighter("Roland");
        character.hit_points.current = 0;
        character.death_saves.failures = 2; // Already have 2 failures
        character
            .conditions
            .push(crate::world::ActiveCondition::new(
                Condition::Unconscious,
                "Dropped to 0 HP",
            ));
        let world = GameWorld::new("Test", character.clone());
        let engine = RulesEngine::new();

        // Take damage at 0 HP - should cause 3rd failure and death
        let resolution = engine.resolve(
            &world,
            Intent::Damage {
                target_id: character.id,
                amount: 5,
                damage_type: DamageType::Slashing,
                source: "Goblin".to_string(),
            },
        );

        // Should have both death save failure and character died effects
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::DeathSaveFailure {
                total_failures: 3,
                ..
            }
        )));
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CharacterDied { .. })));
        assert!(resolution.narrative.contains("DIES"));
    }

    // ========================================================================
    // World Building Tool Tests
    // ========================================================================

    // ------------------------------------------------------------------------
    // NPC Tools
    // ------------------------------------------------------------------------

    #[test]
    fn test_create_npc() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Verify world starts with no NPCs
        assert!(world.npcs.is_empty());

        let intent = Intent::CreateNpc {
            name: "Gundren Rockseeker".to_string(),
            description: "A grizzled dwarf merchant".to_string(),
            personality: "Gruff but loyal".to_string(),
            occupation: Some("Merchant".to_string()),
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec!["Has a map to Wave Echo Cave".to_string()],
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the NpcCreated effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::NpcCreated { name, .. } if name == "Gundren Rockseeker")));

        // Apply effects and verify the NPC was added
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(world.npcs.len(), 1);

        let npc = world.npcs.values().next().unwrap();
        assert_eq!(npc.name, "Gundren Rockseeker");
    }

    #[test]
    fn test_create_npc_duplicate_detection() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create an NPC
        let intent = Intent::CreateNpc {
            name: "Thorek Ironbrew".to_string(),
            description: "A grizzled dwarf innkeeper".to_string(),
            personality: "Gruff but fair".to_string(),
            occupation: Some("Innkeeper".to_string()),
            disposition: "Neutral".to_string(),
            location: Some("The Crossroads Inn".to_string()),
            known_information: vec![],
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(world.npcs.len(), 1);

        // Try to create the same NPC again (should fail with helpful message)
        let duplicate_intent = Intent::CreateNpc {
            name: "Thorek Ironbrew".to_string(), // Same name
            description: "A different description".to_string(),
            personality: "Different personality".to_string(),
            occupation: Some("Barkeep".to_string()),
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec![],
        };

        let duplicate_resolution = engine.resolve(&world, duplicate_intent);

        // Verify no NpcCreated effect was produced
        assert!(
            !duplicate_resolution
                .effects
                .iter()
                .any(|e| matches!(e, Effect::NpcCreated { .. })),
            "Should not produce NpcCreated effect for duplicate NPC"
        );

        // Verify the narrative contains the error message
        assert!(
            duplicate_resolution
                .narrative
                .contains("DUPLICATE NPC ERROR"),
            "Should contain DUPLICATE NPC ERROR in narrative"
        );
        assert!(
            duplicate_resolution.narrative.contains("update_npc"),
            "Should suggest using update_npc"
        );

        // Verify NPC count didn't change
        apply_effects(&mut world, &duplicate_resolution.effects);
        assert_eq!(world.npcs.len(), 1, "NPC count should not increase");
    }

    #[test]
    fn test_create_npc_duplicate_case_insensitive() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create an NPC
        let intent = Intent::CreateNpc {
            name: "Mira the Smith".to_string(),
            description: "A skilled blacksmith".to_string(),
            personality: "Hardworking".to_string(),
            occupation: Some("Blacksmith".to_string()),
            disposition: "Neutral".to_string(),
            location: None,
            known_information: vec![],
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);

        // Try with different casing (should still detect as duplicate)
        let duplicate_intent = Intent::CreateNpc {
            name: "MIRA THE SMITH".to_string(), // Different case
            description: "Another description".to_string(),
            personality: "Different".to_string(),
            occupation: None,
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec![],
        };

        let duplicate_resolution = engine.resolve(&world, duplicate_intent);

        // Should detect as duplicate (case-insensitive)
        assert!(
            duplicate_resolution
                .narrative
                .contains("DUPLICATE NPC ERROR"),
            "Should detect case-insensitive duplicate"
        );
    }

    #[test]
    fn test_update_npc_disposition() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // First create an NPC
        let create_intent = Intent::CreateNpc {
            name: "Sildar Hallwinter".to_string(),
            description: "A human warrior".to_string(),
            personality: "Honorable and brave".to_string(),
            occupation: Some("Knight".to_string()),
            disposition: "Neutral".to_string(),
            location: None,
            known_information: vec![],
        };
        let create_resolution = engine.resolve(&world, create_intent);
        apply_effects(&mut world, &create_resolution.effects);

        // Now update the NPC's disposition
        let update_intent = Intent::UpdateNpc {
            npc_name: "Sildar Hallwinter".to_string(),
            disposition: Some("Friendly".to_string()),
            add_information: vec!["Knows about the Redbrands".to_string()],
            new_description: None,
            new_personality: None,
        };

        let update_resolution = engine.resolve(&world, update_intent);

        // Verify we got the NpcUpdated effect
        assert!(update_resolution.effects.iter().any(|e| matches!(
            e,
            Effect::NpcUpdated { npc_name, changes }
            if npc_name == "Sildar Hallwinter" && changes.contains("disposition changed")
        )));

        // Verify narrative mentions the update
        assert!(update_resolution.narrative.contains("Sildar Hallwinter"));
        assert!(update_resolution.narrative.contains("updated"));
    }

    #[test]
    fn test_update_nonexistent_npc() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to update an NPC that doesn't exist
        let update_intent = Intent::UpdateNpc {
            npc_name: "Ghost NPC".to_string(),
            disposition: Some("Hostile".to_string()),
            add_information: vec![],
            new_description: None,
            new_personality: None,
        };

        let resolution = engine.resolve(&world, update_intent);

        // Should have no effects (NPC not found)
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("not found"));
    }

    #[test]
    fn test_move_npc() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create a destination location
        let tavern = Location::new("The Sleeping Giant", LocationType::Building);
        let tavern_id = tavern.id;
        world.known_locations.insert(tavern_id, tavern);

        // Create an NPC
        let create_intent = Intent::CreateNpc {
            name: "Barthen".to_string(),
            description: "A shopkeeper".to_string(),
            personality: "Helpful".to_string(),
            occupation: Some("Shopkeeper".to_string()),
            disposition: "Friendly".to_string(),
            location: None,
            known_information: vec![],
        };
        let create_resolution = engine.resolve(&world, create_intent);
        apply_effects(&mut world, &create_resolution.effects);

        // Move the NPC to the tavern
        let move_intent = Intent::MoveNpc {
            npc_name: "Barthen".to_string(),
            destination: "The Sleeping Giant".to_string(),
            reason: Some("Going for a drink".to_string()),
        };

        let move_resolution = engine.resolve(&world, move_intent);

        // Verify we got the NpcMoved effect
        assert!(move_resolution.effects.iter().any(|e| matches!(
            e,
            Effect::NpcMoved { npc_name, to_location, .. }
            if npc_name == "Barthen" && to_location == "The Sleeping Giant"
        )));

        // Apply effects and verify the NPC's location changed
        apply_effects(&mut world, &move_resolution.effects);

        let npc = world.npcs.values().find(|n| n.name == "Barthen").unwrap();
        assert_eq!(npc.location_id, Some(tavern_id));
    }

    #[test]
    fn test_remove_npc() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create an NPC
        let create_intent = Intent::CreateNpc {
            name: "Glasstaff".to_string(),
            description: "A villainous wizard".to_string(),
            personality: "Cunning and cruel".to_string(),
            occupation: Some("Wizard".to_string()),
            disposition: "Hostile".to_string(),
            location: None,
            known_information: vec![],
        };
        let create_resolution = engine.resolve(&world, create_intent);
        apply_effects(&mut world, &create_resolution.effects);

        assert_eq!(world.npcs.len(), 1);

        // Remove the NPC
        let remove_intent = Intent::RemoveNpc {
            npc_name: "Glasstaff".to_string(),
            reason: "Defeated by the party".to_string(),
            permanent: true,
        };

        let remove_resolution = engine.resolve(&world, remove_intent);

        // Verify we got the NpcRemoved effect
        assert!(remove_resolution.effects.iter().any(|e| matches!(
            e,
            Effect::NpcRemoved { npc_name, reason }
            if npc_name == "Glasstaff" && reason == "Defeated by the party"
        )));

        // Apply effects and verify the NPC was removed
        apply_effects(&mut world, &remove_resolution.effects);
        assert!(world.npcs.is_empty());
    }

    // ------------------------------------------------------------------------
    // Location Tools
    // ------------------------------------------------------------------------

    #[test]
    fn test_create_location() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // World starts with the starting location
        let initial_location_count = world.known_locations.len();

        let intent = Intent::CreateLocation {
            name: "Phandalin".to_string(),
            location_type: "Town".to_string(),
            description: "A small frontier town".to_string(),
            parent_location: None,
            items: vec!["Notice Board".to_string()],
            npcs_present: vec![],
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the LocationCreated effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::LocationCreated { name, location_type }
            if name == "Phandalin" && location_type == "Town"
        )));

        // Apply effects and verify the location was added
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(world.known_locations.len(), initial_location_count + 1);

        // Find the new location
        let phandalin = world
            .known_locations
            .values()
            .find(|l| l.name == "Phandalin");
        assert!(phandalin.is_some());
    }

    #[test]
    fn test_connect_locations() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create two locations
        let town = Location::new("Phandalin", LocationType::Town);
        let town_id = town.id;
        world.known_locations.insert(town_id, town);

        let cave = Location::new("Wave Echo Cave", LocationType::Cave);
        let cave_id = cave.id;
        world.known_locations.insert(cave_id, cave);

        // Connect them
        let intent = Intent::ConnectLocations {
            from_location: "Phandalin".to_string(),
            to_location: "Wave Echo Cave".to_string(),
            direction: Some("east".to_string()),
            travel_time_minutes: Some(120),
            bidirectional: true,
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the LocationsConnected effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::LocationsConnected { from, to, direction }
            if from == "Phandalin" && to == "Wave Echo Cave" && direction.as_deref() == Some("east")
        )));

        // Apply effects and verify the connection was added
        apply_effects(&mut world, &resolution.effects);

        let phandalin = world.known_locations.get(&town_id).unwrap();
        assert_eq!(phandalin.connections.len(), 1);
        assert_eq!(phandalin.connections[0].destination_id, cave_id);
        assert_eq!(phandalin.connections[0].destination_name, "Wave Echo Cave");
        assert_eq!(phandalin.connections[0].direction, Some("east".to_string()));
    }

    #[test]
    fn test_update_location() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create a location
        let tavern = Location::new("Stonehill Inn", LocationType::Building);
        world.known_locations.insert(tavern.id, tavern);

        // Update the location
        let intent = Intent::UpdateLocation {
            location_name: "Stonehill Inn".to_string(),
            new_description: Some("A cozy inn with warm hearth".to_string()),
            add_items: vec!["Ale Barrel".to_string(), "Fireplace".to_string()],
            remove_items: vec![],
            add_npcs: vec!["Toblen Stonehill".to_string()],
            remove_npcs: vec![],
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the LocationUpdated effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::LocationUpdated { location_name, changes }
            if location_name == "Stonehill Inn" && changes.contains("description updated")
        )));

        // Verify narrative mentions the changes
        assert!(resolution.narrative.contains("Stonehill Inn"));
        assert!(resolution.narrative.contains("updated"));
    }

    #[test]
    fn test_update_nonexistent_location() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to update a location that doesn't exist
        let intent = Intent::UpdateLocation {
            location_name: "Ghost Town".to_string(),
            new_description: Some("A spooky place".to_string()),
            add_items: vec![],
            remove_items: vec![],
            add_npcs: vec![],
            remove_npcs: vec![],
        };

        let resolution = engine.resolve(&world, intent);

        // Should have no effects (location not found)
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("not found"));
    }

    // ------------------------------------------------------------------------
    // Gameplay Tools
    // ------------------------------------------------------------------------

    #[test]
    fn test_modify_ability_score() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Get initial strength
        let initial_strength = world.player_character.ability_scores.strength;

        let intent = Intent::ModifyAbilityScore {
            ability: Ability::Strength,
            modifier: 2,
            source: "Bull's Strength".to_string(),
            duration: Some("1 hour".to_string()),
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the AbilityScoreModified effect
        assert!(resolution.effects.iter().any(|e| matches!(
            e,
            Effect::AbilityScoreModified { ability, modifier, source }
            if *ability == Ability::Strength && *modifier == 2 && source == "Bull's Strength"
        )));

        // Apply effects and verify the ability score changed
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(
            world.player_character.ability_scores.strength,
            initial_strength + 2
        );
    }

    #[test]
    fn test_modify_ability_score_negative() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Get initial dexterity
        let initial_dexterity = world.player_character.ability_scores.dexterity;

        let intent = Intent::ModifyAbilityScore {
            ability: Ability::Dexterity,
            modifier: -4,
            source: "Ray of Enfeeblement".to_string(),
            duration: Some("1 minute".to_string()),
        };

        let resolution = engine.resolve(&world, intent);

        // Apply effects and verify the ability score decreased
        apply_effects(&mut world, &resolution.effects);
        assert_eq!(
            world.player_character.ability_scores.dexterity,
            initial_dexterity - 4
        );
    }

    #[test]
    fn test_modify_ability_score_clamped() {
        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to reduce intelligence below 1
        let intent = Intent::ModifyAbilityScore {
            ability: Ability::Intelligence,
            modifier: -20, // Character has 10 INT, this would make it -10
            source: "Feeblemind".to_string(),
            duration: None,
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);

        // Should be clamped to minimum of 1
        assert_eq!(world.player_character.ability_scores.intelligence, 1);
    }

    #[test]
    fn test_restore_spell_slot() {
        use crate::world::{Ability, SlotInfo, SpellSlots, SpellcastingData};

        let mut character = create_sample_fighter("Roland");
        // Give the fighter some spellcasting (e.g., Eldritch Knight)
        character.spellcasting = Some(SpellcastingData {
            ability: Ability::Intelligence,
            spells_known: vec!["Shield".to_string()],
            spells_prepared: vec![],
            cantrips_known: vec!["Fire Bolt".to_string()],
            spell_slots: SpellSlots {
                slots: [
                    SlotInfo { total: 3, used: 2 }, // Level 1: 1 remaining
                    SlotInfo { total: 2, used: 2 }, // Level 2: 0 remaining
                    SlotInfo { total: 0, used: 0 }, // Level 3
                    SlotInfo { total: 0, used: 0 }, // Level 4
                    SlotInfo { total: 0, used: 0 }, // Level 5
                    SlotInfo { total: 0, used: 0 }, // Level 6
                    SlotInfo { total: 0, used: 0 }, // Level 7
                    SlotInfo { total: 0, used: 0 }, // Level 8
                    SlotInfo { total: 0, used: 0 }, // Level 9
                ],
            },
        });

        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Get initial available slots
        let initial_level1_used = world
            .player_character
            .spellcasting
            .as_ref()
            .unwrap()
            .spell_slots
            .slots[0]
            .used;

        let intent = Intent::RestoreSpellSlot {
            slot_level: 1,
            source: "Arcane Recovery".to_string(),
        };

        let resolution = engine.resolve(&world, intent);

        // Verify we got the SpellSlotRestored effect
        assert!(resolution
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SpellSlotRestored { level: 1, .. })));

        // Apply effects and verify the spell slot was restored
        apply_effects(&mut world, &resolution.effects);

        let new_level1_used = world
            .player_character
            .spellcasting
            .as_ref()
            .unwrap()
            .spell_slots
            .slots[0]
            .used;

        assert_eq!(new_level1_used, initial_level1_used - 1);
    }

    #[test]
    fn test_restore_spell_slot_invalid_level() {
        let character = create_sample_fighter("Roland");
        let world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Try to restore an invalid spell slot level
        let intent = Intent::RestoreSpellSlot {
            slot_level: 10, // Invalid: max is 9
            source: "Invalid Source".to_string(),
        };

        let resolution = engine.resolve(&world, intent);

        // Should have no effects (invalid level)
        assert!(resolution.effects.is_empty());
        assert!(resolution.narrative.contains("Invalid"));
    }

    #[test]
    fn test_create_npc_with_location() {
        use crate::world::{Location, LocationType};

        let character = create_sample_fighter("Roland");
        let mut world = GameWorld::new("Test Campaign", character);
        let engine = RulesEngine::new();

        // Create a location first
        let tavern = Location::new("The Yawning Portal", LocationType::Building);
        let tavern_id = tavern.id;
        world.known_locations.insert(tavern_id, tavern);

        // Create an NPC at that location
        let intent = Intent::CreateNpc {
            name: "Durnan".to_string(),
            description: "A retired adventurer and tavern keeper".to_string(),
            personality: "Stoic and mysterious".to_string(),
            occupation: Some("Tavern Keeper".to_string()),
            disposition: "Neutral".to_string(),
            location: Some("The Yawning Portal".to_string()),
            known_information: vec!["Knows about Undermountain".to_string()],
        };

        let resolution = engine.resolve(&world, intent);
        apply_effects(&mut world, &resolution.effects);

        // Verify NPC was created at the correct location
        let npc = world.npcs.values().find(|n| n.name == "Durnan").unwrap();
        assert_eq!(npc.location_id, Some(tavern_id));
    }
}

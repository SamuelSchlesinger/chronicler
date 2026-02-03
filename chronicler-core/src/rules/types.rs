//! Core types for the Intent/Effect rules system.

use crate::dice::RollResult;
use crate::world::{Ability, CharacterId, Condition, Skill};
use serde::{Deserialize, Serialize};

/// An intent represents what a character wants to do.
/// The AI generates intents, the RulesEngine resolves them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// Attack a target with a weapon
    Attack {
        attacker_id: CharacterId,
        target_id: CharacterId,
        weapon_name: String,
        advantage: crate::dice::Advantage,
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
        advantage: crate::dice::Advantage,
        description: String,
    },

    /// Make an ability check (raw ability, not skill)
    AbilityCheck {
        character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: crate::dice::Advantage,
        description: String,
    },

    /// Make a saving throw
    SavingThrow {
        character_id: CharacterId,
        ability: Ability,
        dc: i32,
        advantage: crate::dice::Advantage,
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

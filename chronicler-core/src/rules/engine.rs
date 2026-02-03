//! The RulesEngine struct and main resolve() dispatch method.

use crate::rules::types::{Intent, Resolution};
use crate::world::GameWorld;

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
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

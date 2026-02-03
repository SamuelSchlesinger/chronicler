//! Inventory management resolution methods.

use crate::rules::helpers::roll_with_fallback;
use crate::rules::types::{Effect, Resolution};
use crate::rules::RulesEngine;
use crate::world::{CharacterId, Condition, GameWorld, ItemType};

impl RulesEngine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_add_item(
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

    pub(crate) fn resolve_remove_item(
        &self,
        world: &GameWorld,
        item_name: &str,
        quantity: u32,
    ) -> Resolution {
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

    pub(crate) fn resolve_equip_item(&self, world: &GameWorld, item_name: &str) -> Resolution {
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

    pub(crate) fn resolve_unequip_item(&self, world: &GameWorld, slot: &str) -> Resolution {
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

    pub(crate) fn resolve_use_item(
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

    pub(crate) fn resolve_adjust_gold(
        &self,
        world: &GameWorld,
        amount: i32,
        reason: &str,
    ) -> Resolution {
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

    pub(crate) fn resolve_adjust_silver(
        &self,
        world: &GameWorld,
        amount: i32,
        reason: &str,
    ) -> Resolution {
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
}

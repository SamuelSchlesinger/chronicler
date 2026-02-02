//! Combat-related tools: damage, healing, conditions, and combat flow.

use claude::Tool;
use serde_json::json;

/// Apply damage to a character or creature.
pub fn apply_damage() -> Tool {
    Tool {
        name: "apply_damage".to_string(),
        description: "Apply damage to a character or creature.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "Amount of damage to apply"
                },
                "damage_type": {
                    "type": "string",
                    "enum": ["slashing", "piercing", "bludgeoning", "fire", "cold",
                            "lightning", "thunder", "acid", "poison", "necrotic",
                            "radiant", "force", "psychic"],
                    "description": "Type of damage"
                },
                "source": {
                    "type": "string",
                    "description": "Source of the damage"
                },
                "target": {
                    "type": "string",
                    "enum": ["player", "npc"],
                    "description": "Who receives the damage"
                }
            },
            "required": ["amount", "damage_type", "source"]
        }),
    }
}

/// Heal a character.
pub fn apply_healing() -> Tool {
    Tool {
        name: "apply_healing".to_string(),
        description: "Heal a character.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "Amount of HP to restore"
                },
                "source": {
                    "type": "string",
                    "description": "Source of the healing"
                }
            },
            "required": ["amount", "source"]
        }),
    }
}

/// Apply a condition to a character.
pub fn apply_condition() -> Tool {
    Tool {
        name: "apply_condition".to_string(),
        description: "Apply a condition to a character.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "condition": {
                    "type": "string",
                    "enum": ["blinded", "charmed", "deafened", "frightened", "grappled",
                            "incapacitated", "invisible", "paralyzed", "petrified", "poisoned",
                            "prone", "restrained", "stunned", "unconscious"],
                    "description": "The condition to apply"
                },
                "source": {
                    "type": "string",
                    "description": "What caused the condition"
                },
                "duration_rounds": {
                    "type": "integer",
                    "description": "How many rounds the condition lasts (omit for indefinite)"
                }
            },
            "required": ["condition", "source"]
        }),
    }
}

/// Remove a condition from a character.
pub fn remove_condition() -> Tool {
    Tool {
        name: "remove_condition".to_string(),
        description: "Remove a condition from a character.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "condition": {
                    "type": "string",
                    "enum": ["blinded", "charmed", "deafened", "frightened", "grappled",
                            "incapacitated", "invisible", "paralyzed", "petrified", "poisoned",
                            "prone", "restrained", "stunned", "unconscious"],
                    "description": "The condition to remove"
                }
            },
            "required": ["condition"]
        }),
    }
}

/// Start a combat encounter.
pub fn start_combat() -> Tool {
    Tool {
        name: "start_combat".to_string(),
        description: "Start a combat encounter. Initiative will be rolled for all combatants. Provide enemy stats based on D&D 5e SRD creatures."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "enemies": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Enemy name (e.g., 'Goblin', 'Orc', 'Wolf')"
                            },
                            "max_hp": {
                                "type": "integer",
                                "description": "Maximum hit points (e.g., Goblin: 7, Orc: 15, Wolf: 11)"
                            },
                            "armor_class": {
                                "type": "integer",
                                "description": "Armor class (e.g., Goblin: 15, Orc: 13, Wolf: 13)"
                            },
                            "initiative_modifier": {
                                "type": "integer",
                                "description": "Initiative modifier based on DEX (e.g., Goblin: +2, Orc: +1, Wolf: +2)"
                            }
                        },
                        "required": ["name"]
                    },
                    "description": "List of enemy combatants with their stats"
                }
            },
            "required": ["enemies"]
        }),
    }
}

/// End the current combat encounter.
pub fn end_combat() -> Tool {
    Tool {
        name: "end_combat".to_string(),
        description: "End the current combat encounter.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

/// Advance to the next turn in combat.
pub fn next_turn() -> Tool {
    Tool {
        name: "next_turn".to_string(),
        description: "Advance to the next turn in combat.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

/// Make a death saving throw.
pub fn death_save() -> Tool {
    Tool {
        name: "death_save".to_string(),
        description: "Make a death saving throw for a character at 0 HP. Roll d20: 10+ = success, <10 = failure, nat 20 = regain 1 HP, nat 1 = 2 failures. 3 successes = stable, 3 failures = death.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

/// Make a concentration check.
pub fn concentration_check() -> Tool {
    Tool {
        name: "concentration_check".to_string(),
        description: "Make a concentration check when a concentrating spellcaster takes damage. DC = max(10, damage/2). CON save to maintain concentration.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "damage_taken": {
                    "type": "integer",
                    "description": "Amount of damage taken that triggered the check"
                },
                "spell_name": {
                    "type": "string",
                    "description": "Name of the spell being concentrated on"
                }
            },
            "required": ["damage_taken", "spell_name"]
        }),
    }
}

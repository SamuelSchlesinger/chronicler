//! Relevance checking for surfacing appropriate context.
//!
//! Uses a fast, cheap model (Haiku) to determine which stored consequences
//! and facts are relevant to the current player input, enabling semantic
//! matching instead of just keyword matching.

use super::story_memory::{ConsequenceId, EntityId, FactId, StoryMemory};
use claude::{Claude, Message, Request};
use serde::Deserialize;
use thiserror::Error;

/// Default model for relevance checking (fast and cheap).
const RELEVANCE_MODEL: &str = "claude-3-5-haiku-20241022";

/// Maximum tokens for relevance check response.
const RELEVANCE_MAX_TOKENS: usize = 500;

/// Errors from relevance checking.
#[derive(Debug, Error)]
pub enum RelevanceError {
    #[error("API error: {0:?}")]
    ApiError(#[from] claude::Error),

    #[error("Failed to parse relevance response: {0}")]
    ParseError(String),
}

/// Result of a relevance check.
#[derive(Debug, Clone, Default)]
pub struct RelevanceResult {
    /// Consequence IDs that should trigger based on current context.
    pub triggered_consequences: Vec<ConsequenceId>,

    /// Fact IDs that are relevant to surface in context.
    pub relevant_facts: Vec<FactId>,

    /// Entity IDs that are relevant but weren't explicitly mentioned.
    pub relevant_entities: Vec<EntityId>,

    /// Raw explanation from the model (for debugging).
    pub explanation: Option<String>,
}

impl RelevanceResult {
    /// Check if any consequences were triggered.
    pub fn has_triggered_consequences(&self) -> bool {
        !self.triggered_consequences.is_empty()
    }

    /// Check if any relevant context was found.
    pub fn has_relevant_context(&self) -> bool {
        !self.relevant_facts.is_empty() || !self.relevant_entities.is_empty()
    }

    /// Check if this result is empty (nothing relevant found).
    pub fn is_empty(&self) -> bool {
        self.triggered_consequences.is_empty()
            && self.relevant_facts.is_empty()
            && self.relevant_entities.is_empty()
    }
}

/// Response format we expect from Haiku.
#[derive(Debug, Deserialize)]
struct RelevanceResponse {
    #[serde(default)]
    triggered_consequences: Vec<String>,
    #[serde(default)]
    relevant_entities: Vec<String>,
    #[serde(default)]
    explanation: Option<String>,
}

/// Checks relevance of stored consequences and facts against player input.
pub struct RelevanceChecker {
    client: Claude,
    model: String,
}

impl RelevanceChecker {
    /// Create a new relevance checker with the given API client.
    pub fn new(client: Claude) -> Self {
        Self {
            client,
            model: RELEVANCE_MODEL.to_string(),
        }
    }

    /// Create from environment (ANTHROPIC_API_KEY).
    pub fn from_env() -> Result<Self, claude::Error> {
        let client = Claude::from_env()?;
        Ok(Self::new(client))
    }

    /// Set a custom model for relevance checking.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Check relevance of stored context against player input.
    ///
    /// This uses a fast model (Haiku) to determine:
    /// 1. Which pending consequences should trigger
    /// 2. Which entities are semantically relevant (even if not mentioned by name)
    pub async fn check_relevance(
        &self,
        player_input: &str,
        current_location: &str,
        story_memory: &StoryMemory,
    ) -> Result<RelevanceResult, RelevanceError> {
        // Get pending consequences
        let consequences = story_memory.pending_consequences_by_importance();

        // If no consequences to check, return early
        if consequences.is_empty() {
            return Ok(RelevanceResult::default());
        }

        // Build the consequences list for the prompt
        let consequences_text = story_memory.build_consequences_for_relevance();

        // Build the prompt
        let prompt = format!(
            r#"You are checking if any pending consequences should trigger based on a player's action in a D&D game.

## Player Action
"{player_input}"

## Current Location
{current_location}

## Pending Consequences
{consequences_text}

## Instructions
Analyze the player's action and determine:
1. Which consequences (if any) should TRIGGER based on this action
2. Which entities/NPCs might be relevant even if not explicitly mentioned

A consequence should trigger if the player's action matches or is closely related to its trigger condition. Be generous with semantic matching - "I enter the village" should trigger a consequence about "entering Riverside" if Riverside is a village.

Respond with ONLY a JSON object (no markdown, no explanation outside the JSON):
{{
  "triggered_consequences": ["id1", "id2"],
  "relevant_entities": ["Baron Aldric", "Town Guards"],
  "explanation": "Brief explanation of matches"
}}

If nothing is relevant, return empty arrays."#
        );

        // Make the API call
        let request = Request::new(vec![Message::user(&prompt)])
            .with_model(&self.model)
            .with_max_tokens(RELEVANCE_MAX_TOKENS)
            .with_temperature(0.0); // Deterministic for relevance checking

        let response = self.client.complete(request).await?;
        let response_text = response.text();

        // Parse the response
        self.parse_response(&response_text, story_memory)
    }

    /// Parse the Haiku response into a RelevanceResult.
    fn parse_response(
        &self,
        response: &str,
        story_memory: &StoryMemory,
    ) -> Result<RelevanceResult, RelevanceError> {
        // Try to extract JSON from the response (handle potential markdown wrapping)
        let json_str = extract_json(response);

        // Parse the JSON
        let parsed: RelevanceResponse = serde_json::from_str(json_str)
            .map_err(|e| RelevanceError::ParseError(format!("{e}: {json_str}")))?;

        // Convert string IDs back to typed IDs
        let mut result = RelevanceResult {
            explanation: parsed.explanation,
            ..Default::default()
        };

        // Parse consequence IDs
        for id_str in parsed.triggered_consequences {
            // Try to find the consequence by ID string
            for consequence in story_memory.pending_consequences() {
                if consequence.id.to_string() == id_str {
                    result.triggered_consequences.push(consequence.id);
                    break;
                }
            }
        }

        // Parse entity names to IDs
        for name in parsed.relevant_entities {
            if let Some(id) = story_memory.find_entity_id(&name) {
                if !result.relevant_entities.contains(&id) {
                    result.relevant_entities.push(id);
                }
            }
        }

        Ok(result)
    }
}

// =============================================================================
// State Inference
// =============================================================================

/// An inferred state change detected from narrative text.
#[derive(Debug, Clone)]
pub struct InferredStateChange {
    /// The entity whose state changed.
    pub entity_name: String,
    /// What type of state changed.
    pub state_type: String,
    /// The inferred new value.
    pub new_value: String,
    /// Why this was inferred (quote from narrative).
    pub evidence: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f32,
    /// Target entity for relationships.
    pub target_entity: Option<String>,
}

/// Response format for state inference.
#[derive(Debug, Deserialize)]
struct StateInferenceResponse {
    #[serde(default)]
    inferred_changes: Vec<InferredChange>,
}

#[derive(Debug, Deserialize)]
struct InferredChange {
    entity_name: String,
    state_type: String,
    new_value: String,
    evidence: String,
    confidence: f32,
    #[serde(default)]
    target_entity: Option<String>,
}

/// Infers state changes from DM narrative text.
///
/// This uses a fast model (Haiku) to detect when narrative implies state changes
/// that the DM didn't explicitly record with tools. For example:
/// - "Mira smiles warmly" → disposition changed to friendly
/// - "The guard captain storms off to the gate" → location changed
pub struct StateInferrer {
    client: Claude,
    model: String,
}

impl StateInferrer {
    /// Create a new state inferrer with the given API client.
    pub fn new(client: Claude) -> Self {
        Self {
            client,
            model: RELEVANCE_MODEL.to_string(),
        }
    }

    /// Set a custom model for state inference.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Infer state changes from a DM narrative response.
    ///
    /// Returns changes that should be applied, filtered by confidence threshold.
    pub async fn infer_state_changes(
        &self,
        narrative: &str,
        known_entities: &[String],
        confidence_threshold: f32,
    ) -> Result<Vec<InferredStateChange>, RelevanceError> {
        // Skip if narrative is too short
        if narrative.len() < 20 {
            return Ok(Vec::new());
        }

        // Skip if no entities to track
        if known_entities.is_empty() {
            return Ok(Vec::new());
        }

        let entities_list = known_entities.join(", ");

        let prompt = format!(
            r#"Analyze this D&D narrative for implied state changes that weren't explicitly recorded.

## Narrative
"{narrative}"

## Known Entities
{entities_list}

## Instructions
Look for IMPLIED state changes in the narrative:
- Disposition: attitude changes (smiles, glares, thanks warmly, becomes hostile)
- Location: movement (storms off, follows, arrives at)
- Status: condition changes (injured, recovered, disappeared)
- Relationship: connection changes (befriends, betrays, owes a debt to)

Only report changes with high confidence (>0.7). Require explicit evidence in the text.

Respond with ONLY a JSON object (no markdown):
{{
  "inferred_changes": [
    {{
      "entity_name": "Mira",
      "state_type": "disposition",
      "new_value": "friendly",
      "evidence": "She smiles warmly and thanks you",
      "confidence": 0.9,
      "target_entity": null
    }}
  ]
}}

If no state changes are implied, return empty array: {{"inferred_changes": []}}"#
        );

        let request = Request::new(vec![Message::user(&prompt)])
            .with_model(&self.model)
            .with_max_tokens(500)
            .with_temperature(0.0);

        let response = self.client.complete(request).await?;
        let response_text = response.text();

        // Parse response
        let json_str = extract_json(&response_text);
        let parsed: StateInferenceResponse = serde_json::from_str(json_str)
            .map_err(|e| RelevanceError::ParseError(format!("{e}: {json_str}")))?;

        // Filter by confidence and convert
        let changes: Vec<InferredStateChange> = parsed
            .inferred_changes
            .into_iter()
            .filter(|c| c.confidence >= confidence_threshold)
            .map(|c| InferredStateChange {
                entity_name: c.entity_name,
                state_type: c.state_type,
                new_value: c.new_value,
                evidence: c.evidence,
                confidence: c.confidence,
                target_entity: c.target_entity,
            })
            .collect();

        Ok(changes)
    }
}

/// Extract JSON from a response that might have markdown code blocks.
fn extract_json(text: &str) -> &str {
    let text = text.trim();

    // Handle ```json ... ``` blocks
    if let Some(start) = text.find("```json") {
        let content_start = start + 7;
        if let Some(end) = text[content_start..].find("```") {
            return text[content_start..content_start + end].trim();
        }
    }

    // Handle ``` ... ``` blocks (without json specifier)
    if let Some(start) = text.find("```") {
        let content_start = start + 3;
        if let Some(end) = text[content_start..].find("```") {
            return text[content_start..content_start + end].trim();
        }
    }

    // Just return the text as-is
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let text = r#"{"triggered_consequences": [], "relevant_entities": []}"#;
        assert_eq!(extract_json(text), text);
    }

    #[test]
    fn test_extract_json_markdown() {
        let text = r#"```json
{"triggered_consequences": ["abc"], "relevant_entities": ["Guard"]}
```"#;
        let expected = r#"{"triggered_consequences": ["abc"], "relevant_entities": ["Guard"]}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_markdown_no_specifier() {
        let text = r#"```
{"triggered_consequences": []}
```"#;
        let expected = r#"{"triggered_consequences": []}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_with_whitespace() {
        let text = r#"
  {"triggered_consequences": []}
  "#;
        let result = extract_json(text);
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_extract_json_nested_backticks() {
        // If there's text before the code block
        let text = r#"Here is the JSON:
```json
{"triggered_consequences": ["id1"]}
```"#;
        let expected = r#"{"triggered_consequences": ["id1"]}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_relevance_result_empty() {
        let result = RelevanceResult::default();
        assert!(result.is_empty());
        assert!(!result.has_triggered_consequences());
        assert!(!result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_consequences() {
        let result = RelevanceResult {
            triggered_consequences: vec![ConsequenceId::new()],
            relevant_facts: vec![],
            relevant_entities: vec![],
            explanation: None,
        };
        assert!(!result.is_empty());
        assert!(result.has_triggered_consequences());
        assert!(!result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_facts() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![FactId::new()],
            relevant_entities: vec![],
            explanation: None,
        };
        assert!(!result.is_empty());
        assert!(!result.has_triggered_consequences());
        assert!(result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_entities() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![],
            relevant_entities: vec![EntityId::new()],
            explanation: None,
        };
        assert!(!result.is_empty());
        assert!(!result.has_triggered_consequences());
        assert!(result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_explanation() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![],
            relevant_entities: vec![],
            explanation: Some("Test explanation".to_string()),
        };
        // Having only an explanation doesn't make it non-empty
        assert!(result.is_empty());
        assert_eq!(result.explanation, Some("Test explanation".to_string()));
    }

    #[test]
    fn test_relevance_response_parsing() {
        // Test that RelevanceResponse can be deserialized
        let json = r#"{"triggered_consequences": ["id1", "id2"], "relevant_entities": ["Guard", "King"], "explanation": "Test"}"#;
        let response: RelevanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.triggered_consequences.len(), 2);
        assert_eq!(response.relevant_entities.len(), 2);
        assert_eq!(response.explanation, Some("Test".to_string()));
    }

    #[test]
    fn test_relevance_response_defaults() {
        // Test that missing fields default correctly
        let json = r#"{}"#;
        let response: RelevanceResponse = serde_json::from_str(json).unwrap();
        assert!(response.triggered_consequences.is_empty());
        assert!(response.relevant_entities.is_empty());
        assert!(response.explanation.is_none());
    }
}

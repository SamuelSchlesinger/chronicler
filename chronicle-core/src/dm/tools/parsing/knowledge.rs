//! Parsing for knowledge tracking tools.

use crate::rules::Intent;
use serde_json::Value;

/// Parse knowledge-related tool calls.
pub fn parse_knowledge_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "share_knowledge" => {
            let knowing_entity = input["knowing_entity"].as_str()?.to_string();
            let content = input["content"].as_str()?.to_string();
            let source = input["source"].as_str()?.to_string();
            let verification = input["verification"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            let context = input["context"].as_str().map(|s| s.to_string());

            Some(Intent::ShareKnowledge {
                knowing_entity,
                content,
                source,
                verification,
                context,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_share_knowledge() {
        let input = json!({
            "knowing_entity": "Mira",
            "content": "The old mine is haunted",
            "source": "player",
            "verification": "true"
        });

        let intent = parse_knowledge_tool("share_knowledge", &input);
        assert!(intent.is_some());

        if let Some(Intent::ShareKnowledge {
            knowing_entity,
            content,
            source,
            verification,
            context,
        }) = intent
        {
            assert_eq!(knowing_entity, "Mira");
            assert_eq!(content, "The old mine is haunted");
            assert_eq!(source, "player");
            assert_eq!(verification, "true");
            assert!(context.is_none());
        } else {
            panic!("Expected ShareKnowledge intent");
        }
    }

    #[test]
    fn test_parse_share_knowledge_with_context() {
        let input = json!({
            "knowing_entity": "Guard",
            "content": "The baron is planning something",
            "source": "observation",
            "verification": "unknown",
            "context": "Overheard in the courtyard"
        });

        let intent = parse_knowledge_tool("share_knowledge", &input);
        assert!(intent.is_some());

        if let Some(Intent::ShareKnowledge { context, .. }) = intent {
            assert_eq!(context, Some("Overheard in the courtyard".to_string()));
        } else {
            panic!("Expected ShareKnowledge intent");
        }
    }

    #[test]
    fn test_parse_share_knowledge_default_verification() {
        let input = json!({
            "knowing_entity": "Mira",
            "content": "Something happened",
            "source": "player"
        });

        let intent = parse_knowledge_tool("share_knowledge", &input);
        assert!(intent.is_some());

        if let Some(Intent::ShareKnowledge { verification, .. }) = intent {
            assert_eq!(verification, "unknown");
        } else {
            panic!("Expected ShareKnowledge intent");
        }
    }
}

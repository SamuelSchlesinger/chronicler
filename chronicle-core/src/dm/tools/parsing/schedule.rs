//! Parsing for scheduled event tools.

use crate::rules::Intent;
use serde_json::Value;

/// Parse schedule-related tool calls.
pub fn parse_schedule_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "schedule_event" => {
            let description = input["description"].as_str()?.to_string();

            // Parse relative time
            let minutes = input["minutes"].as_u64().map(|v| v as u32);
            let hours = input["hours"].as_u64().map(|v| v as u32);

            // Parse absolute time
            let day = input["day"].as_u64().map(|v| v as u8);
            let month = input["month"].as_u64().map(|v| v as u8);
            let year = input["year"].as_i64().map(|v| v as i32);
            let hour = input["hour"].as_u64().map(|v| v as u8);

            // Parse daily time
            let daily_hour = input["daily_hour"].as_u64().map(|v| v as u8);
            let daily_minute = input["daily_minute"].as_u64().map(|v| v as u8);

            // Parse other fields
            let location = input["location"].as_str().map(|s| s.to_string());
            let involved_entities = input["involved_entities"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let visibility = input["visibility"].as_str().unwrap_or("public").to_string();
            let repeating = input["repeating"].as_bool().unwrap_or(false);

            // Validate that at least one time specification is provided
            let has_relative = minutes.is_some() || hours.is_some();
            let has_absolute = day.is_some() && month.is_some() && year.is_some();
            let has_daily = daily_hour.is_some();

            if !has_relative && !has_absolute && !has_daily {
                // No time specified - still valid, just won't trigger on time
                // Could be used for events triggered by other conditions
            }

            Some(Intent::ScheduleEvent {
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
                involved_entities,
                visibility,
                repeating,
            })
        }

        "cancel_event" => {
            let event_description = input["event_description"].as_str()?.to_string();
            let reason = input["reason"].as_str()?.to_string();

            Some(Intent::CancelEvent {
                event_description,
                reason,
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
    fn test_parse_schedule_event_relative() {
        let input = json!({
            "description": "Guard patrol arrives",
            "minutes": 30,
            "location": "East Gate"
        });

        let intent = parse_schedule_tool("schedule_event", &input);
        assert!(intent.is_some());

        if let Some(Intent::ScheduleEvent {
            description,
            minutes,
            location,
            ..
        }) = intent
        {
            assert_eq!(description, "Guard patrol arrives");
            assert_eq!(minutes, Some(30));
            assert_eq!(location, Some("East Gate".to_string()));
        } else {
            panic!("Expected ScheduleEvent intent");
        }
    }

    #[test]
    fn test_parse_schedule_event_absolute() {
        let input = json!({
            "description": "The festival begins",
            "day": 5,
            "month": 3,
            "year": 1492,
            "hour": 10,
            "visibility": "public"
        });

        let intent = parse_schedule_tool("schedule_event", &input);
        assert!(intent.is_some());

        if let Some(Intent::ScheduleEvent {
            description,
            day,
            month,
            year,
            hour,
            visibility,
            ..
        }) = intent
        {
            assert_eq!(description, "The festival begins");
            assert_eq!(day, Some(5));
            assert_eq!(month, Some(3));
            assert_eq!(year, Some(1492));
            assert_eq!(hour, Some(10));
            assert_eq!(visibility, "public");
        } else {
            panic!("Expected ScheduleEvent intent");
        }
    }

    #[test]
    fn test_parse_schedule_event_daily() {
        let input = json!({
            "description": "Morning market opens",
            "daily_hour": 8,
            "daily_minute": 0,
            "repeating": true,
            "location": "Town Square"
        });

        let intent = parse_schedule_tool("schedule_event", &input);
        assert!(intent.is_some());

        if let Some(Intent::ScheduleEvent {
            description,
            daily_hour,
            daily_minute,
            repeating,
            ..
        }) = intent
        {
            assert_eq!(description, "Morning market opens");
            assert_eq!(daily_hour, Some(8));
            assert_eq!(daily_minute, Some(0));
            assert!(repeating);
        } else {
            panic!("Expected ScheduleEvent intent");
        }
    }

    #[test]
    fn test_parse_cancel_event() {
        let input = json!({
            "event_description": "Meeting with the merchant",
            "reason": "The merchant has fled town"
        });

        let intent = parse_schedule_tool("cancel_event", &input);
        assert!(intent.is_some());

        if let Some(Intent::CancelEvent {
            event_description,
            reason,
        }) = intent
        {
            assert_eq!(event_description, "Meeting with the merchant");
            assert_eq!(reason, "The merchant has fled town");
        } else {
            panic!("Expected CancelEvent intent");
        }
    }
}

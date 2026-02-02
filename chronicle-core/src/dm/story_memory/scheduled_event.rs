//! Scheduled events for time-based triggers.
//!
//! This module tracks events that should trigger based on time passage,
//! not just player actions. Events can be one-time or repeating.

use serde::{Deserialize, Serialize};

/// Unique identifier for scheduled events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduledEventId(pub u32);

/// When an event should trigger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventTrigger {
    /// Trigger at a specific game time (absolute).
    AtTime {
        year: i32,
        month: u8,
        day: u8,
        hour: Option<u8>,
    },
    /// Trigger after a duration from when scheduled.
    AfterDuration {
        /// Minutes from when the event was scheduled.
        minutes_from_creation: u32,
        /// The absolute minute count when this should trigger.
        trigger_at_minute: u64,
    },
    /// Trigger at a specific time of day (daily).
    TimeOfDay { hour: u8, minute: u8 },
}

/// Visibility of the event to the player.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum EventVisibility {
    /// Player knows about this event.
    #[default]
    Public,
    /// Player doesn't know this will happen.
    Private,
    /// Player knows something will happen but not details.
    Hinted,
}

impl std::fmt::Display for EventVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventVisibility::Public => write!(f, "public"),
            EventVisibility::Private => write!(f, "private"),
            EventVisibility::Hinted => write!(f, "hinted"),
        }
    }
}

impl EventVisibility {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "public" => Some(EventVisibility::Public),
            "private" | "secret" => Some(EventVisibility::Private),
            "hinted" | "hint" => Some(EventVisibility::Hinted),
            _ => None,
        }
    }
}

/// Current status of a scheduled event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum EventStatus {
    /// Event is waiting to trigger.
    #[default]
    Scheduled,
    /// Event has triggered.
    Triggered,
    /// Event was cancelled before triggering.
    Cancelled,
}

/// A scheduled event that triggers based on time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: ScheduledEventId,
    /// What happens when this event triggers.
    pub description: String,
    /// When the event should trigger.
    pub trigger: EventTrigger,
    /// Where this event occurs (optional).
    pub location: Option<String>,
    /// Who is involved in this event (NPC names, etc.).
    pub involved_entities: Vec<String>,
    /// Whether the player knows about this event.
    pub visibility: EventVisibility,
    /// Whether this event repeats.
    pub repeating: bool,
    /// For repeating events, interval in minutes.
    pub repeat_interval_minutes: Option<u32>,
    /// Current status.
    pub status: EventStatus,
    /// When this event was scheduled (story turn).
    pub scheduled_at_turn: u32,
    /// The minute count when this was scheduled (for duration-based events).
    pub scheduled_at_minute: u64,
}

impl ScheduledEvent {
    /// Create a new scheduled event.
    pub fn new(
        id: ScheduledEventId,
        description: impl Into<String>,
        trigger: EventTrigger,
        current_turn: u32,
        current_minute: u64,
    ) -> Self {
        Self {
            id,
            description: description.into(),
            trigger,
            location: None,
            involved_entities: Vec::new(),
            visibility: EventVisibility::default(),
            repeating: false,
            repeat_interval_minutes: None,
            status: EventStatus::Scheduled,
            scheduled_at_turn: current_turn,
            scheduled_at_minute: current_minute,
        }
    }

    /// Set the location for this event.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Add involved entities.
    pub fn with_entities(mut self, entities: Vec<String>) -> Self {
        self.involved_entities = entities;
        self
    }

    /// Set the visibility.
    pub fn with_visibility(mut self, visibility: EventVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Make this a repeating event.
    pub fn repeating(mut self, interval_minutes: u32) -> Self {
        self.repeating = true;
        self.repeat_interval_minutes = Some(interval_minutes);
        self
    }

    /// Check if this event is still pending.
    pub fn is_pending(&self) -> bool {
        self.status == EventStatus::Scheduled
    }

    /// Mark the event as triggered.
    pub fn trigger(&mut self) {
        self.status = EventStatus::Triggered;
    }

    /// Cancel the event.
    pub fn cancel(&mut self) {
        self.status = EventStatus::Cancelled;
    }

    /// Reschedule a repeating event after it triggers.
    pub fn reschedule(&mut self, new_trigger_minute: u64) {
        if self.repeating {
            self.status = EventStatus::Scheduled;
            if let EventTrigger::AfterDuration {
                minutes_from_creation,
                ..
            } = &mut self.trigger
            {
                self.trigger = EventTrigger::AfterDuration {
                    minutes_from_creation: *minutes_from_creation,
                    trigger_at_minute: new_trigger_minute,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = ScheduledEvent::new(
            ScheduledEventId(1),
            "The festival begins",
            EventTrigger::AtTime {
                year: 1492,
                month: 3,
                day: 5,
                hour: Some(12),
            },
            0,
            0,
        );

        assert_eq!(event.description, "The festival begins");
        assert!(event.is_pending());
    }

    #[test]
    fn test_event_with_location() {
        let event = ScheduledEvent::new(
            ScheduledEventId(1),
            "Market opens",
            EventTrigger::TimeOfDay { hour: 8, minute: 0 },
            0,
            0,
        )
        .with_location("Town Square");

        assert_eq!(event.location, Some("Town Square".to_string()));
    }

    #[test]
    fn test_repeating_event() {
        let event = ScheduledEvent::new(
            ScheduledEventId(1),
            "Daily market",
            EventTrigger::TimeOfDay { hour: 8, minute: 0 },
            0,
            0,
        )
        .repeating(24 * 60); // Repeat daily

        assert!(event.repeating);
        assert_eq!(event.repeat_interval_minutes, Some(24 * 60));
    }

    #[test]
    fn test_event_trigger_and_cancel() {
        let mut event = ScheduledEvent::new(
            ScheduledEventId(1),
            "Test event",
            EventTrigger::AfterDuration {
                minutes_from_creation: 60,
                trigger_at_minute: 60,
            },
            0,
            0,
        );

        assert!(event.is_pending());
        event.trigger();
        assert!(!event.is_pending());
        assert_eq!(event.status, EventStatus::Triggered);

        // Test cancellation
        let mut event2 = ScheduledEvent::new(
            ScheduledEventId(2),
            "Another event",
            EventTrigger::AfterDuration {
                minutes_from_creation: 120,
                trigger_at_minute: 120,
            },
            0,
            0,
        );
        event2.cancel();
        assert_eq!(event2.status, EventStatus::Cancelled);
    }

    #[test]
    fn test_visibility_parsing() {
        assert_eq!(
            EventVisibility::parse("public"),
            Some(EventVisibility::Public)
        );
        assert_eq!(
            EventVisibility::parse("private"),
            Some(EventVisibility::Private)
        );
        assert_eq!(
            EventVisibility::parse("secret"),
            Some(EventVisibility::Private)
        );
        assert_eq!(
            EventVisibility::parse("hinted"),
            Some(EventVisibility::Hinted)
        );
        assert_eq!(EventVisibility::parse("unknown"), None);
    }
}

//! Terminal User Interface for D&D game
//!
//! Built with ratatui, providing:
//! - Narrative display with streaming support
//! - Character sheet panels
//! - Combat tracker
//! - Animated dice rolls
//! - Modal overlays

pub mod layout;
pub mod render;
pub mod theme;
pub mod widgets;

pub use render::render;
pub use theme::GameTheme;

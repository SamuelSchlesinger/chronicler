//! Effect-to-UI mapping.
//!
//! This module translates game Effects into UI state updates,
//! sounds, and screen shake.
//!
//! The module is organized into separate concerns:
//! - `sound_mapping`: Maps effects to sound effects
//! - `narrative`: Formats effects into narrative text
//! - `screen_shake`: Determines screen shake intensity
//! - `processor`: Coordinates all effect processing

mod narrative;
mod processor;
mod screen_shake;
mod sound_mapping;

// Re-export the main public API
pub use processor::process_effect;

// Re-export internal types for potential use by other modules
// (These are currently only used internally but may be useful for testing or extensions)
#[allow(unused_imports)]
pub use narrative::NarrativeOutput;
#[allow(unused_imports)]
pub use screen_shake::screen_shake_for_effect;
#[allow(unused_imports)]
pub use sound_mapping::sound_for_effect;

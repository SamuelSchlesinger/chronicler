//! # Agentic
//!
//! A Rust framework for building AI agents with sophisticated planning, memory,
//! and safety capabilities.
//!
//! ## Core Concepts
//!
//! - **Agent**: The central abstraction that processes messages and produces responses
//! - **Tool**: Executable functions that agents can invoke
//! - **Memory**: Episodic, semantic, and procedural memory systems
//! - **Safety**: Guardrails, approval workflows, and audit logging
//!
//! ## Example
//!
//! ```rust,ignore
//! use agentic::prelude::*;
//! use async_trait::async_trait;
//! use serde_json::json;
//!
//! // Define a custom tool
//! struct EchoTool;
//!
//! #[async_trait]
//! impl Tool for EchoTool {
//!     fn name(&self) -> &str { "echo" }
//!     fn description(&self) -> &str { "Echoes the input back" }
//!     fn input_schema(&self) -> &serde_json::Value {
//!         static SCHEMA: once_cell::sync::Lazy<serde_json::Value> =
//!             once_cell::sync::Lazy::new(|| json!({
//!                 "type": "object",
//!                 "properties": { "message": { "type": "string" } },
//!                 "required": ["message"]
//!             }));
//!         &SCHEMA
//!     }
//!
//!     async fn execute(
//!         &self,
//!         params: serde_json::Value,
//!         _ctx: &ToolContext,
//!     ) -> Result<ToolOutput, ToolError> {
//!         let msg = params["message"].as_str().unwrap_or("no message");
//!         Ok(ToolOutput::text(format!("Echo: {msg}")))
//!     }
//! }
//!
//! // Register tools in a registry
//! let mut registry = ToolRegistry::new();
//! registry.register(EchoTool);
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod action;
pub mod agent;
pub mod context;
pub mod error;
pub mod id;
pub mod llm;
pub mod memory;
pub mod message;
pub mod safety;
pub mod tool;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::action::*;
    pub use crate::agent::{Agent, AgentMetadata, Capabilities, Context, Response};
    pub use crate::error::*;
    pub use crate::id::*;
    pub use crate::llm::{CompletionRequest, CompletionResponse, LlmProvider};
    pub use crate::memory::{EpisodicMemory, MemoryManager, ProceduralMemory, SemanticMemory};
    pub use crate::message::*;
    pub use crate::safety::{Guardrail, SafetyResult, SafetyValidator};
    pub use crate::tool::{Tool, ToolAnnotations, ToolContext, ToolOutput, ToolRegistry};
}

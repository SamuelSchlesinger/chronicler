//! Shared tokio runtime for async operations.
//!
//! This module provides a lazy-initialized global tokio runtime to avoid
//! creating a new runtime for each async operation, which is wasteful and
//! can potentially fail.

use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Global shared tokio runtime for all async operations.
pub static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create tokio runtime"));

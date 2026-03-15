//! Circuit breaker for provider calls — prevents cascading failures
//! when the AI provider is down or rate-limited.
//!
//! **Re-exported from `bizclaw_core::circuit_breaker`** so all crates share the
//! same implementation. This module exists for backward compatibility.

pub use bizclaw_core::circuit_breaker::*;

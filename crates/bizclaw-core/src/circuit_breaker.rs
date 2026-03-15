//! Circuit breaker for external API calls — prevents cascading failures
//! when a service is down or rate-limited.
//!
//! Lives in `bizclaw-core` so both `bizclaw-agent` (provider calls) and
//! `bizclaw-channels` (channel API calls) can share the same implementation.
//!
//! ## State Machine
//! ```text
//! Closed --(failures >= threshold)--> Open
//! Open   --(recovery timeout elapsed)--> HalfOpen
//! HalfOpen --(success)--> Closed
//! HalfOpen --(failure)--> Open (doubled timeout)
//! ```

use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Maximum recovery timeout (5 minutes).
const MAX_RECOVERY_TIMEOUT: Duration = Duration::from_secs(300);

/// Maximum backoff duration (30 seconds).
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Circuit breaker states.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — requests pass through.
    Closed = 0,
    /// Service is failing — requests are rejected immediately.
    Open = 1,
    /// Testing recovery — one request allowed through.
    HalfOpen = 2,
}

impl From<u8> for CircuitState {
    fn from(v: u8) -> Self {
        match v {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Circuit breaker that tracks consecutive failures and prevents cascading
/// failures by rejecting requests when the service appears down.
///
/// Thread-safe via atomics + mutex.
pub struct CircuitBreaker {
    /// Human-readable name for logging (e.g., "telegram", "discord", "provider").
    name: String,
    state: AtomicU8,
    consecutive_failures: AtomicU64,
    failure_threshold: u64,
    /// Base recovery timeout (doubles on repeated failures).
    base_recovery_timeout: Duration,
    /// When the circuit opened (for recovery timing).
    opened_at: Mutex<Option<Instant>>,
    /// Current recovery timeout (doubles each time half-open fails).
    current_recovery_timeout: Mutex<Duration>,
    /// Total failure count (never resets, for metrics).
    total_failures: AtomicU64,
    /// Total success count (for metrics).
    total_successes: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with custom thresholds.
    pub fn new(failure_threshold: u64, base_recovery_timeout: Duration) -> Self {
        Self {
            name: String::new(),
            state: AtomicU8::new(CircuitState::Closed as u8),
            consecutive_failures: AtomicU64::new(0),
            failure_threshold,
            base_recovery_timeout,
            opened_at: Mutex::new(None),
            current_recovery_timeout: Mutex::new(base_recovery_timeout),
            total_failures: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
        }
    }

    /// Create a named circuit breaker (name appears in log messages).
    pub fn named(name: &str, failure_threshold: u64, base_recovery_timeout: Duration) -> Self {
        let mut cb = Self::new(failure_threshold, base_recovery_timeout);
        cb.name = name.to_string();
        cb
    }

    /// Get the circuit breaker name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Format a log prefix with the circuit breaker name.
    fn log_prefix(&self) -> String {
        if self.name.is_empty() {
            "Circuit breaker".to_string()
        } else {
            format!("Circuit breaker [{}]", self.name)
        }
    }

    /// Check whether a request is allowed to proceed.
    ///
    /// - **Closed**: always returns `true`.
    /// - **Open**: returns `true` (and transitions to HalfOpen) only if the
    ///   recovery timeout has elapsed.
    /// - **HalfOpen**: returns `true` — the test request is allowed through.
    pub fn can_execute(&self) -> bool {
        let state: CircuitState = self.state.load(Ordering::SeqCst).into();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let timeout = {
                    let t = self.current_recovery_timeout.lock().unwrap();
                    *t
                };
                let should_try = {
                    let opened = self.opened_at.lock().unwrap();
                    match *opened {
                        Some(at) => at.elapsed() >= timeout,
                        None => true,
                    }
                };
                if should_try {
                    self.state
                        .store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
                    info!(
                        timeout_secs = timeout.as_secs(),
                        "⚡ {} Open → HalfOpen — allowing test request",
                        self.log_prefix()
                    );
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful call. Resets the circuit to Closed.
    pub fn record_success(&self) {
        let prev: CircuitState = self.state.load(Ordering::SeqCst).into();
        self.state
            .store(CircuitState::Closed as u8, Ordering::SeqCst);
        self.consecutive_failures.store(0, Ordering::SeqCst);
        self.total_successes.fetch_add(1, Ordering::Relaxed);

        // Reset recovery timeout to base
        {
            let mut t = self.current_recovery_timeout.lock().unwrap();
            *t = self.base_recovery_timeout;
        }

        if prev != CircuitState::Closed {
            info!(
                previous_state = %prev,
                "✅ {} closed — service recovered",
                self.log_prefix()
            );
        }
    }

    /// Record a failed call. May transition the circuit to Open.
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        let state: CircuitState = self.state.load(Ordering::SeqCst).into();

        match state {
            CircuitState::Closed => {
                if failures >= self.failure_threshold {
                    self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
                    {
                        let mut opened = self.opened_at.lock().unwrap();
                        *opened = Some(Instant::now());
                    }
                    let timeout = {
                        let t = self.current_recovery_timeout.lock().unwrap();
                        *t
                    };
                    warn!(
                        failures = failures,
                        threshold = self.failure_threshold,
                        recovery_timeout_secs = timeout.as_secs(),
                        "🔴 {} opened — {} consecutive failures",
                        self.log_prefix(),
                        failures
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Test request failed — reopen with doubled timeout
                self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
                {
                    let mut opened = self.opened_at.lock().unwrap();
                    *opened = Some(Instant::now());
                }
                let new_timeout = {
                    let mut t = self.current_recovery_timeout.lock().unwrap();
                    *t = (*t * 2).min(MAX_RECOVERY_TIMEOUT);
                    *t
                };
                warn!(
                    new_recovery_timeout_secs = new_timeout.as_secs(),
                    "🔴 {} re-opened from HalfOpen — doubled recovery timeout",
                    self.log_prefix()
                );
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Return the current circuit state.
    pub fn state(&self) -> CircuitState {
        self.state.load(Ordering::SeqCst).into()
    }

    /// Get the number of consecutive failures.
    pub fn consecutive_failures(&self) -> u64 {
        self.consecutive_failures.load(Ordering::SeqCst)
    }

    /// Get total failure count (lifetime).
    pub fn total_failures(&self) -> u64 {
        self.total_failures.load(Ordering::Relaxed)
    }

    /// Get total success count (lifetime).
    pub fn total_successes(&self) -> u64 {
        self.total_successes.load(Ordering::Relaxed)
    }

    /// Compute an exponential backoff duration with deterministic jitter.
    ///
    /// Formula: `min(30s, 1s * 2^attempt)` with ±25% jitter.
    /// Jitter is deterministic (based on attempt number) — no `rand` dependency.
    pub fn backoff_duration(attempt: u32) -> Duration {
        let base_ms = 1000u64.saturating_mul(1u64.checked_shl(attempt).unwrap_or(u64::MAX));
        let capped_ms = base_ms.min(MAX_BACKOFF.as_millis() as u64);

        // Deterministic jitter: ±25% based on attempt number.
        let jitter_pattern: [i64; 8] = [-25, 15, -10, 20, -5, 25, -20, 10];
        let jitter_pct = jitter_pattern[(attempt as usize) % jitter_pattern.len()];
        let jitter_ms = (capped_ms as i64 * jitter_pct) / 100;
        let final_ms = (capped_ms as i64 + jitter_ms).max(1) as u64;

        Duration::from_millis(final_ms)
    }

    /// Reset the circuit breaker to initial state.
    pub fn reset(&self) {
        self.state
            .store(CircuitState::Closed as u8, Ordering::SeqCst);
        self.consecutive_failures.store(0, Ordering::SeqCst);
        {
            let mut t = self.current_recovery_timeout.lock().unwrap();
            *t = self.base_recovery_timeout;
        }
        {
            let mut opened = self.opened_at.lock().unwrap();
            *opened = None;
        }
        info!("🔄 {} reset to Closed", self.log_prefix());
    }

    /// Get a summary of the circuit breaker state for monitoring.
    pub fn summary(&self) -> String {
        if self.name.is_empty() {
            format!(
                "CircuitBreaker {{ state: {}, consecutive_failures: {}, total_failures: {}, total_successes: {} }}",
                self.state(),
                self.consecutive_failures(),
                self.total_failures(),
                self.total_successes()
            )
        } else {
            format!(
                "CircuitBreaker[{}] {{ state: {}, consecutive_failures: {}, total_failures: {}, total_successes: {} }}",
                self.name,
                self.state(),
                self.consecutive_failures(),
                self.total_failures(),
                self.total_successes()
            )
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closed_allows_execution() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());
    }

    #[test]
    fn test_named_circuit_breaker() {
        let cb = CircuitBreaker::named("telegram", 3, Duration::from_secs(10));
        assert_eq!(cb.name(), "telegram");
        assert!(cb.summary().contains("telegram"));
    }

    #[test]
    fn test_opens_after_threshold_failures() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_open_rejects_execution() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(60));

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Should reject — timeout hasn't elapsed (60s)
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_half_open_after_recovery_timeout() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(1));

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(5));

        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_success_closes() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(1));

        cb.record_failure();
        std::thread::sleep(Duration::from_millis(5));
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_failure_doubles_timeout() {
        let base_timeout = Duration::from_millis(10);
        let cb = CircuitBreaker::new(1, base_timeout);

        cb.record_failure();
        std::thread::sleep(Duration::from_millis(15));
        assert!(cb.can_execute());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        let current_timeout = {
            let t = cb.current_recovery_timeout.lock().unwrap();
            *t
        };
        assert_eq!(current_timeout, base_timeout * 2);
    }

    #[test]
    fn test_backoff_increases() {
        let d0 = CircuitBreaker::backoff_duration(0);
        let d1 = CircuitBreaker::backoff_duration(1);
        let d2 = CircuitBreaker::backoff_duration(2);
        assert!(d1 > d0);
        assert!(d2 > d1);
    }

    #[test]
    fn test_backoff_caps_at_max() {
        let d = CircuitBreaker::backoff_duration(20);
        assert!(d <= Duration::from_millis(37_500)); // 30s + 25% jitter
        assert!(d >= Duration::from_millis(22_500)); // 30s - 25% jitter
    }

    #[test]
    fn test_reset() {
        let cb = CircuitBreaker::new(1, Duration::from_secs(30));
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    #[test]
    fn test_metrics_tracking() {
        let cb = CircuitBreaker::new(10, Duration::from_secs(30));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        cb.record_failure();

        assert_eq!(cb.total_failures(), 3);
        assert_eq!(cb.total_successes(), 1);
    }

    #[test]
    fn test_display_state() {
        assert_eq!(format!("{}", CircuitState::Closed), "Closed");
        assert_eq!(format!("{}", CircuitState::Open), "Open");
        assert_eq!(format!("{}", CircuitState::HalfOpen), "HalfOpen");
    }
}

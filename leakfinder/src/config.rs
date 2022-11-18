use std::{sync::Arc, time::Duration};

use crate::policy::PolicyHolder;

pub type TimestampSource = Arc<dyn TimestampProvider>;

pub struct Config {
    pub timestamp_source: TimestampSource,
    pub policy: Arc<PolicyHolder>,
}

pub trait TimestampProvider: Send + Sync + 'static {
    /// Duration since arbitrarym, static, past time
    fn elapsed(&self) -> Duration;
    /// UNIX epoch in nanoseconds
    fn epoch_ns(&self) -> u64;
}

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Instant, SystemTime};

#[cfg(not(target_arch = "wasm32"))]
pub struct StdTimestampProvider {
    start: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for StdTimestampProvider {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TimestampProvider for StdTimestampProvider {
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    fn epoch_ns(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

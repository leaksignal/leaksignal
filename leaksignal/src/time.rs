use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use leakfinder::TimestampProvider;
use proxy_wasm::traits::Context;

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
fn proxy_get_current_time_nanoseconds(return_time: *mut u64) -> proxy_wasm::types::Status {
    unsafe {
        *return_time = 0;
    }
    proxy_wasm::types::Status::Ok
}

struct TimeContext;
impl Context for TimeContext {}

lazy_static::lazy_static! {
    pub static ref TIMESTAMP_PROVIDER: Arc<WasmTimestampProvider> = Arc::new(WasmTimestampProvider::default());
}

pub struct WasmTimestampProvider {
    start: SystemTime,
}

impl Default for WasmTimestampProvider {
    fn default() -> Self {
        Self {
            start: TimeContext.get_current_time(),
        }
    }
}

impl TimestampProvider for WasmTimestampProvider {
    fn elapsed(&self) -> Duration {
        TimeContext
            .get_current_time()
            .duration_since(self.start)
            .unwrap_or_default()
    }

    fn epoch_ns(&self) -> u64 {
        TimeContext
            .get_current_time()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

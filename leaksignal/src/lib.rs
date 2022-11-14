use std::{
    ffi::c_void,
    time::{Duration, SystemTime},
};

use dlmalloc::GlobalDlmalloc;
use log::error;
use proxy_wasm::{
    traits::{Context, RootContext},
    types::LogLevel,
};
use root::EnvoyRootContext;

mod config;
mod env;
mod evaluator;
mod http_response;
mod low_entropy_hash;
mod metric;
mod parsers;
mod pipe;
mod policy;
mod root;

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

mod proto {
    include!(concat!(env!("OUT_DIR"), "/leaksignal.rs"));
}

lazy_static::lazy_static! {
    pub static ref GIT_COMMIT: &'static str = {
        let raw = env!("GIT_COMMIT").trim();
        if raw.len() > 7 {
            &raw[..7]
        } else {
            raw
        }
    };
}

#[no_mangle]
#[cfg(target_family = "wasm")]
pub fn _start() {
    init();
}

fn init() {
    std::panic::set_hook(Box::new(|info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            format!("{}", s)
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            format!("{}", s)
        } else {
            format!("<couldn't parse panic message> {:?}", info)
        };
        if let Some(location) = info.location() {
            error!(
                "panic occurred at {}:{} with message: {}",
                location.file(),
                location.line(),
                message
            );
        } else {
            error!("panic occurred with message: {}", message);
        }
    }));
    proxy_wasm::set_log_level(LogLevel::Trace);
    if !env::ENVIRONMENT.is_empty() {
        log::warn!(
            "leaksignal found environment variables {}",
            env::ENVIRONMENT
                .keys()
                .map(|x| &**x)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(EnvoyRootContext::default())
    });
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[no_mangle]
pub extern "C" fn free(from: *mut c_void) {
    unsafe { Box::from_raw(from as *mut u8) };
}

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

pub fn proxywasm_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let get_time = || {
        TimeContext
            .get_current_time()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    };

    for buf in buf.chunks_mut(8) {
        let entropy = get_time() | get_time();
        buf.copy_from_slice(&entropy.to_be_bytes()[..buf.len()]);
    }
    Ok(())
}

lazy_static::lazy_static! {
    //todo: deal with system clock changes?
    static ref START: SystemTime = TimeContext.get_current_time();
}

pub fn elapsed() -> Duration {
    TimeContext
        .get_current_time()
        .duration_since(*START)
        .unwrap_or_default()
}

getrandom::register_custom_getrandom!(proxywasm_getrandom);

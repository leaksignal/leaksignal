use dlmalloc::GlobalDlmalloc;
use leakfinder::TimestampProvider;
use log::error;
use proxy_wasm::{traits::RootContext, types::LogLevel};
use root::EnvoyRootContext;
use time::TIMESTAMP_PROVIDER;

mod config;
mod env;
mod http_response;
mod metric;
mod root;
mod service;
mod time;

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

pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[no_mangle]
#[cfg(target_family = "wasm")]
pub fn _start() {
    init();
}

fn init() {
    std::panic::set_hook(Box::new(|info| {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.to_string()
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
pub extern "C" fn free(from: *mut std::ffi::c_void) {
    unsafe { Box::from_raw(from as *mut u8) };
}

pub fn proxywasm_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let get_time = || TIMESTAMP_PROVIDER.epoch_ns();

    for buf in buf.chunks_mut(8) {
        let entropy = get_time() | get_time();
        buf.copy_from_slice(&entropy.to_be_bytes()[..buf.len()]);
    }
    Ok(())
}

getrandom::register_custom_getrandom!(proxywasm_getrandom);

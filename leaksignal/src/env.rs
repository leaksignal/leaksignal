use std::ffi::CStr;

use anyhow::{Context, Result};
use indexmap::IndexMap;

fn read_environment() -> Result<IndexMap<String, String>> {
    let mut out = IndexMap::new();
    let (count, size) = unsafe { wasi::environ_sizes_get()? };
    let mut entries: Vec<*mut u8> = Vec::with_capacity(count);

    let mut buf: Vec<u8> = Vec::with_capacity(size);
    unsafe { wasi::environ_get(entries.as_mut_ptr(), buf.as_mut_ptr())? };
    unsafe { entries.set_len(count) };
    // buf must never be accessed

    for entry in entries {
        let cstr = unsafe { CStr::from_ptr(entry as *const i8) };
        let (name, value) = cstr
            .to_str()?
            .split_once('=')
            .context("missing = in environment variable")?;
        out.insert(name.to_string(), value.to_string());
    }

    Ok(out)
}

lazy_static::lazy_static! {
    pub static ref ENVIRONMENT: IndexMap<String, String> = read_environment().expect("failed to read environment");
}

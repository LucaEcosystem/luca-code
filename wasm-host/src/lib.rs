//! Minimal WASM host for Luca Code (M3 spike, experimental).
//!
//! This crate consumes `luca-code` through its public API only and exposes
//! a narrow C ABI returning serialized strings. It duplicates no language
//! semantics: execution, validation, and diagnostics all live in `luca-code`.
//!
//! # ABI and memory ownership
//!
//! - Handles are opaque: `luca_host_new` returns a `Box<Host>` pointer as
//!   `usize`. Every other call takes that handle. Handles must come from
//!   `luca_host_new`, must not be forged, must not be used after
//!   `luca_host_free`, and must be freed exactly once. Misuse traps inside
//!   the instance; it cannot corrupt the caller beyond aborting the call.
//! - Byte buffers cross the boundary as `(ptr, len)` pairs using UTF-8.
//!   `luca_alloc` returns zeroed guest memory the caller fills and then
//!   passes back; every `luca_alloc` result and every response buffer must
//!   be released exactly once with `luca_free(ptr, same_len)`.
//! - `luca_run` / `luca_check` write the response length to `*out_len`
//!   and return the response pointer, or null when the handle or `out_len`
//!   itself is unusable. All Luca-level outcomes — including malformed
//!   UTF-8 input — arrive as response documents, never as traps.
//! - Responses are JSON objects parsed by the host environment's built-in
//!   JSON support. No custom parser exists on either side: requests carry
//!   raw source text plus discrete setup calls, and only guest-produced
//!   response strings are JSON-encoded (with a minimal output escaper).
//! - Entry points validate every argument and contain no panicking index
//!   operations; a residual Rust panic traps per wasm semantics instead of
//!   unwinding across the boundary.
//! - Each handle owns an isolated host (own files, input, captures).
//!   Repeated calls on one handle share its scripted files; per-call
//!   responses contain only output and warnings produced by that call.

use luca_code::{
    check::{check_file_with_platform, diagnostics_to_json},
    error::LucaError,
    interpreter::{Interpreter, RunOutcome},
    FakePlatform, Platform, PlatformError,
};
use std::{
    path::PathBuf,
    ptr,
    slice,
};

/// Fixed virtual filesystem root used for module search paths.
const VIRTUAL_ROOT: &str = "/";
/// Deterministic web-host labels reported for `System.os`/`System.arch`.
const WEB_OS: &str = "web";
const WEB_ARCH: &str = "wasm32";
/// Detail used when the host denies interactive input.
const INPUT_DENIED: &str = "input is not available in this host";

/// Setup-call result codes: `0` success, negative values are host misuse.
/// Luca-level failures never surface here; they arrive as responses.
pub const HOST_OK: i32 = 0;
const HOST_BAD_HANDLE: i32 = -1;
const HOST_BAD_POINTER: i32 = -2;
const HOST_BAD_UTF8: i32 = -3;

struct Host {
    platform: FakePlatform,
}

impl Host {
    fn new() -> Self {
        let platform = FakePlatform::new();
        platform.set_os_arch(WEB_OS, WEB_ARCH);
        platform.set_current_dir(Ok(PathBuf::from(VIRTUAL_ROOT)));
        Self { platform }
    }

    fn interpreter(&self) -> Interpreter {
        Interpreter::with_search_paths(vec![PathBuf::from(VIRTUAL_ROOT)])
            .with_platform(Box::new(self.platform.clone()))
    }

    /// Output lines captured since `warn_from` (warnings route through the
    /// platform sink during execution, unlike program output, which
    /// `run_outcome` returns owned).
    fn new_warnings(&self, warn_from: usize) -> Vec<(usize, usize, String)> {
        self.platform
            .warnings()
            .get(warn_from..)
            .unwrap_or(&[])
            .to_vec()
    }
}

/// Resolve an opaque handle. Handles originate only from `luca_host_new`;
/// anything else (including `0` and freed handles) yields `None`, and the
/// caller reports misuse without touching memory.
///
/// # Safety
///
/// The caller must pass a live handle. This is the single trusted cast in
/// the crate; every entry point routes through it.
unsafe fn host_of(handle: usize) -> Option<&'static mut Host> {
    if handle == 0 {
        return None;
    }
    // SAFETY: by contract `handle` is a live `Box<Host>` pointer created by
    // `luca_host_new` and not yet passed to `luca_host_free`. `&mut` is sound
    // because all ABI calls are synchronous and non-reentrant.
    Some(unsafe { &mut *(handle as *mut Host) })
}

/// Borrow caller-supplied bytes. Empty inputs never touch the pointer, so a
/// null pointer with length zero is valid; any nonzero length requires a
/// readable pointer.
///
/// # Safety
///
/// The caller must guarantee `[ptr, ptr+len)` is readable for the call.
unsafe fn caller_bytes<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    if ptr.is_null() {
        return None;
    }
    // SAFETY: guarded above; the caller owns the allocation for the call.
    Some(unsafe { slice::from_raw_parts(ptr, len) })
}

/// Read one caller-supplied string argument, distinguishing an unreadable
/// pointer ([`HOST_BAD_POINTER`]) from invalid UTF-8 ([`HOST_BAD_UTF8`]).
///
/// # Safety
///
/// Same contract as `caller_bytes`.
unsafe fn caller_arg(ptr: *const u8, len: usize) -> Result<String, i32> {
    // SAFETY: `caller_bytes` validates before any access.
    let bytes = unsafe { caller_bytes(ptr, len) }.ok_or(HOST_BAD_POINTER)?;
    core::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| HOST_BAD_UTF8)
}

/// Emit a guest-owned response buffer and publish its length.
///
/// # Safety
///
/// The caller must pass a writable `out_len` and later release the
/// returned pointer exactly once with `luca_free(ptr, reported_len)`.
unsafe fn respond(out_len: *mut usize, body: String) -> *mut u8 {
    if out_len.is_null() {
        return ptr::null_mut();
    }
    let bytes = body.into_bytes();
    // SAFETY: `out_len` was checked; the caller reads the length before free.
    unsafe {
        *out_len = bytes.len();
    }
    leak_bytes(&bytes)
}

/// Move bytes into a guest-owned buffer with matching length and capacity
/// so `luca_free` can reclaim them exactly.
fn leak_bytes(bytes: &[u8]) -> *mut u8 {
    let mut owned = Vec::with_capacity(bytes.len());
    owned.extend_from_slice(bytes);
    let ptr = owned.as_mut_ptr();
    core::mem::forget(owned);
    ptr
}

/// Minimal output escaper for response documents (quotes, backslashes, and
/// control characters only). Response parsing belongs to the caller's
/// built-in JSON support; this escaper only guarantees valid strings.
fn escape_into(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    escape_into(&mut out, value);
    out.push('"');
    out
}

fn json_string_array(items: &[String]) -> String {
    let mut out = String::from("[");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&json_string(item));
    }
    out.push(']');
    out
}

fn json_warnings(warnings: &[(usize, usize, String)]) -> String {
    let mut out = String::from("[");
    for (index, (line, column, message)) in warnings.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&format!(" [{line},{column},{}]", json_string(message)));
    }
    out.push(']');
    out
}

fn json_error(message: &str, line: usize, column: usize) -> String {
    format!(
        "{{\"status\":\"error\",\"message\":{},\"line\":{line},\"column\":{column}}}",
        json_string(message)
    )
}

fn completed_json(output: Vec<String>, warnings: Vec<(usize, usize, String)>) -> String {
    format!(
        "{{\"status\":\"ok\",\"output\":{},\"warnings\":{}}}",
        json_string_array(&output),
        json_warnings(&warnings)
    )
}

fn exited_json(code: i32, output: Vec<String>, warnings: Vec<(usize, usize, String)>) -> String {
    format!(
        "{{\"status\":\"exit\",\"code\":{code},\"output\":{},\"warnings\":{}}}",
        json_string_array(&output),
        json_warnings(&warnings)
    )
}

fn error_json(error: &LucaError) -> String {
    json_error(&error.message, error.line, error.column)
}

/// Create an isolated host. The returned handle must be passed to
/// `luca_host_free` exactly once.
#[no_mangle]
pub extern "C" fn luca_host_new() -> usize {
    Box::into_raw(Box::new(Host::new())) as usize
}

/// Destroy a host created by `luca_host_new`. Returns [`HOST_OK`], or
/// [`HOST_BAD_HANDLE`] for a null handle.
///
/// # Safety
///
/// The handle must be live and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn luca_host_free(handle: usize) -> i32 {
    if handle == 0 {
        return HOST_BAD_HANDLE;
    }
    // SAFETY: contract as documented above; dropping reclaims the box.
    let _ = unsafe { Box::from_raw(handle as *mut Host) };
    HOST_OK
}

/// Allocate zeroed guest memory of `size` bytes for caller-written input.
/// Release exactly once with `luca_free(ptr, size)`.
#[no_mangle]
pub extern "C" fn luca_alloc(size: usize) -> *mut u8 {
    leak_bytes(&vec![0u8; size])
}

/// Release memory obtained from `luca_alloc` or a response pointer, using
/// the exact size it was created with.
///
/// # Safety
///
/// The pointer must come from this instance with matching `size`, be
/// currently live, and be freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn luca_host_free_buffer(ptr: *mut u8, size: usize) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: contract as documented above; lengths always match because
    // every buffer is created with length equal to capacity.
    let _ = unsafe { Vec::from_raw_parts(ptr, size, size) };
}

/// Alias named for the documented `luca_free` pairing.
///
/// # Safety
///
/// Same contract as `luca_host_free_buffer`.
#[no_mangle]
pub unsafe extern "C" fn luca_free(ptr: *mut u8, size: usize) {
    // SAFETY: identical contract; single implementation, two stable names.
    unsafe { luca_host_free_buffer(ptr, size) };
}

/// Script a virtual file at an absolute path (e.g. `/lib/Utils.lucc`).
/// Returns [`HOST_OK`], or a negative misuse code.
///
/// # Safety
///
/// Byte ranges must be readable as documented for input buffers.
#[no_mangle]
pub unsafe extern "C" fn luca_add_file(
    handle: usize,
    path_ptr: *const u8,
    path_len: usize,
    content_ptr: *const u8,
    content_len: usize,
) -> i32 {
    // SAFETY: validated through `host_of` / `caller_arg` below.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return HOST_BAD_HANDLE;
    };
    // SAFETY: `caller_arg` validates before any access.
    let path = match unsafe { caller_arg(path_ptr, path_len) } {
        Ok(path) => path,
        Err(code) => return code,
    };
    // SAFETY: `caller_arg` validates before any access.
    let content = match unsafe { caller_arg(content_ptr, content_len) } {
        Ok(content) => content,
        Err(code) => return code,
    };
    host.platform.add_file(path, content);
    HOST_OK
}

/// Script a virtual directory (satisfies `path_exists`, never `is_file`).
/// Returns [`HOST_OK`], or a negative misuse code.
///
/// # Safety
///
/// Byte ranges must be readable as documented for input buffers.
#[no_mangle]
pub unsafe extern "C" fn luca_add_dir(handle: usize, path_ptr: *const u8, path_len: usize) -> i32 {
    // SAFETY: validated through `host_of` / `caller_arg` below.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return HOST_BAD_HANDLE;
    };
    // SAFETY: `caller_arg` validates before any access.
    let path = match unsafe { caller_arg(path_ptr, path_len) } {
        Ok(path) => path,
        Err(code) => return code,
    };
    host.platform.add_dir(path);
    HOST_OK
}

/// Queue one input line for `ask`. Returns [`HOST_OK`], or a negative
/// misuse code.
///
/// # Safety
///
/// Byte ranges must be readable as documented for input buffers.
#[no_mangle]
pub unsafe extern "C" fn luca_push_input(handle: usize, ptr: *const u8, len: usize) -> i32 {
    // SAFETY: validated through `host_of` / `caller_arg` below.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return HOST_BAD_HANDLE;
    };
    // SAFETY: `caller_arg` validates before any access.
    let line = match unsafe { caller_arg(ptr, len) } {
        Ok(line) => line,
        Err(code) => return code,
    };
    host.platform.push_input(line);
    HOST_OK
}

/// Toggle the deny-input policy. Nonzero `deny` makes `read_line` fail
/// with `Unsupported`, which the runtime maps to the frozen
/// `Failed to read input:` diagnostic; zero restores queued input.
/// Returns [`HOST_OK`], or [`HOST_BAD_HANDLE`].
#[no_mangle]
pub extern "C" fn luca_set_deny_input(handle: usize, deny: i32) -> i32 {
    // SAFETY: handle-only call; no memory is touched.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return HOST_BAD_HANDLE;
    };
    if deny != 0 {
        host.platform
            .fail_input(PlatformError::unsupported(INPUT_DENIED));
    } else {
        host.platform.clear_input_error();
    }
    HOST_OK
}

/// Add one `System.env` entry. Returns [`HOST_OK`], or a negative misuse
/// code.
///
/// # Safety
///
/// Byte ranges must be readable as documented for input buffers.
#[no_mangle]
pub unsafe extern "C" fn luca_add_env(
    handle: usize,
    key_ptr: *const u8,
    key_len: usize,
    value_ptr: *const u8,
    value_len: usize,
) -> i32 {
    // SAFETY: validated through `host_of` / `caller_arg` below.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return HOST_BAD_HANDLE;
    };
    // SAFETY: `caller_arg` validates before any access.
    let key = match unsafe { caller_arg(key_ptr, key_len) } {
        Ok(key) => key,
        Err(code) => return code,
    };
    // SAFETY: `caller_arg` validates before any access.
    let value = match unsafe { caller_arg(value_ptr, value_len) } {
        Ok(value) => value,
        Err(code) => return code,
    };
    let mut env = host.platform.env_vars();
    if let Some(slot) = env.iter_mut().find(|(k, _)| *k == key) {
        slot.1 = value;
    } else {
        env.push((key, value));
    }
    host.platform.set_env(env);
    HOST_OK
}

/// Execute Luca source and return a JSON response document.
///
/// The source runs with virtual search paths (`/`) against this handle's
/// scripted files. Responses (written to `*out_len`, released with
/// `luca_free`):
/// - `{"status":"ok","output":[...],"warnings":[[line,column,"msg"],...]}`
/// - `{"status":"exit","code":N,"output":[...],"warnings":[...]}`
///   (`System.exit` is returned, never terminates the caller)
/// - `{"status":"error","message":"...","line":L,"column":C}`
/// - `{"status":"host-error","message":"..."}` for unreadable source text.
///   Only `output`/`warnings` produced by this call are included, even
///   though scripted files persist across calls on the same handle.
///   Returns null only when the handle or `out_len` is unusable.
///
/// # Safety
///
/// Byte ranges and `out_len` must be valid as documented for input
/// buffers and responses.
#[no_mangle]
pub unsafe extern "C" fn luca_run(
    handle: usize,
    src_ptr: *const u8,
    src_len: usize,
    out_len: *mut usize,
) -> *mut u8 {
    // SAFETY: validated through `host_of` below; no memory touched yet.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return ptr::null_mut();
    };
    if out_len.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: `caller_arg` validates before any access.
    let source = match unsafe { caller_arg(src_ptr, src_len) } {
        Ok(source) => source,
        Err(_) => {
            return unsafe { respond(out_len, host_error_json("invalid source: expected UTF-8 text")) };
        }
    };
    let warn_from = host.platform.warnings().len();
    let mut interpreter = host.interpreter();
    let body = match interpreter.run_outcome(&source) {
        Ok(RunOutcome::Completed(output)) => {
            completed_json(output, host.new_warnings(warn_from))
        }
        Ok(RunOutcome::Exited { code, output }) => {
            exited_json(code, output, host.new_warnings(warn_from))
        }
        Err(error) => error_json(&error),
    };
    // SAFETY: `out_len` was checked above.
    unsafe { respond(out_len, body) }
}

/// Validate the scripted file at `entry_path` and return a JSON response
/// document: `{"status":"ok","diagnostics":[...]}` or
/// `{"status":"error","message":"..."}` for unreadable entries.
/// Nothing is executed. Script the entry with `luca_add_file` first.
///
/// # Safety
///
/// Same contract as `luca_run`.
#[no_mangle]
pub unsafe extern "C" fn luca_check(
    handle: usize,
    entry_ptr: *const u8,
    entry_len: usize,
    out_len: *mut usize,
) -> *mut u8 {
    // SAFETY: validated through `host_of` below; no memory touched yet.
    let Some(host) = (unsafe { host_of(handle) }) else {
        return ptr::null_mut();
    };
    if out_len.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: `caller_arg` validates before any access.
    let entry = match unsafe { caller_arg(entry_ptr, entry_len) } {
        Ok(entry) => entry,
        Err(_) => {
            return unsafe { respond(out_len, host_error_json("invalid entry path: expected UTF-8 text")) };
        }
    };
    let search = [PathBuf::from(VIRTUAL_ROOT)];
    let body = match check_file_with_platform(PathBuf::from(entry).as_path(), &search, &host.platform) {
        Ok(diagnostics) => format!(
            "{{\"status\":\"ok\",\"diagnostics\":{}}}",
            diagnostics_to_json(&diagnostics)
        ),
        Err(failure) => host_error_json(&failure.to_string()),
    };
    // SAFETY: `out_len` was checked above.
    unsafe { respond(out_len, body) }
}

fn host_error_json(message: &str) -> String {
    format!("{{\"status\":\"host-error\",\"message\":{}}}", json_string(message))
}

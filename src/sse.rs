//! High-level wrappers around the host SSE streaming API.
//!
//! The host exposes three low-level FFI functions (`host_sse_post`,
//! `host_sse_read`, `host_sse_close`) that mirror the subprocess
//! `host_spawn`/`host_read`/`host_close` model.  This module provides
//! ergonomic wrappers on top of them.
//!
//! # Typical usage
//!
//! ```ignore
//! let handle = sse_post(&url, Some(&headers), &body)?;
//! for_each_sse_line(handle, |line| {
//!     // parse the SSE line, e.g. strip "data: " prefix
//!     // return false to stop early
//!     true
//! });
//! // sse_close is called automatically by for_each_sse_line
//! ```

use crate::ffi;

const SSE_BUF_SIZE: usize = 4096;

/// Open a streaming HTTP POST connection and return an opaque handle.
///
/// `headers` — optional slice of `(name, value)` pairs that will be
/// serialised to a JSON object and passed to the host.
///
/// Returns `Ok(handle)` where handle ≥ 1, or an error string if the host
/// could not open the connection.
pub fn sse_post(url: &str, headers: Option<&[(&str, &str)]>, body: &str) -> Result<i32, String> {
    // Serialise headers map to JSON.
    let headers_json: String = match headers {
        Some(h) if !h.is_empty() => {
            let pairs: Vec<String> = h
                .iter()
                .map(|(k, v)| {
                    format!(
                        "\"{}\":\"{}\"",
                        k.replace('\\', "\\\\").replace('"', "\\\""),
                        v.replace('\\', "\\\\").replace('"', "\\\"")
                    )
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        }
        _ => String::new(),
    };

    let handle = unsafe {
        ffi::host_sse_post(
            url.as_ptr(),
            url.len() as i32,
            if headers_json.is_empty() {
                core::ptr::null()
            } else {
                headers_json.as_ptr()
            },
            headers_json.len() as i32,
            body.as_ptr(),
            body.len() as i32,
        )
    };

    if handle <= 0 {
        Err(format!("host_sse_post returned {}", handle))
    } else {
        Ok(handle)
    }
}

/// Read up to `buf.len()` raw bytes from the stream into `buf`.
///
/// Returns:
/// - `n > 0` — bytes written
/// - `0` — no data yet (stream still open)
/// - `-1` — stream ended or error
pub fn sse_read_raw(handle: i32, buf: &mut [u8]) -> i32 {
    unsafe { ffi::host_sse_read(handle, buf.as_mut_ptr(), buf.len() as i32) }
}

/// Close and release an SSE stream handle.
pub fn sse_close(handle: i32) -> i32 {
    unsafe { ffi::host_sse_close(handle) }
}

/// Iterate over every line in an SSE stream, calling `cb` for each one.
///
/// - Lines are split on `'\n'` (bare `\n` or `\r\n` are both handled).
/// - Empty lines (after stripping `\r`) are skipped.
/// - Return `false` from `cb` to stop iteration early.
/// - `sse_close` is called automatically when iteration ends (whether
///   naturally or via early return).
///
/// Busy-waits when no data is available yet (yields via a tight spin on
/// `host_sse_read` returning 0).  For LLM inference this is fine because
/// the host task is writing chunks continuously; the WASM turn is synchronous
/// so there is no scheduler to yield to anyway.
pub fn for_each_sse_line<F>(handle: i32, mut cb: F)
where
    F: FnMut(&str) -> bool,
{
    let mut partial: Vec<u8> = Vec::new();
    let mut buf = [0u8; SSE_BUF_SIZE];

    'outer: loop {
        let n = sse_read_raw(handle, &mut buf);
        if n < 0 {
            // Stream ended or error.
            break;
        }
        if n == 0 {
            // No data yet — spin.
            continue;
        }

        partial.extend_from_slice(&buf[..n as usize]);

        // Split on '\n', yielding complete lines.
        loop {
            match partial.iter().position(|&b| b == b'\n') {
                None => break, // No complete line yet; wait for more data.
                Some(pos) => {
                    let line_bytes = partial.drain(..=pos).collect::<Vec<u8>>();
                    // Strip trailing \r\n / \n.
                    let line = std::str::from_utf8(&line_bytes)
                        .unwrap_or("")
                        .trim_end_matches('\n')
                        .trim_end_matches('\r');

                    if !line.is_empty() {
                        if !cb(line) {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    sse_close(handle);
}

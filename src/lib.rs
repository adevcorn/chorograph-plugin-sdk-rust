pub mod ai;
pub mod ffi;
pub mod process;
pub mod sse;
pub mod ui;

use serde::{Deserialize, Serialize};

/// A single interactive or callable entry point exposed by a project.
/// For WebAPI projects these are HTTP routes; for WebApp projects these are
/// page/action endpoints; for console/worker/native apps these are the
/// program's main entry methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    /// Human-readable label, e.g. "GET /api/products" or "Main()"
    pub label: String,
    /// Source file path relative to the project root, e.g. "Controllers/ProductsController.cs"
    pub path: String,
    /// 1-based line number of the entry point definition, if known
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// HTTP verb or invocation kind, e.g. "GET", "POST", "MAIN", "EXECUTE"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Optional description or doc comment summary
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// How this entry point was detected: "lsp" or "regex"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detection_source: Option<String>,
}

/// The live status of a single orchestrated resource (e.g. an Aspire project or container).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    /// Resource name as declared in the orchestrator (e.g. "api", "db").
    pub name: String,
    /// Resource kind: "project", "container", "executable", or a raw Add* suffix.
    pub kind: String,
    /// Lifecycle state reported by the orchestrator: "Running", "Starting", "Stopped", "Failed", etc.
    pub state: String,
    /// The URL the resource is listening on, if known and running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// The run status of a project as determined by a plugin implementing `detect_run_status`.
/// Returned for web-facing project types (WebAPI, WebApp) where a TCP port can be probed.
/// Plugins that do not support run-status detection should simply not export the function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStatus {
    /// Whether the project appears to be running right now.
    pub is_running: bool,
    /// The base URL the project is listening on, if known (e.g. "https://localhost:7001").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The process ID, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Per-resource statuses, if the orchestrator exposes them (e.g. Aspire Dashboard API).
    /// Empty when the plugin cannot determine per-resource detail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<ResourceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProfile {
    pub category: String,
    pub tags: Vec<String>,
    /// Entry points discovered by the plugin. Defaults to empty if not populated.
    #[serde(default)]
    pub entry_points: Vec<EntryPoint>,
}

pub fn plugin_print(msg: &str) {
    unsafe { ffi::print(msg.as_ptr(), msg.len() as i32) }
}

pub fn read_host_file(path: &str) -> Result<String> {
    let packed = unsafe { ffi::host_read_file(path.as_ptr(), path.len() as i32) };
    let ptr = (packed >> 32) as *mut u8;
    let len = (packed & 0xFFFFFFFF) as usize;

    if len == 0 || ptr.is_null() {
        return Err(PluginError::Other(format!(
            "Failed to read host file: {}",
            path
        )));
    }

    unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        let s =
            String::from_utf8(bytes).map_err(|e| PluginError::SerializationError(e.to_string()))?;
        Ok(s)
    }
}

/// A symbol returned by the host LSP workspace/symbol query.
/// `kind` is the raw LSP SymbolKind integer (e.g. 12 = Function, 6 = Method).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspSymbolInfo {
    pub name: String,
    pub kind: u32,
    pub file_path: String,
    pub line: u32,
}

/// Ask the host LSP orchestrator for all workspace symbols under `root`.
/// Returns `Ok(vec)` — possibly empty — if the host LSP session exists,
/// or `Err` if the session is unavailable or the call fails.
/// Plugins should treat an empty vec as "LSP not ready; fall back to static analysis".
pub fn workspace_symbols_from_host(root: &str) -> Result<Vec<LspSymbolInfo>> {
    let packed = unsafe { ffi::host_workspace_symbols(root.as_ptr(), root.len() as i32) };
    let ptr = (packed >> 32) as *mut u8;
    let len = (packed & 0xFFFFFFFF) as usize;

    if len == 0 || ptr.is_null() {
        return Err(PluginError::Other("No LSP session available".into()));
    }

    unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        let s =
            String::from_utf8(bytes).map_err(|e| PluginError::SerializationError(e.to_string()))?;
        serde_json::from_str::<Vec<LspSymbolInfo>>(&s)
            .map_err(|e| PluginError::SerializationError(e.to_string()))
    }
}

/// Probe whether a TCP port is listening on `host` (e.g. "localhost").
/// The host performs a non-blocking connect with a ~200 ms timeout.
/// Returns `true` if the port is open, `false` otherwise.
pub fn tcp_probe(host: &str, port: u16) -> bool {
    unsafe { ffi::host_tcp_probe(host.as_ptr(), host.len() as i32, port as i32) == 1 }
}

/// The response returned by [`http_get`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code (e.g. 200, 404, 500).
    pub status: u16,
    /// Response body as a UTF-8 string.
    pub body: String,
}

/// Perform an HTTP GET request via the host network stack (bypasses the WASM sandbox).
/// `url` — fully-qualified URL to request.
/// `headers` — optional extra request headers as key-value pairs.
/// Returns `Ok(HttpResponse)` on success (including non-2xx status codes).
/// Returns `Err` if the host cannot reach the URL or the call fails entirely.
pub fn http_get(url: &str, headers: Option<&[(&str, &str)]>) -> Result<HttpResponse> {
    let headers_json = match headers {
        Some(h) => {
            let map: std::collections::HashMap<&str, &str> = h.iter().cloned().collect();
            serde_json::to_string(&map)
                .map_err(|e| PluginError::SerializationError(e.to_string()))?
        }
        None => String::new(),
    };

    let (headers_ptr, headers_len) = if headers_json.is_empty() {
        (std::ptr::null(), 0i32)
    } else {
        (headers_json.as_ptr(), headers_json.len() as i32)
    };

    let packed =
        unsafe { ffi::host_http_get(url.as_ptr(), url.len() as i32, headers_ptr, headers_len) };

    let ptr = (packed >> 32) as *mut u8;
    let len = (packed & 0xFFFF_FFFF) as usize;

    if len == 0 || ptr.is_null() {
        return Err(PluginError::Other(format!(
            "host_http_get returned no data for url: {}",
            url
        )));
    }

    let json = unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        String::from_utf8(bytes).map_err(|e| PluginError::SerializationError(e.to_string()))?
    };

    serde_json::from_str::<HttpResponse>(&json)
        .map_err(|e| PluginError::SerializationError(e.to_string()))
}

/// Perform an HTTP POST request via the host network stack (bypasses the WASM sandbox).
/// `url` — fully-qualified URL to request.
/// `headers` — optional extra request headers as key-value pairs.
/// `body` — request body as a UTF-8 string (e.g. JSON).
/// Returns `Ok(HttpResponse)` on success (including non-2xx status codes).
/// Returns `Err` if the host cannot reach the URL or the call fails entirely.
pub fn http_post(url: &str, headers: Option<&[(&str, &str)]>, body: &str) -> Result<HttpResponse> {
    let headers_json = match headers {
        Some(h) => {
            let map: std::collections::HashMap<&str, &str> = h.iter().cloned().collect();
            serde_json::to_string(&map)
                .map_err(|e| PluginError::SerializationError(e.to_string()))?
        }
        None => String::new(),
    };

    let (headers_ptr, headers_len) = if headers_json.is_empty() {
        (std::ptr::null(), 0i32)
    } else {
        (headers_json.as_ptr(), headers_json.len() as i32)
    };

    let (body_ptr, body_len) = if body.is_empty() {
        (std::ptr::null(), 0i32)
    } else {
        (body.as_ptr(), body.len() as i32)
    };

    let packed = unsafe {
        ffi::host_http_post(
            url.as_ptr(),
            url.len() as i32,
            headers_ptr,
            headers_len,
            body_ptr,
            body_len,
        )
    };

    let ptr = (packed >> 32) as *mut u8;
    let len = (packed & 0xFFFF_FFFF) as usize;

    if len == 0 || ptr.is_null() {
        return Err(PluginError::Other(format!(
            "host_http_post returned no data for url: {}",
            url
        )));
    }

    let json = unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        String::from_utf8(bytes).map_err(|e| PluginError::SerializationError(e.to_string()))?
    };

    serde_json::from_str::<HttpResponse>(&json)
        .map_err(|e| PluginError::SerializationError(e.to_string()))
}

/// Read a single string value from the host's UserDefaults store.
/// Returns `Some(value)` if the key exists, `None` if not set or the call fails.
pub fn get_user_default(key: &str) -> Option<String> {
    let packed = unsafe { ffi::host_get_user_default(key.as_ptr(), key.len() as i32) };
    let ptr = (packed >> 32) as *mut u8;
    let len = (packed & 0xFFFF_FFFF) as usize;

    if len == 0 || ptr.is_null() {
        return None;
    }

    unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        String::from_utf8(bytes).ok()
    }
}

/// Emit a `toolCall` event so the host activity log shows a tool-use entry.
/// `name` is the display string; use well-known prefixes for colour coding:
/// `"READ <path>"`, `"WRITE <path>"`, `"SEARCH <query>"` — anything else gets
/// a generic purple terminal icon.
pub fn push_tool_call(name: &str) {
    use ui::{push_ai_event, AIEvent};
    push_ai_event(
        "",
        &AIEvent::ToolCall {
            name: name.to_string(),
        },
    );
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::plugin_print(&format!($($arg)*));
    }
}

pub mod prelude {
    pub use crate::ai::{AIProvider, AIProviderRegistration, ModelInfo};
    pub use crate::log;
    pub use crate::process::{ChildProcess, PipeType, ProcessStatus, ReadResult};
    pub use crate::sse::{for_each_sse_line, sse_close, sse_post};
    pub use crate::ui::{
        push_ai_event, push_ui, update_state, AIEvent, ChatMessage, ChatPayload, ReplyPayload,
    };
    pub use crate::{
        get_user_default, http_get, http_post, push_tool_call, read_host_file, tcp_probe,
        workspace_symbols_from_host, EntryPoint, HttpResponse, LspSymbolInfo, PluginError,
        ProjectProfile, ResourceStatus, Result, RunStatus,
    };
    pub use chorograph_plugin_macros::chorograph_plugin;
}

pub use chorograph_plugin_macros::chorograph_plugin;
pub use serde_json;

#[derive(Debug)]
pub enum PluginError {
    HostError(i32),
    SerializationError(String),
    Other(String),
}

pub type Result<T> = std::result::Result<T, PluginError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[chorograph_plugin]
    fn init() {}

    #[chorograph_plugin]
    fn handle_action(action_id: String, payload: serde_json::Value) {
        assert_eq!(action_id, "test_action");
        assert_eq!(payload["foo"], "bar");
    }

    #[chorograph_plugin]
    fn on_workspace_change(event: serde_json::Value) {
        assert_eq!(event["type"], "file_mod");
    }

    #[test]
    fn test_handle_action_ffi() {
        let action = "test_action";
        let payload = r#"{"foo":"bar"}"#;

        unsafe {
            __ffi_handle_action(
                action.as_ptr(),
                action.len(),
                payload.as_ptr(),
                payload.len(),
            );
        }
    }

    #[test]
    fn test_on_workspace_change_ffi() {
        let event = r#"{"type":"file_mod"}"#;

        unsafe {
            __ffi_on_workspace_change(event.as_ptr(), event.len());
        }
    }

    #[test]
    fn test_allocate_deallocate() {
        let size = 1024;
        let ptr = allocate(size);
        assert!(!ptr.is_null());

        unsafe {
            // Write to memory to ensure it's valid
            for i in 0..size {
                *ptr.add(i) = (i % 256) as u8;
            }

            // Read back to verify
            for i in 0..size {
                assert_eq!(*ptr.add(i), (i % 256) as u8);
            }
        }

        deallocate(ptr, size);
    }
}

pub mod ai;
pub mod ffi;
pub mod process;
pub mod ui;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProfile {
    pub category: String,
    pub tags: Vec<String>,
}

pub fn plugin_print(msg: &str) {
    unsafe { ffi::print(msg.as_ptr(), msg.len() as i32) }
}

pub fn read_host_file(path: &str) -> Result<String> {
    let packed = unsafe { ffi::host_read_file(path.as_ptr(), path.len() as i32) };
    let ptr = (packed >> 32) as *mut u8;
    let len = (packed & 0xFFFFFFFF) as usize;

    if len == 0 || ptr.is_null() {
        return Err(PluginError::Other(format!("Failed to read host file: {}", path)));
    }

    unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        let s = String::from_utf8(bytes).map_err(|e| PluginError::SerializationError(e.to_string()))?;
        Ok(s)
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::plugin_print(&format!($($arg)*));
    }
}

pub mod prelude {
    pub use crate::ai::{AIProvider, ModelInfo, AIProviderRegistration};
    pub use crate::process::{ChildProcess, PipeType, ReadResult, ProcessStatus};
    pub use crate::ui::{AIEvent, push_ui, update_state, push_ai_event};
    pub use crate::{Result, PluginError, ProjectProfile, read_host_file};
    pub use chorograph_plugin_macros::chorograph_plugin;
    pub use crate::log;
}

pub use serde_json;
pub use chorograph_plugin_macros::chorograph_plugin;

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
                action.as_ptr(), action.len(),
                payload.as_ptr(), payload.len()
            );
        }
    }

    #[test]
    fn test_on_workspace_change_ffi() {
        let event = r#"{"type":"file_mod"}"#;
        
        unsafe {
            __ffi_on_workspace_change(
                event.as_ptr(), event.len()
            );
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

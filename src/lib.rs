pub mod ffi;

pub use serde_json;
pub use chorograph_plugin_macros::chorograph_plugin;

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

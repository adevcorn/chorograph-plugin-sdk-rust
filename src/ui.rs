use serde::{Serialize, Deserialize};
use crate::ffi;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AIEvent {
    #[serde(rename_all = "camelCase")]
    AssistantReply {
        session_id: String,
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    TurnCompleted {
        session_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Info {
        message: String,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        message: String,
    },
}

pub fn push_ui(json: &str) -> i32 {
    unsafe { ffi::host_push_ui(json.as_ptr(), json.len()) }
}

pub fn update_state<T: Serialize>(delta: &T) -> i32 {
    if let Ok(json) = serde_json::to_string(delta) {
        unsafe { ffi::host_update_state(json.as_ptr(), json.len()) }
    } else {
        -2
    }
}

pub fn push_ai_event(session_id: &str, event: &AIEvent) -> i32 {
    if let Ok(json) = serde_json::to_string(event) {
        unsafe {
            ffi::host_push_ai_event(
                session_id.as_ptr(),
                session_id.len(),
                json.as_ptr(),
                json.len(),
            )
        }
    } else {
        -2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_event_serialization() {
        let event = AIEvent::AssistantReply {
            session_id: "123".to_string(),
            text: "hello".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"assistantReply\""));
        assert!(json.contains("\"sessionId\":\"123\""));
        assert!(json.contains("\"text\":\"hello\""));
    }
}

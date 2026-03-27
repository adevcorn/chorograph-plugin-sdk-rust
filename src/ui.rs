use crate::ffi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AIEvent {
    #[serde(rename_all = "camelCase")]
    AssistantReply { session_id: String, text: String },
    #[serde(rename_all = "camelCase")]
    TurnCompleted { session_id: String },
    #[serde(rename_all = "camelCase")]
    Info { message: String },
    #[serde(rename_all = "camelCase")]
    Error { message: String },
    #[serde(rename_all = "camelCase")]
    StreamingDelta { session_id: String, text: String },
    #[serde(rename_all = "camelCase")]
    Reasoning { session_id: String, text: String },
    #[serde(rename_all = "camelCase")]
    PlanGenerated {
        session_id: String,
        files: Vec<String>,
    },
    /// Emitted when the AI needs a clarifying answer from the user before
    /// continuing.  The host will surface a reply field in the UI and send
    /// the user's answer back via the "reply" action.
    #[serde(rename_all = "camelCase")]
    Question { session_id: String, text: String },
    /// A discrete tool invocation (file read, write, search, API call, …).
    /// `name` should use the same prefixes the host recognises for colour coding:
    ///   "READ <path>", "WRITE <path>", "SEARCH <query>", or any other string
    ///   for a generic purple terminal icon.
    ToolCall { name: String },
}

/// A single entry in the conversation history sent with every "chat"/"reply" action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    /// "user" or "assistant"
    pub role: String,
    pub text: String,
}

/// Payload for the "chat" action (initial turn or first message in a session).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatPayload {
    pub session_id: String,
    /// Full message history up to and including the latest user message.
    pub messages: Vec<ChatMessage>,
    /// Optional AST skeletons for the files in scope.
    #[serde(default)]
    pub skeletons: Vec<serde_json::Value>,
}

/// Payload for the "reply" action (user answered an AI question).
/// Structurally identical to ChatPayload — the "reply" action ID is the
/// distinguisher so plugins can skip redundant planning steps.
pub type ReplyPayload = ChatPayload;

pub fn push_ui(json: &str) -> i32 {
    unsafe { ffi::host_push_ui(json.as_ptr(), json.len() as i32) }
}

pub fn update_state<T: Serialize>(delta: &T) -> i32 {
    if let Ok(json) = serde_json::to_string(delta) {
        unsafe { ffi::host_update_state(json.as_ptr(), json.len() as i32) }
    } else {
        -2
    }
}

pub fn push_ai_event(session_id: &str, event: &AIEvent) -> i32 {
    if let Ok(json) = serde_json::to_string(event) {
        unsafe {
            ffi::host_push_ai_event(
                session_id.as_ptr(),
                session_id.len() as i32,
                json.as_ptr(),
                json.len() as i32,
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

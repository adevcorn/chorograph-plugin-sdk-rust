use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AIEvent {
    ReadFile { path: String },
    WriteFile { path: String },
    PatchFile { path: String },
    ToolCall { name: String, input: HashMap<String, String> },
    AssistantReply { session_id: String, text: String },
    TurnFinished { session_id: String },
    Connected,
    Info(String),
    Error(String),
    Other { type_name: String },
}

fn main() {
    let event = AIEvent::Info("test".to_string());
    match serde_json::to_string(&event) {
        Ok(json) => println!("JSON: {}", json),
        Err(e) => println!("Error: {}", e),
    }
}

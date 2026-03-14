use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AIEvent {
    Connected,
    Info(String),
}

fn main() {
    let event = AIEvent::Connected;
    println!("{}", serde_json::to_string(&event).unwrap());

    let event = AIEvent::Info("test".to_string());
    println!("{}", serde_json::to_string(&event).unwrap());
}

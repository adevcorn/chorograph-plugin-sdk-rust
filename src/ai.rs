use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIProviderRegistration {
    pub id: String,
    pub display_name: String,
    pub supported_models: Vec<ModelInfo>,
}

pub trait AIProvider {
    fn id(&self) -> String;
    fn display_name(&self) -> String;
    fn get_models(&self) -> Vec<ModelInfo>;
    fn send_message(&self, session_id: &str, text: &str) -> Result<()>;
}

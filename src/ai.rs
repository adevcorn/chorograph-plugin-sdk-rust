//! AI provider plugin types.
//!
//! Implement [`AIProvider`] and register it with the Chorograph host to expose
//! a custom AI backend (any OpenAI-compatible API, local model, CLI wrapper, etc.)
//! in the chat panel.
//!
//! # Quick start
//! ```rust,ignore
//! use chorograph_plugin_sdk::ai::{AIProvider, AIProviderRegistration, ModelInfo};
//!
//! struct MyProvider;
//! impl AIProvider for MyProvider { /* ... */ }
//! ```

use crate::Result;
use serde::{Deserialize, Serialize};

/// Metadata for a single model offered by an AI provider.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    /// Unique model identifier sent in API requests (e.g. `"gpt-4o"`).
    pub id: String,
    /// Human-readable display name shown in the model picker.
    pub name: String,
}

/// Registration payload emitted by [`AIProvider::id`] and friends.
/// Returned by the `register_ai_provider` WASM export.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIProviderRegistration {
    /// Stable identifier for this provider (e.g. `"com.example.myprovider"`).
    pub id: String,
    /// Name shown in the Chorograph provider picker.
    pub display_name: String,
    /// Models this provider supports.
    pub supported_models: Vec<ModelInfo>,
}

/// Trait that AI provider plugins must implement.
///
/// Export `register_ai_provider`, `get_models`, and `send_message` WASM functions
/// (via the `#[chorograph_plugin]` macro) and delegate to these methods.
pub trait AIProvider {
    fn id(&self) -> String;
    fn display_name(&self) -> String;
    fn get_models(&self) -> Vec<ModelInfo>;
    /// Send `text` as part of session `session_id`.
    /// Stream assistant tokens back with [`crate::ui::push_ai_event`].
    fn send_message(&self, session_id: &str, text: &str) -> Result<()>;
}

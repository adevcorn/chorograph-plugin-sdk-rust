/// Capability tokens declared in a plugin's `provides` field in the registry.
///
/// The host uses these tokens to discover plugins by what they do rather than by ID,
/// which allows any third-party plugin to fulfil a capability without requiring changes
/// to the host application.
///
/// # Example registry entry
/// ```json
/// {
///   "id": "com.example.my-test-runner",
///   "provides": ["testRunner"],
///   ...
/// }
/// ```
///
/// # Dispatch semantics
/// | Token               | Host dispatch    | Notes                                        |
/// |---------------------|------------------|----------------------------------------------|
/// | `TEST_RUNNER`       | All providers    | Multiple test runners can coexist (one per language) |
/// | `SERVER_LIFECYCLE`  | First found      | Manages the background dev server process    |
/// | `EMBEDDED_TERMINAL` | First found      | Provides the embedded terminal panel UI      |
/// | `SERVER_STATUS`     | First found      | Exposes a live server URL/status for the HUD |
pub mod capabilities {
    /// Plugin manages a background server process.
    /// The host sends a `"shutdown"` action (see [`actions::SHUTDOWN`]) on app quit.
    pub const SERVER_LIFECYCLE: &str = "serverLifecycle";

    /// Plugin provides an embedded terminal panel UI.
    pub const EMBEDDED_TERMINAL: &str = "embeddedTerminal";

    /// Plugin can run tests for the current workspace.
    /// The host dispatches `"run_tests"` (see [`actions::RUN_TESTS`]) to **all** providers,
    /// so multiple test runners can coexist (e.g. Swift + Rust in the same repo).
    pub const TEST_RUNNER: &str = "testRunner";

    /// Plugin exposes a live server URL and status for the HUD status pill.
    pub const SERVER_STATUS: &str = "serverStatus";
}

/// Action IDs that the host dispatches to plugins for specific capabilities.
///
/// If your plugin declares a capability in [`capabilities`], you **must** handle
/// the corresponding action ID in your `handle_action` export.
///
/// # Example
/// ```rust
/// use chorograph_plugin_sdk_rust::prelude::*;
/// use chorograph_plugin_sdk_rust::actions;
///
/// #[chorograph_plugin]
/// fn handle_action(action_id: String, payload: serde_json::Value) {
///     match action_id.as_str() {
///         actions::RUN_TESTS => {
///             let cwd = payload["cwd"].as_str().unwrap_or("");
///             // run tests in cwd...
///         }
///         _ => {}
///     }
/// }
/// ```
pub mod actions {
    /// Dispatched to all plugins providing [`capabilities::TEST_RUNNER`].
    /// Payload: `{ "cwd": "<workspace_root>" }`
    pub const RUN_TESTS: &str = "run_tests";

    /// Dispatched to the plugin providing [`capabilities::SERVER_LIFECYCLE`] on app quit.
    /// Payload: `{}`
    pub const SHUTDOWN: &str = "shutdown";
}

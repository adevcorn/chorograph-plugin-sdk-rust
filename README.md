# chorograph-plugin-sdk-rust

Rust SDK for building [Chorograph](https://chorograph.app) plugins compiled to WebAssembly.

## Overview

Chorograph plugins are WASM modules that run inside a sandboxed Wasmtime host embedded
in the macOS app. They can:

- Identify project types and entry points in the spatial map
- Detect run/build status of projects
- Spawn host processes and stream their output
- Perform HTTP requests (bypassing the WASM network sandbox)
- Read and write files within the current workspace
- Emit AI conversation events and render custom SwiftUI panels
- Store and retrieve configuration via the host's UserDefaults

## Quick start

Add the SDK to your plugin's `Cargo.toml`:

```toml
[dependencies]
chorograph-plugin-sdk-rust = { git = "https://github.com/adevcorn/chorograph-plugin-sdk-rust", tag = "v0.2.4" }

[lib]
crate-type = ["cdylib"]
```

Build for WASM:

```sh
cargo build --target wasm32-unknown-unknown --release
```

## Writing a plugin

Use the `#[chorograph_plugin]` macro to export the required WASM entry points:

```rust
use chorograph_plugin_sdk::prelude::*;

#[chorograph_plugin]
fn init() {
    log!("Hello from my plugin!");
}

#[chorograph_plugin]
fn identify_project(workspace_root: String, root_files: Vec<String>) -> Option<ProjectProfile> {
    if root_files.iter().any(|f| f == "Cargo.toml") {
        Some(ProjectProfile {
            category: "Rust".into(),
            tags: vec!["rust".into()],
            entry_points: vec![],
        })
    } else {
        None
    }
}

#[chorograph_plugin]
fn handle_action(action_id: String, payload: serde_json::Value) {
    match action_id.as_str() {
        "build" => { /* spawn cargo build */ }
        _ => {}
    }
}
```

## Capability declarations

Declare which host capabilities your plugin provides in the registry `provides` array.
Use the constants from [`capabilities`] and [`actions`] to avoid typos:

```rust
use chorograph_plugin_sdk::capabilities;
// capabilities::TEST_RUNNER  → "testRunner"
// capabilities::EMBEDDED_TERMINAL → "embeddedTerminal"
// capabilities::SERVER_LIFECYCLE  → "serverLifecycle"
// capabilities::SERVER_STATUS     → "serverStatus"
```

## Key APIs

| Function | Description |
|---|---|
| `read_host_file(path)` | Read a file from the host filesystem |
| `write_host_file(path, bytes)` | Write bytes to a file (sandboxed to workspace root) |
| `get_user_default(key)` | Read a value from macOS UserDefaults |
| `set_user_default(key, value)` | Write a value to macOS UserDefaults |
| `http_get(url, headers)` | HTTP GET via the host network stack |
| `http_post(url, headers, body)` | HTTP POST via the host network stack |
| `tcp_probe(host, port)` | Check if a TCP port is open |
| `workspace_symbols_from_host(root)` | Query the host LSP for workspace symbols |
| `ChildProcess::spawn(cmd, args, cwd, env)` | Spawn a host process |
| `push_ui(json)` | Push a SwiftUI JSON description to the plugin panel |
| `push_ai_event(session_id, event)` | Emit an AI conversation event |

## Modules

- [`ai`] — AI provider registration types and trait
- [`capabilities`] — Capability and action name constants
- [`process`] — Host process spawning and I/O
- [`sse`] — Streaming HTTP (SSE / chunked transfer) helpers
- [`ui`] — Host UI push and AI event helpers
- [`ffi`] — Raw FFI declarations (use the safe wrappers above instead)

## License

MIT

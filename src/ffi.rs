extern "C" {
    pub fn print(ptr: *const u8, len: i32);
    /// Perform an HTTP GET request from the host (bypassing the WASM network sandbox).
    /// `url_ptr`/`url_len` — the request URL.
    /// `headers_ptr`/`headers_len` — optional JSON object of extra request headers, or null/0.
    /// Returns a packed u64: high 32 bits = guest pointer, low 32 bits = byte length of a
    /// JSON-encoded `HttpResponse` object allocated in guest memory.
    /// Returns 0 if the host cannot fulfil the request.
    pub fn host_http_get(
        url_ptr: *const u8,
        url_len: i32,
        headers_ptr: *const u8,
        headers_len: i32,
    ) -> u64;
    /// Perform an HTTP POST request from the host (bypassing the WASM network sandbox).
    /// `url_ptr`/`url_len` — the request URL.
    /// `headers_ptr`/`headers_len` — optional JSON object of extra request headers, or null/0.
    /// `body_ptr`/`body_len` — request body bytes, or null/0 for an empty body.
    /// Returns a packed u64: high 32 bits = guest pointer, low 32 bits = byte length of a
    /// JSON-encoded `HttpResponse` object allocated in guest memory.
    /// Returns 0 if the host cannot fulfil the request.
    pub fn host_http_post(
        url_ptr: *const u8,
        url_len: i32,
        headers_ptr: *const u8,
        headers_len: i32,
        body_ptr: *const u8,
        body_len: i32,
    ) -> u64;
    /// Read a single string value from the host's UserDefaults store.
    /// `key_ptr`/`key_len` — the UserDefaults key to read.
    /// Returns a packed u64: high 32 bits = guest pointer, low 32 bits = byte length of the
    /// UTF-8 value string allocated in guest memory.
    /// Returns 0 if the key is not set or the call fails.
    pub fn host_get_user_default(key_ptr: *const u8, key_len: i32) -> u64;
    /// Write a string value to the host's UserDefaults store.
    /// `key_ptr`/`key_len` — the UserDefaults key.
    /// `val_ptr`/`val_len` — the UTF-8 value to store.
    pub fn host_set_user_default(
        key_ptr: *const u8,
        key_len: i32,
        val_ptr: *const u8,
        val_len: i32,
    );
    /// Write `content_len` bytes from `content_ptr` to the file at `path`.
    /// The path must be inside the workspace root; writes outside it are blocked by the host.
    /// Returns 0 on success, negative on error:
    ///   -1 invalid pointer, -2 outside workspace root, -3 mkdir failed, -4 write failed.
    pub fn host_write_file(
        path_ptr: *const u8,
        path_len: i32,
        content_ptr: *const u8,
        content_len: i32,
    ) -> i32;
    pub fn host_spawn(
        cmd_ptr: *const u8,
        cmd_len: i32,
        args_ptr: *const u8,
        args_len: i32,
        cwd_ptr: *const u8,
        cwd_len: i32,
        env_ptr: *const u8,
        env_len: i32,
    ) -> i32;
    pub fn host_read(handle: i32, pipe: i32, buf_ptr: *mut u8, buf_len: i32) -> i32;
    pub fn host_write(handle: i32, buf_ptr: *const u8, buf_len: i32) -> i32;
    pub fn host_wait_for_data(handle: i32, timeout_ms: i32) -> i32;
    pub fn host_get_status(handle: i32) -> i32;
    pub fn host_kill(handle: i32) -> i32;
    pub fn host_close(handle: i32) -> i32;
    pub fn host_push_ui(json_ptr: *const u8, json_len: i32) -> i32;
    pub fn host_update_state(json_ptr: *const u8, json_len: i32) -> i32;
    pub fn host_push_ai_event(
        session_ptr: *const u8,
        session_len: i32,
        event_ptr: *const u8,
        event_len: i32,
    ) -> i32;
    pub fn host_read_file(path_ptr: *const u8, path_len: i32) -> u64;
    /// Query the host LSP orchestrator for workspace symbols under `root`.
    /// Returns a packed u64: high 32 bits = guest pointer, low 32 bits = byte length
    /// of a JSON-encoded array of `LspSymbolInfo` objects allocated in guest memory.
    /// Returns 0 if no LSP session is available or the query fails.
    pub fn host_workspace_symbols(root_ptr: *const u8, root_len: i32) -> u64;
    /// Probe whether a TCP port is listening on `host`.
    /// The host performs a non-blocking connect attempt with a short timeout (~200 ms).
    /// Returns 1 if the port is open (something is listening), 0 otherwise.
    pub fn host_tcp_probe(host_ptr: *const u8, host_len: i32, port: i32) -> i32;

    /// Open a streaming HTTP POST connection (SSE / chunked transfer).
    /// `url_ptr`/`url_len` — request URL.
    /// `headers_ptr`/`headers_len` — optional JSON object of extra request headers, or null/0.
    /// `body_ptr`/`body_len` — request body bytes, or null/0 for an empty body.
    /// Returns a handle (>= 1) on success, or 0 on error.
    pub fn host_sse_post(
        url_ptr: *const u8,
        url_len: i32,
        headers_ptr: *const u8,
        headers_len: i32,
        body_ptr: *const u8,
        body_len: i32,
    ) -> i32;

    /// Read up to `buf_len` bytes from an open SSE stream into `buf_ptr`.
    /// Returns:
    ///   > 0  — bytes written into buffer
    ///     0  — no data available yet (stream still open)
    ///    -1  — stream ended or an error occurred (or handle is invalid)
    pub fn host_sse_read(handle: i32, buf_ptr: *mut u8, buf_len: i32) -> i32;

    /// Close and release an SSE stream handle.
    /// Returns 0 on success, -1 if the handle was not found.
    pub fn host_sse_close(handle: i32) -> i32;
}

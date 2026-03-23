extern "C" {
    pub fn print(ptr: *const u8, len: i32);
    pub fn host_spawn(cmd_ptr: *const u8, cmd_len: i32, args_ptr: *const u8, args_len: i32, cwd_ptr: *const u8, cwd_len: i32, env_ptr: *const u8, env_len: i32) -> i32;
    pub fn host_read(handle: i32, pipe: i32, buf_ptr: *mut u8, buf_len: i32) -> i32;
    pub fn host_write(handle: i32, buf_ptr: *const u8, buf_len: i32) -> i32;
    pub fn host_wait_for_data(handle: i32, timeout_ms: i32) -> i32;
    pub fn host_get_status(handle: i32) -> i32;
    pub fn host_kill(handle: i32) -> i32;
    pub fn host_close(handle: i32) -> i32;
    pub fn host_push_ui(json_ptr: *const u8, json_len: i32) -> i32;
    pub fn host_update_state(json_ptr: *const u8, json_len: i32) -> i32;
    pub fn host_push_ai_event(session_ptr: *const u8, session_len: i32, event_ptr: *const u8, event_len: i32) -> i32;
    pub fn host_read_file(path_ptr: *const u8, path_len: i32) -> u64;
}

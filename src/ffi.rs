extern "C" {
    pub fn host_spawn(cmd_ptr: *const u8, cmd_len: usize, args_ptr: *const u8, args_len: usize, cwd_ptr: *const u8, cwd_len: usize, env_ptr: *const u8, env_len: usize) -> i32;
    pub fn host_read(handle: i32, pipe: i32, buf_ptr: *mut u8, buf_len: usize) -> i32;
    pub fn host_write(handle: i32, buf_ptr: *const u8, buf_len: usize) -> i32;
    pub fn host_wait_for_data(handle: i32, timeout_ms: u32) -> i32;
    pub fn host_get_status(handle: i32) -> i32;
    pub fn host_kill(handle: i32) -> i32;
    pub fn host_close(handle: i32) -> i32;
    pub fn host_push_ui(json_ptr: *const u8, json_len: usize) -> i32;
    pub fn host_update_state(json_ptr: *const u8, json_len: usize) -> i32;
    pub fn host_push_ai_event(session_ptr: *const u8, session_len: usize, event_ptr: *const u8, event_len: usize) -> i32;
}

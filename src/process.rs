use crate::{ffi, Result, PluginError};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pipe {
    Stdout = 1,
    Stderr = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Signaled(i32),
}

pub struct ChildProcess {
    handle: i32,
}

impl ChildProcess {
    /// Spawns a process. Returns a handle or error.
    pub fn spawn(
        cmd: &str,
        args: &[String],
        cwd: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<Self> {
        let cmd_ptr = cmd.as_ptr();
        let cmd_len = cmd.len();

        let args_json = serde_json::to_vec(args)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        let args_ptr = args_json.as_ptr();
        let args_len = args_json.len();

        let (cwd_ptr, cwd_len) = if let Some(cwd) = cwd {
            (cwd.as_ptr(), cwd.len())
        } else {
            (std::ptr::null(), 0)
        };

        let env_json = serde_json::to_vec(env)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        let env_ptr = env_json.as_ptr();
        let env_len = env_json.len();

        let handle = unsafe {
            ffi::host_spawn(
                cmd_ptr, cmd_len,
                args_ptr, args_len,
                cwd_ptr, cwd_len,
                env_ptr, env_len,
            )
        };

        if handle < 0 {
            Err(PluginError::HostError(handle))
        } else {
            Ok(Self { handle })
        }
    }

    /// Reads data. Returns bytes read, 0 for EOF, -1 for Empty, -2 for Error.
    /// Mapping to Rust: 0 -> Ok(0), -1 -> Ok(0) (if we want to return 0 for non-blocking read with no data)
    /// Actually, let's return Result<Option<usize>> where None means would block/empty.
    pub fn read(&self, pipe: Pipe, buf: &mut [u8]) -> Result<Option<usize>> {
        let res = unsafe {
            ffi::host_read(self.handle, pipe as i32, buf.as_mut_ptr(), buf.len())
        };

        match res {
            r if r >= 0 => Ok(Some(r as usize)),
            -1 => Ok(None),
            -2 => Err(PluginError::HostError(-2)),
            other => Err(PluginError::HostError(other)),
        }
    }

    /// Writes to stdin.
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let res = unsafe {
            ffi::host_write(self.handle, data.as_ptr(), data.len())
        };

        if res < 0 {
            Err(PluginError::HostError(res))
        } else {
            Ok(res as usize)
        }
    }

    /// Blocks until data or timeout. Returns true for data, false for timeout.
    pub fn wait_for_data(&self, timeout_ms: u32) -> bool {
        let res = unsafe {
            ffi::host_wait_for_data(self.handle, timeout_ms)
        };
        res == 1
    }

    /// Returns status (256=Running, 0-255=Exit, 512+=Signal).
    pub fn get_status(&self) -> ProcessStatus {
        let res = unsafe { ffi::host_get_status(self.handle) };
        if res == 256 {
            ProcessStatus::Running
        } else if res >= 512 {
            ProcessStatus::Signaled(res - 512)
        } else if res >= 0 {
            ProcessStatus::Exited(res)
        } else {
            // Treat host errors as exited with -1 for now
            ProcessStatus::Exited(-1)
        }
    }

    /// Terminates process.
    pub fn kill(&self) -> Result<()> {
        let res = unsafe { ffi::host_kill(self.handle) };
        if res == 0 {
            Ok(())
        } else {
            Err(PluginError::HostError(res))
        }
    }

    pub fn handle(&self) -> i32 {
        self.handle
    }
}

impl Drop for ChildProcess {
    fn drop(&mut self) {
        unsafe {
            ffi::host_close(self.handle);
        }
    }
}

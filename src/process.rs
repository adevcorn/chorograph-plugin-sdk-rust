use crate::{ffi, PluginError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeType {
    Stdout = 1,
    Stderr = 2,
}

#[derive(Debug)]
pub enum ReadResult {
    Data(Vec<u8>),
    EOF,
    Empty,
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
        args: Vec<&str>,
        cwd: Option<&str>,
        env: HashMap<String, String>,
    ) -> Result<Self> {
        let cmd_ptr = cmd.as_ptr();
        let cmd_len = cmd.len() as i32;

        let args_json = serde_json::to_vec(&args)
            .map_err(|e| PluginError::SerializationError(e.to_string()))?;
        let args_ptr = args_json.as_ptr();
        let args_len = args_json.len() as i32;

        let (cwd_ptr, cwd_len) = if let Some(cwd) = cwd {
            (cwd.as_ptr(), cwd.len() as i32)
        } else {
            (std::ptr::null(), 0)
        };

        let env_json =
            serde_json::to_vec(&env).map_err(|e| PluginError::SerializationError(e.to_string()))?;
        let env_ptr = env_json.as_ptr();
        let env_len = env_json.len() as i32;

        let handle = unsafe {
            ffi::host_spawn(
                cmd_ptr, cmd_len, args_ptr, args_len, cwd_ptr, cwd_len, env_ptr, env_len,
            )
        };

        if handle < 0 {
            Err(PluginError::HostError(handle))
        } else {
            Ok(Self { handle })
        }
    }

    /// Reads data into a buffer. Returns bytes read, 0 for EOF, -1 for Empty, -2 for Error.
    pub fn read_raw(&self, pipe: PipeType, buf: &mut [u8]) -> Result<Option<usize>> {
        let res =
            unsafe { ffi::host_read(self.handle, pipe as i32, buf.as_mut_ptr(), buf.len() as i32) };

        match res {
            r if r >= 0 => Ok(Some(r as usize)),
            -1 => Ok(None),
            -2 => Err(PluginError::HostError(-2)),
            other => Err(PluginError::HostError(other)),
        }
    }

    pub fn read(&self, pipe: PipeType) -> Result<ReadResult> {
        let mut buf = [0u8; 4096];
        match self.read_raw(pipe, &mut buf)? {
            Some(0) => Ok(ReadResult::EOF),
            Some(n) => Ok(ReadResult::Data(buf[..n].to_vec())),
            None => Ok(ReadResult::Empty),
        }
    }

    /// Writes to stdin.
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let res = unsafe { ffi::host_write(self.handle, data.as_ptr(), data.len() as i32) };

        if res < 0 {
            Err(PluginError::HostError(res))
        } else {
            Ok(res as usize)
        }
    }

    /// Blocks until data or timeout. Returns true for data, false for timeout.
    pub fn wait_for_data(&self, timeout_ms: u32) -> bool {
        let res = unsafe { ffi::host_wait_for_data(self.handle, timeout_ms as i32) };
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

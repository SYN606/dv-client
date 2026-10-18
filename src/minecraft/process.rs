//! Game Process Supervisor and Terminal Output Streamer.
//!
//! Spawns Minecraft directly as an isolated child process without shell execution,
//! streams real-time stdout/stderr, applies in-flight credential scrubbing, and tracks exit codes.

use crate::security::redaction::{sanitize_cmd_args, sanitize_text};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Stopped,
    Starting,
    Running,
    Crashed(i32),
    Exited(i32),
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("I/O error starting or supervising process: {0}")]
    Io(#[from] std::io::Error),
    #[error("No executable argument provided")]
    MissingExecutable,
    #[error("Game process is already running")]
    AlreadyRunning,
}

#[derive(Clone)]
pub struct ProcessSupervisor {
    child_handle: Arc<Mutex<Option<Child>>>,
    pub output_tx: broadcast::Sender<String>,
    pub state_tx: broadcast::Sender<ProcessState>,
    is_running: Arc<AtomicBool>,
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        let (output_tx, _) = broadcast::channel(1024);
        let (state_tx, _) = broadcast::channel(32);

        Self {
            child_handle: Arc::new(Mutex::new(None)),
            output_tx,
            state_tx,
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Launch Minecraft process directly and stream sanitized stdout and stderr.
    pub async fn launch<P: AsRef<Path>>(
        &self,
        args: Vec<String>,
        game_dir: P,
    ) -> Result<i32, ProcessError> {
        self.launch_with_env(args, game_dir, Vec::new()).await
    }

    /// Launch Minecraft process with custom environment variables (e.g. GPU offloading).
    pub async fn launch_with_env<P: AsRef<Path>>(
        &self,
        args: Vec<String>,
        game_dir: P,
        env_vars: Vec<(String, String)>,
    ) -> Result<i32, ProcessError> {
        if self.is_active() {
            return Err(ProcessError::AlreadyRunning);
        }

        if args.is_empty() {
            return Err(ProcessError::MissingExecutable);
        }

        let game_dir = game_dir.as_ref().to_path_buf();
        fs::create_dir_all(&game_dir)?;
        let logs_dir = game_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;
        let log_file = logs_dir.join("latest.log");

        let _ = self.state_tx.send(ProcessState::Starting);
        self.is_running.store(true, Ordering::SeqCst);

        let sanitized_cmd = sanitize_cmd_args(&args).join(" ");
        info!("Launching Minecraft: {}", sanitized_cmd);
        let _ = self.output_tx.send(format!("[DV] Spawning: {}\n", sanitized_cmd));

        let program = &args[0];
        let prog_args = &args[1..];

        let mut cmd = Command::new(program);
        cmd.args(prog_args)
            .current_dir(&game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000);
        }

        for (k, v) in &env_vars {
            cmd.env(k, v);
            info!("Applied process environment: {}={}", k, v);
        }

        let mut child = cmd.spawn()?;

        #[cfg(target_os = "windows")]
        if let Some(pid) = child.id() {
            assign_child_to_kill_on_close_job(pid);
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut handle = self.child_handle.lock().await;
            *handle = Some(child);
        }

        let _ = self.state_tx.send(ProcessState::Running);

        // Spawn stream reader tasks
        let out_sender = self.output_tx.clone();
        let log_path_clone = log_file.clone();

        let stdout_task = tokio::spawn(async move {
            if let Some(out) = stdout {
                let mut reader = BufReader::new(out).lines();
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path_clone)
                    .ok();

                while let Ok(Some(line)) = reader.next_line().await {
                    let sanitized = sanitize_text(&line);
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "{}", sanitized);
                    }
                    let _ = out_sender.send(format!("{}\n", sanitized));
                }
            }
        });

        let err_sender = self.output_tx.clone();
        let stderr_task = tokio::spawn(async move {
            if let Some(err) = stderr {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let sanitized = sanitize_text(&line);
                    let _ = err_sender.send(format!("[STDERR] {}\n", sanitized));
                }
            }
        });

        let (stdout_res, stderr_res) = tokio::join!(stdout_task, stderr_task);
        let _ = stdout_res;
        let _ = stderr_res;

        let exit_status = {
            let mut handle = self.child_handle.lock().await;
            if let Some(mut child) = handle.take() {
                child.wait().await.ok()
            } else {
                None
            }
        };

        self.is_running.store(false, Ordering::SeqCst);

        let code = exit_status.and_then(|s| s.code()).unwrap_or(-1);

        if code == 0 {
            info!("Game process exited cleanly with code 0");
            let _ = self.output_tx.send("[DV] Process exited cleanly.\n".to_string());
            let _ = self.state_tx.send(ProcessState::Exited(0));
        } else {
            error!("Game process exited with error code: {}", code);
            let _ = self.output_tx.send(format!("[DV] Process exited with code {}\n", code));
            let _ = self.state_tx.send(ProcessState::Crashed(code));
        }

        Ok(code)
    }

    /// Forcibly terminate the running game process.
    pub async fn terminate(&self) {
        let mut handle = self.child_handle.lock().await;
        if let Some(mut child) = handle.take() {
            let _ = child.kill().await;
            info!("Terminated child game process");
        }
        self.is_running.store(false, Ordering::SeqCst);
        let _ = self.state_tx.send(ProcessState::Stopped);
    }
}

#[cfg(target_os = "windows")]
fn assign_child_to_kill_on_close_job(pid: u32) {
    #[repr(C)]
    struct IO_COUNTERS {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        basic_limit_information: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        io_info: IO_COUNTERS,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_limit: usize,
        peak_job_memory_limit: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateJobObjectW(lpJobAttributes: *const std::ffi::c_void, lpName: *const u16) -> *mut std::ffi::c_void;
        fn SetInformationJobObject(
            hJob: *mut std::ffi::c_void,
            JobObjectInfoClass: i32,
            lpJobObjectInfo: *const std::ffi::c_void,
            cbJobObjectInfoLength: u32,
        ) -> i32;
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
        fn AssignProcessToJobObject(hJob: *mut std::ffi::c_void, hProcess: *mut std::ffi::c_void) -> i32;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }

    unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if !job.is_null() {
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.basic_limit_information.limit_flags = 0x00002000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            let set_res = SetInformationJobObject(
                job,
                9, // JobObjectExtendedLimitInformation
                &info as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if set_res != 0 {
                // PROCESS_SET_QUOTA (0x0100) | PROCESS_TERMINATE (0x0001)
                let proc = OpenProcess(0x0100 | 0x0001, 0, pid);
                if !proc.is_null() {
                    let _ = AssignProcessToJobObject(job, proc);
                    CloseHandle(proc);
                }
            }
            let _ = job;
        }
    }
}

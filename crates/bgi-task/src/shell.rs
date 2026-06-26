use crate::{Result, TaskError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShellConfig {
    pub disable: bool,
    pub timeout: i32,
    pub no_window: bool,
    pub output: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            disable: false,
            timeout: 60,
            no_window: true,
            output: true,
        }
    }
}

impl ShellConfig {
    pub fn from_value(value: Option<&Value>) -> Self {
        value
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellTaskParam {
    pub command: String,
    pub timeout_seconds: i32,
    pub no_window: bool,
    pub output: bool,
    pub disable: bool,
    pub working_directory: PathBuf,
}

impl ShellTaskParam {
    pub fn build_from_config(
        command: impl Into<String>,
        config: ShellConfig,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            command: command.into(),
            timeout_seconds: config.timeout,
            no_window: config.no_window,
            output: config.output,
            disable: config.disable,
            working_directory: working_directory.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellExecutionStatus {
    Disabled,
    EmptyCommand,
    Started,
    Completed,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExecutionResult {
    pub command: String,
    pub working_directory: PathBuf,
    pub timeout_seconds: i32,
    pub no_window: bool,
    pub output_enabled: bool,
    pub status: ShellExecutionStatus,
    pub waited_for_exit: bool,
    pub exit_code: Option<i32>,
    pub output_shell: String,
    pub output: String,
}

impl ShellExecutionResult {
    pub fn has_output(&self) -> bool {
        !self.output_shell.is_empty() || !self.output.is_empty()
    }
}

pub fn execute_shell_task(param: &ShellTaskParam) -> Result<ShellExecutionResult> {
    execute_shell_task_with_cancel(param, || false)
}

pub fn execute_shell_task_with_cancel(
    param: &ShellTaskParam,
    mut is_cancelled: impl FnMut() -> bool,
) -> Result<ShellExecutionResult> {
    if param.disable {
        return Ok(shell_result(param, ShellExecutionStatus::Disabled, false));
    }
    if param.command.trim().is_empty() {
        return Ok(shell_result(
            param,
            ShellExecutionStatus::EmptyCommand,
            false,
        ));
    }

    let mut child = shell_command(param)
        .spawn()
        .map_err(TaskError::ShellStart)?;
    {
        let mut stdin = child.stdin.take().ok_or_else(|| {
            TaskError::ShellIo(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "shell stdin was not captured",
            ))
        })?;
        writeln!(stdin, "{}", param.command).map_err(TaskError::ShellIo)?;
        stdin.flush().map_err(TaskError::ShellIo)?;
    }

    if param.timeout_seconds <= 0 {
        return Ok(shell_result(param, ShellExecutionStatus::Started, false));
    }

    let deadline = Duration::from_secs(param.timeout_seconds as u64);
    let start = std::time::Instant::now();
    loop {
        if is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(ShellExecutionResult {
                status: ShellExecutionStatus::Cancelled,
                waited_for_exit: true,
                ..shell_result(param, ShellExecutionStatus::Cancelled, true)
            });
        }
        if let Some(status) = child.try_wait().map_err(TaskError::ShellIo)? {
            let output = child.wait_with_output().map_err(TaskError::ShellIo)?;
            let (output_shell, output_text) = if param.output {
                split_legacy_shell_output(&String::from_utf8_lossy(&output.stdout))
            } else {
                (String::new(), String::new())
            };
            return Ok(ShellExecutionResult {
                command: param.command.clone(),
                working_directory: param.working_directory.clone(),
                timeout_seconds: param.timeout_seconds,
                no_window: param.no_window,
                output_enabled: param.output,
                status: ShellExecutionStatus::Completed,
                waited_for_exit: true,
                exit_code: status.code(),
                output_shell,
                output: output_text,
            });
        }
        if start.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(ShellExecutionResult {
                status: ShellExecutionStatus::TimedOut,
                waited_for_exit: true,
                ..shell_result(param, ShellExecutionStatus::TimedOut, true)
            });
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn shell_result(
    param: &ShellTaskParam,
    status: ShellExecutionStatus,
    waited_for_exit: bool,
) -> ShellExecutionResult {
    ShellExecutionResult {
        command: param.command.clone(),
        working_directory: param.working_directory.clone(),
        timeout_seconds: param.timeout_seconds,
        no_window: param.no_window,
        output_enabled: param.output,
        status,
        waited_for_exit,
        exit_code: None,
        output_shell: String::new(),
        output: String::new(),
    }
}

#[cfg(windows)]
fn shell_command(param: &ShellTaskParam) -> Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = Command::new("cmd.exe");
    command.arg("/k").arg("@echo off");
    if param.no_window {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    configure_shell_command(&mut command, param);
    command
}

#[cfg(not(windows))]
fn shell_command(param: &ShellTaskParam) -> Command {
    let mut command = Command::new("sh");
    configure_shell_command(&mut command, param);
    command
}

fn configure_shell_command(command: &mut Command, param: &ShellTaskParam) {
    command.current_dir(&param.working_directory);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
}

fn split_legacy_shell_output(stdout: &str) -> (String, String) {
    let normalized = stdout.replace("\r\n", "\n");
    let mut lines = normalized.lines();
    let output_shell = lines.next().unwrap_or_default().to_string();
    let output = lines.collect::<Vec<_>>().join("\n");
    (output_shell, output)
}

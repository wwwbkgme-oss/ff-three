//! Isolated code executor.
//!
//! Execution strategy (priority order):
//!   1. Docker container – resource-limited, network-off (production)
//!   2. Subprocess – Python/Node only (development fallback)
//!
//! The security scanner is always called first; a violation aborts immediately.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::Utc;
use tokio::{process::Command, time::timeout};

use types::{ProgrammingLanguage, SandboxStatus, SecurityScan};

use super::scanner::SecurityScanner;

// ── Output type ───────────────────────────────────────────────────────────────

/// Result of one sandbox execution.
#[derive(Debug)]
pub struct ExecutionResult {
    pub status:            SandboxStatus,
    pub stdout:            Option<String>,
    pub stderr:            Option<String>,
    pub exit_code:         Option<i32>,
    pub execution_time_ms: i64,
    pub memory_used_kb:    i64,
    pub security_scan:     SecurityScan,
}

// ── Executor ──────────────────────────────────────────────────────────────────

pub struct SandboxExecutor {
    pub timeout_secs:   u64,
    pub max_memory_mb:  u64,
}

impl Default for SandboxExecutor {
    fn default() -> Self { Self { timeout_secs: 30, max_memory_mb: 128 } }
}

impl SandboxExecutor {
    pub fn new(timeout_secs: u64, max_memory_mb: u64) -> Self {
        Self { timeout_secs, max_memory_mb }
    }

    /// Execute `code` in the appropriate sandbox.
    pub async fn execute(
        &self,
        code:     &str,
        language: &ProgrammingLanguage,
    ) -> ExecutionResult {
        // 1. Security gate – always first.
        let scan = SecurityScanner::scan(code, language);
        if !scan.passed {
            return ExecutionResult {
                status:            SandboxStatus::SecurityViolation,
                stdout:            None,
                stderr:            Some(format!("Blocked: {}", scan.threats_detected.join(", "))),
                exit_code:         Some(126),
                execution_time_ms: 0,
                memory_used_kb:    0,
                security_scan:     scan,
            };
        }

        // 2. Docker (preferred) or subprocess fallback.
        if Self::docker_available().await {
            self.run_docker(code, language, scan).await
        } else {
            self.run_subprocess(code, language, scan).await
        }
    }

    // ── Docker path ───────────────────────────────────────────────────────────

    async fn docker_available() -> bool {
        Command::new("docker").arg("info").output().await
            .map(|o| o.status.success()).unwrap_or(false)
    }

    async fn run_docker(
        &self,
        code:     &str,
        language: &ProgrammingLanguage,
        scan:     SecurityScan,
    ) -> ExecutionResult {
        let tmp = match tempfile::tempdir() {
            Ok(d)  => d,
            Err(e) => return self.io_error(&format!("tempdir: {e}"), scan),
        };
        let file = tmp.path().join(format!("solution.{}", language.file_extension()));
        if std::fs::write(&file, code).is_err() {
            return self.io_error("write failed", scan);
        }

        let cmd = Self::docker_run_cmd(language, &file);
        let start = Instant::now();

        let output = timeout(
            Duration::from_secs(self.timeout_secs),
            Command::new("docker")
                .args([
                    "run", "--rm", "--network=none",
                    "--memory",  &format!("{}m", self.max_memory_mb),
                    "--cpus",    "0.5",
                    "-v", &format!("{}:/code:ro", file.display()),
                    language.docker_image(),
                    "sh", "-c", &cmd,
                ])
                .output(),
        )
        .await;

        self.interpret(output, start, scan)
    }

    fn docker_run_cmd(lang: &ProgrammingLanguage, file: &PathBuf) -> String {
        let fname = file.file_name().and_then(|n| n.to_str()).unwrap_or("solution");
        match lang {
            ProgrammingLanguage::Python     => format!("python3 /code/{fname}"),
            ProgrammingLanguage::JavaScript => format!("node /code/{fname}"),
            ProgrammingLanguage::TypeScript => format!("npx ts-node /code/{fname}"),
            ProgrammingLanguage::Rust       => format!("rustc /code/{fname} -o /tmp/bin && /tmp/bin"),
            ProgrammingLanguage::Go         => format!("go run /code/{fname}"),
            ProgrammingLanguage::Java       => format!("javac /code/{fname} && java -cp /tmp Solution"),
            ProgrammingLanguage::Cpp        => format!("g++ /code/{fname} -o /tmp/bin && /tmp/bin"),
        }
    }

    // ── Subprocess fallback ───────────────────────────────────────────────────

    async fn run_subprocess(
        &self,
        code:     &str,
        language: &ProgrammingLanguage,
        scan:     SecurityScan,
    ) -> ExecutionResult {
        let tmp = match tempfile::tempdir() {
            Ok(d)  => d,
            Err(e) => return self.io_error(&format!("tempdir: {e}"), scan),
        };
        let file = tmp.path().join(format!("solution.{}", language.file_extension()));
        if std::fs::write(&file, code).is_err() {
            return self.io_error("write failed", scan);
        }

        let (prog, args): (&str, Vec<String>) = match language {
            ProgrammingLanguage::Python     => ("python3", vec![file.to_string_lossy().into()]),
            ProgrammingLanguage::JavaScript => ("node",    vec![file.to_string_lossy().into()]),
            _ => {
                return ExecutionResult {
                    status:            SandboxStatus::Failed,
                    stdout:            None,
                    stderr:            Some(format!("{language:?} requires Docker for subprocess execution")),
                    exit_code:         Some(1),
                    execution_time_ms: 0,
                    memory_used_kb:    0,
                    security_scan:     scan,
                };
            }
        };

        let start = Instant::now();
        let output = timeout(
            Duration::from_secs(self.timeout_secs),
            Command::new(prog).args(&args).output(),
        )
        .await;

        self.interpret(output, start, scan)
    }

    // ── Shared output interpreter ─────────────────────────────────────────────

    fn interpret(
        &self,
        output: Result<Result<std::process::Output, std::io::Error>, tokio::time::error::Elapsed>,
        start:  Instant,
        scan:   SecurityScan,
    ) -> ExecutionResult {
        let elapsed = start.elapsed().as_millis() as i64;
        match output {
            Err(_) => ExecutionResult {
                status: SandboxStatus::Timeout,
                stdout: None,
                stderr: Some("Execution timed out".into()),
                exit_code: None, execution_time_ms: elapsed, memory_used_kb: 0, security_scan: scan,
            },
            Ok(Err(e)) => ExecutionResult {
                status: SandboxStatus::Failed,
                stdout: None,
                stderr: Some(format!("Spawn error: {e}")),
                exit_code: Some(1), execution_time_ms: elapsed, memory_used_kb: 0, security_scan: scan,
            },
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
                let code   = out.status.code().unwrap_or(1);
                ExecutionResult {
                    status:    if out.status.success() { SandboxStatus::Completed } else { SandboxStatus::Failed },
                    stdout:    if stdout.is_empty() { None } else { Some(stdout) },
                    stderr:    if stderr.is_empty() { None } else { Some(stderr) },
                    exit_code: Some(code),
                    execution_time_ms: elapsed,
                    memory_used_kb:    0,
                    security_scan:     scan,
                }
            }
        }
    }

    fn io_error(&self, msg: &str, scan: SecurityScan) -> ExecutionResult {
        ExecutionResult {
            status:            SandboxStatus::Failed,
            stdout:            None,
            stderr:            Some(msg.to_owned()),
            exit_code:         Some(1),
            execution_time_ms: 0,
            memory_used_kb:    0,
            security_scan:     scan,
        }
    }
}

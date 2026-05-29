//! `sandbox` – runtime: sandboxed code execution and security scanning.
//!
//! Exposes two independent concerns:
//!   • [`scanner::SecurityScanner`] – static threat detection (pure, no I/O)
//!   • [`executor::SandboxExecutor`] – isolated process / Docker execution (I/O)

pub mod executor;
pub mod scanner;

pub use executor::{ExecutionResult, SandboxExecutor};
pub use scanner::SecurityScanner;

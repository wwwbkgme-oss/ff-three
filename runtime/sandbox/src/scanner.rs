//! Static security scanner – pure analysis, no I/O.
//!
//! Detects dangerous code patterns before any execution occurs.
//! Called by the executor as the first gate.

use chrono::Utc;

use types::{ProgrammingLanguage, SecurityScan};

pub struct SecurityScanner;

impl SecurityScanner {
    /// Scan `code` for dangerous patterns and return a populated [`SecurityScan`].
    /// Deterministic: same code always produces the same result.
    pub fn scan(code: &str, language: &ProgrammingLanguage) -> SecurityScan {
        let mut threats = Vec::new();

        // ── Universal patterns ────────────────────────────────────────────────
        for (pattern, label) in UNIVERSAL_PATTERNS {
            if code.contains(pattern) {
                threats.push(label.to_string());
            }
        }

        // ── Language-specific patterns ────────────────────────────────────────
        let lang_patterns: &[(&str, &str)] = match language {
            ProgrammingLanguage::Python     => PYTHON_PATTERNS,
            ProgrammingLanguage::Rust       => RUST_PATTERNS,
            ProgrammingLanguage::JavaScript
            | ProgrammingLanguage::TypeScript => JS_PATTERNS,
            _ => &[],
        };

        for (pattern, label) in lang_patterns {
            if code.contains(pattern) {
                threats.push(label.to_string());
            }
        }

        threats.dedup();

        let risk_level = match threats.len() {
            0 => "none",
            1 => "low",
            2 => "medium",
            _ => "high",
        };

        SecurityScan {
            passed: threats.is_empty(),
            threats_detected: threats,
            risk_level: risk_level.to_string(),
            scanned_at: Utc::now(),
        }
    }
}

// ── Pattern tables ────────────────────────────────────────────────────────────

const UNIVERSAL_PATTERNS: &[(&str, &str)] = &[
    ("eval(",         "eval() usage"),
    ("exec(",         "exec() usage"),
    ("/etc/passwd",   "Sensitive file reference"),
    ("/etc/shadow",   "Sensitive file reference"),
    ("127.0.0.1",     "Localhost network reference"),
    ("0.0.0.0",       "Wildcard bind"),
];

const PYTHON_PATTERNS: &[(&str, &str)] = &[
    ("import os",          "OS module import"),
    ("import sys",         "sys module import"),
    ("import subprocess",  "subprocess import"),
    ("import socket",      "socket import"),
    ("import ctypes",      "ctypes import"),
    ("__import__",         "Dynamic import"),
    ("open(",              "File I/O"),
    ("os.system",          "Shell execution"),
    ("os.popen",           "Shell execution"),
];

const RUST_PATTERNS: &[(&str, &str)] = &[
    ("std::process::Command", "Shell execution"),
    ("std::fs::File",         "File I/O"),
    ("std::net::TcpStream",   "Network I/O"),
    ("unsafe {",              "Unsafe block"),
    ("#[no_mangle]",          "FFI symbol export"),
];

const JS_PATTERNS: &[(&str, &str)] = &[
    ("require('fs')",             "File-system access"),
    ("require(\"fs\")",           "File-system access"),
    ("require('child_process')",  "Shell execution"),
    ("require('net')",            "Network access"),
    ("process.env",               "Environment access"),
    ("fetch(",                    "Network access"),
];

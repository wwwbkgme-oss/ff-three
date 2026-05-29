# `runtime/sandbox`

Code execution sandbox for student submissions.

**Layer rule:** I/O is allowed here (process spawning, filesystem).

## What it does

Executes student-submitted code in an isolated process with:
- Configurable timeout (`SANDBOX_TIMEOUT_SECS`, default 30s)
- Memory limit (`SANDBOX_MAX_MEMORY_MB`, default 128 MB)
- Output capture (stdout + stderr)
- Exit status detection

## Usage (from `runtime/server`)

```rust
let result = sandbox.run(SandboxRun {
    language:   ProgrammingLanguage::Python,
    code:       student_submission.clone(),
    timeout_ms: config.sandbox_timeout_secs * 1000,
}).await?;
// result.stdout, result.exit_code, result.duration_ms
```

## Security note

The current implementation uses process isolation only.
For production deployments handling untrusted code, add:
- OS-level syscall filtering (`seccomp-bpf` on Linux)
- cgroup v2 resource limits
- Filesystem namespacing (`nsjail` / `bubblewrap`)

See `docs/ROADMAP.md` Phase 1.1 (ff-one) for the reference hardening plan.

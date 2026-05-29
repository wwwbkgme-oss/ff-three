//! `drivers` — runtime I/O adapter layer.
//!
//! Contains transport implementations that bridge domain logic with the
//! outside world.  **No domain logic lives here.**
//!
//! ## Layer position
//!
//! ```text
//! foundation  ← pure types, traits
//! domain      ← pure business logic
//! runtime/drivers ← I/O adapters (this crate)
//! runtime/server  ← HTTP server, wires everything together
//! ```
//!
//! ## Current adapters
//!
//! | Module  | What it adapts                                    |
//! |---------|---------------------------------------------------|
//! | `llm`   | Free-tier LLM providers (Groq, SambaNova, …)     |
//!
//! ## Future adapters (see NEXT.md)
//!
//! | Module       | What it adapts                            |
//! |--------------|-------------------------------------------|
//! | `storage`    | S3 / local object store                   |
//! | `notify`     | Push notifications (webhook, SSE)         |
//!
//! ## What does NOT belong here
//!
//! - Domain logic → goes in `domain/`
//! - Domain-behaviour plugins → goes in `plugins/`
//! - HTTP routing → goes in `runtime/server`
//!
//! See `docs/PLUGIN_VS_DRIVER.md` for the canonical boundary spec.

pub mod llm;

pub use llm::FreeClient;
pub use llm::types::{ChatMessage, ProviderKind};

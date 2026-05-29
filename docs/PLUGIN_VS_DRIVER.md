# Plugin vs Driver — Boundary Specification

**Status: FROZEN** — this document defines a hard architectural constraint.
Any change requires explicit team consensus.

---

## One-sentence rule

> **Plugins extend domain behaviour.  Drivers adapt infrastructure.**

---

## Definitions

| Concept    | Definition |
|---|---|
| **Plugin** | A module that adds or modifies **what the domain does** — new goals, quest types, NPC strategies, economy rules. Pure logic; no I/O. |
| **Driver** | A module that adapts **how the runtime talks to the outside world** — HTTP clients, LLM APIs, object storage, queues, notification services. All I/O; no domain logic. |

---

## Layer map

```
foundation/types      ← IDs, traits, WorldTick, TickContext, DeterministicRng
foundation/events     ← event types, EventStore trait, EventEnvelope

domain/characters     ← Character aggregate, GOAP planner, reducer
domain/quests         ← quest lifecycle
domain/world          ← biome engine
domain/agents         ← AgentStrategy (abstract, no I/O)

runtime/drivers/      ← I/O ADAPTERS  ← this boundary
  llm/                   HTTP clients for LLM providers
  (future: storage, notify, search)

runtime/server/       ← HTTP routing, wires drivers into domain via AppState
runtime/sandbox/      ← code execution sandbox

plugins/              ← DOMAIN EXTENSIONS  ← this boundary
  (empty — reserved for future domain-behaviour plugins)
```

---

## Decision table

| Question | Plugin | Driver |
|---|---|---|
| Does it perform I/O (HTTP, DB, FS, queue)? | ✗ | ✔ |
| Does it implement a domain trait (`AgentStrategy`, `Reducer`, …)? | ✔ | ✗ |
| Would it be incorrect if called twice with the same input? | ✗ | ✔ (network can fail) |
| Is it replay-safe / deterministic? | ✔ required | ✗ not required |
| Does the domain layer import it? | ✔ allowed | ✗ **forbidden** |
| Does it carry API keys or base URLs? | ✗ | ✔ |

---

## Concrete examples

| Module | Classification | Reason |
|---|---|---|
| `runtime/drivers/llm/` | **Driver** | HTTP calls to LLM APIs |
| `domain/characters/src/planner.rs` | **Domain** | Pure GOAP logic, no I/O |
| `domain/agents/` | **Domain** | `AgentStrategy` implementations, prompt builders |
| `plugins/npc-economy` (future) | **Plugin** | Adds economy goal types to the domain |
| `plugins/quest-generator-v2` (future) | **Plugin** | Extends quest generation rules |

---

## The `AgentKind` rule

Domain code must **never** reference a concrete provider (`Groq`, `SambaNova`, etc.).
If the domain needs to express "use an LLM", it goes through the abstract contract:

```rust
// foundation/types — abstract, no I/O
pub trait AgentStrategy: Send + Sync { … }
```

The runtime wires the concrete driver:

```rust
// runtime/server/src/state.rs — concrete, I/O allowed
pub llm: Option<Arc<drivers::FreeClient>>
```

If a selection config is needed (e.g. "use Groq for quests, SambaNova for hints"),
it lives in the **runtime layer** as:

```rust
// runtime only — never domain
pub enum ProviderSelection { Groq, SambaNova, Llm7, … }
```

`ProviderKind` from `runtime/drivers` must **never** be imported by `foundation/` or `domain/`.

---

## Enforcement

1. `cargo check` must always pass without `llm-free` or `drivers` in any `domain/*/Cargo.toml`.
2. CI should fail if `drivers` appears in `foundation/` or `domain/` deps.
3. This document must be linked from `ARCHITECTURE.md`.

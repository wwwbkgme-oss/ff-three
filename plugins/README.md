# `plugins/`

Reserved for **domain-behaviour extensions** only.

**Layer rule:** Plugins must be pure domain logic — no I/O of any kind.

## What belongs here

- New `GoalType` variants and their GOAP preconditions
- Custom quest generation rules
- Extended NPC behaviour (new `AgentStrategy` implementations)
- Economy rules, faction systems

## What does NOT belong here

- LLM providers → `runtime/drivers/llm/`
- Storage adapters → `runtime/drivers/` (future)
- HTTP handlers → `runtime/server/`

## Plugin ABI

All plugins must export:

```rust
#[no_mangle]
pub extern "C" fn ff_plugin_init(ctx: *mut FfPluginCtx) -> i32 { ... }
#[no_mangle]
pub extern "C" fn ff_plugin_tick(ctx: *mut FfPluginCtx) -> i32 { ... }
#[no_mangle]
pub extern "C" fn ff_plugin_shutdown(ctx: *mut FfPluginCtx) { ... }
```

And a `plugin.toml` manifest:

```toml
[plugin]
id          = "forgefabrik.{domain}"
version     = "0.1.0"
kind        = "domain-behaviour"
api_version = 1
```

See [forge-core `docs/PLUGIN_ABI.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/PLUGIN_ABI.md)
for the canonical ABI specification.

## Current plugins

*(none — reserved)*

## Boundary spec

[`docs/PLUGIN_VS_DRIVER.md`](../docs/PLUGIN_VS_DRIVER.md)

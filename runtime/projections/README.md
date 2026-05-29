# `runtime/projections`

Event-sourced read models — lightweight views rebuilt from the event log.

**Rule:** Projections are read-only w.r.t. the event store. They never emit events.

## `CharacterView`

Flattened character snapshot for API responses and UI rendering.
Cheaper than loading the full `Character` aggregate.

```rust
// Build from full aggregate (e.g. after snapshot load)
let view = CharacterView::from_character(&char, current_tick, global_offset);

// Update incrementally (O(1) per event)
let was_relevant = view.apply(&event, global_offset);

// Persist view + checkpoint so restarts resume here
store.upsert_character_view(&view).await?;
```

## Projection worker

`runtime/server/src/sim/projection_worker.rs` runs as a background task:

1. Load `checkpoint` from `character_views` table
2. Call `store.load_since(checkpoint, BATCH_SIZE)`
3. Apply each event to the relevant `CharacterView`
4. Upsert updated views + advance checkpoint

## character_views table (migration 006)

```sql
CREATE TABLE character_views (
    id         UUID PRIMARY KEY,
    data       JSONB NOT NULL,        -- serialised CharacterView
    checkpoint BIGINT NOT NULL,       -- last global_offset applied
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

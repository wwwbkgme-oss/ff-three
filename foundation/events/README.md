# `foundation/events`

All emitted facts — the canonical event definitions for the Academy domain.

**Rule: Events are truth. State is projection.**

## Key exports

| Module | Contents |
|---|---|
| `CharacterEvent` | 18 variants: lifecycle, movement, goals, memory, social, factions, mood, stats |
| `AcademyEvent` | Student enroll, quest start/complete, XP, biome, groups, achievements |
| `EventEnvelope<E>` | Wraps every event: `event_id`, `causation_id`, `correlation_id`, `tick`, `actor`, `realm` |
| `EventStore` trait | `append`, `load_stream`, `load_since`, `stream_version` |
| `InMemoryEventStore` | Zero-dep test implementation (Arc<RwLock<BTreeMap>>) |
| `StoredEvent` | Event with position metadata: `stream_id`, `sequence`, `global_offset`, `payload: Value` |
| `ExpectedVersion` | `Any` · `NoStream` · `Exact(u64)` — optimistic concurrency guard |
| `StreamId` | Identifies one aggregate's event stream |

## Usage pattern

```rust
// Emit
let event = CharacterEvent::Moved { id, from, to, at };
let payload = serde_json::to_value(&event)?;
store.append(StreamId::from_uuid(id.inner()), ExpectedVersion::Exact(version), vec![payload]).await?;

// Replay
let stored = store.load_stream(stream_id).await?;
let events: Vec<CharacterEvent> = stored.iter().map(|s| s.deserialize()).collect::<Result<_,_>>()?;
let state = Character::replay(blank, &events);
```

## Production implementation

`PgEventStore` lives in `runtime/server/src/event_store.rs` — Postgres-backed with transaction + OCC.

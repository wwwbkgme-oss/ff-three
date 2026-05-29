# `domain/world`

Biome state engine and knowledge graph operations.

**Layer rule:** Deterministic. Same score → same `BiomeState`, always.

## Biome state machine

```
avg_score < 0.30  →  Confused
avg_score < 0.60  →  Clouded
avg_score < 0.85  →  Enlightened
avg_score ≥ 0.85  →  Mastered
```

`BiomeStateEngine::score_to_state(avg: f64) -> BiomeState` — pure function.

## Knowledge graph

`KnowledgeGraph` tracks concept mastery per student:

```rust
kg.update_mastery("rust-ownership", 0.87);
let mastery = kg.get_mastery("rust-ownership"); // 0.87
```

Keys are plain strings (concept slugs), values are `f64` in `[0.0, 1.0]`.

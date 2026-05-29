# Contributing to ff-three (ForgeFabrik Academy)

> **This repo is part of the ForgeFabrik federation.**
> Before architectural changes read [`AGENTS.md`](AGENTS.md) and
> [forge-core SYNC_CONTRACT](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/SYNC_CONTRACT.md).

---

## Quick start

```bash
git clone https://github.com/wwwbkgme-oss/ff-three.git
cd ff-three
cp .env.example .env         # fill DATABASE_URL, REDIS_URL, JWT_SECRET + one LLM key
docker-compose up -d         # Postgres + Redis
cargo test --workspace       # must pass
```

---

## Layer rules (hard boundary — violations block PRs)

```
foundation/   no I/O · no Utc::now() · no rand::thread_rng()
domain/       no HTTP · no DB · deterministic only
runtime/      I/O allowed · drivers go here, not in plugins/
plugins/      pure domain behaviour only · no I/O
```

Full boundary spec: [`docs/PLUGIN_VS_DRIVER.md`](docs/PLUGIN_VS_DRIVER.md)

---

## Event-first mandate

```
Command → AggregateRoot::handle() → Vec<Event>
                                          ↓
                                   EventStore::append()
                                          ↓
                                   Reducer::apply() → State
```

Never mutate `Character` (or any aggregate) directly.
All persistence goes through `PgEventStore::append`.

---

## Determinism requirements

Every change to `domain/` must maintain:

| Rule | How enforced |
|---|---|
| No wall-clock time | `WorldTick` only, never `Utc::now()` |
| No non-seeded RNG | `DeterministicRng` only, never `thread_rng()` |
| Same events → same state | `Character::replay` must be stable |
| OCC on every write | `ExpectedVersion::Exact(char.version)` |

Run before every PR:

```bash
cargo test -p characters      # 18 determinism tests
cargo check --workspace
cargo clippy -- -D warnings
```

---

## Adding a free LLM provider

1. Add constants to `runtime/drivers/src/llm/providers/<name>.rs`
2. Add the model list to `runtime/drivers/src/llm/catalog.rs`
3. Extend `build_chain()` in `runtime/drivers/src/llm/mod.rs`
4. Update `runtime/drivers/README.md` provider table
5. Update `.env.example`

Criterion: **permanently free, no credit card, no trial expiry.**

---

## PR checklist

- [ ] `cargo check --workspace` clean
- [ ] `cargo test -p characters` — 18 pass
- [ ] `cargo clippy -- -D warnings` clean
- [ ] No `Utc::now()` / `thread_rng()` in `foundation/` or `domain/`
- [ ] New domain logic has a determinism test
- [ ] Relevant `.md` files updated
- [ ] `FORGE_CORE_SYNC.md` updated if canonical names change
- [ ] `CHANGELOG.md` entry added

---

## Commit format

```
type(scope): short description

Body (optional).

Co-Authored-By: Name <email>
```

Types: `feat` `fix` `refactor` `docs` `test` `chore` `build`

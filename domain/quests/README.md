# `domain/quests`

Quest lifecycle, progression rules, and XP formulae for the Academy.

**Layer rule:** Deterministic, no I/O. All mutations expressed as `AcademyEvent`.

## Key concepts

| Concept | Description |
|---|---|
| `Quest` | Generated learning challenge (title, description, difficulty, XP reward) |
| `QuestStatus` | `Available → Active → Completed / Failed` |
| `QuestReducer` | Applies `AcademyEvent` to `StudentQuest` state |
| `QuestRules` | `PASS_THRESHOLD = 0.70`, `MAX_ATTEMPTS = 10` |

## XP formula

```rust
// xp_for_difficulty(difficulty: i32) -> i32
difficulty.max(1) * 10
```

Difficulty 1 → 10 XP · difficulty 10 → 100 XP.

## Domain rules (enforced by command handler)

- Score ≥ 0.70 → `QuestCompleted` event
- Score < 0.70 → `QuestFailed` event
- Attempts > `MAX_ATTEMPTS` → command rejected with `ForgeError::DomainViolation`

//! Snapshot types and the `DeterministicHash` contract.
//!
//! Snapshots allow: replay-from-checkpoint, rollback, world branching,
//! cross-node state verification.

use serde::{Deserialize, Serialize};

use crate::{ids::{EventId, SnapshotId}, time::WorldTick};

/// A verified checkpoint of the world state at a specific tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub id:      SnapshotId,
    pub at_tick: WorldTick,
    /// Blake3 / SHA-256 hash of the entire projected state.
    /// `state_hash` is the primary equality / verification key.
    pub state_hash: [u8; 32],
    /// Last event included in this snapshot.
    pub cursor: EventId,
    /// Link to the previous snapshot → forms a verifiable chain.
    /// `None` for the genesis snapshot.
    pub parent_snapshot: Option<SnapshotId>,
    /// Compressed state payload (CBOR / JSON depending on runtime).
    pub payload: serde_json::Value,
}

impl WorldSnapshot {
    /// Returns `true` when a freshly recomputed hash matches the stored one.
    pub fn verify(&self, recomputed: [u8; 32]) -> bool {
        self.state_hash == recomputed
    }
}

/// Every aggregate that participates in deterministic replay must implement this.
///
/// Implementors MUST guarantee:
///   `apply(apply(s, e1), e2) == apply(s, e1+e2)` (associativity)
///   Same event sequence → same hash (determinism)
pub trait DeterministicHash {
    /// Produce a 32-byte deterministic hash of the current state.
    ///
    /// **Requirements:**
    ///   - No wall-clock time.
    ///   - No randomness.
    ///   - Field ordering must be stable (use sorted keys in maps).
    fn state_hash(&self) -> [u8; 32];
}

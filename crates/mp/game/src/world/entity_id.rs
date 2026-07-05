//! `EntityId` — the entity handle (porting-rules §B5).
//!
//! The type itself now lives in `mp_qshared` (`common::mp::entity_id`) so that
//! `gentity_t`'s stored entity fields can name it. This module
//! re-exports it (and the `ent_id`/`ent_id_opt` seam helpers) at the historical
//! `crate::world::EntityId` path so every existing game-side use keeps compiling
//! unchanged.
//!
//! Source: `docs/architecture/state-ownership.md` § `EntityId`.

pub use mp_qshared::common::mp::entity_id::{ent_id, ent_id_opt, EntityId};

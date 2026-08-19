//! `EntityId` is the entity handle (porting-rules §B5).
//!
//! `EntityId` and `to_num` live in `mp_qshared` (`common::mp::entity_id`), so `gentity_t`'s stored entity fields can name them.
//! The `ent_id` and `ent_id_opt` pointer-to-index helper functions live in `mp_game` (`crate::ent_id`, DEC-26).
//! They run `offset_from` over the concrete `gentity_t`.
//! This module re-exports all four at the historical `crate::world::EntityId` path, so every existing game-side use keeps compiling unchanged.
//!
//! Source: `docs/architecture/state-ownership.md` § `EntityId`.

pub use crate::ent_id::{ent_id, ent_id_opt};
pub use mp_qshared::common::mp::entity_id::{to_num, EntityId};

//! `EntityId` — the entity handle (porting-rules §B5).
//!
//! `EntityId`/`to_num` live in `mp_qshared` (`common::mp::entity_id`) so that
//! `gentity_t`'s stored entity fields can name them; the `ent_id`/`ent_id_opt`
//! pointer↔index seam helpers live in `mp_game` (`crate::ent_id`, DEC-26) since
//! they do `offset_from` over the concrete `gentity_t`. This module re-exports
//! all four at the historical `crate::world::EntityId` path so every existing
//! game-side use keeps compiling unchanged.
//!
//! Source: `docs/architecture/state-ownership.md` § `EntityId`.

pub use crate::ent_id::{ent_id, ent_id_opt};
pub use mp_qshared::common::mp::entity_id::{to_num, EntityId};

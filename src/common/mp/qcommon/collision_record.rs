//! MP Ghoul2 collision record types copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:1871-1888`

#![allow(non_camel_case_types)]

/// Ghoul2 model collision hit record.
///
/// Raven uses this as an entry in `G2Trace_t`, described as the map of Ghoul2
/// model parts hit by a trace. Usage in Ghoul2 collision code treats
/// `mEntityNum == -1` as an unused record; populated records carry hit
/// distance, entity/model/surface indexes, collision position/normal, flags,
/// material, location, and barycentric hit coordinates.
pub use crate::shared::CollisionRecord_t;

/*
Ghoul2 Insert Start
*/
pub const MAX_G2_COLLISIONS: usize = 16;

pub type G2Trace_t = [CollisionRecord_t; MAX_G2_COLLISIONS]; // map that describes all of the parts of ghoul2 models that got hit

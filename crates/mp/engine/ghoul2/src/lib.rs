//! `mp_engine_ghoul2` — the server-side Ghoul2 + renderer bone/model pipeline
//! (`docs/subsystems/ghoul2-server.md`, `G2SV-D5`). The `Engine.g2` subsystem
//! lands entirely here; loader model memory is reached over `EngineHost`, not a
//! `mp_renderer` crate edge.

// The bone/render pipeline reaches world state through raw pointers
// (`(*bc.root_bone_list)[i]`, `(*ghoul2).blist[i]`), so container indexing on
// those paths implicitly autorefs through the deref — the exact pattern this
// deny-by-default lint flags. The refs are intentional (seam-confined unsafe,
// `G2SV-D9` bone-cache handles); silenced crate-wide as in `mp_game`. Revisit
// when the safe-state migration lands.
#![allow(dangerous_implicit_autorefs)]

pub mod ghoul2_system;

pub mod info_array;

pub mod api_bolts;
pub mod api_bones;
pub mod api_collision;
pub mod api_gore;
pub mod api_models;
pub mod api_ragdoll;
pub mod api_saveload;
pub mod api_surfaces;

pub mod bolts;
pub mod bones;
pub mod misc;
pub mod ragdoll;
pub mod ragdoll_update_params;
pub mod surfaces;

pub mod gore;
pub mod matcomp;
pub mod render;
pub mod shared;

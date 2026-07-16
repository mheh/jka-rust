//! Multiplayer common code shared within MP modules, but not yet proven common with SP.

pub mod bg;
pub mod botlib;
pub mod cgame;
pub mod ent_fn_ids;
pub mod entity_id;
pub mod game;
pub mod gentity;
// The forward-decl struct is named `gentity_s` (matching Raven's `struct
// gentity_s`), so the module carrying it cannot also be named `gentity_s` (a
// same-name module+type collides in the type namespace). `#[path]` keeps the
// file `gentity_s.rs` while re-exporting the struct flat at `common::mp::gentity_s`
// — the ergonomic path the abi seam names, mirroring the old `common::mp::gentity_t`.
#[path = "gentity_s.rs"]
mod gentity_s_impl;
pub mod ghoul2;
pub mod playerstate;
pub mod qcommon;
pub mod rmg;
pub mod trace_t;
pub mod ui;

pub use entity_id::EntityId;
pub use gentity_s_impl::gentity_s;

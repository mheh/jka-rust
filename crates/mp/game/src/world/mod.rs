//! `world` — the MP module island: the owned `GameWorld`, its `EntityId`
//! handle, and the `GameContext` dispatch receiver (STATE-Q11 resolution,
//! round-6: `crates/{mp,sp}/game/src/world/{mod,game_world,game_context,entity_id}.rs`).
//!
//! Thin `impl Dispatch<C> for GameContext` adapters colocate in
//! `game_context.rs` (round-6 pinning); per-command logic stays
//! one-fn-per-file elsewhere in the crate.

pub mod entity_id;
pub mod game_context;
pub mod game_scratch;
pub mod game_world;

pub use entity_id::{ent_id, ent_id_opt, to_num, EntityId};
pub use game_context::GameContext;
pub use game_scratch::GameScratch;
pub use game_world::GameWorld;

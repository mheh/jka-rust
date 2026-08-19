//! The `world` module holds the owned `GameWorld`, its `EntityId` handle, and the `GameContext` dispatch receiver.
//! The files are `crates/{mp,sp}/game/src/world/{mod,game_world,game_context,entity_id}.rs`.
//!
//! Thin `impl Dispatch<C> for GameContext` adapters colocate in `game_context.rs`.
//! Per-command logic stays one-fn-per-file elsewhere in the crate.

pub mod entity_id;
pub mod game_context;
pub mod game_scratch;
pub mod game_world;
pub mod guarded_entities;
pub mod shared_buffer;

pub use entity_id::{ent_id, ent_id_opt, to_num, EntityId};
pub use game_context::GameContext;
pub use game_scratch::GameScratch;
pub use game_world::GameWorld;
pub use guarded_entities::GuardedEntities;
pub use shared_buffer::SharedBuffer;

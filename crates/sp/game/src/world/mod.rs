//! `world` — the SP module island: the owned `GameWorld`, its `EntityId`
//! handle, and the per-export `GameContext` receiver (STATE-Q11 resolution,
//! round-6: `crates/{mp,sp}/game/src/world/{mod,game_world,game_context,entity_id}.rs`).

pub mod entity_id;
pub mod game_context;
pub mod game_world;

pub use entity_id::EntityId;
pub use game_context::GameContext;
pub use game_world::GameWorld;

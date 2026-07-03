//! `mp_game` — MP server-game module (`g_*`), the game-side of the QVM boundary.
//!
//! The core `g_local.h` data model is ported (client/entity/level + AI/teams/npc
//! types), verified against oracle with size/offset asserts. //TODO: Port the
//! gameplay logic (g_*.c functions).

#![allow(non_camel_case_types, non_snake_case)]

pub mod ai;
pub mod botai;
pub mod client;
pub mod entity;
pub mod g_init_game;
pub mod g_shutdown_game;
pub mod level;
pub mod npc;
pub mod saber;
pub mod say;
pub mod teams;
pub mod trap;
pub mod world;

pub use world::{EntityId, GameContext, GameWorld};

// The export-command enum, re-exported so the jampgame shell names it through
// its existing two edges (round-7 item 25; SEAM-D10's exactly-two-edges shell
// property stays intact — the shell sees the seam through the logic crate).
pub use mp_abi::game::exports::MpGameExport;

// The per-command vmMain call types the shell's dispatch match names, seen
// through the logic crate on the same item-25 principle (checkpoint-7 finding:
// mechanical extension — the shell's arms need the C marker/Args types).
pub mod vmcalls {
    pub use mp_abi::game::vmcalls::GAME_INIT::GameInit;
    pub use mp_abi::game::vmcalls::GAME_SHUTDOWN::GameShutdown;
}

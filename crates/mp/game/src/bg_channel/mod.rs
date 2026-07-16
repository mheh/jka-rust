//! `bg_channel` — game-tier view of the bg state channel.
//!
//! safe-state S5-6: the channel's state and its two seam traits
//! ([`BgState`]/[`PmoveContext`]/[`BgTraps`]/[`GameCallbacks`]) moved down to
//! `mp_bg::bg_channel`. Only the game-tier implementations stay here
//! ([`GameBgTraps`]/[`GameCallbacksImpl`], in [`game_impl`]). The moved
//! submodules and types are re-exported under their old `crate::bg_channel::*`
//! paths so game importers and the prelude glob keep resolving unchanged.

pub mod game_impl;

pub use mp_bg::bg_channel::{bg_state, bg_traps, game_callbacks, pmove_context, rng};
pub use mp_bg::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext, Rng};

pub use game_impl::{GameBgTraps, GameCallbacksImpl};
pub mod traps {
    pub use super::game_impl::GameBgTraps;
}

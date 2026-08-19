//! `bg_channel` is the game-tier view of the bg state channel.
//!
//! The channel's state (`BgState`) and its pmove context (`PmoveContext`) moved down to `mp_bg::bg_channel`.
//! Its two traits (`BgTraps`, `GameCallbacks`) moved down to `mp_bg::bg_channel` too.
//! Only the game-tier implementations stay here ([`GameBgTraps`]/[`GameCallbacksImpl`], in [`game_impl`]).
//! The moved submodules and types are re-exported under their old `crate::bg_channel::*` paths.
//! Game importers and the prelude glob keep resolving them unchanged.

pub mod game_impl;

pub use mp_bg::bg_channel::{bg_state, bg_traps, game_callbacks, pmove_context, rng};
pub use mp_bg::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext, Rng};

pub use game_impl::{GameBgTraps, GameCallbacksImpl};
pub mod traps {
    pub use super::game_impl::GameBgTraps;
}

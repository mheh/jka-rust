//! `bg_channel` — the pass-3 bg state channel.
//!
//! The bg tier's state and its two seam traits, kept in one place. The
//! `mp_bg` crate split is deferred to post-parity, so these live in `mp_game`
//! for now; the trait boundary (`BgTraps`/`GameCallbacks`) — not a crate
//! wall — enforces `bg < game`.
//!
//! - [`BgState`] — session-lifetime bg tables + the faithful LCG [`Rng`],
//!   owned by `GameWorld`.
//! - [`PmoveContext`] — the per-`Pmove`-call working set.
//! - [`BgTraps`] — bg→engine outbound surface.
//! - [`GameCallbacks`] — bg→game upcalls.
//! - [`GameBgTraps`]/[`GameCallbacksImpl`] — the game-tier implementations
//!   (would move to game proper when the bg crate splits out).

pub mod bg_state;
pub mod bg_traps;
pub mod game_callbacks;
pub mod game_impl;
pub mod pmove_context;
pub mod rng;
pub mod traps {
    pub use super::game_impl::GameBgTraps;
}

pub use bg_state::BgState;
pub use bg_traps::BgTraps;
pub use game_callbacks::GameCallbacks;
pub use game_impl::{GameBgTraps, GameCallbacksImpl};
pub use pmove_context::PmoveContext;
pub use rng::Rng;

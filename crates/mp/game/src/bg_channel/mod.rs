//! `bg_channel` — the pass-3 bg state channel (fork rulings 12-16, 19).
//!
//! The bg tier's state and its two seam traits, kept in one place. Ruling 19
//! defers the `mp_bg` crate split to post-parity, so these live in `mp_game`
//! for pass 3; the trait boundary (`BgTraps`/`GameCallbacks`) — not a crate
//! wall — enforces `bg < game`.
//!
//! - [`BgState`] — session-lifetime bg tables + the fork-3 [`Rng`], owned by
//!   `GameWorld` (ruling 12/15).
//! - [`PmoveContext`] — the per-`Pmove`-call working set (rulings 12/8a).
//! - [`BgTraps`] — bg→engine outbound surface (ruling 13).
//! - [`GameCallbacks`] — bg→game upcalls (ruling 16).
//! - [`GameBgTraps`]/[`GameCallbacksImpl`] — the game-tier implementations
//!   (would move to game proper when the bg crate splits out).

pub mod bg_state;
pub mod bg_traps;
pub mod game_callbacks;
pub mod game_impl;
pub mod pmove_context;
pub mod rng;

pub use bg_state::BgState;
pub use bg_traps::BgTraps;
pub use game_callbacks::GameCallbacks;
pub use game_impl::{GameBgTraps, GameCallbacksImpl};
pub use pmove_context::PmoveContext;
pub use rng::Rng;

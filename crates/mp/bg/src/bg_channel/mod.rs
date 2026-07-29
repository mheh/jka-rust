//! `bg_channel` — the bg state channel.
//!
//! The bg tier's session/per-call state and its two seam traits.
//!
//! - [`BgState`] — session-lifetime bg tables + the faithful LCG [`Rng`],
//!   owned by the game's `GameWorld`.
//! - [`PmoveContext`] — the per-`Pmove`-call working set.
//! - [`BgTraps`] — bg→engine outbound surface.
//! - [`GameCallbacks`] — bg→game upcalls.
//!
//! The game-tier implementations (`GameBgTraps`/`GameCallbacksImpl`, in
//! `mp_game`'s `bg_channel::game_impl`) implement these traits over the engine
//! and world handles.

pub mod bg_host;
pub mod bg_state;
pub mod bg_traps;
pub mod game_callbacks;
pub mod pmove_context;
pub mod rng;

pub use bg_host::BgHost;
pub use bg_state::BgState;
pub use bg_traps::BgTraps;
pub use game_callbacks::GameCallbacks;
pub use pmove_context::PmoveContext;
pub use rng::Rng;

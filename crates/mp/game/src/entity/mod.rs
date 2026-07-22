//! MP server-side game entity (`g_local.h`): the `gentity_t` struct and the
//! game-private entity constants.
//!
//! The `gentity_t` struct lives here (`gentity` submodule, DEC-26); the abi tier
//! carries entity pointers opaquely as `gentity_s`. The game-private constants
//! and typedefs still live in `mp_qshared` (`common::mp::gentity`), imported by
//! the struct and by the engine crates.

pub mod damage_redirect;
pub mod flags;
pub mod gentity;
pub mod hit_location;
pub mod mover_state;

pub use gentity::{gentity_t, PrefixSet, PrefixSlot};

pub use mover_state::moverState_t;

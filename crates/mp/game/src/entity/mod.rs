//! MP server-side game entity (`g_local.h`): the game-private entity constants.
//!
//! The `gentity_t` struct itself lives in `mp_qshared` (it crosses the game
//! syscall boundary as `gentity_t*`); it is re-exported here for convenience.

pub mod damage_redirect;
pub mod flags;
pub mod hit_location;
pub mod mover_state;

pub use mp_qshared::common::mp::gentity::gentity_t;

pub use mover_state::moverState_t;

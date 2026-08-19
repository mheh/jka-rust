//! MP `MOVER_*` state constants.
//!
//! `moverState_t` itself lives in `mp_qshared` as a `c_int` alias, because `gentity_t.moverState` needs it.
//! These constants are the named states.
//! Source: `oracle/codemp/game/g_local.h:88-94`

pub use mp_qshared::common::mp::gentity::moverState_t;

// movers are things like doors, plats, buttons, etc
pub const MOVER_POS1: moverState_t = 0;
pub const MOVER_POS2: moverState_t = 1;
pub const MOVER_1TO2: moverState_t = 2;
pub const MOVER_2TO1: moverState_t = 3;

#![allow(non_camel_case_types, non_snake_case)]

/// Raven `moverState_t` — mover entity state.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:107-113`
#[repr(i32)]
pub enum moverState_t {
    MOVER_POS1 = 0,
    MOVER_POS2 = 1,
    MOVER_1TO2 = 2,
    MOVER_2TO1 = 3,
}

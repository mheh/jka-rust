#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EG2_Collision` — collision type for Ghoul2 bones.
///
/// Type definition source: `oracle/code/game/ghoul2_shared.h:484-489`
#[repr(i32)]
pub enum EG2_Collision {
    G2_NOCOLLIDE,
    G2_COLLIDE,
    G2_RETURNONHIT,
}

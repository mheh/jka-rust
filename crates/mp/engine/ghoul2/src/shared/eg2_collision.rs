#![allow(non_camel_case_types, non_snake_case)]

/// Raven `EG2_Collision` — ghoul2 collision type.
///
/// Raven: (no comment in source).
/// Type definition source: `oracle/codemp/ghoul2/ghoul2_shared.h:465-470`
#[repr(i32)]
pub enum EG2_Collision {
    G2_NOCOLLIDE = 0,
    G2_COLLIDE = 1,
    G2_RETURNONHIT = 2,
}

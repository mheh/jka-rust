#![allow(non_camel_case_types, non_snake_case)]

/// Raven `visibility_t`: visibility state of a target.
///
/// Type definition source: `oracle/codemp/game/b_public.h:68-68`
// `GameGlobals` stores this by value (`enemyVisibility`, `NPC.c:38`).
// Read sites copy it out of `ctx.world.globals` and compare it with `==`, so it derives `Clone`, `Copy`, `PartialEq`, `Eq`, and `Default`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum visibility_t {
    #[default]
    VIS_UNKNOWN,
    VIS_NOT,
    VIS_PVS,
    VIS_360,
    VIS_FOV,
    VIS_SHOOT,
}

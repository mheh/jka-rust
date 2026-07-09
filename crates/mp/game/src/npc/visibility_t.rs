#![allow(non_camel_case_types, non_snake_case)]

/// Raven `visibility_t` — visibility state of a target.
///
/// Type definition source: `oracle/codemp/game/b_public.h:68-68`
// `Clone, Copy, PartialEq, Eq, Default` backfilled (pass-3): `GameGlobals`
// stores this by value (`enemyVisibility`, `NPC.c:38`) and porters compare it
// with `==`/copy it out of `ctx.world.globals` at every read site.
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

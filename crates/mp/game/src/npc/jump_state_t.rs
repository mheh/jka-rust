#![allow(non_camel_case_types, non_snake_case)]

/// Raven `jumpState_t` — jump animation state for AI.
///
/// Type definition source: `oracle/oracle/codemp/game/b_public.h:77-84`
#[repr(i32)]
pub enum jumpState_t {
    JS_WAITING = 0,
    JS_FACING,
    JS_CROUCHING,
    JS_JUMPING,
    JS_LANDING,
}

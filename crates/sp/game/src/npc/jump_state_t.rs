#![allow(non_camel_case_types, non_snake_case)]

/// Raven `jumpState_t` — jump state enumeration.
///
/// Type definition source: `oracle/oracle/code/game/b_public.h:97-104`
#[repr(i32)]
pub enum jumpState_t {
    JS_WAITING = 0,
    JS_FACING,
    JS_CROUCHING,
    JS_JUMPING,
    JS_LANDING,
}

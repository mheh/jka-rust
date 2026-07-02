#![allow(non_camel_case_types, non_snake_case)]

/// Raven `blockedFunc_t` — enumeration of entity blocked callback function IDs.
///
/// Type definition source: `oracle/oracle/code/game/g_functions.h:269-276`
#[repr(i32)]
pub enum blockedFunc_t {
    blockedF_NULL = 0,
    //
    blockedF_Blocked_Door,
    blockedF_Blocked_Mover,
}

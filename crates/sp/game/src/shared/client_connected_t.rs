#![allow(non_camel_case_types, non_snake_case)]

/// Raven `clientConnected_t` — client connection state.
///
/// Type definition source: `oracle/oracle/code/game/g_shared.h:240-244`
#[repr(i32)]
pub enum clientConnected_t {
    CON_DISCONNECTED = 0,
    CON_CONNECTING = 1,
    CON_CONNECTED = 2,
}

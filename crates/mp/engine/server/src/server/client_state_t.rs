#![allow(non_camel_case_types, non_snake_case)]

/// Raven `clientState_t` — client connection state enumeration.
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:114-121`
#[repr(i32)]
pub enum clientState_t {
    /// can be reused for a new connection
    CS_FREE = 0,
    /// client has been disconnected, but don't reuse connection for a couple seconds
    CS_ZOMBIE = 1,
    /// has been assigned to a client_t, but no gamestate yet
    CS_CONNECTED = 2,
    /// gamestate has been sent, but client hasn't sent a usercmd
    CS_PRIMED = 3,
    /// client is fully in game
    CS_ACTIVE = 4,
}

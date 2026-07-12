#![allow(non_camel_case_types, non_snake_case)]

/// Raven `clientState_t` — the state of a client connection.
///
/// Raven: enumerates connection states from free to actively playing.
/// Type definition source: `oracle/code/server/server.h:89-96`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

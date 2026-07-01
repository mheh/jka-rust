#![allow(non_camel_case_types)]

/// Raven `connstate_t` — client connection state.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2518-2528`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum connstate_t {
    CA_UNINITIALIZED,
    /// Raven: not talking to a server
    CA_DISCONNECTED,
    /// Raven: sending request packets to the server
    CA_CONNECTING,
    /// Raven: sending challenge packets to the server
    CA_CHALLENGING,
    /// Raven: netchan_t established, getting gamestate
    CA_CONNECTED,
    /// Raven: only during cgame initialization, never during main loop
    CA_LOADING,
    /// Raven: got gamestate, waiting for first frame
    CA_PRIMED,
    /// Raven: game views should be displayed
    CA_ACTIVE,
    /// Raven: playing a cinematic or a static pic, not connected to a server
    CA_CINEMATIC,
}

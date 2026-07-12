//! `net_chan.cpp`-local packet/loopback constants (non-`_XBOX` branch).
//!
//! Source: `oracle/codemp/qcommon/net_chan.cpp:33-38,474`

/// Raven `MAX_PACKETLEN` — max size of a network packet.
/// Source: `oracle/codemp/qcommon/net_chan.cpp:33`
pub const MAX_PACKETLEN: i32 = 1400;

/// Raven `FRAGMENT_SIZE`.
/// Source: `oracle/codemp/qcommon/net_chan.cpp:34`
pub const FRAGMENT_SIZE: i32 = MAX_PACKETLEN - 100;

/// Raven `PACKET_HEADER` — two ints and a short.
/// Source: `oracle/codemp/qcommon/net_chan.cpp:35`
pub const PACKET_HEADER: i32 = 10;

/// Raven `FRAGMENT_BIT`.
/// Source: `oracle/codemp/qcommon/net_chan.cpp:38`
pub const FRAGMENT_BIT: i32 = 1 << 31;

// there needs to be enough loopback messages to hold a complete
// gamestate of maximum size
/// Raven `MAX_LOOPBACK`.
/// Source: `oracle/codemp/qcommon/net_chan.cpp:474`
pub const MAX_LOOPBACK: i32 = 16;

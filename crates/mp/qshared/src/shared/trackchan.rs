#![allow(non_camel_case_types)]

/// Raven `trackchan_t` sound tracking channels.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2056-2064`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum trackchan_t {
    TRACK_CHANNEL_NONE = 50,
    TRACK_CHANNEL_1,
    TRACK_CHANNEL_2,
    TRACK_CHANNEL_3,
    TRACK_CHANNEL_4,
    TRACK_CHANNEL_5,
    NUM_TRACK_CHANNELS,
}

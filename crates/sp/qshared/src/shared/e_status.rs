#![allow(non_camel_case_types)]

/// Raven `e_status` — cinematic playback state.
///
/// SP-vs-MP: SP declares this as a named `typedef enum` (a real enum); MP declares
/// it as `typedef int` with a separate anonymous enum of constants.
///
/// Type definition source: `oracle/code/game/q_shared.h:2671-2679`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum e_status {
    FMV_IDLE,
    /// Raven: play
    FMV_PLAY,
    /// Raven: all other conditions, i.e. stop/EOF/abort
    FMV_EOF,
    FMV_ID_BLT,
    FMV_ID_IDLE,
    FMV_LOOPED,
    FMV_ID_WAIT,
}

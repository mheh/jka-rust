#![allow(non_camel_case_types)]

/// Raven `errorParm_t` — parameters to the main Error routine.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:251-256`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum errorParm_t {
    /// Raven: exit the entire game with a popup window
    ERR_FATAL,
    /// Raven: print to console and disconnect from game
    ERR_DROP,
    /// Raven: don't kill server
    ERR_DISCONNECT,
    /// Raven: pop up the need-cd dialog
    ERR_NEED_CD,
}

#![allow(non_camel_case_types)]

/// Raven `WL_e` system-wide print levels.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:428-433`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WL_e {
    WL_ERROR = 1,
    WL_WARNING,
    WL_VERBOSE,
    WL_DEBUG,
}

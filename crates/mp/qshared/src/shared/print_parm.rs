#![allow(non_camel_case_types)]

/// Raven `printParm_t` renderer print levels.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:438-443`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum printParm_t {
    PRINT_ALL,
    /// Raven: only print when "developer 1"
    PRINT_DEVELOPER,
    PRINT_WARNING,
    PRINT_ERROR,
}

#![allow(non_camel_case_types)]

/// Raven `printParm_t` — print levels from the renderer.
///
/// Type definition source: `oracle/code/game/q_shared.h:243-248`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum printParm_t {
    PRINT_ALL,
    /// Raven: only print when "developer 1"
    PRINT_DEVELOPER,
    PRINT_WARNING,
    PRINT_ERROR,
}

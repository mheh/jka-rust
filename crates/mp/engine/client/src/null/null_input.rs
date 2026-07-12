//! Raven's `null` input stubs — the DEDICATED/no-renderer build's `IN_*` /
//! `Sys_SendKeyEvents` entry points, every body an intentional no-op.
//!
//! Source: `oracle/codemp/null/null_input.cpp`

/// Raven `IN_Init`.
///
/// Source: `oracle/codemp/null/null_input.cpp:3-4`
pub fn IN_Init() {}

/// Raven `IN_Frame`.
///
/// Source: `oracle/codemp/null/null_input.cpp:6-7`
pub fn IN_Frame() {}

/// Raven `IN_Shutdown`.
///
/// Source: `oracle/codemp/null/null_input.cpp:9-10`
pub fn IN_Shutdown() {}

/// Raven `Sys_SendKeyEvents`.
///
/// Source: `oracle/codemp/null/null_input.cpp:12-13`
pub fn Sys_SendKeyEvents() {}

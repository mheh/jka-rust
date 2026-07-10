#![allow(non_camel_case_types, non_snake_case)]

// Raven: defines and structures required for fonts; must match the defines
// in `stmparse.h`.

/// Raven `STYLE_DROPSHADOW`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:570`
pub const STYLE_DROPSHADOW: u32 = 0x80000000;

/// Raven `STYLE_BLINK`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:571`
pub const STYLE_BLINK: u32 = 0x40000000;

/// Raven `SET_MASK`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:572`
pub const SET_MASK: u32 = 0x00ffffff;

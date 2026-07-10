#![allow(non_camel_case_types, non_snake_case)]

// Raven: Light Style Constants.

/// Raven `LS_NORMAL`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:311`
pub const LS_NORMAL: u8 = 0x00;

/// Raven `LS_UNUSED`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:312`
pub const LS_UNUSED: u8 = 0xfe;

/// Raven `LS_LSNONE`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:313`
pub const LS_LSNONE: u8 = 0xff;

/// Raven `MAX_LIGHT_STYLES`.
///
/// Source: `oracle/codemp/qcommon/qfiles.h:314`
pub const MAX_LIGHT_STYLES: usize = 64;

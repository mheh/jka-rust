#![allow(non_camel_case_types, non_snake_case)]

// Referenced flags — these are in loop specific order so don't change the
// order.

/// Raven `FS_GENERAL_REF`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:500`
pub const FS_GENERAL_REF: i32 = 0x01;

/// Raven `FS_UI_REF`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:501`
pub const FS_UI_REF: i32 = 0x02;

/// Raven `FS_CGAME_REF`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:502`
pub const FS_CGAME_REF: i32 = 0x04;

/// Raven `FS_QAGAME_REF`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:503`
pub const FS_QAGAME_REF: i32 = 0x08;

/// Raven `NUM_ID_PAKS` — number of id paks that will never be autodownloaded
/// from base.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:505`
pub const NUM_ID_PAKS: i32 = 9;

/// Raven `MAX_FILE_HANDLES`.
///
/// Raven `#define`s `MAX_FILE_HANDLES` twice — 16 under `_XBOX`, 64
/// otherwise (`qcommon.h:508-510`); the engine never builds `_XBOX`, so the
/// non-`_XBOX` value is the one that applies.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:510`
pub const MAX_FILE_HANDLES: usize = 64;

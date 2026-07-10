#![allow(non_camel_case_types)]

//! `l_struct.h` `fielddef_t::type` field-type constants and flags.
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly. Typed
//! `i32` to match `fielddef_t::r#type`'s width.
//!
//! Source: `oracle/codemp/botlib/l_struct.h:18-28`

/// Raven `FT_CHAR` — char.
/// Source: `oracle/codemp/botlib/l_struct.h:18`
pub const FT_CHAR: i32 = 1;

/// Raven `FT_INT` — int.
/// Source: `oracle/codemp/botlib/l_struct.h:19`
pub const FT_INT: i32 = 2;

/// Raven `FT_FLOAT` — float.
/// Source: `oracle/codemp/botlib/l_struct.h:20`
pub const FT_FLOAT: i32 = 3;

/// Raven `FT_STRING` — char `[MAX_STRINGFIELD]`.
/// Source: `oracle/codemp/botlib/l_struct.h:21`
pub const FT_STRING: i32 = 4;

/// Raven `FT_STRUCT` — struct (sub structure).
/// Source: `oracle/codemp/botlib/l_struct.h:22`
pub const FT_STRUCT: i32 = 6;

/// Raven `FT_TYPE` — only type, clear subtype.
/// Source: `oracle/codemp/botlib/l_struct.h:24`
pub const FT_TYPE: i32 = 0x00FF;

/// Raven `FT_ARRAY` — array of type.
/// Source: `oracle/codemp/botlib/l_struct.h:26`
pub const FT_ARRAY: i32 = 0x0100;

/// Raven `FT_BOUNDED` — bounded value.
/// Source: `oracle/codemp/botlib/l_struct.h:27`
pub const FT_BOUNDED: i32 = 0x0200;

/// Raven `FT_UNSIGNED`.
/// Source: `oracle/codemp/botlib/l_struct.h:28`
pub const FT_UNSIGNED: i32 = 0x0400;

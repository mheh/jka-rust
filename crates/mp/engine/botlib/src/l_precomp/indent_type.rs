#![allow(non_camel_case_types, non_snake_case)]

// Raven: `indent_t::type` bits for the preprocessor's `#if`/`#ifdef` stack.

/// Raven `INDENT_IF`.
///
/// Source: `oracle/codemp/botlib/l_precomp.h:48`
pub const INDENT_IF: i32 = 0x0001;

/// Raven `INDENT_ELSE`.
///
/// Source: `oracle/codemp/botlib/l_precomp.h:49`
pub const INDENT_ELSE: i32 = 0x0002;

/// Raven `INDENT_ELIF`.
///
/// Source: `oracle/codemp/botlib/l_precomp.h:50`
pub const INDENT_ELIF: i32 = 0x0004;

/// Raven `INDENT_IFDEF`.
///
/// Source: `oracle/codemp/botlib/l_precomp.h:51`
pub const INDENT_IFDEF: i32 = 0x0008;

/// Raven `INDENT_IFNDEF`.
///
/// Source: `oracle/codemp/botlib/l_precomp.h:52`
pub const INDENT_IFNDEF: i32 = 0x0010;

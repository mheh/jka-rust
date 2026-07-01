//! `native_types` — Raven-free scalar/handle primitives that are byte-identical
//! across SP and MP `q_shared.h`. Cross-mode; re-exported by each mode's
//! `qshared` umbrella.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `qboolean`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h`
pub type qboolean = c_int;

/// Raven `fileHandle_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:187`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:362`
pub type fileHandle_t = c_int;

/// Raven `clipHandle_t` collision model handle.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:188`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:363`
pub type clipHandle_t = c_int;

/// Raven `qhandle_t`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:183`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:358`
pub type qhandle_t = c_int;

/// Raven `mdxaBone_t`.
///
/// Type definition source: `oracle/oracle/code/renderer/mdx_format.h:137`
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:3078`
/// Type definition source: `oracle/oracle/codemp/renderer/mdx_format.h:137`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

pub const QFALSE: qboolean = 0;
pub const QTRUE: qboolean = 1;

/// Raven `MAX_QPATH`.
///
/// Definition source: `oracle/oracle/code/game/q_shared.h:215`
/// Definition source: `oracle/oracle/codemp/game/q_shared.h:393`
pub const MAX_QPATH: usize = 64;

//! `native_types` — Raven-free scalar/handle primitives that are byte-identical
//! across SP and MP `q_shared.h`. Cross-mode; re-exported by each mode's
//! `qshared` umbrella.

#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_uchar, c_ulong, c_ushort};

pub mod anim;
pub mod say;

pub use anim::anim_event_type::animEventType_t;
pub use anim::anim_number::{animNumber_t, SABER_ANIM_GROUP_SIZE};
pub use anim::footstep_type::footstepType_t;
pub use say::saying_t::saying_t;

/// Raven `byte`.
///
/// Type definition source: `oracle/code/game/q_shared.h:176`
/// Type definition source: `oracle/codemp/game/q_shared.h:349`
pub type byte = c_uchar;

/// Raven `word`.
///
/// Type definition source: `oracle/code/game/q_shared.h:174`
/// Type definition source: `oracle/codemp/game/q_shared.h:350`
pub type word = c_ushort;

/// Raven `ulong`.
///
/// Type definition source: `oracle/code/game/q_shared.h:173`
/// Type definition source: `oracle/codemp/game/q_shared.h:351`
pub type ulong = c_ulong;

/// Raven `qboolean`.
///
/// Type definition source: `oracle/code/game/q_shared.h`
/// Type definition source: `oracle/codemp/game/q_shared.h`
pub type qboolean = c_int;

/// Raven `qfalse`/`qtrue` — the `qboolean` enum values, in Raven's lowercase
/// spelling.
///
/// Definition source: `oracle/code/game/q_shared.h`
/// Definition source: `oracle/codemp/game/q_shared.h`
#[allow(non_upper_case_globals)]
pub const qfalse: qboolean = 0;
#[allow(non_upper_case_globals)]
pub const qtrue: qboolean = 1;

/// Raven `fileHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:187`
/// Type definition source: `oracle/codemp/game/q_shared.h:362`
pub type fileHandle_t = c_int;

/// Raven `clipHandle_t` collision model handle.
///
/// Type definition source: `oracle/code/game/q_shared.h:188`
/// Type definition source: `oracle/codemp/game/q_shared.h:363`
pub type clipHandle_t = c_int;

/// Raven `qhandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:183`
/// Type definition source: `oracle/codemp/game/q_shared.h:358`
pub type qhandle_t = c_int;

/// Raven `thandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:184`
/// Type definition source: `oracle/codemp/game/q_shared.h:359`
pub type thandle_t = c_int;

/// Raven `fxHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:185`
/// Type definition source: `oracle/codemp/game/q_shared.h:360`
pub type fxHandle_t = c_int;

/// Raven `sfxHandle_t`.
///
/// Type definition source: `oracle/code/game/q_shared.h:186`
/// Type definition source: `oracle/codemp/game/q_shared.h:361`
pub type sfxHandle_t = c_int;

/// Raven `mdxaBone_t`.
///
/// Type definition source: `oracle/code/renderer/mdx_format.h:137`
/// Type definition source: `oracle/codemp/game/q_shared.h:3078`
/// Type definition source: `oracle/codemp/renderer/mdx_format.h:137`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

const _: () = assert!(core::mem::size_of::<mdxaBone_t>() == 48);
const _: () = assert!(core::mem::offset_of!(mdxaBone_t, matrix) == 0);

/// Raven `MAX_QPATH`.
///
/// Definition source: `oracle/code/game/q_shared.h:215`
/// Definition source: `oracle/codemp/game/q_shared.h:393`
pub const MAX_QPATH: usize = 64;

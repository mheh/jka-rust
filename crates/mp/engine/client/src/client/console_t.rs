#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{qboolean, vec4_t};

/// `CON_TEXTSIZE`.
///
/// Source: `oracle/codemp/client/client.h:354`
pub const CON_TEXTSIZE: usize = 32768;

/// `NUM_CON_TIMES`.
///
/// Source: `oracle/codemp/client/client.h:356`
pub const NUM_CON_TIMES: usize = 4;

/// Raven `console_t` — the client console scrollback buffer and display state.
///
/// Type definition source: `oracle/codemp/client/client.h:358-380`
#[repr(C)]
pub struct console_t {
    pub initialized: qboolean,

    pub text: [i16; CON_TEXTSIZE],
    /// line where next message will be printed
    pub current: i32,
    /// offset in current line for next print
    pub x: i32,
    /// bottom of console displays this line
    pub display: i32,

    /// characters across screen
    pub linewidth: i32,
    /// total lines in console scrollback
    pub totallines: i32,

    /// for wide aspect screens
    pub xadjust: f32,
    /// for wide aspect screens
    pub yadjust: f32,

    /// aproaches finalFrac at scr_conspeed
    pub displayFrac: f32,
    /// 0.0 to 1.0 lines of console to display
    pub finalFrac: f32,

    /// in scanlines
    pub vislines: i32,

    /// cls.realtime time the line was generated
    /// for transparent notify lines
    pub times: [i32; NUM_CON_TIMES],
    pub color: vec4_t,
}

const _: () = assert!(core::mem::size_of::<console_t>() == 65612);
const _: () = assert!(core::mem::offset_of!(console_t, initialized) == 0);
const _: () = assert!(core::mem::offset_of!(console_t, text) == 4);
const _: () = assert!(core::mem::offset_of!(console_t, current) == 65540);
const _: () = assert!(core::mem::offset_of!(console_t, x) == 65544);
const _: () = assert!(core::mem::offset_of!(console_t, display) == 65548);
const _: () = assert!(core::mem::offset_of!(console_t, linewidth) == 65552);
const _: () = assert!(core::mem::offset_of!(console_t, totallines) == 65556);
const _: () = assert!(core::mem::offset_of!(console_t, xadjust) == 65560);
const _: () = assert!(core::mem::offset_of!(console_t, yadjust) == 65564);
const _: () = assert!(core::mem::offset_of!(console_t, displayFrac) == 65568);
const _: () = assert!(core::mem::offset_of!(console_t, finalFrac) == 65572);
const _: () = assert!(core::mem::offset_of!(console_t, vislines) == 65576);
const _: () = assert!(core::mem::offset_of!(console_t, times) == 65580);
const _: () = assert!(core::mem::offset_of!(console_t, color) == 65596);

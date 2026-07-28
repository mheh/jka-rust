#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_float;

use super::gen_func_t::genFunc_t;

/// Raven `waveForm_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:287-294`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct waveForm_t {
    pub func: genFunc_t,

    pub base: c_float,
    pub amplitude: c_float,
    pub phase: c_float,
    pub frequency: c_float,
}

const _: () = assert!(core::mem::size_of::<waveForm_t>() == 20);
const _: () = assert!(core::mem::offset_of!(waveForm_t, func) == 0);
const _: () = assert!(core::mem::offset_of!(waveForm_t, base) == 4);
const _: () = assert!(core::mem::offset_of!(waveForm_t, amplitude) == 8);
const _: () = assert!(core::mem::offset_of!(waveForm_t, phase) == 12);
const _: () = assert!(core::mem::offset_of!(waveForm_t, frequency) == 16);

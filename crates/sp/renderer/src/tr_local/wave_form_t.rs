#![allow(non_camel_case_types, non_snake_case)]

use super::gen_func_t::genFunc_t;

/// Raven `waveForm_t` — waveform generator parameters.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:276-283`
#[repr(C)]
pub struct waveForm_t {
    pub func: genFunc_t,

    pub base: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub frequency: f32,
}

const _: () = assert!(core::mem::size_of::<waveForm_t>() == 20);
const _: () = assert!(core::mem::offset_of!(waveForm_t, func) == 0);
const _: () = assert!(core::mem::offset_of!(waveForm_t, base) == 4);
const _: () = assert!(core::mem::offset_of!(waveForm_t, amplitude) == 8);
const _: () = assert!(core::mem::offset_of!(waveForm_t, phase) == 12);
const _: () = assert!(core::mem::offset_of!(waveForm_t, frequency) == 16);

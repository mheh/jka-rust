#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `T_G_ICARUS_GETSETIDFORSTRING`.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:920-923`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct T_G_ICARUS_GETSETIDFORSTRING {
    pub string: [c_char; 2048],
}

const _: () = assert!(core::mem::size_of::<T_G_ICARUS_GETSETIDFORSTRING>() == 2048);
const _: () = assert!(core::mem::offset_of!(T_G_ICARUS_GETSETIDFORSTRING, string) == 0);

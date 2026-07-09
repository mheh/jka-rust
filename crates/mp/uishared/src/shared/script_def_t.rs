#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

// Raven `#define MAX_SCRIPT_ARGS 12`.
// Source: `oracle/codemp/ui/ui_shared.h:76`
const MAX_SCRIPT_ARGS: usize = 12;

/// Raven `scriptDef_t` — a UI script command plus its argument list.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.h:106-109`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct scriptDef_t {
    pub command: *const c_char,
    pub args: [*const c_char; MAX_SCRIPT_ARGS],
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<scriptDef_t>() == 104);
const _: () = assert!(core::mem::offset_of!(scriptDef_t, command) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(scriptDef_t, args) == 8);

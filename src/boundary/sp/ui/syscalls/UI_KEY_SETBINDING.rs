use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_KEY_SETBINDING`.
///
/// Raven wrapper: `syscall( UI_KEY_SETBINDING, keynum, binding );`
/// Raven transport: `Key_SetBinding( args[1], (const char *)VMA(2) ); return 0;`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:187`
/// Args source (SP): `oracle/oracle/code/client/cl_ui.cpp:480-481`
/// Transport/switch source (SP): `oracle/oracle/code/client/cl_ui.cpp:480-482`
pub struct UiKeySetbindingArgs {
    keynum: c_int,
    binding: *const c_char,
}

impl UiKeySetbindingArgs {
    pub const fn new(keynum: c_int, binding: *const c_char) -> Self {
        Self { keynum, binding }
    }

    pub const fn keynum(&self) -> c_int {
        self.keynum
    }

    pub const fn binding(&self) -> *const c_char {
        self.binding
    }
}

/// `UI_KEY_SETBINDING` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:187`
pub struct UiKeySetbinding;

impl OutboundSysCall for UiKeySetbinding {
    type Import = SpUiImport;
    type Args = UiKeySetbindingArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_KEY_SETBINDING;
}

impl EncodeSysCall for UiKeySetbinding {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.keynum() as isize, ptr_to_word(args.binding())])
    }
}

impl DecodeSysCallReturn for UiKeySetbinding {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}

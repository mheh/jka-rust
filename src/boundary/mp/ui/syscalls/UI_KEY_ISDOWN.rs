use core::ffi::c_int;

use super::super::MpUiImport;
use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_KEY_ISDOWN`.
///
/// C ABI: `qboolean trap_Key_IsDown(int keynum)`.
/// Raven's wrapper forwards the key number as the only payload word, and the
/// client switch reads it from `args[1]`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:525-526`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:989-990`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgKeyIsdownArgs {
    /// Key number, read by Raven as `args[1]`.
    keynum: c_int,
}

impl CgKeyIsdownArgs {
    pub const fn new(keynum: c_int) -> Self {
        Self { keynum }
    }

    pub const fn keynum(&self) -> c_int {
        self.keynum
    }
}

/// `UI_KEY_ISDOWN` MP cgame imports syscall boundary token.
///
/// Raven wrapper: `return syscall( UI_KEY_ISDOWN, keynum );`
/// Raven transport: `return Key_IsDown( args[1] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:194`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:525-526`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:525-526`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:989-990`
pub struct CgKeyIsdown;

impl OutboundSysCall for CgKeyIsdown {
    type Import = MpUiImport;
    type Args = CgKeyIsdownArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_ISDOWN;
}

impl EncodeSysCall for CgKeyIsdown {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.keynum() as isize])
    }
}

impl DecodeSysCallReturn for CgKeyIsdown {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}

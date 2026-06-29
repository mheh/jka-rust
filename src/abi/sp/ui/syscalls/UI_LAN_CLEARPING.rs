use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_CLEARPING`.
///
/// Raven wrapper: `syscall( UI_LAN_CLEARPING, n );`
/// Raven transport: `LAN_ClearPing( args[1] );`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:199`
/// Args source (SP): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_CLEARPING`.
/// Fallback args/source: `oracle/oracle/codemp/ui/ui_syscalls.c:306-307`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1073-1074`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:969`
pub struct UiLanClearpingArgs {
    n: c_int,
}

impl UiLanClearpingArgs {
    pub const fn new(n: c_int) -> Self {
        Self { n }
    }

    pub const fn n(&self) -> c_int {
        self.n
    }
}

/// `UI_LAN_CLEARPING` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:199`
pub struct UiLanClearping;

impl OutboundSysCall for UiLanClearping {
    type Import = SpUiImport;
    type Args = UiLanClearpingArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_CLEARPING;
}

impl EncodeSysCall for UiLanClearping {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.n() as isize])
    }
}

impl DecodeSysCallReturn for UiLanClearping {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}

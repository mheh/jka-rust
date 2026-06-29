use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_CLEARPING`.
///
/// Raven wrapper: `syscall( UI_LAN_CLEARPING, n );`
/// Raven transport: `LAN_ClearPing( args[1] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:306-307`
#[derive(Debug)]
pub struct UiLanClearpingArgs {
    n: c_int,
}

impl UiLanClearpingArgs {
    pub fn new(n: c_int) -> Self {
        Self { n }
    }

    pub fn n(&self) -> c_int {
        self.n
    }
}

/// `UI_LAN_CLEARPING` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_CLEARPING, n );`
/// Raven transport: `LAN_ClearPing( args[1] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:66`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:306-307`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:969`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1073-1075`
pub struct UiLanClearping;

impl OutboundSysCall for UiLanClearping {
    type Import = MpUiImport;
    type Args = UiLanClearpingArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_CLEARPING;
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

use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_GETSERVERPING`.
///
/// Raven wrapper: `return syscall( UI_LAN_GETSERVERPING, source, n );`
/// Raven transport: `return LAN_GetServerPing( args[1], args[2] );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:282-283`
#[derive(Debug)]
pub struct UiLanGetserverpingArgs {
    source: c_int,
    n: c_int,
}

impl UiLanGetserverpingArgs {
    pub fn new(source: c_int, n: c_int) -> Self {
        Self { source, n }
    }

    pub fn source(&self) -> c_int {
        self.source
    }

    pub fn n(&self) -> c_int {
        self.n
    }
}

/// `UI_LAN_GETSERVERPING` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_GETSERVERPING, source, n );`
/// Raven transport: `return LAN_GetServerPing( args[1], args[2] );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:112`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:282-283`
/// Output source: `oracle/codemp/ui/ui_local.h:967`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1096-1097`
pub struct UiLanGetserverping;

impl OutboundSysCall for UiLanGetserverping {
    type Import = MpUiImport;
    type Args = UiLanGetserverpingArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETSERVERPING;
}

impl EncodeSysCall for UiLanGetserverping {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize, args.n() as isize])
    }
}

impl DecodeSysCallReturn for UiLanGetserverping {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}

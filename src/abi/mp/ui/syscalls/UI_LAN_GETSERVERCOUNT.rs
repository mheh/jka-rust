use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_GETSERVERCOUNT`.
///
/// Raven wrapper: `return syscall( UI_LAN_GETSERVERCOUNT, source );`
/// Raven transport: `return LAN_GetServerCount(args[1]);`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:270-271`
#[derive(Debug)]
pub struct UiLanGetservercountArgs {
    source: c_int,
}

impl UiLanGetservercountArgs {
    pub fn new(source: c_int) -> Self {
        Self { source }
    }

    pub fn source(&self) -> c_int {
        self.source
    }
}

/// `UI_LAN_GETSERVERCOUNT` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_GETSERVERCOUNT, source );`
/// Raven transport: `return LAN_GetServerCount(args[1]);`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:95`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:270-271`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:964`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1085-1086`
pub struct UiLanGetservercount;

impl OutboundSysCall for UiLanGetservercount {
    type Import = MpUiImport;
    type Args = UiLanGetservercountArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_GETSERVERCOUNT;
}

impl EncodeSysCall for UiLanGetservercount {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize])
    }
}

impl DecodeSysCallReturn for UiLanGetservercount {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}

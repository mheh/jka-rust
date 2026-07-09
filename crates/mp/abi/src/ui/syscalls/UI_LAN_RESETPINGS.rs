use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_LAN_RESETPINGS`.
///
/// Raven wrapper: `syscall( UI_LAN_RESETPINGS, n );`
/// Raven transport: `LAN_ResetPings( args[1] );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:302-303`
#[derive(Debug)]
pub struct UiLanResetpingsArgs {
    n: c_int,
}

impl UiLanResetpingsArgs {
    pub fn new(n: c_int) -> Self {
        Self { n }
    }

    pub fn n(&self) -> c_int {
        self.n
    }
}

/// `UI_LAN_RESETPINGS` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_RESETPINGS, n );`
/// Raven transport: `LAN_ResetPings( args[1] );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:100`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:302-303`
/// Output source: `oracle/codemp/ui/ui_local.h:979`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1109-1111`
pub struct UiLanResetpings;

impl OutboundSysCall for UiLanResetpings {
    type Import = MpUiImport;
    type Args = UiLanResetpingsArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_RESETPINGS;
}

impl EncodeSysCall for UiLanResetpings {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.n() as isize])
    }
}

impl DecodeSysCallReturn for UiLanResetpings {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}

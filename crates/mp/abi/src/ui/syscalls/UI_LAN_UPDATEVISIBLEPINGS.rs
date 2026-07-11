use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `UI_LAN_UPDATEVISIBLEPINGS`.
///
/// Raven wrapper: `return syscall( UI_LAN_UPDATEVISIBLEPINGS, source );`
/// Raven transport: `return LAN_UpdateVisiblePings( args[1] );`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:326-327`
#[derive(Debug)]
pub struct UiLanUpdatevisiblepingsArgs {
    source: c_int,
}

impl UiLanUpdatevisiblepingsArgs {
    pub fn new(source: c_int) -> Self {
        Self { source }
    }

    pub fn source(&self) -> c_int {
        self.source
    }
}

/// `UI_LAN_UPDATEVISIBLEPINGS` MP UI imports syscall ABI token.
///
/// Raven wrapper: `return syscall( UI_LAN_UPDATEVISIBLEPINGS, source );`
/// Raven transport: `return LAN_UpdateVisiblePings( args[1] );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:99`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:326-327`
/// Output source: `oracle/codemp/ui/ui_local.h:976`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1106-1107`
pub struct UiLanUpdatevisiblepings;

impl OutboundSysCall for UiLanUpdatevisiblepings {
    type Import = MpUiImport;
    type Args = UiLanUpdatevisiblepingsArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_UPDATEVISIBLEPINGS;
}

impl EncodeSysCall for UiLanUpdatevisiblepings {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize])
    }
}

impl DecodeSysCallReturn for UiLanUpdatevisiblepings {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}

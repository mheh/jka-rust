use core::ffi::c_int;

use super::super::SpUiImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_RESETPINGS`.
///
/// Raven wrapper: `LAN_ResetPings(source);`
/// Raven transport: `LAN_ResetPings(args[1]);`
///
/// Enum source: `oracle/code/ui/ui_public.h:222`
/// Args source (SP fallback): `oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_RESETPINGS`.
/// Args source (fallback): `oracle/codemp/ui/ui_local.h:979`
/// Transport/switch source (fallback): `oracle/codemp/client/cl_ui.cpp:1109-1110`
/// Output source fallback: `oracle/codemp/ui/ui_local.h:979`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLanResetpingsArgs {
    source: c_int,
}

impl UiLanResetpingsArgs {
    pub const fn new(source: c_int) -> Self {
        Self { source }
    }

    pub const fn source(&self) -> c_int {
        self.source
    }
}

/// `UI_LAN_RESETPINGS` SP UI imports syscall ABI token.
///
/// Raven: 70
/// Source: `oracle/code/ui/ui_public.h:222`
pub struct UiLanResetpings;

impl OutboundSysCall for UiLanResetpings {
    type Import = SpUiImport;
    type Args = UiLanResetpingsArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_RESETPINGS;
}

impl EncodeSysCall for UiLanResetpings {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize])
    }
}

impl DecodeSysCallReturn for UiLanResetpings {
    fn decode_return(_word: isize) -> Self::Output {}
}

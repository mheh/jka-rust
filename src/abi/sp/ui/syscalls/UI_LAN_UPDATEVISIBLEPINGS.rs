use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_LAN_UPDATEVISIBLEPINGS`.
///
/// Raven wrapper: `return syscall( UI_LAN_UPDATEVISIBLEPINGS, source );`
/// Raven transport: `return LAN_UpdateVisiblePings( args[1] );`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:221`
/// Args source (SP fallback): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_UPDATEVISIBLEPINGS`.
/// Args source (fallback): `oracle/oracle/codemp/ui/ui_local.h:976`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1106-1107`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:976`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLanUpdatevisiblepingsArgs {
    source: c_int,
}

impl UiLanUpdatevisiblepingsArgs {
    pub const fn new(source: c_int) -> Self {
        Self { source }
    }

    pub const fn source(&self) -> c_int {
        self.source
    }
}

/// `UI_LAN_UPDATEVISIBLEPINGS` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:221`
pub struct UiLanUpdatevisiblepings;

impl OutboundSysCall for UiLanUpdatevisiblepings {
    type Import = SpUiImport;
    type Args = UiLanUpdatevisiblepingsArgs;
    type Output = qboolean;

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_UPDATEVISIBLEPINGS;
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

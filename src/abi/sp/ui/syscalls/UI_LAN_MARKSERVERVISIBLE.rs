use core::ffi::c_int;

use super::super::SpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `UI_LAN_MARKSERVERVISIBLE`.
///
/// Raven wrapper: `return syscall( UI_LAN_MARKSERVERVISIBLE, source, n, visible );`
/// Raven transport: `LAN_MarkServerVisible( args[1], args[2], (qboolean)args[3] );`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:220`
/// Args source (SP fallback): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_LAN_MARKSERVERVISIBLE`.
/// Args source (fallback): `oracle/oracle/codemp/ui/ui_local.h:974`
/// Transport/switch source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:1099-1101`
/// Output source fallback: `oracle/oracle/codemp/ui/ui_local.h:974`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLanMarkservervisibleArgs {
    source: c_int,
    n: c_int,
    visible: qboolean,
}

impl UiLanMarkservervisibleArgs {
    pub const fn new(source: c_int, n: c_int, visible: qboolean) -> Self {
        Self { source, n, visible }
    }

    pub const fn source(&self) -> c_int {
        self.source
    }

    pub const fn n(&self) -> c_int {
        self.n
    }

    pub const fn visible(&self) -> qboolean {
        self.visible
    }
}

/// `UI_LAN_MARKSERVERVISIBLE` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:220`
pub struct UiLanMarkservervisible;

impl OutboundSysCall for UiLanMarkservervisible {
    type Import = SpUiImport;
    type Args = UiLanMarkservervisibleArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_LAN_MARKSERVERVISIBLE;
}

impl EncodeSysCall for UiLanMarkservervisible {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.source() as isize,
            args.n() as isize,
            args.visible() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiLanMarkservervisible {
    fn decode_return(_word: isize) -> Self::Output {}
}

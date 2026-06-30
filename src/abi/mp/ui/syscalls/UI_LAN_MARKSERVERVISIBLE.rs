use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::types::qboolean;

/// Arguments for `UI_LAN_MARKSERVERVISIBLE`.
///
/// Raven wrapper: `syscall( UI_LAN_MARKSERVERVISIBLE, source, n, visible );`
/// Raven transport: `LAN_MarkServerVisible( args[1], args[2], (qboolean)args[3] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:318-319`
#[derive(Debug)]
pub struct UiLanMarkservervisibleArgs {
    source: c_int,
    n: c_int,
    visible: qboolean,
}

impl UiLanMarkservervisibleArgs {
    pub fn new(source: c_int, n: c_int, visible: qboolean) -> Self {
        Self { source, n, visible }
    }

    pub fn source(&self) -> c_int {
        self.source
    }

    pub fn n(&self) -> c_int {
        self.n
    }

    pub fn visible(&self) -> qboolean {
        self.visible
    }
}

/// `UI_LAN_MARKSERVERVISIBLE` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_MARKSERVERVISIBLE, source, n, visible );`
/// Raven transport: `LAN_MarkServerVisible( args[1], args[2], (qboolean)args[3] );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:98`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:318-319`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:974`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1099-1101`
pub struct UiLanMarkservervisible;

impl OutboundSysCall for UiLanMarkservervisible {
    type Import = MpUiImport;
    type Args = UiLanMarkservervisibleArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_MARKSERVERVISIBLE;
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
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}

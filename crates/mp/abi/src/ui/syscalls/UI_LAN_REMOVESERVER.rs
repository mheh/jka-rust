use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_LAN_REMOVESERVER`.
///
/// Raven wrapper: `syscall( UI_LAN_REMOVESERVER, source, addr );`
/// Raven transport: `LAN_RemoveServer( args[1], (const char *)VMA(2) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:334-335`
#[derive(Debug)]
pub struct UiLanRemoveserverArgs {
    source: c_int,
    addr: *const c_char,
}

impl UiLanRemoveserverArgs {
    pub fn new(source: c_int, addr: *const c_char) -> Self {
        Self { source, addr }
    }

    pub fn source(&self) -> c_int {
        self.source
    }

    pub fn addr(&self) -> *const c_char {
        self.addr
    }
}

/// `UI_LAN_REMOVESERVER` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_LAN_REMOVESERVER, source, addr );`
/// Raven transport: `LAN_RemoveServer( args[1], (const char *)VMA(2) );`
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:104`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:334-335`
/// Output source: `oracle/oracle/codemp/ui/ui_local.h:978`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1066-1067`
pub struct UiLanRemoveserver;

impl OutboundSysCall for UiLanRemoveserver {
    type Import = MpUiImport;
    type Args = UiLanRemoveserverArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_LAN_REMOVESERVER;
}

impl EncodeSysCall for UiLanRemoveserver {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.source() as isize, ptr_to_word(args.addr())])
    }
}

impl DecodeSysCallReturn for UiLanRemoveserver {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
